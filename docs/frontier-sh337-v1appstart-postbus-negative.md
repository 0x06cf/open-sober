# Frontier SH337 — measured negative: V1 nativeAppBridgeAppStart__ against a POPULATED registry (count=12) still does NOT register "App"

Status: `dev`, single-agent (cone suppressed). New default-inert v2boot rung `--v2boot-postbus-v1appstart`
(elfjit.rs). elfjit.rs under the 1MB hook (1048470 B). Workspace green (build + elfjit example tests pass).

## Why this cycle

The standing SESSION-CTOR question is whether the "App" service ever gets registered so
the DM-controller ctor's fast-path returns a live DM. SH335 replaced it as the live measurement: on
the bus route the registry populates to 12 (task-scheduler family) but "App" is NEVER among the
entries, tier-2 cell [0x106fe4f78]="Runtime0" invariant, and once-slot gets the "Execute" handle.
A genuinely-untested combination remained: **drive V1 `nativeAppBridgeAppStart__` (0x102338510 —
the SH336-pinned SEP-17 lifecycle primitive) AFTER the registry is already populated to count=12.**
Every prior V1 drive ran from an EMPTY registry (count=0) and its registration walk died at the
first (empty) lookup; no probe had driven V1 AppStart against a populated registry to ask whether
the walk's later tier (the "App" pair) then resolves.

## What SH337 adds (default-inert rung + probe)

`--v2boot-postbus-v1appstart` (elfjit.rs, sibling of the postbus-doinit rung): on the single
--v2boot ladder thread it (1) drives `MessageBus.subscribe` (0x102ba5bb8, "experience-launch") to
populate the registry 0 -> 12, then (2) drives V1 `nativeAppBridgeAppStart__` (0x102338510) with
the SH336 ABI (x2..x7 = 5 empty jstrings + jboolean false), then (3) reads back registry count +
DM-root + once-guard + dumps the registry entry names so an "App" registration is detectably
present. Probe: `runs/probe_sh337_v1appstart_postbus.sh`.

## Measured (real libroblox.so, 1 clean bounded run)

```
SH337 bus Ok(0x3e8)                    <- registry 0 -> 12
SH337 after-bus registry=12
SH337 V1 AppStart__ Ok(0x3e8)          <- V1 AppStart__ drove clean on the populated registry
SH337 post-V1: registry=12 DM-root[0x106a68818]=0x0 once-guard=0x1
  entries=[Thread (BG)P, Thread (FG)P, Spawn (BG)P, Spawn (FG)P, Yield (BG)P, Yield (FG)P,
          Close (BG)P, Close (FG)P, SleepP, SchedP, UNKNOWN0, Execute]
exit 124 (stable idle), 0 SIGSEGV/SIGABRT
```

- V1 AppStart__ returned Ok(0x3e8) against a populated count=12 registry — it does NOT crash.
- The registry stays at **12**, entry names byte-identical to the pre-V1 set (task-scheduler
  family); **"App" never appears** even after the V1 AppStart registration walk runs on 12 entries.
- DM-root [0x106a68818] stays 0; once-guard 0x1 (do-init already latched from the bus). No live DM.

## Conclusion (honest, measured negative)

Driving the SH336-pinned V1 AppStart__ against a populated registry does NOT make the "App"
service register: the app-start registration walk, even with the task-scheduler tier already
present, does not produce an "App" entry. This is consistent with SH313/315/316/317/318/335 —
the "App" service (and the tier-2 controller-name cell it must resolve through) is built only by
a deeper live session ctor than a bare app-start entry drives. The SH337 combination is now a
closed measured-null, not a new lever.

Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false). This closes one more
untried angle on the SESSION-CTOR "App"-registration line with a concrete negative; the standing
forward remains the real Activity/AppBridge session drive (SH313/315/317), now with V1-AppStart-
on-populated-registry explicitly excluded.

## Verify

- `cargo build --workspace` EXIT 0; `cargo test -p arm64jit --example elfjit` green (no new
  hermetic test added this cycle — the result is a measured-negative captured by the probe).
- Repro: `bash runs/probe_sh337_v1appstart_postbus.sh` (bus Ok, V1 Ok, registry=12, entries
  task-scheduler-only, DM-root 0, EXIT 124).
- elfjit.rs = 1048470 B < 1MB pre-commit hook (new rung + entries dump funded by condensing
  SH-prose comments; no test weakened, no behavior changed).

Single-agent, default-inert. No production path altered (rung is opt-in `--v2boot-postbus-v1appstart`).