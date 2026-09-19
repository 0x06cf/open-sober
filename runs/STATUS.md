# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this session: SH372 = MEASURED — the DM-creator continuation
continueAfterFlagsLoaded_ (0x102bd1d68, SH371 runs deep) and the settings-state self-drive
(SH284/285) CONVERGE on the identical SH285 persistence-object leaf (pc 0x101db1b08, lr
0x101db1b18, 0xff..ff internal buffer pointer), answering SH371's "same object or different?"
gap with a fresh register dump: SAME object, path-independent. recon-v3 deliverables
re-verified green. Workspace green (614/0).

## Current state

- `dev` HEAD: SH372 (jit.rs sh372 convergence hermetic, arm64jit lib 434/0 + probe + docs).
- Workspace green (cargo test --workspace EXIT 0, 614 passed/0 failed).
- recon-v3 deliverables green (24 task-driven frames swap Ok(0x1), 0 json abort, 0 crash).
- Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0, MH_* all false.

## What advanced this session

- SH371 (prior commit): continueAfterFlagsLoaded_ now EXECUTES DEEP headlessly (overturns the
  SH226/228 "never fires" map); engine-init dispatcher body proven STRAIGHT-LINE.
- SH372 (this commit): fresh full register + guest-stack dump from the continuation path
  proves both engine init paths terminate at the SAME SH285 live-object leaf — same pc
  0x101db1b08, same lr 0x101db1b18, same 0xff..ff uninitialized internal data-pointer. The
  SH285 terminal is PATH-INDEPENDENT (not a benign-body branch one path misses), i.e. the
  object only a real LocalStorageManager/session ctor owns (SH174/204 class). New hermetic
  sh372 pins the shared leaf + caller bl + continuation prologue. No production path edited.

## Honest status

- Route-B live-DM structural gate UNCHANGED. SH174 capture-latch stays the single forward
  hook. SH372 strengthens the measured-closed SH285 record with a second-entry confirmation.
- R1 content half staged+armed+serviceable (SH351/352/354); latent until a live DM drives the
  loader.

## Next-forward candidates

1. (PRIMARY, Route-B) The SESSION half remains THE wall: do-init must own a live DataModel
   (SH184/185). The two measured dead-ends from the now-reached continuation are the SH285
   live-object wall and the F+0x18 controller floor — both measured-closed.
2. Do NOT re-drive LSM sub-call skips (SH349/350/358); do NOT re-arm window-attach once-guard
   (SH367); do NOT re-enter ALooper loop (SH365); at this point any further Route-B seed is a
   stopgap vs. the SESSION-CTOR cause-lever the operator names.