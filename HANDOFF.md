# Open Sober — Agent Handoff
## SH344 (Sep 19, 2026, hermes-worker): Route-B re-attack on the SH343-deepened FULL ladder — the NativeDataModelManager session-ctor line now FIRES (DM-creator band 0 hits -> 1 hit @0x102bd1b98), yet the engine's make_shared<DataModel> still never allocs (0 validated), terminal = the SH285 LSM reader/pop live-object wall 0x101db1b08. re-armed + ran the SH174 DM-capture latch on the ladder (latent hook proven live: caught real allocs, 0 validated).

Single-agent (cone suppressed). Persistence landed (SH343) -> returned to Route B per
the operator's hard directive. Measurement-only (no production editor). Workspace green.

## SH344 (committed): two probes + frontier doc.
- Run 1 (JIT_DM_ALLOC_CAPTURE+DELEGATE): latch ARM confirmed (`CRT operator-new ACTIVE
  hook 0x1067daaf0 -> capture trail`) catching real allocs (0x18, 0x20040) but **0
  [validated]** -> no live DataModel. EXIT 139 bad_function_call = the known
  delegate-path disruption on the deep ladder (SH170/runbook caveat), not a session fault.
- Run 2 (region-watch): governor 22 pcs end-to-end (0x102e9fa84..0x102ea30dc) then
  routeb-dmforce SH164 fabricated manager -> fnB real engine-init guest 0x102bd1b98 ->
  bl 0x102bd8ce8 -> REAL continueAfterFlagsLoaded_ (0x102bd1d68) -> nativeAppBridgeAppStart
  (SH165 manager re-seed + SH243 getter cell + SH245 app-name guard all fired).
  Terminal: guestpc=0x101db1b08 fault=0xffffffffffffffff (SH285 LSM reader/pop live-object
  wall). ScriptContext + setDataModelToCurrent registry 0 hits. EXIT 134.
- CONFIRMS: SH343's LSM keyfix does NOT unlock the DM ctor. The full ladder now
  measureably enters the NativeDataModelManager session-ctor line (distinct from SH340's
  skip-appstart governor-silent path) but the live-DM gate is UNCHANGED — no DM-root,
  MH_* false. SH285 verdict stands: 0x101db1b08 is the live-object class,
  cause-not-symptom only, do NOT repair-seed it.

recon-v3 deliverables (type4_frame_thunk self-drive + JIT_JSON_ZERO_FIX) re-verified
present at HEAD. Route-B live-DM structural gate UNCHANGED. SH174 capture-latch (proven
armed here) stays the single forward hook.

## Next (unchanged, authoritative)
The SEP-17 SESSION-CTOR lever (drive the real Android Activity/AppBridge session init
state machine) remains the primary forward — now with the NativeDataModelManager line
reachable. The next implementable artifact is the scoped re-router of the manager's
flag-completion slot (+0x1f0) toward a real engine-constructed app-shell
(SH165-fwd "Next"), gated by the fact that seeds cannot produce a live DataModel
(SH165-fwd task-1/e2, SH174 runbook).