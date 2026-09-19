# Frontier SH403 — the StartApp boot body advances into the app-bridge pipe -> do-init chain
# (the RECON-V3 convergence line; next-gate after SH402)

Date: 2026-09-20, hermes-worker, single-agent. Workspace green before/after
(`cargo test --workspace` EXIT 0, 632 passed/0 failed, arm64jit lib 452; elfjit.rs
untouched at 45 B under the 1MiB hook; jit.rs condensed to 31 B under the hook after
adding sh403).

## Why this cycle

SH402 measured that the REAL StartAppWithParams boot body (0x258b144) runs DEEP headlessly
through the SH400 ordered substrate (7 block-entry pcs 0x258b144..0x258b268), correcting
SH401's attribution (0x258c6e4 = app-registry hash-insert helper, not the boot body). Its
frontier named the next gate: *"trace the StartApp boot body's interior calls and drive the
one that feeds DMCONT 0x102bd1d68."* SH403 executes that trace.

## What SH403 measured (real libroblox.so, region-watch, 2/2 reproducible, EXIT 124, 0 crash)

Disasm of the boot body showed `bl 0x2baeeec` at 0x10258b2dc — RECON-V3's named app-bridge
pipe ("StartAppWithParams (0x258b144) AND StartLuaAppDM converge on the app-bridge pipe
bl 0x2baeeec -> do-init 0x2206c40"). Region-watching the pipe + do-init + DMCONT on the
SAME SH402 env (genuine-single ordered substrate drive):

- **StartApp boot body runs PAST sh402's last pc (0x258b268)** — region-watch now records
  block-entry pcs through 0x10258b3a0.
- **The app-bridge pipe 0x102baeeec is ENTERED** (1 hit; `sub sp,#0x50` = d10143ff).
- **do-init 0x102206c40 runs its DEEP body** (20 block-entry pcs 0x102206c40..0x102206fac),
  and the **post-do-init worker 0x1023eff4c runs** (1 hit). The SH361 DM-ctor trace fires
  (container+32 non-NULL -> `[obj]vt=0x10635dde8 vt[+48]=0x10258b5d8` @0x2206e24).

So the RECON-V3 convergence chain **StartApp boot body -> app-bridge pipe -> do-init ->
post-do-init worker** now runs headlessly through the ordered substrate — a genuine advance
on the do-init/DM world-build line (do-init + its worker were never reached via the StartApp
path before, only via Standalone StartLuaAppDM).

## Standing gates (unchanged or confirmed)

- **DMCONT 0x102bd1d68 = 0 hits** — the next standing gate, still not reached.
- **The do-init MAIN dispatch BODY 0x10258b5d8 is STILL never entered** (SH362's
  measured-never-executes gate holds: the SH361 dispatch reads the target but the run parks
  cleanly EXIT 124 before control enters the body — 0 block-entry pcs in [0x10258b5d8,0x10258d000)).
- A never-run composition (genuine-single + old SH371 DMCONT-firing rungs) AND a plain re-run
  of the SH371 env both abort at the run-variable live-object arm 0x10284cfa0 (EXIT 134).

## Honest

- Does NOT manufacture a DataModel. DM-root [0x106a68818]=0, MH_* stay false, AppBridgeV2
  [0x106a705e8] = genuine vt 0x1063a3410 (unchanged). once-slot 0x400000b sentinel.
- This is a MEASURE + a new hermetic. The genuine forward: the StartApp boot body now runs
  DEEP (past SH402) and reaches the app-bridge pipe + do-init + post-do-init worker — the
  RECON-V3 convergence line is now executable headlessly; DMCONT + the 0x258b5d8 body remain
  the un-reached do-init construction gates.
- New real-image hermetic `sh403_startapp_pipe_then_doinit_reach_pinned` (arm64jit lib
  451->452) byte-pins: pipe prologue 0x2baeeec = d10143ff, boot body `bl 0x2baeeec`
  @0x10258b2dc = 94188f04, do-init prologue 0x102206c40 = d10303ff, post-do-init worker
  0x1023eff4c = d10603ff.
- Captures: runs/capture_sh403_pipe_doinit.sh, runs/capture_sh403b_dispatch_body.sh.
  Log (outside repo): /home/hermes-worker/runs/sh403-*.txt.

## Next

The two standing do-init construction gates remain: DMCONT 0x102bd1d68 and the 0x258b5d8
dispatch body. The reachable-but-parks behavior (EXIT 124, no fault) at the 0x258b5d8 body
suggests the do-init's main-branch dispatch lands somewhere benign rather than the runner
faulting at SH285 — the boundary where the (still absent) live DM would be read is the next
thing to probe (region-watch the do-init main-branch continuation 0x10258b5d8+ past its
fetched target; keep the same SH400 substrate env). Do-not-re-tread unchanged
(SH285/385/395-398 family, setDataModelToCurrent SH388, EC reader-gate, 0x258b5d8/SH362 body,
window-attach real SH367, ALooper SH365, governor-gates full-ladder SH379, LSM crossings).