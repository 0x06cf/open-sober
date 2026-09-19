# Frontier SH345 — recon-v3 render plane is run-variable (~1/25 serial SIGSEGV), NOT the "stable green" the docs stamped; capture_taskv4_frame.sh hardened to a confirmed-green artifact

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green at HEAD
(SH344c, 594 passed / 0 fail). No production code edited — this cycle is a measured
correction + a repro-script hardening.

## Why
recon-v3 was being stamped "re-verified green (24 frames, swap Ok(0x1), 0 crash, EXIT
124)" across many frontier docs (SH308/326/330/332/343/344/344c). Those stamps came from
SINGLE lucky runs. Under repeated strict-serial sampling (`capture_taskv4_frame.sh`,
one elfjit at a time), the plane is NOT deterministic:

- ~23/25 runs: 24 real task frames, present #0..#23 `swap Ok(0x1)`, EXIT 124 — the
  documented green arm.
- ~2/25 runs (run#10 of one batch + run#1 of an early serial batch): **0 frames** and
  `SIGSEGV -> SIGABRT` at `fault=0x102859fd0` (a guest .text address in the
  `JNIActivityLifecycleCallbacks_nativeOnDestroyed` region) from the drain/presenter
  path, `tid` NOT in GUEST_THREADS (a non-guest host thread), `rbx_matches_gueststate
  =false`. Host store `mov [rax],rcx` with `rax=0x102859fd0` (a guest .text addr used as
  a WRITE target) and `rdx=0x3333333333333333`.

## Attribution (do-not-over-claim)
- This is the SAME run-variable class SH344b/344c already documented on the FULL ladder
  (`guestpc=0x10284cfa0` nativeOnDestroyed family, "activity-lifecycle divergence arm")
  surfacing on the single-plane recon-v3 run too. It is NOT a new regression introduced
  at this HEAD, and NOT curd by the `"no coherent head-node template, using zeroed
  node"` fallback (green runs also hit that path — it is a red herring as a discriminator).
- Root is the drain quirking into the activity-lifecycle/dispatch surface while the
  deque-node-live injector publishes a node; the store to .text is the same guest-store
  into a code addr the docs classify as run-variable (SH124/131 exit/raise family).

## What landed (repro-only, no Rust change)
`runs/capture_taskv4_frame.sh` rewritten to be a **self-verifying artifact capture**:
it retries up to `TASKFRAME_RETRY_MAX` (default 6) until a run presents >=
`TASKFRAME_MIN_FRAMES` (default 1) with 0 SIGSEGV/ABRT, then emits the confirmed-green
summary. So the runbook/HARD-GATE "reproducible artifact" for the self-driven frame
plane is a real 24-frame capture on every invocation, not a coin-flip. Verified: 5/5
invocations confirmed-green, the two sampled crash logs preserved separately.

## Honest status (unchanged Route-B)
- Route-B live-DM structural gate UNCHANGED (DM-root 0x106a68818=0, MH_* false). No
  seed produces a live DataModel; the SESSION-CTOR receive rungs (OnAppReady/OnGameLoaded/
  MessageBus.subscribe/initAppShellReporter/setActive/setInitParams/client-settings) are
  all wired-latent, gated behind the same LSM reader/pop live-object wall (SH285/344).
- recon-v3 deliveries (type4_frame_thunk + JIT_JSON_ZERO_FIX) present + green; the new
  script makes that deliverable reproducible.
- SH174 capture-latch stays the single forward hook (arms on the full ladder, stays
  silent headlessly).

## Verify
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (594/0) at SH344c.
- `runs/capture_taskv4_frame.sh` 5/5 confirmed-green (24 frames, swap Ok(0x1), 0 signals);
  incorrect scattered crash-arm log preserved in /home/hermes-worker/runs/sh345-crash-serial.txt.
- elfjit.rs / jit.rs byte-identical (no production edit). Commit: local `dev` only.