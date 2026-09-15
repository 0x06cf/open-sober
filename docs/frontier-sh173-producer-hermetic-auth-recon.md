# SH173 — Session-producer dispatch HERMETIC proof + auth/R1/render-seam recon corrections

Author: hermes-worker (autonomous loop) + SH171+1 cone (deleg_d585254f task-0/1/2),
Sep 15 2026. Status: implemented (1 code change + hermetic test) + recon
corrections consolidated. Companion ledger: STATUS.md, HANDOFF.md.

## What SH173 implements (crates/arm64jit/src/jit.rs)

**Hermetic test `session_producer_push_epoch_bump_wake_dispatch`** proves the
session-producer handoff's host-side dispatch mechanism WITHOUT a live engine
drain — the piece the operator's "session-producer handoff" spec depends on but
no prior test exercised (recon deleg_d585254f task-0: producer is
implemented-latent, only ever exercised with a running drain).

The test mirrors the exact real producer logic (`elfjit.rs --deque-node-bump`:
write node into the headcell, `[Q] = [Q] + 0x1_0000_0000` high-32-only epoch
bump, `syscall(SYS_futex, Q+8, FUTEX_WAKE, 1)`) against a fake consumer that
parks on the futex latch and re-reads epoch + pops the headcell after waking.
Robust to a lost wake (short-timeout wait-loop re-poll; never hangs — verified
deterministic). Two real futex threads; 0 guest/jit_run.

**Verify markers:** `test jit::...::session_producer_push_epoch_bump_wake_dispatch ... ok`
- consumer woke and popped the pushed node from the headcell
- consumer observed the epoch increment (high-32 0->1) after wake

## Consistency with the operator spec

The cone confirmed the producer push-epoch-wake mechanism IS the real one in
elfjit.rs:7627-7635 (`--deque-node-bump`) and 7660-8067 (`--deque-node-live`,
which does NOT bump epoch — it relies on the drain's finite-timeout poll). It
is `implemented-latent`: it reaches a real node only with a live engine
drainer; MH_APP_READY gating of the producer is spec-only (read as a
diagnostic at elfjit.rs:6893/6964/7034, never as a producer gate). This test
makes the host-side mechanism provable headlessly; the guest-drain side still
needs a live session (migration gate unchanged).

## Recon corrections (same cone, READ-ONLY)

1. **AUTH STRING CORRECTED:** the login-persistence artifact is the
   **`.ROBLOSECURITY`** cookie (SH129/earlier recon had it mis-typed
   `.ROBLESECURITY` — confirmed `.ROBLOSECURITY` + `.ROBLOSECURITY=` + the
   `[FLog::WebLoginProtocol] Found .ROBLOSECURITY cookie value` log). The disk
   artifact is NOT any native .so file: zero references to session.db /
   shared_prefs / app_webview / cookies.db / cookies.sqlite / SyncUtil in the
   lib; rbx-storage.db is pure content cache. Auth persists via the JAVA
   universalapp cookie manager: restored on boot by
   `JNICookieManager_setCookiesFromDisk` (JNI 0x2bcbebc) -> injected into the
   native cookie jar via `NativeSettingsInterface_nativeSetMultipleCookies`
   (0x2202ff8) -> SessionService/UserInfo ->
   `initializeLuaAppWithLoggedInUser`. The cookie-write+inject is reachable
   BEFORE a live DataModel; turning it into an engine-visible restored session
   is behind the migration gate. SH168's prestage seeds the WRONG artifact for
   auth (cache/rbx-storage.db = content cache + empty dirs; no cookie store) —
   its own docstring says auth is an independent plane it does not touch.
2. **R1 render-seam recon (task-2, HARD-CONFIRMED):** the present-walker
   0x105b2ed48 node loop has ZERO DataModel/ScriptContext/Luau-VM gate — it is
   gated only by host-touchable bytes (R+0x22f blit flag, R+0x288 lazy init,
   R+0x298/R+0x2a0 node-present trio, R+0x260 present-enable) and the
   R+0x180/0x188 0x28-stride node list. So engine RENDERING of fabricated scene
   nodes is host-seedable NOW. But this is the ALREADY-DONE Route-A render
   plane (SH152/153/154 film: harness fabricates nodes, engine's own GLES
   draws them) — NOT Route-B self-construction. Engine SELF-MOUNTING of real
   GuiObject nodes (the R+0x180 list being fed by real GuiObjects via the
   type-4 vector) still needs a live DataModel + LuaVM + ScriptContext +
   CoreScriptLoader. Per the operator directive, do NOT polish Route-A at
   Route-B's expense.
3. **Type-4 vector / producer gating (task-0):** `[0x106829ea8]` stays
   host-install-only (--taskv4-seed); the session-producer handoff is
   latent-but-correct and fires the instant a real session advances (MH_APP_READY
   stays false headlessly). Do not re-derive an in-image install site (SH46/52
   sealed it: only external glue installs it).

## Standing (unchanged)

Live-DM = MIGRATION GATE (~29 recon angles). Workspace green (546/0; arm64jit 368).