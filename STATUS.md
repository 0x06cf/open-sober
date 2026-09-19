# Open-Sober run status (hermes-worker)

Updated this cycle (SH475): pinned the SendAppEventOnAppReady event-name discriminator DECODE
as a tested contract (the operator's "confirm w19-event=0x4"), and staged the real APK assets
so the engine's own content reads have something to serve when a live DM arrives.

## Current state

- `dev` HEAD: (SH475). recon-v3 immediate-priority deliverables re-verified GREEN at fresh HEAD
  (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames, 196 node pops, 0 json abort,
  0 crash, EXIT 124).
- Do-init/Route-B baseline re-probed (capture_sh415): substrate 14/16, once-guard bit0=1, DM-root
  [0x106a68818]=0x0 (LIVE DM=false), MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all true,
  AppBridgeV2 vt resolved, 0 crash.
- SH475: `jit::routeb_appevent_sso_size_to_event_code(b0,sp8)` — pure model of the SendAppEvent
  'Home' discriminator (SSO size4->event 4, 5->1, 12->3, else 0), + hermetic truth table + sh211
  real-image byte pins of the decode chain @0x102bb46b8. Closes the operator's explicit
  "confirm w19-event=0x4" pin. Honest: SH339's measured size-6 fabrication -> 0 (not 4); the
  discriminator genuinely routes "Home" only when a real 4-byte string reaches it.
- Also staged the real APK assets/ (594 files, 81MB) at /tmp/sober_assets_real; measured ZERO
  AAssetManager/rbxasset requests on the currently-reachable path (content stays latent until live DM).

## This cycle's advance

- One small pure helper + hermetic in jit.rs; sh211 real-image byte-pin extension in elfjit.rs.
  Both under the 1MiB hook (jit.rs 1,048,119; elfjit.rs 1,048,491; SH-prose comments condensed).

## Honest status

- No DM (DM-root [0x106a68818]=0, no store reaches it headlessly, LIVE DM=false) — Route-B live-DM
  structural gate UNCHANGED (SH462/467). This is a BUILD-THE-RUNTIME test-contract + content-surface
  advance, not a Route-B seed.

## Next-forward candidates

1. (standing, TOP) do-init completeness / live-DM: aligned lever is the session-ctor/runtime-surface
   drive (engine's OWN session constructs the DM). The real APK assets are now staged so a completed
   do-init's First AAssetManager/rbxasset request has REAL content to serve.
2. The SendAppEvent 'Home' fabricate path (SH339): a real 4-byte "Home" SSO reaching the discriminator
   (currently the fabricated jstring materializes as size 6 -> event 0). Fixing the fabricate side is
   an open question downstream of the live-DM wall.
3. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged).
4. DeviceParams viewport{Width,Height}Mm (338/190): reached via a Java static
   DeviceUtils.getScreenPhysicalSizeInMillimeters→Point→x/y FIELD path — build when a live run touches it.