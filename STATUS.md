# Open-Sober run state (hermes-worker)

## HEAD: `dev` branch, SH327 (force AppStarted factory construction DETERMINISTIC; app-start
## fencepost 0x1025f5300 advances one level deeper to the V2StartAppWithParams params-obj +0x140
## member). SEP-17 "force the construction branch" forward (SH326c premise reversal -> fix).

**State**: `cargo test --workspace` green (0 fail): arm64jit lib 408/0 + elfjit examples 151/0
(150 + sh327) + fsmap + others. `cargo build` EXIT 0. Commit on local `dev` (not pushed; operator
pushes). elfjit.rs 31 B under the 1MB hook (condensed SH-prose to fit).

## This session (SH327)

1. Recon-v3 render plane intact (untouched this cycle).
2. **SH327** — deterministic AppStarted construction. `routeb_patch_appstart_construct_force` patches
   0x2e890f4 `b.ne 0x2e89118` (0x54000121) -> unconditional `b 0x2e89118` (0x14000009), removing the
   factory 0x2e890c4's sole non-construction exit (producer-counter tag==2 -> unconstructed ret). Idempotent
   (short-circuits [x19,#24] @0x2e89130). Gated on JIT_ROUTEB_DM_SEED/DMFORCE (inert by default).
3. MEASURED (3/3 dual-PC dump): construction write fires + lands (`[appstart0x106a6f480]=0x55cb343ab000`,
   real heap obj, now DETERMINISTIC); field-copy helper 0x25f54e8 completes + returns; fault ADVANCES to
   `ldr w3,[x20,#320]` @0x25f5328 (fault=0x140), x20 = V2StartAppWithParams params obj = 0.
4. arm64jit lib 408/0; elfjit examples 150 -> 151/0; +sh327 guard (9 pins); +probe_sh327_construct_force.sh.

## Standing (honest, unchanged)

- **Route-B live-DM structural gate UNCHANGED**: no make_shared<DataModel> fires headlessly; DM-root
  [0x106a68818] stays 0; MH_APP_READY stays false.
- NEW: app-start fencepost is deterministic-constructed; standing wall = V2StartAppWithParams
  params-obj +0x140 member (x0/x20 param of fn 0x25f5270 = 0).
- Everything achievable headlessly (llvmpipe); GPU host for performance later.

## Next-forward candidates

(a) The 0x102256510 terminal (JNI-receive fn reading [x0] with x0=0). Determine its caller/arg
    contract — if x0 is a real object the session should pass, this is the SESSION-CTOR drive; if
    it is a NULL-able callback, a benign early-ret may clear it (SH322/323 pattern).
(b) R1 content path (synthetic CoreScript module) latent until a live DM requests rbxasset://.
(c) SH304 session-gated producer fires the instant a real session owns a live DM.
(d) Re-examine SESSION-CTOR Activity-session lifecycle drive (nativeActivity_onEngineSettingsReceived
    + real surface) to construct the lifecycle registry for real.

## Do-not-re-tread (this session)

- "The SH273/settings-init walls are a single closed live-object wall" — SH322+SH323 prove the
  SH320/321 MAIN-path line crosses FIVE consecutive benign seeds (two lifecycle-notify early-rets
  + two cookie/string globals) before reaching the live-object class at 0x102256510.
- Prior do-not-re-tread list (SH314-322, SH302-306, SH251/255/260 etc.) unchanged.