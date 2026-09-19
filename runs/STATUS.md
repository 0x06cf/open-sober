# Open-Sober run status (hermes-worker)

## SH478 (this cycle): complete InitParams.buildVariant — now "release" (was "")
The authoritative recon-framework-boot-order.md names buildVariant="release"; the JIT
served "". MEASURED in the real .so that buildVariant is consumed in config/telemetry
identity + compared against release/debug/production literals. getBuildVariant now
returns b"release" (7 chars); both pins move out of the empty-default arm (fn-table
len==7 + sh134 empty-set drop). arm64jit 683/0, workspace green.

## Cycle opening (this session, all ingested into SH478)
- recon-v3 deliverable re-verified GREEN at SH477 HEAD and re-re-verified at the SH478
  HEAD after the production change (capture_taskv4_frame.sh attempt 1 BOTH times: 24 real
  task-driven frames `present swap Ok(0x1)`, 195-197 node pops, 0 json abort, 0 crash,
  EXIT 124 stable idle) — the buildVariant production edit did NOT regress the deliverable.
- Do-init/Route-B baseline re-probed (capture_sh415): substrate 14/16 atoms completed
  jit_run (11 non-zero) + 2/16 stopped, once-guard bit0=1, DM-root [0x106a68818]=0x0 ->
  LIVE DM=false, MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY true, AppBridgeV2 vt
  0x1063a3410, 0 crash.

## Current state
- `dev` HEAD = SH478 (d6bf25f). Production change: getBuildVariant -> "release" (jni.rs).
- Workspace green (cargo test --workspace EXIT 0, ~864 passed/0 failed; arm64jit 683/0).
- recon-v3 deliverables green (self-driven task frames swap Ok(0x1), 0 json abort, 0 crash).
- Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0 (structural per SH462/467).

## Honest status
- No live DM. Route-B live-DM structural gate UNCHANGED. SH478 is a BUILD-THE-RUNTIME
  session-config identity completion (the value the engine reads when it serializes its
  params), latent-but-correct — not a Route-B seed.

## Next-forward candidates
1. (standing, TOP) do-init completeness / live-DM: aligned lever is the session-ctor /
   runtime-surface drive (engine's OWN session constructs the DM). Real APK assets staged
   at SOBER_ASSETS_ROOT so a completed do-init's first rbxasset/AAssetManager request has
   REAL content.
2. The SendAppEvent 'Home' fabricate path (SH339): a real 4-byte "Home" SSO reaching the
   discriminator (currently the fabricated jstring materializes as size 6 -> event 0).
   Open and downstream of the live-DM wall.
3. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged).
4. (CLOSED) DeviceParams viewport Mm (SH476).
5. (CLOSED) LocaleList size()/get() flattening (SH477).