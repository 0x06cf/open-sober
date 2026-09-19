# Frontier SH402 — correct the StartAppWithParams attribution + measure the boot-body / app-shell band reach at this HEAD
# (recon v3 / SESSION-CTOR, the frontier's named next gate after SH401)

Date: 2026-09-19/20, hermes-worker, single-agent. Workspace green before/after
(`cargo test --workspace` EXIT 0, 631 passed/0 failed, arm64jit lib 451; elfjit.rs
untouched, jit.rs condensed under the 1MiB hook).

## Why this cycle

SH401 measured the ordered session-substrate drive (SH400, genuine AppBridgeV2 single
vt 0x1063a3410) executing the REAL governor 0x102e9fa84 and its make-call
`bl 0x258c6e4`, which SH401 labeled a "StartAppWithParams" entry hit. Its frontier doc
named the next gate: *"drive StartAppWithParams 0x258c6e4's body toward DMCONT
0x102bd1d68."* This cycle runs exactly that measurement and corrects the attribution
RECON-V3 already flagged in prose: **0x258c6e4 is NOT the StartAppWithParams boot
body — it is the AppBridgeV2 app-registry hash-insert helper.**

## What SH402 measured (real libroblox.so, region-watch, 2/2 reproducible, EXIT 124, 0 crash)

Same env as SH401 (full ladder + the SH400 `--v2boot-session-drive` ordered substrate;
no skip-appstart):

- **0x258c6e4 = hash-insert helper, not the boot body.** It has a clean
  `stp x29,x30,[sp,#-64]!` / `a9bc7bfd` prologue, is a ~0x1d0-byte leaf that computes
  a hash-index lookup and returns, and SH159e's deterministic-param patch (cap=0,
  count=0, float=1.0) makes it INSERT an EMPTY app entry then `ret` — i.e. a benign
  registry no-op under the ladder. It never builds app state.
- **The REAL boot body `nativeAppBridgeV2StartAppWithParams` at 0x258b144
  (`sub sp,#0xf0` = `d103c3ff`) runs DEEP headlessly** — region-watch recorded
  7 block-entry pcs 0x258b144..0x258b268 (prologue through the x23/x19/x20 setup and
  the first adrp/ldr chain). That is the genuine StartApp-pivoted code path, now
  entered through the ordered substrate.
- **The do-init / app-shell ctor band [0x102207b50,0x102209000) is ENTERED** (~10
  distinct block-entry pcs), consistent with SH340/SH360's app-shell world-build band
  (the emptyvec walker block entries 0x102208e4c/0x102208e88 are the SH360-shaped hits).
- **DMCONT 0x102bd1d68 = 0 hits** (still the standing not-reached gate).
- A never-run composition was ALSO measured: the SH400 genuine-single drive combined
  with the OLDER rungs that SH371 had measured FIRING DMCONT DEEP
  (`--v2boot-session --v2boot-surface-handoff --v2boot-send-appevent
  --v2boot-send-game-loaded --v2boot-session-bus`) — under that combination the run
  aborts at the run-variable live-object arm guestpc 0x10284cfa0 (EXIT 134) BEFORE any
  StartApp body / DMCONT; and re-running the plain old SH371 env at THIS HEAD also
  aborts there (behavior drifted across cycles — the live-object arm is run-variable).

## Honest

- Does NOT manufacture a DataModel. DM-root [0x106a68818]=0, MH_* stay false,
  AppBridgeV2 [0x106a705e8] = genuine vt 0x1063a3410 (from SH400, unchanged).
- This is a MEASURE + attribution correction + new hermetic. The genuine forward is
  that the STARTAPP BOOT BODY (not just the hash-insert helper SH401 measured) is now
  reachable through the ordered substrate; DMCONT remains the next standing gate.
- New real-image hermetic `sh402_startapp_boot_body_and_appshell_band_reach_pinned`
  (arm64jit lib 450->451) byte-pins: boot body 0x258b144 = `sub sp,#0xf0` (d103c3ff),
  hash-insert helper 0x258c6e4 = `stp` frame (a9bc7bfd), app-shell band entry
  0x102207b50 = unconditional `b` (14000001), SH360 emptyvec walker block-entry
  0x102208e88 = `ldp` (a9405275).
- Captures: runs/capture_sh402_startapp_body.sh, runs/capture_sh402b_combined.sh,
  runs/capture_sh402c_dmcont_confirm.sh, runs/capture_sh402d_bootbody.sh.
  Logs (outside repo): /home/hermes-worker/runs/sh402-*.txt.

## Next

The standing next gate is still DMCONT 0x102bd1d68 (continueAfterFlagsLoaded_). Now
that the StartApp boot body is confirmed reachable headlessly through the ordered
substrate, the forward is to trace the boot body's own interior calls (its `bl`
targets from the disasm: 0x10626b6d0, 0x102b9ec9c, 0x102248224, 0x102baaa60,
0x102ba3df4 ...) and drive the one that feeds the DM-creator continuation, keeping the
same ladder + SH400 substrate env. Do-not-re-tread unchanged (SH285/385/395-398 family,
setDataModelToCurrent SH388, EC reader-gate, 0x258b5d8/SetInitParams SH362/375,
window-attach real SH367, ALooper SH365, governor-gates full-ladder SH379, LSM
crossings SH385/393/396).