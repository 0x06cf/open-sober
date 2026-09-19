# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this cycle: SH369 = MEASURED structural pin that window-attach COMPLETION
funnels into the CLOSED persistence lane (flags-latch 0x72739d4 -> initStorageManagerNative
0x1db1050), NOT to a live DM — refining SH367's "needs a real surface" reading. recon-v3
immediate-priority deliverables re-verified green at HEAD (24 real task-driven frames, swap
Ok(0x1), 0 json abort, 0 crash). Workspace green (612 passed/0 failed).

## Current state

- `dev` HEAD (local): SH369 (window-attach-completion-persistence-convergence pin; arm64jit lib 432).
- Workspace green (cargo test --workspace EXIT 0, 612 passed/0 failed; arm64jit 432/0).
- recon-v3 immediate-priority deliverables CONFIRMED green this cycle (capture_taskv4_frame.sh, EXIT 0).
- Route-B live-DM structural gate UNCHANGED: DM-root [0x106a68818]=0x0, MH_* all false.

## What advanced this cycle

- **SH369**: pinned the SH367 window-attach completion chain that SH367 stopped at bl 0x22985c0.
  New real-image hermetic (11 word-pins) shows `0x22985c0 -> 0x2270a98 -> 0x2270b24` gates on the
  SAME flags-loaded latch the ladder seeds ([0x72739d4]) and, when set, calls
  initStorageManagerNative 0x1db1050 — the SH285-family persistence lane closed at SH349/350/358.
  So even a REAL EGL surface at window-attach does NOT yield a live DM: completion re-enters a
  measured-unbounded lane. De-risks the SESSION-CTOR window precondition (it cannot be the unblock
  by itself) and keeps Route-B's live-DM gate as the only structural forward.

## Honest status

- Route-B live-DM structural gate UNCHANGED. Do-init still never owns a live DataModel
  (DM-root 0, MH_* false). SH174 capture-latch stays the single forward observer.
- SH369 does not manufacture a DM; it is a disasm-verified pin + a re-verify of recon-v3.

## Next-forward candidates

1. (PRIMARY, standing) The SESSION half remains THE wall: do-init must own a live DM (SH184/185
   four-stacked closure). Reach it only via a real Android Activity/AppBridge session drive
   (genuine EGL surface + onAppReady + real jstring) so the upstream ctor RUNS — per the operator's
   SESSION-CTOR directive. The window-attach completion is now pinned as persistence-lane-bound
   (SH369), so the lever is the live session drive / governor / onAppReady home-cone (SH126+), not
   the window precondition alone.
2. R1 content half is staged + armed + serviceable (SH351/352/354); fires the moment a live DM
   drives the loader.
3. Do NOT re-arm the window-attach once-guard with a fabricated surface (SH367); do NOT re-enter
   the ALooper glue LOOP (SH365); bounded process_cmd is the guarded entry (SH366/SH368); do NOT
   re-drive LSM sub-call skips (SH349/350/358); do NOT seed [0x106a64da0] (SH362); do NOT re-attack
   EC reader-gate (SH356); do NOT seed [0x107275550] (SH359).