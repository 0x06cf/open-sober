# Frontier SH399 — the coherent ordered SESSION-SUBSTRATE (the SEP-17/18 BUILD-THE-RUNTIME deliverable)

Date: 2026-09-21 (this cycle), hermes-worker, single-agent (cone suppressed). Library-level
additive deliverable only: a new `ROUTEB_SESSION_SUBSTRATE` ordered session drive + a real-image
hermetic `sh399` that byte-pins every atom on the real libroblox.so. NO production behavior change
(the table is data; the elfjit ladder thread consumes the order). Workspace green at start and end
(cargo test --workspace EXIT 0, arm64jit lib 447 passed/0 failed). recon-v3 deliverables
re-verified green at this HEAD (capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames
`present #N swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; capture_sh304 session-gated
producer INERT 3 UNGATED/0 GATED; JIT_JSON_ZERO_FIX present).

## Why this cycle

The last 9 SH cycles (SH390-398) were ALL probe-only DM closures — every one measured that no
static composition manufactures a live DataModel, and each landed zero production code. The
operator's SEP-18 directive is explicit: BUILD THE RUNTIME, NOT THE DM. The concrete gap is that
the SEP-17 Activity/AppBridge lifecycle natives (setActive, onEngineSettingsReceived,
client-settings, initAppShellReporter, AppBridgeV2 InitWithParams, StartLuaAppDM, StartApp,
SendAppEventOnAppReady/GameLoaded, MessageBus experience-launch subscribe) exist ONLY as scattered
opt-in `--v2boot-*` rungs in elfjit.rs — they were never assembled into ONE validated, ordered,
host-side session substrate, and each rung's address was never independently byte-pinned on the
real binary in a hermetic test.

## What landed

- `RoutebSessionAtom { name, guest, abi_slots }` + the 16-entry `ROUTEB_SESSION_SUBSTRATE` table
  (jit.rs, pub const), ordered per recon-routeB-globaltinit-unblock.md + the SH184 lifecycle map +
  SH264/275/277: ROUTE-B ladder first (nativeInitializeNativeFlags, nativeGameGlobalInit,
  setTaskSchedulerBackgroundMode, V2InitWithParams, StartLuaAppDM, V2StartAppWithParams,
  V2UpdateSurface), then the SEP-17 native lifecycle (initAppShellReporter, setActive,
  nativeSetInitParams, nativeInitClientSettings, nativeInitClientSettingsSigned,
  onEngineSettingsReceived, SendAppEventOnAppReady(Home in x5), SendAppEventOnGameLoaded,
  MessageBus.subscribe(experience-launch)).
- `sh399_routeb_session_substrate_ordered_pinned` (arm64jit lib 446->447): reads the real
  libroblox.so (skips if absent) and asserts EVERY atom resolves to a real non-leaf fn prologue
  (`sub sp,sp,#imm` 0xd1... or `stp x29,x30,[sp,#imm]!` 0xa9...) at the correct file offset
  (guest = file+0x100000000, .text has file-offset==vaddr), that no address repeats, and that
  every ABI slot is <=5. FAILS if any atom drifts from the real binary.

## Verification

`cargo test -p arm64jit --lib sh399` => ok (446 filtered, 1 passed; real-binary anchors exercised).
`cargo test --workspace` EXIT 0. File size: jit.rs held under the 1MiB commit hook (1,048,451 B,
~125 B headroom) by condensing SH-prose comments (facts/addresses preserved) — elfjit.rs untouched.

## Honest

Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false, AppBridgeV2 0). This is the single validated ORDER the elfjit session
drive consumes, minus the per-atom gate seeds those rungs already carry separately — a
build-the-runtime substrate artifact, not a DM advance. SH174 capture-latch stays the single
forward observer; R1 content half stays staged/armed/serviceable (SH351/352/354). The substrate
fires when a real session advances (do-init owning a live DM constructs the GuiObject tree the
rungs' SendAppEventOnAppReady/GameLoaded + MH_* atoms drive); that live-DM gate is unchanged.