# Open-Sober run status (hermes-worker)

Updated 2026-09-22, this cycle: SH415 — make do-init COMPLETION a first-class
observable of the ordered session substrate. SH400-414 built the executable
ordered session-drive (session boot, G3 content, DM binder, app-start, onAppReady
lifecycle, login gate, input pump), but the EXECUTE-DO-INIT-GATES live-DM markers
were only ever inferred from scattered probes. SH415 adds session.rs
`probe_doinit_completion()` (page-guarded readout of once-guard [0x106a68410].bit0,
DM-root [0x106a68818] + in-image vt, app-DM counter [0x106dca0e88]) wired into the
substrate right after the two do-init-reaching atoms (StartLuaAppDM +
V2StartAppWithParams), reporting do-init completion per-atom instead of guessing.

## Current state

- `dev` HEAD: SH415. Production code only in session.rs (off the 1MiB hooks);
  jit.rs/elfjit.rs untouched. Workspace green (arm64jit lib 469/0; cargo test
  --workspace EXIT 0).
- recon-v3 deliverables re-verified green at HEAD: capture_taskv4_frame.sh
  attempt 1 = 24 real task frames `swap Ok(0x1)`, 197 node pops, 0 json abort,
  0 crash.

## This cycle's advance

- **SH415**: do-init completion is now a REPORTED observable of the runtime.
  MEASURED on real libroblox.so (complete substrate, EXIT clean): substrate 11/16
  Ok; the probe fires twice and reports the same honest verdict — once-guard
  seeded (0x101 bit0=1, the FIRST EXECUTE-DO-INIT-GATES precondition now MET under
  the full substrate) yet DM-root=0, counter=0, LIVE DM=false. Per-run proof the
  do-init once-path constructs nothing live (SH381 consistent). Host lifecycle
  fires (MH_FLAGS/ENGINE_INITIALIZED/APP_READY true, AppBridgeV2 genuine vt).

## Honest status

- No DM (DM-root [0x106a68818]=0, no make_shared, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED. The probe moved no guest byte (pure readout) —
  it makes the completion gate measurable per-run, not a DM advance. The live DM
  still requires the engine's own session to construct it (SEP-18 cause-not-symptom).

## Next-forward candidates

1. (standing, TOP — Route B) do-init completeness / live-DM: the substrate now
   REPORTS the completion markers; keep the SEP-18 runtime build (real
   Activity/AppBridge/JNI-lifecycle/GLES drive) so the engine's own session ctor
   constructs the DM world.
2. DMCONT 0x102bd1d68 = 0 from the MAIN arm.
3. Feed real X-window events into `drive_host_input_pump` on the render/host loop
   (the natural integration point once a live DM advances).
4. Do NOT re-tread: setDataModelToCurrent (SH388), LSM crossings (SH385/393/396),
   SH285/SH341 family, EC reader-gate, window-attach real (SH367), ALooper (SH365),
   governor-gates (SH379).
5. Do NOT run the SH174 latch without JIT_DM_ALLOC_CAPTURE_DELEGATE=1 (SH395).