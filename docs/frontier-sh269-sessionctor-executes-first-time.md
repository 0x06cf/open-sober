# Frontier SH269 — post-ladder SESSION-CTOR rungs execute headlessly for the FIRST time (skip-appstart lever)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed — no subagents).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. New default-inert elfjit flag + 0 production-path changes in the JIT
library (one opt-in seed guard added, fires only under env). Workspace green
(cargo build + cargo test --workspace exit 0).

## The discovery (why SH264-267 all measured the session rungs "latent")

The SEP-17 SESSION-CTOR directive names the post-ladder session rungs as the
PRIMARY lever — `--v2boot-send-appevent` (SendAppEventOnAppReady 0x102bb463c),
`--v2boot-send-game-loaded` (SendAppEventOnGameLoaded 0x102bb429c), and
`--v2boot-session-bus` (MessageBus.subscribe 0x102ba5bb8). SH264-268 added and wired
them — but every SH265/266 measurement reported "latent": the `--v2boot` ladder's
StartLuaAppDM + V2StartAppWithParams rungs SELF-DRIVE DEEP into app-start via the
DMCONT continuation, and terminate the PROCESS (SIGABRT after the LSM/map SIGSEGV)
BEFORE the loop reaches the post-ladder session rungs. So the session rungs — the
whole point of the SEP-17 cause-not-symptom workstream — had NEVER executed headlessly.

## The lever (opt-in `--v2boot-skip-appstart`)

`--v2boot-skip-appstart` skips the two app-start self-driver rungs
(StartLuaAppDM 0x1023efe2c, V2StartAppWithParams 0x10258b144) and the V1
AppStart__ fallback (0x102338510) in the loom loop, so the loop COMPLETES and the
post-ladder session rungs are reached on a non-crashing path. Default (flag absent)
= exact prior behavior (unregressed). This is the A/B side-lever that isolates
whether the session rungs can advance independent of the app-start crash.

## MEASURED (real libroblox.so, full SH267 seed set, deterministic 3/3)

`--v2boot-skip-appstart --v2boot-session-bus`:
- **MessageBus.subscribe returns Ok(0x3e8)** — the FIRST headless execution of a
  post-ladder session-ctor rung, EXIT 124 (stable idle loop), 3/3 deterministic.
- **once-guard [0x6a68410] latched to 0x1** — the do-init's __call_once RAN via the
  REAL session drive (subscribe -> the app-bridge pipe), the first time the
  once-lambda executed from the session path. DM-root [0x106a68818] stays 0x0
  (the do-init's lambda still does not populate a valid DM controller — the
  SH174/SH204 live-DM structural gate, UNCHANGED; no DataModel manufactured).
- MH_FLAGS_LOADED/APP_READY stay false (no live DM to load/app).

`--v2boot-skip-appstart --v2boot-send-game-loaded`: OnGameLoaded rung reached,
EXIT 124 clean (no crash, body self-terminates in the idle loop).

`--v2boot-skip-appstart --v2boot-send-appevent`: SendAppEventOnAppReady now EXECUTES
and drives the governor predicate. Baseline (no govflag seed): SIGSEGV guestpc=
0x102ea0b9c fault=0x0 (governor predicate `ldr x8,[x0]` where x0=[app-governor+1032]
= the app-DM controller = 0). With the new opt-in seed (JIT_ROUTEB_APPSART_GOVFLAG,
flags byte [0x106a64da0] OR bit0): governor NULL-deref GONE, SendAppEventOnAppReady
advances ONE fencepost deeper to SIGSEGV guestpc=0x102bb803c fault=0x0 (x20 = return
of nativePreloadFlagOverrides 0x2dae640 = 0) — the preload-overrides object
[0x106a64d98] is a never-constructed live object (SH174/SH204 class). So the govflag
seed is a ONE-GATE forward seed (advances the session drive past the governor
NULL-controller), then dies at the preload-overrides live-object.

## Honest (do-not-over-claim)
- Does NOT manufacture a DataModel; DM-root stays 0; Route-B live-DM structural gate
  UNCHANGED; MH_* stay false. The session rungs STILL cannot boot Lua (no live DM).
- What IS genuinely new + measured: **the post-ladder session-ctor rungs now EXECUTE
  headlessly for the first time** (MessageBus.subscribe returns Ok(0x3e8), and the
  do-init once-lambda runs via the real session drive, latching once-guard=1). This
  converts SH264-267's "latent" (static judgment) into "executed + measured" at the
  session path — the cause-not-symptom line the SEP-17 directive prioritizes. It is
  the first real advancement OF the session drive, distinct from seed-into-app-start.
- The govflag seed advances SendAppEventOnAppReady one fencepost (governor crossed ->
  preload-overrides live-object), a measured single-gate forward seed, then parks at
  the standing live-object class.

## Code / verify / artefacts
- elfjit.rs: `--v2boot-skip-appstart` (skips the 3 app-start self-drivers in the
  ladder loop + V1 fallback) + direct GOVFLAG byte seed before SendAppEventOnAppReady
  (under JIT_ROUTEB_APPSART_GOVFLAG; the governor block is already JIT-cached so the
  in-crate entry guard can't re-fire mid-session).
- jit.rs: `routeb_govflag_seed_guard` (opt-in JIT_ROUTEB_APPSART_GOVFLAG), wiring in
  the block-entry dispatch.
- Verify: `cargo build --workspace` + `cargo test --workspace` exit 0.
- Repro: `runs/capture_sh269_session_ctor_exec.sh` (MessageBus 3x + OnGameLoaded +
  OnAppReady A/B with/without GOVFLAG).
- Commit: local `dev` only.