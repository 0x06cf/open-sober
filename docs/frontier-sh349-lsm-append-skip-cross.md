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
- The new terminal 0x101d9a708 is another member of the SH285-class family, but its pin is
  DIFFERENT and potentially loader-fixable: a `.got` entry (0x67d16f0) whose relocated value is
  garbage. If [0x1067d16f0] should hold &__stack_chk_guard and the loader/JIT skipped relocating
  the final GOT slot, that is a relocation/dispatch gap (a candidate for real loader work, the
  kind of fix the migration-gate conscience demands hunting before declaring a dead end) — OR
  it's a per-call live-object wall. Both hypotheses are measurable (seed [0x1067d16f0] to a valid
  host canary and re-run; if the ladder advances, the GOT was the lever; if a new wall appears,
  it was one node in the live-object family).
- Recon-v3 self-driven frame plane unchanged (re-verified green in the run env).

## Verify
Workspace green (cargo build + cargo test --workspace exit 0; elfjit example 157/0). A/B repro
above (3/3). Repro: runs/capture_sh349_lsm_append_skip.sh.