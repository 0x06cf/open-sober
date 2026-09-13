# SH100–SH101 — broke the SH55/64 run-variable SIGSEGV class + cleared the globalinit do-init walker dispatch-2 gate

## SH100: process-global top-level-run counter guards the block-cache eviction

**Symptom:** the --v2boot ladder SIGSEGVs at a RUN-VARIABLE site across runs
(observed 0x102208450 / 0x102208504 fault 0x18, / 0x1021db144). This is the
long-documented SH55/64 class ("concurrent top-level jit_runs SIGABRT the shared
block cache").

**Root cause (recon deleg_f09cb718):** `clear_block_cache()` fired on EVERY
top-level `jit_run` (the `nesting==0` branch in `jit_run`). The do-init spawns
guest WORKER THREADS (via `spawn_pthread`) that each call `jit_run` top-level,
so while the render/ladder thread was mid-execution of a translated block, a
concurrent worker's top-level entry evicted every cached block. The eviction
itself doesn't free leaked blocks, but the torn guest-global state two threads
mutate concurrently surfaced as the run-variable SIGSEGV across different guest
pcs.

**Fix:** process-global `ACTIVE_TOP_LEVEL_RUNS: AtomicU32`. On a top-level entry
`begin_top_level()` bumps it and only calls `clear_block_cache()` when this is
the FIRST active run (`prev==0`); a second thread entering top-level does NOT
evict. `end_top_level()` decrements. Deadlock-free (no cross-thread wait) and
does not starve the render thread.

**Result:** the ladder crash became DETERMINISTIC (4/4 runs at the same
guestpc/fault) — the SH55/64 run-variability is gone, leaving pure content seeds
to fix.

## SH101: dispatch-1 scratch leaf — clears the walker's dispatch-2 [0x18] fault

**Symptom:** with SH100 the crash was deterministic at guestpc 0x102208504,
fault=0x18.

**Root cause:** the do-init walker block (0x102208440..0x1022085c0) does two
virtual dispatches on obj=[0x106dcae20]:
- dispatch-1 `ldr x10,[obj]; ldr x8,[x10,#0x10]; blr x8`
- dispatch-2 `ldr x9,[obj]; ldr x9,[x9,#0x18]; blr x9`

The block then runs a guest MEMZERO loop whose base is `x11 = dispatch-1-ret +
(x25<<4)`, and `x25 = ([(vector end)] - [begin])>>4 = 0` (empty seeded vector).
So `x11 = dispatch-1's return in x0`. The identity host leaf (`routeb_singleton_leaf`)
returns its first arg a0 == the obj pointer the caller passed in x0 — so the
memzero erased obj[0] (and obj[8]), and dispatch-2 `ldr x9,[obj]` read 0 ->
`[0+0x18]` fault 0x18.

**Fix:** install a SCRATCH-returning host leaf (`routeb_disptch1_scratch_leaf`,
returns a dedicated leaked writable 0x100 buffer, NOT a0) at dispatch-1's vtable
slot. dispatch-1 reads `[vtab+0x10]` (obj[0]==vtab), so the seed sets
`vtab[0x10] = scratch_leaf`. dispatch-2's slot `[0x106846988]` keeps the identity
leaf (its return feeds a branch, not a memzero).

**Result:** dispatch-2 CLEARED. The ladder advances deeper into nativeGameGlobalInit
to a NEW deterministic gate: guestpc 0x102dade34, `ldrb w8,[x0]` fault=0x0 (x0=0
after bl 0x102b511e0 then bl 0x10221364c) — real globalinit progress past the
walker.

## Verification
- cargo build --workspace: 0 errors.
- cargo test --workspace: 518 passed / 0 failed.
- v2boot ladder: 3/3 deterministic at the new gate (0x102dade34, fault 0x0) —
  no longer run-variable.
- Product path unaffected (both changes gated behind --v2boot seeds / the
  counter only changes conforms the existing top-level eviction contract to only
  evict when single-run).