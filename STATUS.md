# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this cycle: SH371 = MEASURED forward — the DM-creator continuation
continueAfterFlagsLoaded_ (0x102bd1d68) now EXECUTES DEEP headlessly (25+ block-entry pcs
through its app-name guard) with the full Route-B env, overturning the SH226/228 "never
fires" map; the engine-init dispatcher body is hermetic-proven STRAIGHT-LINE (diverge can
only be a leaf return / host-landing, no benign body branch). recon-v3 immediate-priority
deliverables independently re-verified green. Route-B live-DM structural gate UNCHANGED
(DM-root 0, MH_* false).

## Current state

- `dev` HEAD (local): SH371 (jit.rs sh371 hermetic + continuation-executes measurement).
- Workspace green (cargo test --workspace EXIT 0; elfjit 159/0, arm64jit lib 433/0).
- recon-v3 immediate-priority deliverables CONFIRMED green this cycle (capture_taskv4_frame.sh
  attempt 1: 24 real task-driven frames swap Ok(0x1), 197 node pops, 0 json abort, 0 crash;
  JIT_JSON_ZERO_FIX len-clamp at 0x102355d40 present).
- Route-B live-DM structural gate UNCHANGED: DM-root [0x106a68818]=0x0, MH_* all false.

## What advanced this cycle

- **SH371 (measured forward, map-correction)**: region-watching the engine-init dispatcher +
  continueAfterFlagsLoaded_ with the full Route-B seed env (capture_sh344's DMCONT +
  DM_CONT_M48_SEED + CONT_APPNAME_SEED) shows continueAfterFlagsLoaded_ (0x102bd1d68) now
  FIRES and runs deep (0x102bd1d68 .. 0x102bd1f64 app-name guard, SH245/SH248c-seeded) —
  previously recorded as "never entered" (SH226/SH228). It then terminals at the standing
  SH285 persistence-lane wall (guestpc=0x101db1b08), one fencepost before the F+0x18
  controller floor the routeb_dm_manager_cont comment predicts.
- New real-image hermetic `sh371_engineinit_dispatcher_body_straightline_to_sub` (jit.rs):
  scans the dispatcher body [0x2bd8ce8,0x2bd8d64) + sub_2bd8dac body [0x2bd8dac,0x2bd8e28)
  for any control-flow word, rejecting all but the 4 known sites (bl getter, blr vt+0xf8,
  blr vt+0x108, bl sub) + sub's blr vt+0x1f0 — both bodies STRAIGHT-LINE, so SH228's
  "diverge at a leaf" narrows to "a leaf's return never lands back in-image (host landing)".
- Probe runs/capture_sh371_dispatcher_body.sh + docs/frontier-sh371-....md added.

## Honest status

- Route-B live-DM structural gate UNCHANGED. Do-init still never owns a live DataModel
  (DM-root 0, MH_* false). SH174 capture-latch stays the single forward observer.
- SH371 corrects the map (the continuation is env-reachable deep, not "never entered") but
  the continuation dives into the measured-closed SH285 persistence lane. It does NOT
  manufacture a DataModel.

## Next-forward candidates

1. (PRIMARY, standing) The SESSION half remains THE wall: do-init must own a live DM (SH184/185
   four-stacked closure). The two measured dead-ends from the now-reached continuation are the
   SH285 live-object wall and the F+0x18 controller floor behind it — both measured-closed
   (SH349/350; do NOT re-drive LSM sub-call skips).
2. R1 content half is staged + armed + serviceable (SH351/352/354); fires the moment a live DM
   drives the loader. Window-attach completion 0x22985c0 funnels into the same settings-state/LSM
   lane (SH369) — not a DM route.
3. Do NOT re-arm the window-attach once-guard (SH367); do NOT re-enter the ALooper glue loop
   (SH365); bounded process_cmd is the guarded entry (SH366/SH368); do NOT re-drive LSM
   sub-call skips (SH349/350/358); do NOT seed [0x106a64da0] (SH362); do NOT re-attack EC
   reader-gate (SH356); do NOT seed [0x107275550] (SH359).