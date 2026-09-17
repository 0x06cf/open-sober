# SH251 — dynamic DM-ctor trace RE-ATTACK (operator SEP-15 line) measured: pairing the
# genuine manufactured DM with the DMCONT continuation does NOT cross the app-start wall
# (fresh single-agent negative; closes the "reattack via the trace, not a static seed" lever)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.
Repro: runs/batch_sh251_dmcont_realctor.sh (3-run batch), recon-v3 plane re-verified green.

## Why (the operator's SEP-15 directive is the exact framed lever)
The operator's hard Route-B re-attack directive: "re-attack the live-DataModel construction
... re-examining whether the declared 'not seedable' wall can be crossed via the do-init/do-build
completion or a dynamic DM-ctor trace rather than a static seed." The dynamic DM-ctor trace
(routeb_dm_real_ctor_drive_guard, SH187) already CONSTRUCTS a genuine-vptr DM through the real
ctor wrapper 0x1023f5ff8 -> 0x1023f6038 and plants current-DM holder 0x106391908. Namely it is
"not a static seed" — it runs the engine's real DM construction code. But every prior DMCONT
continuation batch (SH248c..f, SH250) ran WITHOUT JIT_ROUTEB_DM_REALCTOR, so app-start's live
host-heap map construction (SH248g wall 0x1021dde34) ALWAYS saw holder=0. This cycle pairs the
two for the FIRST time and measures.

## Measured (real libroblox.so, 3 completing ladders, EXIT 134/139 at the shared wall)
- routeb-realctor fires 3/3: `REAL DM ctor wrapper 0x1023f5ff8 DROVE ok ret x0=...; obj vptr
  set = 0x1067162e8,0x1067163a0,0x1067163f8 GENUINE MATCH` — the trace constructs a genuine DM.
- SH187b plants it: `planted constructed DM ... into current-DM holder 0x106391908` 3/3.
- The DMCONT continuation runs its full serialize body (0x102bd1d68..0x102bd2014 region hits)
  then drives nativeAppBridgeAppStart deep (0x102339004..0x10233907c region hits) — and STILL
  terminates at the SAME live-object wall: `[SIGSEGV] ... guestpc=0x1021dde34 fault=0x0
  EXIT=134` (runs 1,3) / `Aborted` (run 2 = the known SH55/64 second-gate flake).
- The setDataModelToCurrent / DataModelServices registry fan-out is NOT reached (region
  0x1022dbcc0 watch noise only; correct guest is 0x102dbcc10, no genuine fan-out observed).

## Verdict (do-not-re-tread)
The operator's named re-attack lever — "cross the live-DM wall via a dynamic DM-ctor trace
rather than a static seed" — is now MEASURED on the actual combination: a genuine real-code
constructed DM present in the current-DM holder does NOT change app-start's live-object map
construction outcome. Reason (consistent with SH248g/249/250): the 0x1021dde34 wall is a
SEPARATE host-heap LiveObject graph (JNI_OnLoad+0x69e.. region, its own ctor never ran
headlessly), only reachable + completable by a real DataModel ctor upstream of the current-DM
holder the manufactured trace supplies. The trace manufactures the DM and arms the registry
predicate (necessary) but app-start's own construction graph is downstream, under-allocated,
and needs its own live initialization. This is a fresh measured closure of the specific
re-attack the operator named — not a re-tread, because holder-armed + trace + continuation
had never been run together.

## Honest
Does NOT manufacture a GuiObject scene node; Route-B live-DM structural gate UNCHANGED.
SH174 capture-latch arming *(0x106391908) at a real (session) make_shared<DataModel> stays the
single forward hook. recon-v3 deliverables (24 task-driven frames / json zero-fix) re-verified
green at this HEAD.

## Files
- runs/batch_sh251_dmcont_realctor.sh, runs/sh251-dmcont-realctor-batch.txt
- HANDOFF/STATUS ledger entries (this section). No production code path edited this cycle
  (pure re-attack measurement); workspace green.