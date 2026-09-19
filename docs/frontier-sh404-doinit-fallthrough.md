# Frontier SH404 — the do-init FALL-THROUGH arm (app-shell construction) runs deep; the main dispatch `br x1` is bypassed
# (do-init world-build; next-gate after SH403)

Date: 2026-09-20, hermes-worker, single-agent. Workspace green before/after
(`cargo test --workspace` EXIT 0, 633 passed/0 failed, arm64jit lib 453; elfjit.rs
untouched; jit.rs condensed to 221 B under the 1MiB hook).

## Why this cycle

SH403 measured the StartApp boot body -> app-bridge pipe 0x2baeeec -> do-init 0x102206c40 DEEP
body + post-do-init worker 0x1023eff4c all run headlessly, with the SH361 DM-ctor trace firing
(container+32 non-NULL -> vt[+48]=0x10258b5d8 @0x2206e24). Its frontier named the next step:
*"probe the do-init main-branch continuation past its fetched dispatch target — the run parks
EXIT 124 before entering the 0x10258b5d8 body, so where does control actually land?"* SH404
answers it.

## What SH404 measured (real libroblox.so, region-watch, 2/2 reproducible, EXIT 124, 0 crash)

- **The do-init MAIN dispatch `br x1` @0x2206e24 (target vt[+48]=0x10258b5d8) is BYPASSED** —
  0 block-entry hits at 0x2206e24. The SH361 dyn-trace READS the dispatch address from
  [container+32]/vt, but control never actually br's there.
- **The run takes the FALL-THROUGH arm**: 0x102206e30 `bl 0x102206ebc` -> 0x102206e34
  (operator-new, mov w0,#0x18; bl 0x101d96768) -> nested worker 0x102206fac (`sub sp,#0x60` =
  d10183ff).
- **That fall-through runs DEEP into the app-shell ctor band [0x102207000,0x102209000)** —
  25+ distinct block-entry pcs 0x102207c28..0x102207f58 execute (frame stp a9bd7bfd at
  0x102207df8), then the run parks cleanly (EXIT 124, 0 crash, no guestpc fault terminal).
- DMCONT 0x102bd1d68 = 0 hits (standing next gate); the 0x10258b5d8 dispatch BODY = 0 hits.

## Interpretation (refines SH362's root cause)

SH362 attributed "0x10258b5d8 body never enters" to a pre-body fault (e.g. the SH285 lane).
SH404 corrects the root cause: the do-init's MAIN dispatch branch is **not taken at all** —
the SH361 address is computed but the `br x1` never fires because the do-init body selects the
FALL-THROUGH (app-shell construction) arm instead. So the app-shell ctor band executes on the
do-init fall-through path and the DM world-build continues headlessly; the 0x10258b5d8 body is
unreachable not because of an earlier fault but because control flows elsewhere.

## Honest

- Does NOT manufacture a DataModel. DM-root [0x106a68818]=0, MH_* false, AppBridgeV2
  [0x106a705e8] = genuine vt 0x1063a3410 (unchanged). once-slot 0x400000b sentinel.
- This is a MEASURE (2/2) + a new hermetic. It does not build the DM, but it pins precisely
  WHERE the do-init world-build actually goes headlessly (fall-through -> app-shell band) and
  corrects the main-dispatch-bypassed attribution — the actionable map for the next gate.
- New real-image hermetic `sh404_doinit_fallthrough_appshell_band_reach_pinned` (arm64jit lib
  452->453) byte-pins: main dispatch `br x1` @0x2206e24 = d61f0020, fall-through nested worker
  0x102206fac = d10183ff, app-shell band frame 0x102207df8 = a9bd7bfd, dispatch body prologue
  0x10258b5d8 = d105c3ff (sub sp,#0x170).
- Captures: runs/capture_sh404_fallthrough.sh. Log (outside repo):
  /home/hermes-worker/runs/sh404-fallthrough.txt.

## Next

Two un-reached do-init construction gates remain: DMCONT 0x102bd1d68 and the main dispatch body
0x10258b5d8. The fall-through path parks cleanly inside the app-shell band — the next probe is to
find exactly where the fall-through parks (the last block-entry pc 0x102207f58 and beyond:
does the band loop / ret / reach a live-DM read?) and whether a seed (e.g. the SH360 emptyvec /
the app-shell band's own guards) turns the park into a clean RETURN that advances do-init to
completion. Do-not-re-tread unchanged (SH285/385/395-398, setDataModelToCurrent SH388, EC
reader-gate, 0x258b5d8/SetInitParams SH362/375, window-attach real SH367, ALooper SH365,
governor-gates full-ladder SH379, LSM crossings).