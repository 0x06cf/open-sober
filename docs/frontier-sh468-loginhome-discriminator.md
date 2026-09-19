# Frontier SH468 — pin the FULL login-vs-home discriminator through the real NativeHelper dispatch

Worker: hermes-worker · date 2026-09-20 · workspace green (860/0; arm64jit lib
679/0 incl. 1 new sh468 hermetic). Production code UNCHANGED (test-only
addition to crates/arm64jit/src/jni.rs).

## Gap closed
recon-routeB-globaltinit-unblock.md's NativeHelper contract lists 5 ordered
`gameActivity_*` callbacks (onFlagsLoaded -> onEngineInitialized -> onAppReady
-> onDidLogInReceived -> onGameLoaded) as the "missing harness surface" that
lets StartLuaAppDM advance its session. The milestones were hermetically pinned
(session.rs lifecycle_milestones_driven_in_order + jni.rs
nativehelper_game_activity_callbacks_dispatch_void_method), but the **login-vs-
home gate itself was only half-pinned**:
- session.rs pinned the EMPTY-payload -> LOGIN direction (fresh headless
  session, no persisted credential).
- jni.rs's dispatch test only asserted onDidLogInReceived "returns void, does
  not fault" — it passed a3=0 and NEVER asserted that a non-empty payload flips
  MH_LOGGED_IN.

So the remembered-sign-in / HOME branch — the "stay logged in" path the real
app reads from its datastore — had no dispatch-level test; a regression that
broke the HOME direction (payload string ignored, or logged_in always false)
would pass the whole suite. That's exactly the "only non-trivial ABI gap"
recon calls out (VOID-with-String, discriminator reads [sp] payload).

## What was added
jni.rs test `login_payload_discriminates_login_vs_home_through_dispatch`:
1. EMPTY payload (b"") -> MH_LOGIN_RECEIVED=1, MH_LOGGED_IN=0 -> LOGIN
   (the fresh-session first screen).
2. NON-EMPTY payload (b".ROBLOSECURITY=abc123") -> MH_LOGGED_IN=1 -> HOME
   (the remembered-sign-in path). Pins that the DISPATCH really carries the
   payload string into read_cstr(a3) and flips the latch the session reads.
3. Reads the final host discriminator (`logged_in? -> home/login`) resolve to
   HOME exactly — the same expression drive_nativehelper_lifecycle logs.

Public API exercised: `fire_nativehelper_login_payload(&[u8])` -> the same
`jni_call_void_method` shim (slot 61) the engine uses. Pure host logic; no real
binary, no env, parallel-safe.

## Why not a re-tread
The prior two tests each covered ONE direction (session empty-login; jni
"void returns 0"). Neither asserted the HOME branch through the real dispatch.
This closes the discriminator's full truth table at the dispatch level.
Production code untouched (deterministic, byte-identical runtime).

## Honest status
NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root
[0x106a68818]=0). This is BUILD-THE-RUNTIME session-surface contract
completion: the login-vs-home READOUT a real session's onDidLogInReceived
creates is now a tested contract in both directions, so the "login renders
first, home after remembered sign-in" split the operator named is pinned.
Files: docs/frontier-sh468-loginhome-discriminator.md + crates/arm64jit/src/
jni.rs (test-only). Commit (SH468).