# Open Sober — Agent Handoff
## SH326 (Sep 18, 2026, hermes-worker): re-verified the two recon-v3 deliverables (type4_frame_thunk
## self-driven frames + json-zero-fix) are ALREADY landed + green at HEAD (no new code needed), and
## sharpened the standing 0x1025f5300 gate's SEP-17 forward contract. Verified: 24 task-driven frames,
## present #19..#23 swap Ok(0x1), 197 node pops, 0 json abort, EXIT 124. The gate 0x1025f5300 is
## deterministic 3/3 (fault=0x140); its cross is [appstart_obj+24]!=NULL — the genuine AppStarted field
## built by nativeAppBridgeAppStart (bl @0x25f52ec, out-param [sp+328]->x19). Extended sh325 hermetic
## guard with 2 new real-image pins (0x1025f52ec=bl-nativeAppBridgeAppStart, 0x1025f52f0=ldr x19,[sp,#328])
## so the exact session-construct source is pinned. NEW observation: the sh325 ladder probe's SetInitParams
## (--v2boot-session-set) drives the engine deep enough to wake the REAL type-4 TaskScheduler drain, but it
## faults NONDETERMINISTICALLY on leaked host-ptrs (RUN1 0x102855fd0 / RUN4 0x10624e6c0 / RUN5 0x1021df3cc;
## RUN2/6 benign) — run-variable, NOT a stable gate. elfjit examples 150/0, arm64jit lib 408/0, workspace
## green, elfjit.rs under 1MB hook (condensed SH-prose, facts preserved).
## Verify: sh325 1 passed; elfjit 150/0; cargo test --workspace EXIT 0. HONEST: no DM (DM-root 0, MH_* false);
## Route-B live-DM gate UNCHANGED. Next: the standing gate is again a genuine session-constructed AppStarted
## field at [appstart_obj+24] (SEP-17 real-session drive), not a seedable cell.