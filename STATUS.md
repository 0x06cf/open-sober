# Open-Sober run state (hermes-worker)

## HEAD: `dev` branch, SH322 (CROSS the SH273 lifecycle wall the SH320/321 MAIN-path reaches:
## engine settings-init now advances past it — first forward on STATUS candidate (a)). The do-init
## MAIN binder-dispatch climbs into nativePostClientSettingsLoadedInitialization3 (caller 0x2270024
## -> bl 0x21f3748) and SH321 measured fault=0x50 at the SH273 lifecycle-notify body. SH322 JIT
## seed (JIT_ROUTEB_LIFECYCLE_EARLYRET=1) makes fn 0x21f3748's tbnz w8,#1 @0x21f3774 -> 0x21f3870
## take the epilogue canary-check+ret no-op -> SIGSEGV that was at 0x1021f3748 moves forward to
## 0x1021f5078 (fault=0x0, global std::string [0x106ed7a18] = SH248d-class .bss, seedable).

**State**: `cargo test --workspace` green (0 fail): arm64jit lib 407/0 + elfjit examples 147/0
(146 + sh322) + fsmap + others. `cargo build` EXIT 0. Commit b0e2ac3 on local `dev` (not pushed;
operator pushes). elfjit.rs 25 B under the 1MB hook.

## This session (SH322)

1. Recon-v3 render plane intact (headless render plane, not touched this cycle).
2. **SH322** — crossed the SH273 lifecycle wall on the SH320/321 MAIN-path (STATUS candidate (a)).
   The do-init DONE-path MAIN binder-dispatch (0x206df4->vt+0x30->br x1) climbs into real engine
   settings-init `nativePostClientSettingsLoadedInitialization3` (caller 0x2270024->0x2270050
   bl 0x21f3748) and SH321 measured fault=0x50 at the SH273 lifecycle-notify body 0x21f3748
   (`ldrb [x8,#80]` on arg0 [x1]==0). SH322 (block-entry guard, JIT_ROUTEB_LIFECYCLE_EARLYRET=1)
   seeds the caller pair [x1]=obj with byte[+80].bit1=1 so fn 0x21f3748's `tbnz w8,#1` @0x21f3774
   jumps to 0x21f3870 (epilogue canary-check+ret, benign no-op) — bypassing the registry-build.
   A/B (real so): BASELINE EXIT 139 SIGSEGV @0x1021f3748; FORWARD guard fires 4x, wall GONE, first
   SIGSEGV advances to 0x1021f5078 (fault=0x0, EXIT 134). New terminal reads global std::string
   [0x106ed7a18] (below the SH248d cookie-jar globals) — seedable (SH323 natural next).
3. arm64jit lib 406 -> 407/0; elfjit examples 146 -> 147/0; workspace green; elfjit.rs 25 B under 1MB.

## Standing (honest, unchanged)

- **Route-B live-DM structural gate UNCHANGED**: no make_shared<DataModel> fires headlessly; DM-root
  [0x106a68818] stays 0; MH_APP_READY stays false. SH322 advances the engine-settings-init line one
  fencepost past the SH273 lifecycle wall, but it still dies at the SH248d-class live-object wall
  ([0x106ed7a18] .bss std::string) — a SESSION-CTOR cave, not a DM.
- Everything achievable headlessly (llvmpipe); GPU host for performance later.

## Next-forward candidates

(a) Seed the SH322 new terminal's global std::string [0x106ed7a18] = empty SSO string
    (routeb_empty_sso_string, the SH248d helper) so the whitespace-check fn at 0x1021f5078 also
    completes, then chase the next fencepost on the same engine-settings-init line.
(b) R1 content path (synthetic CoreScript module) latent until a live DM requests rbxasset://.
(c) SH304 session-gated producer fires the instant a real session owns a live DM.
(d) Re-examine SESSION-CTOR Activity-session lifecycle drive (nativeActivity_onEngineSettingsReceived
    + real surface) to construct the lifecycle registry for real.

## Do-not-re-tread (this session)

- "The SH273 lifecycle wall is a single unreachable/closed live-object wall with no lever" — SH322
  proves it is CROSSABLE via a benign no-op seed (tbnz -> canary-check+ret) on the SH320/321
  MAIN-path engine-settings-init route.
- Prior do-not-re-tread list (SH314-321, SH302-306, SH251/255/260 etc.) unchanged.