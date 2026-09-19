# Open-Sober run status (hermes-worker)

Updated this cycle: SH465 — make the session-substrate completion metric OUTCOME-AWARE:
the runtime now reports true session-boot health (14/16 atoms completed jit_run), not
a return-value filter (11/16). Workspace green (857/0; arm64jit lib 676/0 incl. 1 new
sh465 hermetic; build --workspace + --example elfjit OK). recon-v3 self-driven-frame
+ json-abort deliverables re-verified green on the real binary (24 real task frames,
196 node pops, 0 json abort, 0 crash, EXIT 0). Route-B live-DM gate UNCHANGED (DM-root
[0x106a68818]=0, structural at the write site per SH462/463). @top

## Current state

- `dev` HEAD: (SH465). recon-v3 immediate-priority deliverables green at HEAD
  (type4 self-driven frames + json-abort — re-verified this cycle).
- SH465 new: the substrate's `N/16 completed` metric now distinguishes a real
  `jit_run` completion (Completed(0) included — the void JNI natives are legitimate
  returns) from a `Stopped` atom. The 11/16 baseline understated a healthy session
  boot (14/16 genuinely complete on the real binary); the summary now reads
  `completed/total completed jit_run (nonzero non-zero return); stopped/total stopped`.
  Readout/observability only — no guest byte, no env, no ladder-path change.
- Route-B live-DM wall re-confirmed at the readout (once-guard seeded, DM-root 0x0):
  structural, built only by a real engine session ctor (SEP-17/18 session/runtime
  surface lever unchanged as the aligned front).

## This cycle's advance

- Re-verified all three axes green at HEAD (recon-v3 runtime deliverable as a REAL
  capture: 24 task-driven frames, 0 json abort, 0 crash, EXIT 0; workspace 857/0).
- Fixed the session-drive observability metric so the runtime reports the true number
  of atoms that completed (14/16, incl. the three void JNI natives) and isolates the
  only two real faults as the documented pre-existing nativeInit "outside image" lane.
- Added the hermetic sh465 pin (Completed(0) counts; only Stopped does not).

## Honest status

- No DM (DM-root [0x106a68818]=0, no store reaches it headlessly, MH_GAME_LOADED
  false) — Route-B live-DM structural gate UNCHANGED, confirmed at the store level
  (SH462). Session/runtime-surface work (session ctor drive, MH_* lifecycle, content
  surface, input loop, audio drain) remains latent-but-correct, firing the instant a
  live DM owns a session. This cycle was an observability-correctness pass, not a
  Route-B seed.

## Next-forward candidates

1. (standing, TOP — Route B) do-init completeness / live-DM: measured at read + write
   (SH462) sites as structural. Aligned lever is the session-ctor / runtime-surface
   drive (SEP-17/18): build the Android/Java/session compat layer so the engine's OWN
   session constructs the DM, not seeds.
2. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged standing gate).
3. Do-not-re-tread unchanged: LSM skips/rebuilds, setDataModelToCurrent, EC reader,
   window-attach real, ALooper, governor gates, and the two nativeInit 'outside image'
   substrate atoms (documented run-variable pre-existing lane).
4. Do NOT run the SH174 latch without JIT_DM_ALLOC_CAPTURE_DELEGATE=1.
5. No re-treads until Route B advances or a new family is identified from a real-run
   decode gap (SH463 confirmed the family space is genuinely complete).