# SH264 — SEP-17 SESSION DRIVE: drive the REAL Android Activity-lifecycle natives the engine asserts on

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · opt-in `--v2boot-session` (default-inert) · workspace green

## What this is

The operator's **SEP-17 hard directive** names the standing `0x1021dde34` live-object map wall
"EXHAUSTED" and redirects the primary effort: **build a REAL Android Activity/AppBridge session
drive** — emulate the lifecycle the engine asserts on, rather than seeding single objects.
SH184's lifecycle map lists the primitives: `JNIAppLifecycleNativeAdapter_setActive`,
`nativeActivity_onEngineSettingsReceived` (client-settings), `initAppShellReporter`,
`nativeAppBridgeAppStart`, plus the messageBus/dataModel-bindings receive entries.

**Measured, fresh:** all four Java→engine lifecycle natives are **JNI-RECEIVE entries with ZERO
in-image `bl` callers** (full-exec-text scan: `SetInitParams` 0x2bcc814, `setActive` 0x21f5de4,
`onResumed` 0x21f5db8, `initAppShellReporter` 0x21f53b8 each have 0 direct callers). Only the
*Java side of a real Activity* invokes them — so the harness **must** drive them as real guest
entries. Nothing in the codebase did before SH264.

## What landed (elfjit.rs `--v2boot-session`, single ladder thread)

Before the GlobalInit/app-start rungs, drive the three lifecycle natives that complete — as real
guest `jit_run` entries **on the SAME single ladder thread** (SH55/64: concurrent top-level
`jit_run`s corrupt the shared block cache; serialized only) — reusing `boot_sp`/`tpidr` + the
fabricated `thiz` + AutoValue init-params jobject, in the order the real Activity asserts them:

1. `initAppShellReporter` (guest `0x1021f53b8`) — once-guarded Reporter init
2. `JNIAppLifecycleNativeAdapter_setActive` (guest `0x1021f5de4`) — its core `0x21f5f80` reads the
   SH248f-fabricated app-lifecycle adapter triplet `[0x106b0bde0]`, so it resolves benignly
3. `nativeAppBridgeSetInitParams` (guest `0x102bcc814`) — MainGameActivity init-params
4. [`--v2boot-session-resumed`, ambulatory-only] `nativeOnResumed` — see "measured" below

Then the normal ladder (nativeInitializeNativeFlags → … → StartLuaAppDM → app-start) runs untouched.

Measure (next/available): `MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY` + AppBridgeV2 singleton
`[0x106a705e8]`.

Hermetic `sh264_activity_lifecycle_natives_jni_receive_pinned` (real-image guard family as
sh260/261, skip-if-absent) byte-pins the four prologues + setActive's adapter-triplet read
(`0x1021f5f80`=0xd00248a9, `0x1021f5f88`=0xa940252a), so a drift breaks loudly.

Repro: `runs/capture_sh264_session_drive.sh` (SH259 full seed set + `--v2boot-session`).
Log: `runs/sh264-session-drive.txt`.

## Measured (real libroblox.so, bounded run)

- **`initAppShellReporter` and `setActive` each RETURN `Ok(0x0)` cleanly** — the first time these
  lifecycle natives have ever executed headlessly (both were previously entirely undriven, not
  even reachable). The SH248f adapter-triplet seed makes setActive's core take the benign path.
- **`SetInitParams`** (guest 0x102bcc814) soft-returns `run_loop: pc 0x106eda000b8` outside image —
  the same singleton-dispatch/soft-return class the ladder already handles (SH111/115 family), NOT
  a new crash. `MH_FLAGS_LOADED=false MH_ENGINE_INITIALIZED=false MH_APP_READY=false`,
  `AppBridgeV2[0x106a705e8]=0x0` after — no milestone, no singleton, consistent with AppBridgeV2
  being a later-stage object.
- **`nativeOnResumed` is a MEASURED dead-end as driven** (ambulatory, opt-in): it tail-branches
  into the SHARED Activity lifecycle-notifier dispatcher `0x21f15a4` whose body derefs a *real*
  lifecycle-callback-registry object at `+0x50` — `SIGSEGV guestpc=0x1021f3748 fault=0x50`, then
  SIGABRT. That is the SH184 live-object/registry class, NOT a seedable cell. Kept behind
  `--v2boot-session-resumed` so it never breaks the default clean session drive.
- **No regression to the app-start line:** with `--v2boot-session` on, the full ladder still runs
  (nativeInitializeNativeFlags → GlobalInit Ok → UpdateAdapterInit Ok → setTaskSchedulerBM Ok →
  V2Init soft-return → StartLuaAppDM), the app-start self-drive still walks **93 distinct region
  pcs**, and the terminal is the SAME parked wall `guestpc=0x101db1d04` (LocalStorageManager
  insert-leaf, SH260) — the lifecycle drive neither advances nor regresses the standing Route-B
  live-DM gate.

## Honest verdict

The SEP-17 session-drive workstream is now **wired + measured-latent-but-correct**: the three
non-aborting lifecycle natives run as real engine session-init entries (initAppShellReporter +
setActive both complete Ok) and are regression-safe. It does **NOT** manufacture a DataModel;
Route-B live-DM structural gate UNCHANGED (the upstream session ctor still doesn't run far enough
to build a DM world). The named final levers — nativeActivity_onEngineSettingsReceived/client-
settings feed into initEngine_, and the messageBus **experience-launch** + dataModel-bindings
receive entries — remain un-driven; they are the honest next candidates on this line.

recon-v3 plane re-verified green at HEAD (`capture_taskv4_frame.sh`: **24** task-driven frames,
present #19..#23 swap Ok(0x1), 197 node pops, 0 json abort, 0 crash, EXIT 124). Workspace green
(`cargo build --workspace` EXIT 0; `cargo test --workspace` 0 failed; arm64jit examples **100/0**).
SH174 capture-latch stays the single forward hook. Single-agent, default-inert, no production path
edited.