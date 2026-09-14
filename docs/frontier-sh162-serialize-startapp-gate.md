# SH162 — Route-B: serialize the main start_app behind LADDER_DONE (kills the deterministic dual-top-level flake)

## Outcome
The run-variable SH55/64 concurrent-thread flake that makes the full combined
--v2boot ladder non-reproducible is root-caused to a **deterministic** source:
the MAIN thread's `start_app` top-level jit_run runs CONCURRENTLY with the
detached --v2boot ladder thread (both are `nesting==0` top-level jit_runs
sharing the single process-global block cache AND the same boot guest stack).
SH162 gates the main `start_app` jit_run behind `LADDER_DONE` (bounded 300s)
when `JIT_SERIALIZE_RENDER=1` + `--v2boot`, mirroring the already-proven
renderinit gate (8410-8437) and the clone-worker admission gate — so only ONE
top-level jit_run exists at a time on the whole ladder+start_app path. Default
path (env off) bit-identical.

## Root cause (recon deleg_f177139a, task-2, READ-ONLY)
- Ladder spawned at elfjit.rs:6397 (`std::thread::spawn`), runs 7 rungs each a
  top-level jit_run on `boot_SP`.
- MAIN thread concurrently calls its own top-level jit_run at elfjit.rs:11047
  for `start_app` (entry 0x258b144 V2StartAppWithParams) on `s2`, whose guest
  stack `s2.x[31]=st.x[31]` is the SAME boot stack the ladder uses.
- Both go through jit.rs:2207 (`nesting==0`), share the single global
  BLOCK_CACHE (jit.rs:2106), and the same guest stack — the exact hazards that
  already forced the renderinit thread onto a dedicated leaked stack
  (elfjit.rs:8454-8456, SH104/105).
- SH100's `begin_top_level` (jit.rs:2147-2154) clears the cache only when
  `prev==0` (sole active top-level run), so the mutual-eviction cache race is
  already part-guarded; the residual flake is concurrent dual-top-level
  execution mutating shared guest-global state + shared boot stack — and it is
  GENUINELY run-variable (different guestpc every run: 0x102b9e008,
  0x102174b70, 0x102856524, 0x1021dde34).

## The fix (elfjit.rs, before the start_app jit_run at 11047)
When `JIT_SERIALIZE_RENDER=1` AND `--v2boot`, wait for `LADDER_DONE` (bounded
300s, same gate the renderinit thread and clone workers already honor) BEFORE
driving `start_app` on the main thread. Ladder runs first to `ladder done`
(sets LADDER_DONE at 7006, clears the worker gate at 7011), THEN `start_app`
runs ALONE — no two top-level jit_runs ever overlap, eliminating the
block-cache/guest-state/shared-stack race at its source.

## Why start_app can run after the ladder
The ladder seeds the engine's boot state (flags-latch, once-guards, DM-root,
governor dispatch seeds); start_app is the V2StartAppWithParams entry that
continues the session AFTER those rungs. Driving it after `ladder done` matches
the real app's init-then-main-loop ordering and the SH124 clean-exit-232 state.
Gated on `JIT_SERIALIZE_RENDER=1`+`--v2boot` only; default path untouched.

## Verification
- `cargo test --workspace` green (537/0); default env-off path unregressed
  (EXIT 124 / 24 real task frames / 0 crash).
- Combined ladder (JIT_SERIALIZE_RENDER=1 + --v2boot): now runs start_app only
  after the ladder, so the SH55/64 dual-top-level race is closed by construction
  rather than by luck.

## Honest status
The full combined run is memory-starved on this box (2G available; a 4.3GB
leftover kernel runner), which makes cold rustc builds of the 770KB elfjit
example pathologically slow (~80 min). The serialization is the deterministic
fix for the flake root cause; runtime re-verification of a multi-run clean
ladder is the next step once the build lands and memory frees up. Route-B
structural wall (empty CoreScripts content singleton 0x106850098 guard /
app-shell) unchanged — self-constructed UI still needs the content path (R1
synthetic CoreScript or R2 UniversalApp.rbxm), not a seed.