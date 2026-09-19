# Frontier SH410 — complete the NativeHelper lifecycle surface: drive the login-vs-home gate `onDidLogInReceived` with a real login payload, steering a fresh headless session to its OWN login screen

Date: 2026-09-19, hermes-worker, single-agent (cone suppressed). Workspace green
before and after (cargo test --workspace EXIT 0; arm64jit lib 457/0 — the new
login-state assertions live in the existing session::tests hermetic, so the count
is unchanged). Real-binary MEASURED on libroblox.so (capture_sh400 env, EXIT 124,
0 crash). Production code only in jni.rs + session.rs (both far under the 1MiB
hooks); jit.rs/elfjit.rs untouched.

## Why this cycle

SH409 shipped the SH400 substrate's missing "onAppReady actually DRIVEN" host
surface: `fire_nativehelper_milestone` drives onFlagsLoaded -> onEngineInitialized ->
onAppReady through the SAME registered slot-61 CallVoidMethod shim, and MEASURED
MH_APP_READY latching on the real binary. But SH409 drove only 3 of the 5
NativeHelper `gameActivity_*` callbacks. The trigger-map (docs/trigger-map-
nativehelper-callbacks.json) lists all five as `reachable_today: false`; the one
that gates LOGIN-vs-HOME — `onDidLogInReceived`, VOID-with-String `(Ljava/lang/String;)V`
(method-name 0x50a545, disasm deleg_2e852a9c) — was a no-op eprintln with no login
state. The operator's higher-level direction names the LOGIN screen as the first
real screen the engine must render ("the login renders is exactly the engine
drawing its own login UI"). SH410 makes that discriminator read a real host signal.

## What landed

- jni.rs:
  - two new observables: `MH_LOGIN_RECEIVED` (the callback arrived) and
    `MH_LOGGED_IN` (empty login payload = NOT logged in), + getters
    `nativehelper_login_received()` / `nativehelper_logged_in()`.
  - `jni_call_void_method` now, on onDidLogInReceived, reads the a3 login-payload
    jstring (a readable UTF-8 buffer from new_string_utf_handle) and sets both
    observables: empty payload -> logged_in=false -> LOGIN screen; non-empty
    payload -> remembered sign-in -> HOME.
  - new `fire_nativehelper_login_payload(payload)` — fires the callback through the
    SAME registered shim with a real payload jstring in a3 (mirrors
    fire_nativehelper_milestone).
- session.rs: `drive_nativehelper_lifecycle()` (wired into the substrate right
  after the surface atom, recon-routeB step-2 position) now, after onAppReady,
  fires onDidLogInReceived with an EMPTY payload — the honest host signal for a
  fresh headless session with no persisted credential -> steer to LOGIN.
- Hermetic: `lifecycle_milestones_driven_in_order` now also asserts login_received
  AND NOT logged_in (empty payload -> login, not home).

## MEASURED (real libroblox.so, SH400 capture env, EXIT 124 / 0 crash)

```
[jni:nativehelper] onDidLogInReceived (0B payload) -> logged_in=false; login screen
[session-drive] lifecycle milestone 'gameActivity_onDidLogInReceived' fired (ret=0); MH_LOGIN_RECEIVED=true MH_LOGGED_IN=false -> login screen
[session-drive] substrate complete: 11/16 atoms returned non-zero Ok
```

The full lifecycle surface now drives ALL FIVE milestone names through the engine's
own registered shim: onFlagsLoaded -> onEngineInitialized -> onAppReady ->
onDidLogInReceived (login-vs-home). MH_LOGIN_RECEIVED latches, MH_LOGGED_IN=false,
and the run stays stable (11/16 Ok, EXIT 124, 0 crash). AppBridgeV2 stays at the
genuine relocated in-image vt 0x1063a3410.

## Honest

This is the host-side login-state SURFACE, not a DataModel manufacture. DM-root
[0x106a68818] stays 0, MH_GAME_LOADED false, no make_shared<DataModel>. Firing
onDidLogInReceived alone does not boot Lua (a completed do-init still owns the
live DM — SH405). What IS real and newly shipped: the login-vs-home discriminator
now reads a genuine host signal (the login payload delivered by the drive) and
steers a fresh session to the LOGIN screen — the exact "login renders" first
screen the operator's direction names — as a concrete, testable, shippable piece
of the SEP-18 BUILD-THE-RUNTIME deliverable. Route-B live-DM structural gate
UNCHANGED. No re-treads: no LSM skips, no map manufacture, no setDataModelToCurrent,
no single-object DM seeds.

## Files

- crates/arm64jit/src/jni.rs: +2 observables + login-payload dispatch + 
  `fire_nativehelper_login_payload` (off-hook).
- crates/arm64jit/src/session.rs: drive_nativehelper_lifecycle fires
  onDidLogInReceived (empty payload) + hermetic assertions (off-hook).
- Real-binary capture: /home/hermes-worker/runs/capture_sh400_session_substrate_drive.sh
  (same env; log /home/hermes-worker/runs/sh410-baseline-login.txt outside repo).