# Frontier SH349 — CROSS the SH285 terminal by RETing the faulty LSM byte-copy sub-call (0x101d9a15c); the Session-CTOR persistence lane advances one fencepost to a GOT/canary read wall at 0x101d9a708

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Additions: default-inert
opt-in `JIT_ROUTEB_LSM_APPEND_SKIP=1` (`routeb_patch_lsm_append_skip`, elfjit.rs) + `sh349`
hermetic (real-image pins: byte-copy prologue / SH285 caller bl / fault store / natural ret)
+ runs/capture_sh349_lsm_append_skip.sh. No production code path edited; new patch env-gated.
Workspace green (arm64jit 416/0 + elfjit example 157/0).

## Why (genuinely new, not a re-run)
SH348 measured that leaf-`ret`ing initStorageManagerNative's ENTRY (0x101d9d8b0) does NOT clear
the SH285 SIGSEGV (`guestpc=0x101db1b08`): the caller block is reached by a mid-function direct
jump that bypasses the entry patch. So "skip the whole init" is the wrong granularity — the
fault is inside a single self-contained leaf sub-call, a libc++-style backward byte-copy:
`ldr w8,[x2]; ...; strb w11,[x10],#-1` (writes through the manager's unconstructed [obj+0x50]=0xff..ff).

## The patch
`routeb_patch_lsm_append_skip` RETs ONLY that byte-copy sub-call (0x101d9a15c, `ldr w8,[x2]` ->
`ret`), a pure-memcpy leaf with zero observable side effects, so EVERY path into the fault is
stubbed regardless of how the caller block is reached. CAUSE-level line-cross (SH117), NOT a
live-object repair (SH248h trap). Block-cache dropped for the append body + SH285 caller block
so no stale translated `strb` block survives.

## MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES + both skips, 3/3)
```
append_skip=1  init_skip=1
OLD terminal GONE: no more SIGSEGV guestpc=0x101db1b08
NEW terminal (3/3, EXIT 134 SIGABRT after): SIGSEGV guestpc=0x101d9a708 fault=0xffffffffffffffff
  -> the SH285 persistence-lane wall is CROSSED
```
0x101d9a708 is a stack-canon name/version-packing helper (`sub sp,sp,#0x50`; `adrp x20,67d1000;
ldr x20,[x20,#1776]` -> x20=[0x67d16f0]; `ldr x8,[x20]; stur x8,[x29,#-8]` = GOT canary read; then
backward byte-copy; `cmp x8; b.ne __stack_chk_fail`). The fault is a LOAD `ldr x8,[x20]` where
x20=[0x67d16f0] = the GOT entry VALUE, which is 0xff..ff (unrelocated / garbage) instead of a valid
canary pointer. It's the final 8 bytes of `.got` (file .got range 0x67c9a28-0x67d16f8).

## What is genuinely new + measured
- **The long-standing, repeatedly-parked SH285 terminal (every SH260/284/285/3444/348 run died at
  0x101db1b08) is now CROSSED headlessly** — the first time any run gets past it. The Session-CTOR
  persistence lane (DMCONT continuation -> initStorageManagerNative) advances one full fencepost.
- SH348's stated next step ("a storage-skip must target the failing sub-call, not the whole init")
  is now implemented and verified to work exactly as predicted.

## Honest (do-not-over-claim)
- Does NOT manufacture a DataModel; DM-root [0x106a68818]=0; MH_* stay false; Route-B live-DM
  structural gate UNCHANGED. The DMCONT continuation still has not reached app-start 0x2bd2058.
- **The GOT/canary hypothesis below is REFUTED by the register dump (SH349+1, this cycle):** at the
  new terminal 0x101d9a708, x20 = 0x56543aa2fbf8 = the plt-PATCHED valid canary address (both GOT
  slots 0x631aa30 AND 0x67d16f0 were patched to 0x56543aa2fc00, log line `[plt] patched
  __stack_chk_guard GOT 0x67d16f0 (0x0) -> 0x56543aa2fc00`). The fault is a BYTE-LOAD from
  x19=0xffffffffffffffff (the CALLER's source pointer = garbage 0xff..ff), NOT a canary read
  failure. So 0x101d9a708 is one more node in the SAME unconstructed-live-object family, not a
  loader/relocation gap.
- **The LSM sub-call-whack-a-mole is UNBOUNDED (measured):** `bl 0x1d9d8b0` (initStorageManagerNative)
  has HUNDREDS of call sites across the whole binary — it is the most-called function in
  libroblox.so, invoked from every app-start/session path. Ret-ing its internal byte-copy leaves
  one-by-one reveals the next unconstructed-field deref of hundreds; this specific single-skip
  lane is a measured dead-end for "skip the next leaf" granularity.
- **CORRECTION to the escape-latch note below (verified before implementing, this cycle):** the
  `[0x683d920]` latch at 0x2bd1fe4/0x2bd1fe8 is the **"app-start already ran" re-entry latch**, NOT
  a bypass to app-start. `tbnz w8,#0, 0x2bd2080` jumps to 0x2bd2080, which is PAST the app-start
  `bl 2338ef4` (0x2bd2058); and [0x683d920] is set to 1 only at 0x2bd2060, AFTER app-start returns.
  So on the FIRST DMCONT pass the latch is 0 and the continuation MUST run both `bl 1d9d8b0`
  calls to populate the app-start name strings before reaching app-start. Seeding the latch to 1
  would SKIP app-start, not reach it (the existing JIT_ROUTEB_DMCONT seed of [0x683d920]=0 is
  correct). **Conclusion: the DMCONT→app-start path has NO clean bypass; the LSM live-object family
  is mandatory and unbounded via sub-call skips. This closes the "single-seed escape" angle.**
- **NEW DISCOVERY (SH349+1, verified):** the DMCONT continuation (NativeDataModelManager
  continuation, file 0x2bd1d68..0x2bd2080) calls initStorageManagerNative via TWO `bl 1d9d8b0`
  sites (0x2bd2008 for the [x19+80] string, 0x2bd2030 for the [x19+128] string) immediately
  before the app-start `bl 2338ef4` (0x2bd2058). Both are guarded by libc++-style SSO byte checks
  (0x2bd1fec bit0 -> populate; else empty-string copy), so the first-pass strings come from LSM.
- Recon-v3 self-driven frame plane unchanged (re-verified green in the run env).

## Verify
Workspace green (cargo build + cargo test --workspace exit 0; elfjit example 157/0). A/B repro
above (3/3). Repro: runs/capture_sh349_lsm_append_skip.sh.