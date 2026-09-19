# Frontier SH409 — the SH400 substrate's missing "onAppReady actually DRIVEN" host surface

Date: 2026-09-19/21, hermes-worker, single-agent. Workspace green before/after
(cargo test --workspace EXIT 0; arm64jit lib 457/0 with the new session test).
Real-binary MEASURED on libroblox.so (runs/capture_sh400_session_substrate_drive.sh).
Production code only in session.rs + jni.rs (both far under the 1MiB hooks; jit.rs
and elfjit.rs untouched — they are at/near the hook).

## Why this cycle

The standing frontier (SH407/408 "Next") named the ONE genuinely-open surface as
"the REAL session-compat runtime (SH400 substrate + a real LSM/EGL/onAppReady host
drive)". Recon-routeB step-2 and the SEP-18 "BUILD-THE-RUNTIME" operator directive
both name the same missing executable step: the NativeHelper lifecycle callbacks
"actually DRIVEN" — a real host invokes onFlagsLoaded -> onEngineInitialized ->
onAppReady on the gameActivity object. The SH400 substrate (session.rs) drove all
16 guest atoms but ONLY WAITED for the engine to reach those CallVoidMethod sites,
so none ever fired headlessly (MH_* all stayed false on every prior SH400-408 run).

## What landed

- jni.rs: `pub fn fire_nativehelper_milestone(name: &[u8]) -> u64` — interns the
  milestone name to a readable NULL-terminated method-id handle (identical to the
  engine's GetMethodID return) and dispatches it through the SAME registered JNI
  CallVoidMethod shim (slot 61) the engine uses, so the MH_* observables transition
  exactly as a real session's own callbacks would. Inert unless a host drive calls it.
- session.rs: `pub fn drive_nativehelper_lifecycle()` drives the ordered sequence
  [gameActivity_onFlagsLoaded, gameActivity_onEngineInitialized,
  gameActivity_onAppReady] through that shim. Wired into
  `drive_routeb_session_substrate` immediately after the surface-handover atom
  (V2UpdateSurfaceAppWithPlatformParams 0x1025f5fec) — the recon-routeB step-2
  position (surface -> forced onAppReady -> SendAppEventOnAppReady).
- New hermetic `lifecycle_milestones_driven_in_order` (session.rs, no real binary).

## MEASURED (real libroblox.so, SH400 capture, EXIT 124 / 11-16 Ok)

The substrate drive now logs the ordered milestone sequence (was absent before):

```
[session-drive] lifecycle milestone 'gameActivity_onFlagsLoaded' fired (ret=0); MH_FLAGS_LOADED=true ...
[session-drive] lifecycle milestone 'gameActivity_onEngineInitialized' fired (ret=0); MH_FLAGS_LOADED=true MH_ENGINE_INITIALIZED=true ...
[session-drive] lifecycle milestone 'gameActivity_onAppReady' fired (ret=0); MH_FLAGS_LOADED=true MH_ENGINE_INITIALIZED=true MH_APP_READY=true
[session-drive] ...post: MH_FLAGS_LOADED=true MH_ENGINE_INITIALIZED=true MH_APP_READY=true MH_GAME_LOADED=false AppBridgeV2[0x106a705e8]=0x1063a3410
```

The three milestones fire IN ORDER (flags-loaded -> engine-initialized -> app-ready)
through the same registered shim, and MH_APP_READY latches on the real binary — the
missing "onAppReady actually DRIVEN" host surface. AppBridgeV2[0x106a705e8] stays at
the genuine relocated in-image vt 0x1063a3410 (unchanged, correct).

## Honest

This is a host-side lifecycle SURFACE, not a DataModel manufacture. DM-root
[0x106a68818] stays 0, once-slot 0x400000b sentinel present, MH_GAME_LOADED false,
no make_shared<DataModel>. Setting the MH_* observables does not by itself boot Lua
(SH405: they are session observables; the do-init must still complete). What IS real
and newly shipped: the substrate now exercises the exact host-to-engine lifecycle
call sequence recon-routeB step-2 / SEP-18 prescribe, in order, through the engine's
own registered shim — a concrete, testable, shippable piece of the BUILD-THE-RUNTIME
deliverable, and it makes MH_APP_READY latched on a real drive (previously always 0).

Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* now host-driven but the
DataModel still needs a completed do-init). No re-treads: no LSM skips, no map
manufacture, no setDataModelToCurrent, no single-object DM seeds.