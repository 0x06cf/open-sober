# Frontier SH400 — the ordered session-substrate DRIVER (SEP-18 BUILD-THE-RUNTIME)

Date: 2026-09-19, hermes-worker, single-agent. Workspace green before/after
(`cargo test --workspace` EXIT 0; arm64jit lib 449 passed/0 failed; +2 new
hermetic in a NEW small module, no change to jit.rs/elfjit.rs size hooks).

## Why this cycle

The previous 9 cycles (SH390-398) were ALL probe-only closures — every one
measured "no live DM" and landed ZERO production code. The operator's SEP-18
directive is explicit: **BUILD THE RUNTIME, NOT THE DM** — "drive the engine's
REAL Activity-session init state machine... in host code, not as one more seed
into the .so." SH399 shipped the coherent ordered 16-atom `ROUTEB_SESSION_SUBSTRATE`
table, but its ONLY consumer was a hermetic address-pin test — the documented
deferred line "the elfjit ladder thread consumes the order" was never implemented.
The table was dead data.

## What landed (SH400)

- **NEW module `crates/arm64jit/src/session.rs`** (small, off the 1MiB hooks):
  - `SessionHandles` (env/thiz/init_params/start_params/bg_name fabricated JNI handles)
  - `substrate_args(name, &handles) -> [u64;8]` — per-atom ABI template mirroring
    every proven `--v2boot-*` rung arg pattern in elfjit.rs (incl. the SH186
    ABI-critical "Home" in x5 for SendAppEventOnAppReady, surface token+params for
    V2UpdateSurface, fabricated manager for onEngineSettingsReceived).
  - `drive_atom(...)` — one serialized `jit_run` per atom (SH55/64), reports Ok/Err
    + post-atom `MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY/GAME_LOADED` +
    `AppBridgeV2[0x106a705e8]`.
  - `drive_routeb_session_substrate(iimg, ib, tpidr, boot_sp) -> usize` — walks the
    FULL ordered `ROUTEB_SESSION_SUBSTRATE` (16 atoms) and drives each in order.
  - **SH82 gate** folded into the driver: at the `nativeGameGlobalInit` atom it
    seeds main-id `[0x106863a68] = pthread_self()` (mirrors the elfjit ladder rung)
    so the rung-1 nanosleep park is crossed.
- Two hermetic tests (no real binary needed): every substrate atom has a non-empty
  ABI template, and the known atom-name set exactly matches the substrate (no
  fallthrough, no drift).
- `pub mod session` in lib.rs.
- elfjit call site: `--v2boot-session-drive` env/arg-gated rung calling
  `arm64jit::session::drive_routeb_session_substrate`.
- Capture `runs/capture_sh400_session_substrate_drive.sh` (env = the furthest-
  advancing full ladder envelope from SH397; adds `--v2boot-session-drive`).

## MEASURED (real libroblox.so, 2/2 reproducible)

```
EXIT=124 (stable idle, 0 crash)
[session-drive] [1/16] nativeInitializeNativeFlags @ 0x10232048c -> stopped (benign host-pc soft return)
[session-drive] [2/16] nativeGameGlobalInit      @ 0x102206404 -> Ok (SH82 gate crossed the park)
[session-drive] [3/16] setTaskSchedulerBM        -> Ok(0x1)
[session-drive] [4/16] V2InitWithParams          -> stopped (benign host-pc)
[session-drive] [5/16] StartLuaAppDM             -> Ok(0x3e8)
... 6-16 all Ok ...
[session-drive] substrate complete: 11/16 atoms returned non-zero Ok
```

**The genuine forward:** the substrate drive constructs the **AppBridgeV2
singleton to its genuine relocated vtable 0x1063a3410** — `AppBridgeV2[0x106a705e8]`
goes `0x0` on the first 4 atoms to `0x1063a3410` from atom [5/16] `StartLuaAppDM`
onward (12 of 16 atoms report the real vtable). Every prior cycle measured this
singleton stuck at `0x0`. This is exactly the SESSION-CTOR lever — StartLuaAppDM
driven in the ordered substrate self-constructs the AppBridgeV2 singleton the
`0x2ea3084` builder populates via `__call_once` (recon-sh156), which is the
gate before the governor `0x102e9fa84` and the deeper DM world-build.

## Honest

- Does NOT manufacture a DataModel (DM-root [0x106a68818] = 0; once-slot =
  0x400000b "Execute" sentinel, SH381). MH_* stay false across all atoms (they are
  session observables that only fire when a real do-init completes with a live DM).
- The AppBridgeV2 singleton advance is REAL and reproducible (2/2), but it is a
  session-state construction, not yet a live DM — it moves the SESSION-CTOR line
  from "AppBridgeV2 0x0" to "AppBridgeV2 genuine vtable", the documented next
  gate (governor 0x102e9fa84).
- This is the first production code landed since SH389 (after 9 probe-only cycles):
  a build-the-runtime deliverable, not a probe.

## Next

Route-B live-DM structural gate mostly unchanged (DM-root 0). The substrate-driver
now EXECUTES the ordered session; the forward is to drive DEEPER past the
AppBridgeV2 singleton — the governor `vt[+0x18]=0x102e9fa84` (guest 0x102e9fa84)
the singleton makes reachable — and keep the do-init/StartLuaAppDM → DM world-build
line going. R1 content half stays staged/armed (SH351/352/354).