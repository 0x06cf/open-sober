# Frontier SH344 — re-armed the Route-B DM-creation trace on the SH343-deepened FULL ladder: the NativeDataModelManager session-ctor line now fires (DM-creator band 0 hits -> 1 hit), yet a live DataModel still never allocs (0 validated make_shared<DataModel>); terminal = SH285 LSM reader/pop live-object wall

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Measurement-only
(no production code edited). Workspace green at HEAD SH343 (cargo test --workspace
exit 0). recon-v3 deliverables re-verified present.

## Why this cycle (the Route-B re-attack, not a re-tread)
The operator's SEP-15 hard directive: once the persistence/cookie line lands
(SH343 crossed the LSM poison fencepost), STOP the persistence lane and RETURN to
Route B as the TOP priority — re-attack the live-DataModel construction via the
ExperienceController / initializeLuaAppWithDataModel / DataModelServices path.
SH164 measured the DM-creator band (NativeDataModelManager getFlagsFromEngine_/
initEngine_ 0x102bd1a30..0x102bd1d08) at **0 hits** — but that was on the
--v2boot-skip-appstart send-appevent reach. The SH343 landing crossed the LSM
poison fencepost on the FULL app-start-driving ladder. This cycle re-runs the
Route-B reach on the SH343-deepened FULL ladder to see whether it now enters the
NativeDataModelManager session-ctor band that the skip-appstart path never reached.

## What was measured (real libroblox.so, SH343 full-ladder env + KEYFIX + DM capture)

### Run 1 — DM-allocation capture latch armed (JIT_DM_ALLOC_CAPTURE + DELEGATE)
- Latch ARMED: `SH167/SH169 routed CRT operator-new ACTIVE hook 0x1067daaf0 ->
  capture trail 0x7f00000001e0 (prev_hook 1021ebaf4) -> all operator-new blr the
  capture trail` — the SH174 forward hook is wired and live on this ladder.
- The trail caught real allocations (`FIRST call#1..#4 bytes=0x18`) + one
  `bytes=0x20040` — but **0 capture lines `[validated]`** => NO `make_shared<DataModel>`
  ever produced a validated in-image-vtable object.
- Run died `EXIT 139` `std::bad_function_call` (libc++abi terminating) — the
  DELEGATE path (calling through the JIT to the engine's allocator hook) is
  disruptive on this deep full ladder, exactly the SH170/runbook-caveated risk.
  Not a session fault — an instrumentation-window artifact.

### Run 2 — Route-B region-watch on the SH343 FULL ladder (KEYFIX, no capture)
JIT_REGION_WATCH across [NativeDataModelManager 0x102bd1a30-0x102bd1d40],
[setDataModelToCurrent registry 0x102dbcc10-0x102dbcd40], [governor
0x102e9fa80-0x102ea3b40], [ScriptContext 0x101f1d8ac-0x101f1d940]:
```
governor         0x102e9fa80..0x102ea3b40 : 22 hits (pc 0x102e9fa84, fb58, fb6c, fbbc,
                                  fbc8, fbdc, fbf8, fc08, fc1c, fc3c, fc54, fc74,
                                  fc80, fc90, fc9c, fcac, fcb4, fcbc, fcc4, +ea3084,
                                  ea30d0, ea30dc)
NativeDataModelManager 0x102bd1b98   : 1 hit (engine-init fnB ENTERED)
ScriptContext loader 0x101f1d8ac       : 0 hits
setDataModelToCurrent reg 0x102dbcc10    : 0 hits
terminal: guestpc=0x101db1b08 fault=0xffffffffffffffff (SH285 LSM reader/pop
          live-object wall); EXIT 134 (SIGABRT after the SEGV)
```

## What is genuinely new (do-not-over-claim)
- On the SH343-deepened FULL app-start-driving ladder, the governor runs end-to-end
  (22 pcs incl. the LSM-tail band 0x102ea30xx) AND the engine's own
  NativeDataModelManager session-ctor line now ENTERS: `routeb-dmforce` SH164
  DISPATCH -> fabricated NativeDataModelManager -> fnB **0x102bd1b98 real
  engine-init entered** -> `bl 0x102bd8ce8` -> manager vt +0x1f0 = **REAL
  continueAfterFlagsLoaded_ (0x102bd1d68)** -> nativeAppBridgeAppStart (the SH165
  manager re-seed + SH243 getter cell + SH245 app-name guard all fired). That is the
  DM-creator band (0x102bd1a30..0x102bd1d08) that SH164 measured at **0 hits** on
  the skip-appstart path — reached here.
- Despite that session-ctor reach, **no live DataModel is produced**: Run 1's
  capture trail got 0 [validated] objects; Run 2 terminates at the SH285 LSM
  reader/pop live-object terminal 0x101db1b08 (fault 0xffffffffffffffff) with the
  session half still uneventful (ScriptContext + setDataModelToCurrent registry
  0 hits). The live-DM structural gate is CONFIRMED unchanged one fencepost deeper.
- SH285's verdict stands: 0x101db1b08 is a guest-heap string-object whose internal
  buffer pointer [obj+0x50] is 0xff..ff (uninitialized) — the SH248h/SH256
  live-object class driven by the cause-not-symptom SESSION-CTOR lever, NOT a
  seedable repair. Do NOT re-drive a repair seed into it.

## Honest conclusion
The SH343 keyfix LSM cross does NOT unlock the DM ctor. The full ladder now
measureably enters the NativeDataModelManager session-ctor line (a Route-B forward
distinct from SH340's skip-appstart governor-silent path), but the engine's
make_shared<DataModel> still never runs headlessly and the run terminates at the
same structural live-object wall. No DM-root (0x106a68818=0), MH_* all false. The
Route-B live-DM gate is UNCHANGED — it needs either the cause-level SESSION-CTOR
drive (real Activity/AppBridge session, the SEP-17 primary lever) or real input +
display (migration runbook), not another static seed. SH174 capture-latch (now
proven armed here) stays the single forward hook.

## SH344b (same cycle, app-shell-ctor band probe)
- Region-watch [app-shell ctor 0x102207b50..0x102209000][post-do-init
  0x1023eff4c..0x1023f0100][ScriptContext 0x101f1d8ac] on the SAME SH343 full ladder:
  **app-shell ctor band 0 hits** — the manager->nativeAppBridgeAppStart continuation
  does NOT proceed into the app-shell ctor under the current seeds (stable negative;
  same conclusion as SH164 for this band). Terminal differs by band config:
  `guestpc=0x10284cfa0 fault=0x0` (EXIT 134) — this is the
  JNIActivityLifecycleCallbacks nativeOnDestroyed family (`ldr x1,[x8]` x8=0 at
  file 0x284cfa4), i.e. the run routes into an activity-lifecycle callback path when
  the app-shell band is watched (seed/timing divergence). Informative for the
  SESSION-CTOR lever: the activity-lifecycle surface is where control goes, not the
  app-shell ctor.

## Files
- Probe: runs/capture_sh344_dmcap.sh (DM-capture arm on SH343 ladder; shows latch
  armed + 0 [validated] + bad_function_call EXIT 139).
- Probe: runs/capture_sh344_routeb_reach.sh (region-watch; governor 22 hits, DM-creator
  1 hit @0x102bd1b98, ScriptContext/setDataModelToCurrent 0, terminal 0x101db1b08).
- Probe: runs/capture_sh344b_appshell.sh (app-shell ctor band 0 hits; terminal
  0x10284cfa0 nativeOnDestroyed family).
- Live captures gitignored (runs/sh344-*.txt).
- No production code changed this cycle; workspace green.

## Next (unchanged, authoritative)
The SEP-17 SESSION-CTOR lever (drive the real Android Activity/AppBridge session
init state machine) remains the primary forward. The next-implementable artifact
on the now-reached NativeDataModelManager line is the scoped re-router of the
manager's flag-completion slot (+0x1f0) deeper toward a real engine-constructed
app-shell, per SH165-fwd "Next" — a session-forward that continues to be gated by
the fakt that seeds cannot produce a live DataModel (SH165-fwd task-1/e2).