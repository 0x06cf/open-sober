# SH174 recon: V1 AppStart fallback = terminal no-op + cookie-ingress reconfirmed behind migration wall

Author: hermes-worker SH174 (recon cone deleg_a092dbb4, both tasks READ-ONLY).
Scope: two headless open lines resolved — the canonical ladder's LAST rung (V1
AppStart__ fallback) and the .ROBLOSECURITY cookie-ingress ("remember sign-in").
Both reconfirm the standing live-DM migration gate; no production code change.

## Task-1: V1 AppStart__ (fallback) rung = CONFIRMED TERMINAL NO-OP for the UI goal

- The "driving V1 AppStart__ (fallback)" rung (elfjit.rs:6838) drives guest
  **0x102338510** = the V1 JNI entry `nativeAppBridgeAppStart__` (file 0x2338510)
  with the CORRECTED 6-arg ABI (x0=env, x1=thiz, x2/x3/x5/x6/x7 = 5 empty
  jstrings, x4 = jboolean false; pinned by the `v1_app_start_six_arg_abi...`
  regression, jni.rs:1923-1949). It is the fallback vs V2 because V1 reads params
  as individual jstrings (no AutoValue jobject / no Call*Method getter), so the
  SH56 params-collapse json-abort can never fire on it.
- Chain: continueAfterFlagsLoaded_ (0x102bd1d68) -> nativeAppBridgeAppStart__
  (0x102338510) -> AppStart governor (0x2338ef4). The closure is
  lifecycle/telemetry/platform-init ONLY (FastLog, base-url, JNIAppLifecycle
  setActive, SendAppEvent/MessageBus, cookie/web-login/crashpad/headers/
  LocalStorage, refcount stubs) — NEVER a DM factory, ZERO GuiObjects, no session
  node (frontier-sh167-dm-alloc-capture.md:13-22; frontier-sh59-v1-appstart.md:51-54).
- It already benign-completes Ok. Reaching it was a headless ceiling, NOT an
  under-driven rung. **Action: MOVE ON — no further seeding of the V1/
  AppBridge-manager line.** Any deeper session needs the migration gate.

## Task-2: .ROBLOSECURITY cookie ingress = RECONFIRMED behind the SAME jar-init wall

- Login persistence = the `.ROBLOSECURITY` cookie; native ingress chain:
  `JNICookieManager_setCookiesFromDisk` (0x2bcbebc) -> `nativeSetMultipleCookies`
  (0x102202ff8, 4-arg JNI thunk) -> pure-native worker 0x102203148 ->
  SessionService -> initializeLuaAppWithLoggedInUser.
- Progress since SH129: **gate 1 [0x72739d4].bit0 is now cleared FOR FREE** — the
  current ladder rung-0 (nativeInitializeNativeFlags 0x10232048c) drives the real
  engine chain that writes `strb #1,[0x72739d4]` (elfjit.rs:6427-6433). The cookie
  worker's gate-1 read (0x22031b0) now passes with no extra seed.
- Still blocked: **gate 2 [0x6dcfc30].bit0 is NOT seeded** (zero hits under
  crates/), the SH129 --cookie-ingress driver was REVERTED (no driver exists), and
  decisively the **cookie-jar-CONSTRUCTION NULL at 0x21fce24 is UNCHANGED** — even
  with both latches forced, the worker derefs the UNBUILT HttpCookieProtocol jar
  container and SIGSEGVs 0x10220331c (fault=0x0). The jar is only built by a real
  app-launch (fuller engine init) the headless ladder never reaches.
- **Verdict: NOT drivable headlessly.** Same structural live-DM/jar-init migration
  wall — NOT a new gate. Only forward = a real GPU-host app-launch session builds
  the jar, then re-drive the two pinned workers (0x102202ff8 -> 0x102203148).

## Standing (unchanged, reconfirmed this session)
live-DM = MIGRATION GATE. No headless .bss/.data seed constructs one (~30 recon
angles across SH167-174). SH167/SH169/SH174 DM allocation-capture latch is the
ready catch at migration; docs/frontier-sh174-migration-runbook.md is the
GPU-host runbook (address-audited CONFIRMED). Workspace green 370/0.