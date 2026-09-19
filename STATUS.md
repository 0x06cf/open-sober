# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this cycle: SH368 = bounded app-command SEQUENCE drive on the guarded SH366
entry — the engine's own process_cmd (0x102bcd6e4) drives cmds {6,8,11} cleanly (INIT_WINDOW
marker fires), session observables measured after each confirm the real dispatcher alone does NOT
self-transition AppBridgeV2/surface; full 20-entry jump table + window-attach contract pinned in a
new real-image hermetic. Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false).

## Current state

- `dev` HEAD (local): SH368 (bounded app-command sequence drive + jump-table/window-contract pins).
- Workspace green (cargo test --workspace EXIT 0, 611 passed/0 failed; arm64jit lib 431).
- Route-B live-DM structural gate UNCHANGED: DM-root [0x106a68818]=0x0, MH_* all false.
- recon-v3 immediate-priority deliverables CONFIRMED green at HEAD.

## What advanced this cycle

- **SH368**: extended the SH366 confirmed-green bounded process_cmd entry to a command SEQUENCE
  (verified-safe {6,8,11}) over a shared fabricated app/inner/win, driving the engine's REAL
  Activity-session init state machine in the operator's SESSION-CTOR sense. MEASURED (attempt 1,
  EXIT 124, 0 crash): all 3 commands return Ok, cmd 11 fires INIT_WINDOW marker [inner+9]=1,
  once-guard stays OFF. Observable readback isolates the wall precisely: the real dispatcher alone
  does NOT self-transition AppBridgeV2 ([0x106a705e8] 0x0->0) nor the surface XID
  ([0x10683d348] stays 0x200000 = wired X11 XID, SH112) — those move only when a live
  session/do-init builds the DM world. +full 20-entry jump table + window-attach contract pinned in
  `sh368` (arm64jit lib 431) + capture + frontier doc.

## Honest status

- Route-B live-DM structural gate UNCHANGED. Do-init still never owns a live DataModel
  (DM-root 0, MH_* false). SH174 capture-latch stays the single forward observer.
- SH368 does not manufacture a DM; it advances the "drive the engine's real command queue" half of
  the SESSION-CTOR directive with pinned addresses + a bounded live readback.

## Next-forward candidates

1. (PRIMARY, standing) The SESSION half remains THE wall: do-init must own a live DM (SH184/185
   four-stacked closure). Reach it only via a REAL Android Activity/AppBridge session drive —
   genuine EGL surface (NOT a fabricated obj; SH367 measured fault) + onAppReady + real jstring —
   so the upstream ctor RUNS and constructs the DM world for real, per the operator's SESSION-CTOR
   directive. Window-attach completion 0x22985c0 converges into the same settings-state/LSM lane.
2. R1 content half is staged + armed + serviceable (SH351/352/354); fires the moment a live DM
   drives the loader.
3. Do NOT re-arm the window-attach once-guard with a fabricated surface (SH367); do NOT re-enter
   the ALooper glue LOOP (SH365); bounded process_cmd is the guarded entry (SH366/SH368); do NOT
   re-drive LSM sub-call skips (SH349/350/358); do NOT seed [0x106a64da0] (SH362); do NOT re-attack
   EC reader-gate (SH356); do NOT seed [0x107275550] (SH359).