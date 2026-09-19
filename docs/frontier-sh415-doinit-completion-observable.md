# Frontier SH415 — do-init completion is now a first-class observable of the ordered session substrate

Date: 2026-09-22, hermes-worker, single-agent (cone suppressed). Workspace green
before (cargo test --workspace EXIT 0) and after (arm64jit lib 469/0, +1 new
SH415 hermetic). Capture runs/capture_sh415_doinit_completion.sh, log outside
repo (/home/hermes-worker/runs/sh415-doinit-completion.txt). Production code
only in session.rs (off the 1MiB hooks); jit.rs/elfjit.rs untouched (both at /
near the hook, byte-unchanged).

## Why this cycle

The recon-v3 immediate-priority deliverables (self-driven frames + JSON fix)
were re-verified green at HEAD first: capture_taskv4_frame.sh attempt 1 = 24
real task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort,
0 crash, EXIT 124. The single genuinely-open front remains Route B — a completed
do-init owning a live DataModel. SH400-414 built the executable ordered
session-substrate drive (session boot, G3 content, DM binder, nativeAppBridgeAppStart,
the onFlagsLoaded->onEngineInitialized->onAppReady->onDidLogInReceived lifecycle,
input pump). But do-init COMPLETION — the EXECUTE-DO-INIT-GATES letter's `DM-root
[0x106a68818] non-NULL with [[+0x20]] vt OK AND app-DM counter [0x106dca0e88]
advanced` markers — was never a REPORTED observable of that substrate; it was
only inferred from scattered one-off probes. SH415 makes it one.

## What landed

- **session.rs**: `probe_doinit_completion() -> DoinitCompletion` — a page-guarded
  (routeb_ensure_writable, no-SIGSEGV) readout of the four EXECUTE-DO-INIT-GATES
  live-DM markers: once-guard [0x106a68410].bit0, DM-root [0x106a68818], the
  genuine-DM vtable check ([root+0x20] -> in-image vt, [vt+0x30] readable), and
  the app-data-model counter [0x106dca0e88]. `DoinitCompletion::liveness()` is the
  single bit: true only when ALL four hold.
- Wired into `drive_routeb_session_substrate` right after the two atoms whose
  jit_run bodies reach the do-init — StartLuaAppDM (0x1023efe2c) and
  V2StartAppWithParams (0x10258b144) — so completion is reported per-atom like
  MH_* / AppBridgeV2, not guessed.
- New hermetic `sh415_doinit_completion_probe_safe_and_aggregates` (arm64jit lib
  468 -> 469): no-live-image reads degrade to 0 without SIGSEGV, liveness()
  aggregates (any single failed marker negates), and the two do-init-reaching
  atoms stay in the substrate table so the wiring cannot silently disconnect.

## Measured (real libroblox.so, complete SH400-414 substrate, libroblox.so)

- Substrate completes **11/16 atoms non-zero Ok** (no regression from SH400), 22
  `returned Ok` / 2 `stopped`, EXIT clean (no SIGSEGV/SIGABRT/guestpc terminal).
- **The probe fires twice (after StartLuaAppDM AND after V2StartAppWithParams) and
  reports the same honest verdict:**
  `once-guard[0x106a68410]=0x101 bit0=1 DM-root[0x106a68818]=0x0 vt=0x0 in-image=0
  app-DM-counter[0x106dca0e88]=0x0 -> LIVE DM = false`.
- The genuinely-new datum: **under the COMPLETE current substrate, the first
  EXECUTE-DO-INIT-GATES precondition is now MET** — the once-guard [0x106a68410]
  is seeded bit0=1 (0x101), so the once-lambda was LET to run — and yet DM-root
  still stays 0x0 and the app-DM counter 0. This is direct per-run proof
  (consistent with SH381's at-the-store measurement) that the do-init once-path's
  construct helper does not populate a live DM: the guard gate being satisfied is
  necessary but not sufficient. Route-B live-DM structural gate UNCHANGED.
- Full host lifecycle fires: MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all
  true, MH_GAME_LOADED false, AppBridgeV2 genuine vt 0x1063a3410.

## Honest

Does NOT manufacture a DataModel; the probe moved no guest byte (pure readout,
page-map-on-demand only). It turns the do-init completion gate into a first-class
REPORTED observable of the ordered runtime — the exact "seed and hope" vs
"measured verdict" discipline the operator demands. The verdict this cycle
re-confirms, now as a substrate-built-in: the do-init once-path stays empty
headlessly, so the live DM still requires the engine's own session to construct
it (the standing SEP-18 BUILD-THE-RUNTIME / cause-not-symptom front). Rule-1
regression check + recon-v3 deliverables all green at HEAD.

## Files

- crates/arm64jit/src/session.rs (+probe_doinit_completion / DoinitCompletion /
  probe_rd, wired after StartLuaAppDM + V2StartAppWithParams, +hermetic sh415)
- runs/capture_sh415_doinit_completion.sh
- docs/frontier-sh415-doinit-completion-observable.md

## Next (standing)

The runtime now REPORTS do-init completion; the marker stays non-live until the
engine's own session ctor builds the DM world. Keep the SEP-18 runtime build
(the one cause-not-symptom front): a real Activity/AppBridge/JNI-lifecycle/GLES
substrate drive. Do-not-re-tread unchanged: LSM skips/rebuilds (SH349/350/358/
373/375/377/378/385/396), setDataModelToCurrent (SH388), EC reader (SH355/356/374),
window-attach real (SH367), ALooper (SH365), governor gates (SH379).