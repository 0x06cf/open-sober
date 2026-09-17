# Frontier SH272 — nativePreloadFlagOverrides: BOTH getter branches structurally dead (mechanized SH270 residual)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Route-B live-DM
structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
Pure regression pinning + mechanism closure; no production path edited,
default-inert. +1 hermetic sh272. Workspace green.

## Why (SH270 left a residual open)
SH270 correctly pinned the SendAppEventOnAppReady post-advance terminal
guestpc=0x102bb803c (`ldr x8,[x20]`) as x20 = nativePreloadFlagOverrides return
(bl 0x102bb801c -> thunk 0x102dae640 -> entry 0x2dae5f0), and MEASURED that
wiring the engine's own constructed singleton base into the value cells
[0x106a64d78]/[0x106a64d98] does NOT move the wall. But SH270 left it as an
empirical negative ("the getter's load path ignores those cells / re-reads
elsewhere") without the structural WHY. This cycle disassembles the full getter
dual-path and pins the terminal words so a future cycle does not re-attack either
branch as a one-store seed.

## Measured (fresh disasm, real libroblox.so, guest = file vaddr + 0x100000000)

The getter entry 0x2dae5f0:
  bl 0x1057816f0          @0x2dae5f8  guard-acquire helper (Meyers lazy once)
  tbz w0,#0 -> 0x2dae624              @0x2dae5fc  select construct vs value-cell

**Value-cell branch** (guard helper returned bit0 SET, falls through):
  0x2dae600 adrp x8,0x6a64000
  0x2dae604 ldr x0,[x8,#3448]        x0 = [0x106a64d78]
  0x2dae608 cbz x0 -> 0x2dae638 (ret)   // returns 0 when cell empty
  0x2dae60c ldr x8,[x0]
  0x2dae610 ldr x2,[x8,#16]          vt[+16]
  0x2dae620 br x2                    // a VTABLE DISPATCH, not a soft-return cell
-> This is NOT a "parcel the pointer back" read; it dispatches the object's
   vt[+16] method. It needs a REAL preload-overrides object with a functioning
   vt[+16] target. A host wire of the object base would cbz (if first word null)
   or br into garbage. SH270's wire was inert precisely because this branch is a
   live-object dispatch, not a value read.

**Construct branch** (tbz taken -> 0x2dae624, bl ctor 0x101df8ff8, then b tail
helper 0x2daf5c8):
  ctor 0x1df8ff8 zero-INITS the singleton:
    0x1df9058 stp x0,xzr,[x19,#72]   // [obj+72]=allocation, [obj+80]=xzr
    ... bl 284cf5c ... b 0x1df9014 -> ret obj base 0x106d2dd20
  tail helper 0x2daf5c8:
    0x2daf5ec ldr x0,[x19,#80]       // reads [obj+80] as its null-flag
    0x2daf5f0 cbz x0 -> 0x2daf608    // [obj+80]==0 after a headless construct
    0x2daf608 mov x20,xzr; ... ret   // => returns 0 ALWAYS
-> The construct branch is equally a live-object wall: the ctor deliberately
   leaves [obj+80] zero (a real session later populates it / a nested object is
   built by the vt[+16] path), so the helper short-circuits to 0 every headless
   run. Not a one-store seed either.

Guard helper 0x57816f0's own Meyers once byte: adrp x8,0x6d2d000 + add #0xf30
= [0x6d2df30] (0x57816fc/0x5781700) — a distinct cell from the 0x106a64d78
value cell SH270 wired.

## Verdict (do-not-re-tread either branch)
guestpc=0x102bb803c is the SH174/SH204 live-object class: the getter needs the
REAL session to construct AND populate the object (its vt[+16] dispatch target +
non-null [obj+80]). Neither a value-cell wire (SH270) nor a construct-path drive
can produce that headlessly. Route-B live-DM structural gate UNCHANGED; SH174
capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the
single forward hook.

## Honest (do-not-over-claim)
Does NOT manufacture a DataModel; DM-root stays 0; MH_* stay false. New+measured
this cycle: the structural WHY both getter branches yield 0 (value = vtable
dispatch, construct ctor zeroes [obj+80]) at the byte level — converting SH270's
empirical "inert" into a mechanism + durable pins.

## Code / verify / artefacts
- elfjit.rs hermetic `sh272_preload_getter_both_branches_structurally_dead_pinned`
  (real-image guard, skip-if-absent): 10 anchors — value-cell ldr/cbz/ldr-vt/
  br-vt16 (0x102dae604/608/60c/610/620), construct ctor zero-store (0x101df9058)
  + sh270 construct bl (0x102dae624), helper null-flag ldr/cbz (0x102daf5ec/5f0),
  guard helper adrp/add (0x1057816fc/0x105781700). 4-aligned + in-window asserts.
- Verify: `cargo test -p arm64jit --example elfjit sh272` = 1 passed (real-image
  anchors asserted, pin printed on libroblox.so). `cargo test -p arm64jit --examples`
  = **106/0** (was 105). `cargo test --workspace` green.
- recon-v3 plane re-verified at HEAD: runs/capture_taskv4_frame.sh = 24 task-driven
  frames present #19..#23 swap Ok(0x1), 193 node pops, dispatch #2652000, 0 json
  abort, 0 SIGSEGV/SIGABRT, EXIT 124.
- Repro (pre-existing SH269 SendAppEvent terminal) runs/sh272-repro.txt = EXIT 134
  at guestpc=0x102bb803c (baseline parity), eligible for current state.
- Commit: local `dev` only.