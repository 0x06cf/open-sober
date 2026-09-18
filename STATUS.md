# Open-Sober run state (hermes-worker)

## HEAD: `dev` branch, SH323 (advance the SH320/321 MAIN-path engine-settings-init line two more
## fenceposts + cross a SECOND lifecycle-notify copy; five consecutive crossings since SH321:
## 0x1021f3748 -> 0x1021f5078 -> 0x1025f370c -> 0x102256510). STATUS candidate (a), continued.

**State**: `cargo test --workspace` green (0 fail): arm64jit lib 408/0 + elfjit examples 148/0
(147 + sh323) + fsmap + others. `cargo build` EXIT 0. Commit <FILL> on local `dev` (not pushed;
operator pushes). elfjit.rs 48 B under the 1MB hook.

## This session (SH323)

1. Recon-v3 render plane intact (headless render plane, not touched this cycle).
2. **SH323** — extended the SH320/321 MAIN-path engine-settings-init advance. New default-inert
   guard JIT_ROUTEB_SETTINGS_SSO_SEED seeds two NULL cookie/string globals with an empty SSO
   string: CELL_A [0x106ed7a18] (fn 0x21f5078 whitespace-check, clears the 0x1021f5078 wall) and
   CELL_B [0x106ed7a28] (StartAppWithParams `bl 0x221364c` -> ldrb [x0] @0x1025f370c, seeded at the
   confirmed block entry 0x1025f36ac). SH322's early-ret guard now also covers the second
   lifecycle-notify copy 0x1021f4538. A/B (real so): BASELINE first SIGSEGV 0x1021f5078; FORWARD
   seed=2, wall gone, terminal advances to 0x102256510 (fault=0x0). Five consecutive crossings on
   the settings-init line.
3. arm64jit lib 407 -> 408/0; elfjit examples 147 -> 148/0; workspace green; elfjit.rs 48 B under 1MB.

## Standing (honest, unchanged)

- **Route-B live-DM structural gate UNCHANGED**: no make_shared<DataModel> fires headlessly; DM-root
  [0x106a68818] stays 0; MH_APP_READY stays false. The MAIN-path engine-settings-init line now
  advances five fenceposts past SH321's SH273 wall but still terminates at the live-object class
  (0x102256510, a JNI-receive entered with NULL this) — a SESSION-CTOR cave, not a DM.
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