# Frontier SH348 — measured negative: leaf-`ret`ing LocalStorageManager::initStorageManagerNative does NOT clear the SH285 terminal (the crash reaches 0x101db1b08 by a path the entry patch cannot stop)

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert production patch
(opt-in `JIT_ROUTEB_LSM_INIT_SKIP=1`) + +1 sh348 hermetic (real-image pins). Workspace green
(cargo test --workspace exit 0, arm64jit 416/0 + 1 new sh348). elfjit.rs held < 1MB hook.

## Why (a genuinely-new cause-level leg, not a re-run)
Every prior Session-CTOR cycle peaked at the SH285 persistence-lane live-object wall: a byte-copy
inside `initStorageManagerNative` writes through an unconstructed string buffer
`[obj+0x50]=0xff..ff` at guest 0x101db1b08 (SH285/286 classification), reached deep inside the
function on the DMCONT/StartLuaAppDM drive, one hop before the SH344c app-start bl 0x2bd2058.
SH344b measured the app-shell ctor band (0x102207b50..0x102209000) at **0 hits**. The frontier
said "do NOT re-drive a repair seed into [obj+0x50]" (SH248h/SH256 trap). This cycle tried a
DIFFERENT action, consistent with SH117/SH93 line-cross: make the whole persistence-lane init a
clean no-op by leaf-`ret`ing its entry, so the Session-CTOR continuation is not required to have
the live LSM object at all.

## What SH348 does (default-inert)
`routeb_patch_lsm_init_skip()` (elfjit.rs, env-gated): mprotect-RW, rewrite initStorageManagerNative
entry 0x101d9d8b0 prologue `0xa9bd7bfd` (stp x29,x30,[sp,#-0x30]!) -> `0xd65f03c0` (ret), restore
RWX, drop the whole function body + sh324 caller from the block cache so no pre-patch cached interior
block (incl. the deep byte-copy at 0x101db1b08) is reachable. +hermetic `sh348` pinning the entry
prologue, the sh324 caller bl (0x102256608=0x97ed1caa), the SH285 terminal site (0x101db1b08=0xd10083a2),
and the app-start bl region (0x102bd2058 b-format).

## MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES, 3 runs)
```
SH348 leaf-`ret` initStorageManagerNative @0x101d9d8b0 (a9bd7bfd -> d65f03c0)   <- patch fires
SIGSEGV guestpc=0x101db1b08 fault=0xffffffffffffffff                            <- SH285 terminal PERSISTS 3/3
[SIGABRT] guestpc=0x0 (host libc++ bad_function_call / abort)                   <- Arm A/C as before
app-shell ctor band 0x102207b50: 79 hits AT THE SINGLE ENTRY PC 0x102207b50     <- FastLog warmer (SH344c), NOT ctor body
```
Even after widening the block-cache drop to the FULL function body [0x101d9d000..0x101dbe000], the
terminal stays `guestpc=0x101db1b08` (1 run, EXIT 139 SIGSEGV direct). The 79 hits at the one address
0x102207b50 are the __cxa_guard acquisition loop (the SH344c FastLog warmer), confirmed by the deep
pcs 0x102208e8c/0x102208eac destructor-loop — NOT genuine app-shell construction.

## Honest conclusion (do-not-over-claim)
SH348 is a MEASURED NEGATIVE that closes the "skip initStorageManagerNative to cross the SH285 lane"
candidate: the entry-ret does not move the terminal. The crash reaches 0x101db1b08 through a call
path that the entry patch cannot stop (either the deep byte-copy is inside a DIFFERENT function whose
entry is past 0x101d9d8b0, or a caller block jumps to the interior directly). This REFINES the SH285
attribution: initStorageManagerNative's entry is not the only gate to the fault site, so a storage-
skip must target the specific failing sub-call, not the whole init. The app-shell band "firing" is
the known FastLog warmer, consistent with SH344c — not a Route-B advance. Route-B live-DM structural
gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward
hook. The sh348 pins are a real-image regression anchor; the guard is default-inert.

## Verify
- `cargo build --example elfjit` EXIT 0; `cargo test --workspace` EXIT 0 (arm64jit 416/0 + sh348);
  `cargo test -p arm64jit --example elfjit sh348` OK (real-image pins pass on libroblox.so).
- Repro: `bash runs/capture_sh348_lsm_init_skip.sh` -> SH348 patch line, SH285 terminal persists,
  app-shell band = 79 hits at single entry pc (warmer). Logs runs/sh348-runN.txt (gitignored).
- elfjit.rs 1,044,985 B (under the 1MB pre-commit hook).

## Files
- `crates/arm64jit/examples/elfjit.rs`: +`routeb_patch_lsm_init_skip` (env-gated, default-inert) +
  call in --v2boot + hermetic `sh348`.
- `runs/capture_sh348_lsm_init_skip.sh` (new repro).
- Frontier doc: docs/frontier-sh348-lsm-init-skip.md.