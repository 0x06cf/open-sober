# SH225 — do-init DM-construction dispatch fork RESOLVED to a pinned contract (single-agent Route-B re-attack)

Status: implemented + verified on the real binary (+1 hermetic sh225, elfjit example
73/0). Frontier: Route-B. No production path edited — the SH186 "judged same-difficulty,
never built out" fork is now byte-anchored so the next drive (or a proof-of-dead-end)
starts from a concrete dispatch contract.

## Why

SH186 recon task-0 (frontier-sh186) mapped the ONE reachable DM-touching path —
`nativeAppBridgeStartLuaAppDM` (0x1023efe2c) -> dispatcher 0x102baeeec -> GlobalInit
do-init (0x102206c40) — and concluded that "DM construction is buried inside a
scheduled app-start; the do-init path builds a std::function closure and blrs through
a captured vtable; no entry point makes a DM in isolation." It then JUDGED (not
measured) that fabricating the binder + scheduler runnable and driving the app-start
was "same-difficulty as the static-seed route" and never built it out. The operator's
doctrine demands converting that *judgment* into either a concrete fabrication recipe
or a measured proof-of-dead-end. This cycle disassembles the fork to ground and pins
the dispatch contract.

## Method

Fresh disasm of the real libroblox.so (`crates/arm64jit/examples/dm_construction_fork.rs`:
load_elf_image + decode, with resolved branch targets). guest = file vaddr + 0x100000000.

## Measured (real libroblox.so, fresh decode)

### do-init 0x102206c40 (GlobalInit do-init)
- 0x102206c78 `add x8, <page>, #0x410` -> **the once-guard `[0x106a68410]`**
- 0x102206c7c `ldar w9,[x8]` (acquire-load the once-guard)
- 0x102206c84 `tbz w9,#0, ->0x102206d10` : **if bit0 clear (first call), take the init path.**
- init path 0x102206d10: 0x102206d18 `bl ->0x10284ce54` = **the `__call_once_impl` SH196
  measured to self-latch to a strcmp string-intern (0x400000b), NOT a DM**; then
  0x102206d20 `cbz w0,-152 ->0x206c88` loops while NOT-done; then builds the
  registry-key strings (0x102206d24..).
- after the once completes, do-init calls the closure-build:
  - 0x102206cdc `bl ->0x102206db8` (the closure build)
  - 0x102206ce4 `bl ->0x10221942c` (a second short fork: stp/[sp,#-16]! prologue,
    calls a helper, `ret`).

### closure-build / match dispatch 0x102206db8
- 0x102206de8 `bl ->0x1062d62b0` (a helper)
- 0x102206df4 `ldr x0,[x19,#4]` — **the app-bridge/binder object** (from the callee-saved
  x19 threaded in from the dispatcher chain)
- 0x102206df8 `cbz x0, ->0x102206ea4` (NULL binder => skip to epilogue = the current
  headless benign soft-return `Ok(0x3e8)`)
- 0x102206dfc `ldr x8,[x0]` — x8 = *x0 = the binder object's **vtable**
- 0x102206e00 `ldr x1,[x8,#0x30]` — x1 = **vt+0x30 slot**
- 0x102206e24 `br x1` — **dispatch to vt+0x30 of the binder object at [x19+4]**

## The pinned contract (new vs SH186's prose)

**`GlobalInit do-init`'s first-call path runs `__call_once` and then dispatches the
DM-construction entry via `br x1` where `x1 = vt+0x30` of the app-bridge/binder object
at `[x19+4]`.** That vtable slot is the exact "scheduled app-start" entry a fabricated
binder must satisfy to make do-init advance past the benign `Ok(0x3e8)` soft-return.

Note (reconciliation with SH224): SH224 showed the *DM object's own* genuine vtable's
vt+0x30 = 0x1057d1b9c (a tiny accessor). This do-init dispatch is the *binder /
app-bridge object's* vtable — a DIFFERENT class — so it is not the DM's own vt+0x30.
They must not be conflated.

## Honest boundary (do-not-over-claim)

This cycle is recon + a regression pin. It does NOT manufacture a binder, does NOT
drive do-init further, and does NOT produce a DataModel. It turns SH186's "judged
same-difficulty" fork into a concrete, byte-anchored dispatch contract (binder object
[@x19+4]'s vt+0x30) so the NEXT Route-B drive can target a real object instead of
re-arguing reachability. Route-B live-DM structural gate UNCHANGED (SH209/218/223).

## Code

- `crates/arm64jit/examples/elfjit.rs` hermetic `sh225_doinit_dm_construction_dispatch_fork_pinned`
  (real-image guard family as sh224/sh223/sh222/sh219/sh213): byte-pins the 9 dispatch
  words (do-init prologue, once-guard acquire+tbz, both `bl` fork call sites, the
  `[x19+4]`-load / `[x0]`-vtable / `vt+0x30`-slot / `br x1` dispatch), the once-guard
  8-alignment, the resolved `__call_once` target (0x10284ce54) from the init-path `bl`,
  and the fork-0x10221942c prologue.
- `crates/arm64jit/examples/dm_construction_fork.rs` (repro tool; prints resolved disasm).

## Verify
- `cargo test --example elfjit` = **73 passed / 0 failed** (was 72; +1 sh225).
- `cargo test --workspace` green.