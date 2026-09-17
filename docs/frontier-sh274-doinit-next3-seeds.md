# Frontier SH274 — ROUTE-B NEXT-3 do-init SEED VALUES implemented + measured

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Route-B live-DM
structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
+1 hermetic test; default-inert (env JIT_ROUTEB_DOINIT_NEXT3). Workspace green
(cargo test --workspace 578/0; arm64jit examples 107/0).

## Why (the authoritative NEXT-3 do-init seeds were never value-written)
The ROUTE-B RECON V3 doc (deleg_8d5648cf, authoritative) lists THREE do-init /
app-shell ctor seeds that clear concrete SEGVs:
  #1 thread-init singleton [0x1067333aa0] = ptr to a 0x20 zeroed buffer
     (clears SEGV 0x102207ef0: `ldr x0,[x0,#16]; cbz x0,ret` — a NULL-global deref).
  #2 telemetry once-cell [0x106dcd380]  = -1
     (clears the 2b4cd1c pthread_cond_wait park / async gate).
  #3 map page + set [0x10673336d8].bit0 = 1
     (clears SEGV 0x102212838: `ldarb w8,[x0]; tbz w8,#0` spin-on-flag).
SH156 already MAPS the containing pages but NEVER writes these values. This
cycle implements the VALUE seeds (two mechanisms, both default-inert) and
measures whether they advance the do-init/app-shell world-build.

## Code
- jit.rs: `routeb_doinit_next3_seed_guard` (opt-in JIT_ROUTEB_DOINIT_NEXT3) —
  fires on ANY block entry in the do-init band [0x102206c40, 0x102213000);
  idempotent (thread-init only when 0, telemetry only when != -1, map-page ORs
  bit0). Wired into the block-entry dispatch.
- elfjit.rs: DIRECT ladder seed at the SH156 page-map synthesis point (same opt
  env) — the GOVFLAG lesson (SH269): the app-shell ctor body blocks are already
  JIT-cached by the prior StartLuaAppDM rung, so the entry guard alone is
  partially latent. Both are value-seeded here so the seed fires at synthesis.
- jit.rs test `routeb_doinit_next3_guard_env_pc_gated_and_seeds_values`:
  (a) inert without env, (b) pc-gated to the band, (c) writes the three
  documented values, (d) idempotent. 1 passed; arm64jit lib now 399/0.

## MEASURED (real libroblox.so, full SH258/SH259 seed set, A/B)
- A (base): EXIT 134, terminal guestpc=0x101db1d04 (standing LSM insert-leaf
  wall, SH260), regionpcs=169 (do-init band + map wall).
- B (+JIT_ROUTEB_DOINIT_NEXT3): EXIT 134, ALL THREE seeds FIRE headlessly at
  pc=0x1022076f8 (thread-init -> leaked 0x20 buffer, telemetry -> -1, map-page
  bit0 -> 1), regionpcs=168, terminal **UNCHANGED** at 0x101db1d04.
- Repro runs/capture_sh274_doinit_next3_ab.sh.

## Honest verdict (do-not-over-claim)
The NEXT-3 seed VALUES are now implemented and provably fire headlessly (the
three ctor-global writes land at synthesis). The A/B measurement shows they do
NOT shift the app-start terminal — the do-init/app-shell ctor SEGVs (0x102207ef0,
2b4cd1c, 0x102212838) are NOT the binding floor on the full --v2boot ladder;
control dies earlier at the SH260 LSM insert-leaf wall (0x101db1d04), which is
the parked persistence-detour wall (SEP-15 directive). The seeds are latent-but-
correct forward world-build prep (they clear three real ctor faults a later
crossing of the LSM wall would hit), default-inert, regression-free.
Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED;
SH174 capture-latch stays the single forward hook.

## Verify
- cargo build --workspace EXIT 0; cargo test --workspace 578/0;
  cargo test -p arm64jit --example elfjit 107/0; new unit 1 passed.
- Commit: local dev only.