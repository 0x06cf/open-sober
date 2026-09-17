# SH248c — cross the -9 bad_alloc AND the app-name NULL-store gate; continueAfterFlagsLoaded_ now enters nativeAppBridgeAppStart for the first time headlessly

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## What was done (two concrete seed fixes, both default-inert env-gated)
The DMCONT continuation (real `continueAfterFlagsLoaded_` 0x102bd1d68) was stopped at
two distinct gates; SH248b located them precisely, this cycle clears both and measures
the NEXT fencepost.

1. **-9 bad_alloc (SH248 root cause) fixed** — `routeb_dm_manager_cont`'s M+0x48 app-name
   seed wrote `cap=1` (only the long-flag bit0, ZERO capacity). The engine's
   serialize-assign at 0x2bd1dfc reads destination `[this]&~1` = 0 as capacity -> not
   enough room -> grow path -> `oldcap-1` underflows to 0xffff..ff > max_size ->
   `b.hi 0x2b50690` -> `x23=-9` -> `operator_new(-9)`->NULL->`std::bad_alloc`. Fix: seed
   a VALID long capacity `[m+0x48]=0x11` (cap 0x10, bit0=1 long) with the correct
   long-form layout ([0]=cap, [8]=size, [16]=data ptr) and a 64-byte leaked "Home\0"
   buffer. MEASURED: the `OPERATOR_NEW size=-9` line is GONE; all 20+ serialize-assign
   block entries (0x2bd1dfc..0x2bd1f5c) run clean.

2. **App-name NULL-store fault (0x2bd1fd4) crossed** — the engine's own serializer
   (driven from the all-zeroed flags-holder F) OVERWRITES M+0x48 with an EMPTY string
   (size=0), so the guard at 0x2bd1f64 (`cbnz [M+0x50]`) falls through to the deliberate
   `mov x8,xzr; strb w9,[x8]` NULL store (SIGSEGV). New guard
   `routeb_cont_appname_seed_guard` (opt-in `JIT_ROUTEB_CONT_APPNAME_SEED=1`) fires at
   the guard's block entry 0x102bd1f64 and re-seeds `[M+0x50]=5` (long-form size,
   M+0x58 still holds the "Home" data ptr) so the cbnz skips to 0x2bd1fe0. MEASURED:
   the continuation passes 0x2bd1fd4 and reaches the app-start call path.

## Measured (real libroblox.so, canonical --v2boot ladder, DMFORCE+DMCONT+SH245 env,
## plus M48_SEED=1 + CONT_APPNAME_SEED=1)
- BEFORE these fixes the continuation died at 0x2b50600 `-9 bad_alloc` (SH248 series) /
  0x2bd1fd4 NULL-store (SH245, when the M+0x48 cap happened to be enough).
- AFTER: the continuation runs its ENTIRE serialize body (region-watch block entries to
  0x102bd1f64 then 0x102bd2014) and **enters `nativeAppBridgeAppStart` / the
  NativeAppBridge app-start path for the FIRST time headlessly** (reproduced 2/6 ladder
  runs; run-variable flake ~1/3). The next fencepost is a SIGSEGV inside the app-start
  string construction: `guestpc=0x102b504e4 (string assign) fault=0x0, x0=0x0 (NULL
  destination), x1=0x55dc.. src, x2=0x55dc..` — a NULL-`this` std::string copy in the
  live app-start/controller construction (the SH174/SH204-class object-lifetime wall,
  now reached from inside the real nativeAppBridgeAppStart rather than a harness-driven
  path).

## Honest (do-not-over-claim)
- Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED. The single
  forward hook (SH174 capture-latch at a real `make_shared<DataModel>`) is untouched.
- What IS new and measured: the continuation and its serialize body complete cleanly, and
  the engine's own app-start entry executes headlessly for the first time. This moves the
  DMCONT line from "bad_alloc" to "inside nativeAppBridgeAppStart, NULL-controller string
  copy" — a genuine forward execution boundary, gated by a live constructed object the
  flags-holder F can't supply (empty headlessly).

## Next (honest, single-agent)
The NULL-destination string assign inside app-start is the post-nativeAppBridgeAppStart
fencepost. Both candidate directions are the fabricatable-object-graph class (SH174/SH204
map): seed the string destination F holds for the app-start controller, or supply a live
controller object. Standing forward hook unchanged.

## Code / files
- crates/arm64jit/src/jit.rs: `routeb_dm_manager_cont` M+0x48 seed -> valid long cap 0x11 +
  correct layout + 64B "Home" buffer; `CONT_MANAGED_M` static +
  `routeb_cont_managed_m()` + `routeb_cont_appname_seed_guard` (opt-in
  JIT_ROUTEB_CONT_APPNAME_SEED=1, fires at pc=0x102bd1f64); wired into the block-entry
  dispatch after `routeb_alloc_probe_guard`. All default-inert (no default-config change).
- repro runs/capture_sh248c_cont_next.sh + runs/batch_sh248c_cont_next.sh.
- cargo build --workspace / cargo test --workspace green.