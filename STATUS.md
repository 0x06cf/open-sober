# Open-Sober run status (hermes-worker)

Updated 2026-09-19/20, this session: SH367 = MEASURED NEGATIVE — arming the real window-attach
GL-surface path faults (SH366 next-forward executed); the SH366 clean INIT_WINDOW drive is
preserved and the fault is pinned one level deeper into the deep GL post-init 0x22985c0.
Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false).

## Current state

- `dev` HEAD (local): SH367 (window-attach real-path measured negative; SH366 clean drive preserved).
- Workspace green (cargo test --workspace EXIT 0, 610 passed/0 failed; arm64jit lib 430).
- Route-B live-DM structural gate UNCHANGED: DM-root [0x106a68818]=0x0, MH_* all false.
- recon-v3 immediate-priority deliverables CONFIRMED green at HEAD.

## What advanced this session

- **SH367**: executed + MEASURED the SH366 next-forward (hand a REAL wired ANativeWindow so
  window-attach 0x2bd29a0 takes its real GL-surface path). Arming the once-guard
  ([win+0x268].bit0=1) + crafting [win+0x278] + registering XID=0x200000 made the run HARD-FAULT
  (8/8 EXIT 134/139, marker never set, fault guestpc=0x7f0000001f50 fault=0x7f818c0097) because the
  deep GL post-init 0x22985c0 needs a REAL EGL surface/context — a fabricated obj cannot satisfy it.
  **Reverted** to the SH366 clean drive; re-verified clean (process_cmd Ok, [inner+9] marker set,
  artifact attempt 1: EXIT 124 crash 0). Added read-only routeb_glue_realattach_guard + sh367
  hermetic (arm64jit lib 430). This CONFIRMS the window precondition is a real GL-surface wall on
  the SESSION-CTOR line, not a seedable global.

## Honest status

- Route-B live-DM structural gate UNCHANGED. Do-init still never owns a live DataModel
  (DM-root 0, MH_* false). SH174 capture-latch stays the single forward observer.
- SH367 does not manufacture a DM; it closes the \"arm the once-guard to force the real
  window-attach path\" candidate with evidence and pins the real GL-surface completion as a
  genuine Session-Ctor wall.

## Next-forward candidates

1. (PRIMARY, standing) The SESSION half remains THE wall: do-init must own a live DM (SH184/185
   four-stacked closure). Persistence lane measured unbounded (SH349/350/358), dispatch body
   unreachable (SH362), receive cb never entered (SH347/364), app-command drain dead (SH365),
   real window-attach GL completion faults on a fabricated obj (SH367) — all point to a REAL
   Android Activity/AppBridge session drive with a genuine EGL surface + onAppReady + real jstring,
   per the operator's SESSION-CTOR directive.
2. The window-attach COMPLETION (0x22985c0 deep GL post-init) needs a genuine EGL surface/context —
   wire a real ANativeWindow/surface into the session drive (not a fabricated obj; SH367).
3. R1 content half is staged + armed + serviceable (SH351/352/354); fires the moment a live DM
   drives the loader.
4. Do NOT re-arm the window-attach once-guard (SH367 measured fault); do NOT re-enter the ALooper
   glue LOOP (SH365); bounded process_cmd remains the guarded entry (SH366); do NOT re-drive LSM
   sub-call skips (SH349/350/358); do NOT seed [0x106a64da0] (SH362); do NOT re-attack EC
   reader-gate (SH356); do NOT seed [0x107275550] (SH359).