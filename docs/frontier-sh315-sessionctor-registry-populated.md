# SH315 — SESSION-CTOR binder route populates the service registry headlessly (count 0→12, first ever)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh315` (real-image, 6 pins). elfjit examples 140/0.
Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false). Recon-v3 re-verified green.

## 1. The finding (measured, 5/5 deterministic)

The `MessageBus.subscribe` SESSION-CTOR route (`--v2boot-skip-appstart` + `--v2boot-session-bus`,
the SEP-17 PRIMARY-LEVER path) **populates the service registry headlessly for the first time**:
`service-registry-count[0x106fe2f08]` = **12** — every prior SH312/313 static + runtime measurement
had it **0** (the DM-controller ctor's name->service lookup cbz's on empty and returns NULL, so
do-init's once-lambda stores NULL into DM-root [0x106a68818] and the app-shell/DM never builds).

That count cell is the exact gate the whole Route-B chain hangs on (SH312/313 byte-pinned). The
skip-appstart+bus run drives app-start's REAL session registration walk; SH313 said ONLY a real
session run writes it (whole-image scan: zero static writers, 0xa7e-gated ctor increment). So this
is the first measured evidence a real (not DMCONT-continuation, not harness-seeded) session path
constructs engine state headlessly.

## 2. What got registered (measured entry dump)

Dump of the registry array [0x106fe6180] (0x60 stride, name string AT the entry addr, service ptr
at entry-16) shows 12 genuine DM-task scheduler services:

    Thread (BG), Thread (FG), Spawn (BG), Spawn (FG), Yield (BG), Yield (FG),
    Close (BG), Close (FG), Sleep, Sched, UNKNOWN, Execute

Critically **"App" is NOT among them** — the DM-controller ctor (bl 0x2168798 from 0x61e311c)
looks up its "App"/"Execute" name pair, so even with count=12 the ctor's fast-path `cbnz x0`
@0x61e3124 still misses and DM-root stays 0. The registry now registers the task-scheduler family
but not yet the DM "App" service the ctor fast-path needs.

## 3. Post-bus do-init re-drive does NOT link the two (measured, honest)

Implemented `--v2boot-postbus-doinit` (elfjit.rs): drive bus (populates registry to 12), then
CLEAR once-guard [0x106a68410].bit0 + re-seed main-id (SH126 pattern), then re-drive StartLuaAppDM
so the do-init once-lambda re-runs the DM-controller ctor with a NON-empty registry.

Result (probe B region-watch): the DM-controller ctor region 0x1061e30bc-0x1061e3340 **does fire**
(0x1061e310/311/312/31e/32c/32d), and the do-init once-lambda 0x2206d10 fires — but the once-guard
**never self-latches** (stays 0 after the re-drive) and DM-root stays 0. So the do-init's `__call_once`
does not complete on the ladder thread even with a populated registry; the ctor fast-path is reachable
but the once-lambda's completion is what gates reading a root. This is a live-state/completion gate,
not a fixable seed — consistent with the SESSION-CTOR "drive the real session to completion" premise.

## 4. What this means for Route B

- The SESSION-CTOR binder route now demonstrably runs app-start's real registration walk headlessly
  (count 0→12) — a genuine cause-not-symptom advance, the first time engine state that SH313 proved
  is ONLY session-constructed got constructed by a headless run.
- The remaining gap is: register the "App" service (the task-scheduler family it currently registers
  is a different tier), AND get the do-init once-lambda to complete on the ladder thread so it reads
  the ctor's fast-path result. Both are still SESSION-CTOR live-state work, not seeds.
- No DM manufactured. DM-root 0, MH_* false. Route-B live-DM structural gate UNCHANGED.

## Repro

- `cargo test -p arm64jit --example elfjit -- sh315` (1 passed; 140/0 total).
- Registry population: `runs/probe_sh315d_regentry.sh` (5/5, EXIT 124, count=12, entry dump).
- Post-bus do-init re-drive: `runs/probe_sh315_postbus_doinit.sh` + `runs/probe_sh315b_regions.sh`.
- Recon-v3 re-verified green at this HEAD (`runs/capture_taskv4_frame.sh`: 24 frames, swap Ok(0x1),
  197 pops, 0 json abort, EXIT 124).

Single-agent. Workspace green (arm64jit lib 405/0; elfjit examples 140/0).