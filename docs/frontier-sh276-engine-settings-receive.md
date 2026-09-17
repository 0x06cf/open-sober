# SH276 — drive nativeActivity_onEngineSettingsReceived (the SEP-17 directive's OTHER named engine-settings primitive)

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · opt-in `--v2boot-session-engine` (default-inert) · workspace green

## What this is

SH275 fed the CLIENT-settings input (`nativeInitClientSettingsSigned`) that `initEngine_` consumes, but the
SEP-17 directive's *other* named primitive — the engine-settings **receive** `nativeActivity_onEngineSettingsReceived`
(guest `0x2bd1c38`, file `0x2bd1c38`+`0x1_0000_0000`) — was never driven. SH264-275 drove lifecycle natives
(initAppShellReporter/setActive/SetInitParams) + both client-settings variants + the messageBus/dataModel-bindings
receives, but not this engine-settings state-transition on the NativeDataModelManager instance.

## Location (fresh `aarch64-linux-gnu-objdump` on real libroblox.so)

- Entry `0x2bd1c38` = `sub sp,#0x40 ; stp x29,x30,[sp,#32]` (its FLog `[FLog::NativeDM] nativeActivity_onEngineSettingsReceived:`
  sits at rodata `0x4997a7`, referenced by the version-gate log `adrp x3,499000; add x3,x3,#0x7a7` @ `0x2bd1c7c`).
- Body (both branches fall through to the same state transition):
  - reads version word `[adrp 0x683d000 + #2296]` = `[0x10683d8f8]`; `and w9,w8,#0xff; and x10,x8,#0xfc00; cmp w9,#0x6; ccmp ...; b.eq <main>`.
  - if NOT version 6: logs the FLog (debug/legacy path), then falls through.
  - `add x0,x19,#0x14; bl 2b53a68` → **pthread_mutex_lock** on `[this+0x14]`.
  - `ldrb w8,[x19,#649]; mov w9,#1; strb w9,[x19,#648]` → LATCH `[this+648]=1` (engine-settings-received),
    and `cbz w8` → if `[this+649]!=0` also `str w8,[x19,#16]` (state → 3).
  - `add x0,x19,#0x14; bl 2b53abc` → **pthread_mutex_unlock**; `ret` (stack-check).

A ZEROED `pthread_mutex_t` is the static initializer, so lock/unlock on a freshly zeroed manager returns immediately
single-threaded; a leaked zeroed `0x800` buffer keeps `[this+648]/[this+649]` in-bounds.

## What landed (elfjit.rs `--v2boot-session-engine`, single ladder thread)

Seeds the version word `[0x10683d8f8]=6` (this method's own read), then drives `0x102bd1c38` as a real guest
`jit_run` on the same single ladder thread (SH55/64 serialized discipline), `this` = a fabricated zeroed `0x800`
manager, reusing `boot_sp`/`tpidr`. Reports the return + post-run `[this+648]` (engine-settings-received latch),
`[this+16]` (state), MH_* flags, and dumps `[0x106829ea8]`.

Hermetic `sh276_engine_settings_receive_transition_pinned` (real-image guard, skip-if-absent): pins the receive
prologue (`0x102bd1c38`=0xd10103ff, `+4`=0xa9027bfd), version-word decode (`0x102bd1c5c` adrp 683d000,
`0x102bd1c60` ldr [x8,#2296]→[0x10683d8f8], `0x102bd1c6c` cmp w9,#0x6, `0x102bd1c74` b.eq), FLog-0x4997a7 adrp
`0x102bd1c7c`, mutex-lock `bl 2b53a68`@0x2bd1ca8, the latch `strb w1,[this,#0x288]`@0x102bd1cb4, state store
`str w8,[this,#16]`@0x102bd1cc0, mutex-unlock `bl 2b53abc`@0x2bd1cc8.

## Measured (real libroblox.so, SH269 full seed set + `--v2boot-skip-appstart` + `--v2boot-session-engine`, 2-run batch)

**Deterministic 2/2**: `EngSettingsReceived returned Ok(0x0): [this+648](engine-settings-received)=1 [this+16](state)=0
MH_FLAGS_LOADED=false MH_APP_READY=false`, EXIT 124 clean. The SEP-17-named engine-settings receive executes
headlessly for the FIRST time and latches its engine-settings-received flag on the fabricated manager. No crash,
no regression to the app-start line (once-guard latches 0x1, DM-root stays 0x0).

## Honest framing

This is a **consumer** state-transition on a fabricated `this`, NOT a DataModel ctor — nil-milestone either way.
The value is cause-not-symptom: it converts the SEP-17 directive's named `nativeActivity_onEngineSettingsReceived`
receive from "never driven" into a measured clean execution that latches its flag, on the exact manager-instance
receive the real Activity would drive after it constructs a manager. Route-B live-DM structural gate UNCHANGED
(DM-root 0, MH_* false); SH174 capture-latch (arm at a real `make_shared<DataModel>`) stays the single forward hook.

## Verify

`cargo test -p arm64jit --example elfjit sh276` = 1 passed with real-image pins; workspace green
(`cargo build --workspace` + `cargo test --workspace` EXIT 0). Repro: `runs/capture_sh276_engine_recv.sh`.
recon-v3 render plane re-verified green at HEAD (`runs/capture_taskv4_frame.sh`). elfjit.rs trimmed (comments)
to stay under the 1MB pre-commit hook.