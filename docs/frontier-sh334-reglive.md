# Frontier SH334 — LIVE answer to the "App"-registration question (candidate #1) via a block-entry registry readout at the ctor lookup, surviving the FMOD/LSM crash

Status: `dev`, single-agent (cone suppressed). New guard `routeb_registry_live_guard` in
crates/arm64jit/src/jit.rs (opt-in `JIT_ROUTEB_REG_LIVE=1`, default-inert, read-only, fires once).
New repro probe runs/probe_sh334_reglive.sh. 2 hermetic tests added (arm64jit lib 408->410).
Workspace green (lib 410/0, elfjit examples 154/0, all crates 0 fail). elfjit.rs byte-identical at
1048570 B (unchanged, at the 1MB pre-commit hook).

## Why this cycle

STATUS candidate #1 (SESSION-CTOR "App"-registration) has been "open but UNCHANGED" for many
cycles: SH332/333 measured that on the SH320/322/323 MAIN-path run the service-registration walk
(fn 0x21e2a90) + name->service lookup (fn 0x2168798) DO execute, and the SH330 +0x408 app-start
fork CROSSES — but the run dies at the FMOD/AAudio wall (0x106240cb8) or the LSM reader
(0x101dcab68) BEFORE the post-ladder `dump()` (which reads registry count + names + DM-root + the
tier-2 cell) ever fires. So the decisive "does the crossed app-start register 'App'?" question could
not be answered — the only registry readout lived in dump(), which was unreachable. This cycle adds
a LIVE readout at the exact moment the DM-controller ctor's lookup executes, so the answer no longer
depends on surviving the crash.

## The guard

`routeb_registry_live_guard(state, pc)` — opt-in `JIT_ROUTEB_REG_LIVE=1`, default-inert (env-gated).
Fires once (OnceLock) when a block at guest pc 0x102168798 = the DM-ctor name->service lookup fn
0x2168798 entry is entered, and snapshots the same cells the post-ladder dump reads:
- service-registry count [0x106fe2f08] + the first 16 entry NAMEs (array base 0x106fe6180, stride 0x60)
- DM-root [0x106a68818] and once-slot [0x106a68408] (SH316 distinct cells)
- tier-2 controller-name cell for entry 0 (SH318 fixidx per-entry): cell = 0x106fe2f00 + fixidx0*0x5c + 0x2078

Read-only: it only eprintln!'s the snapshot; no guest state is written. Because it fires at the
lookup ENTRY, before the lookup walks the (possibly) empty registry, its output answers whether the
registration-walk had already populated "App" by the time the ctor needs it — regardless of the
downstream crash that kills dump().

## Measured (real libroblox.so, runs/probe_sh334_reglive.sh, 3/3 deterministic)

At the DM-ctor lookup on the SH332-style MAIN-path run (no --v2boot-skip-appstart, full
SH320/322/323 + SH330 stack):

```
[routeb-reglive] SH334 LIVE at lookup 0x2168798 entry pc=0x102168798:
  service-registry-count[0x106fe2f08]=0 entries=[]
  DM-root[0x106a68818]=0x0 once-slot[0x106a68408]=0x0
  fixidx0=0x0 tier2-cell[0x106fe4f78]=""
```

Terminal run-variable (FMOD 0x106240cb8 in one earlier run; LSM reader 0x101dcab68 + SIGABRT
leaked-host-pc in the rest) — the known SH332/SH330-class downstream.

## What this resolves (candidate #1)

The "App"-registration question is now MEASURED, not open: on the SH332-style MAIN-path run, when the
DM-controller ctor's name->service lookup fires, the service-registry COUNT is 0 (empty) — no
entries at all, hence no "App". So the crossed +0x408 app-start does NOT register "App" before the
run dies; the ctor fast-path `cbnz x0` @0x61e3124 would still miss (empty registry), DM-root stays 0.
This is consistent with SH315's measured baseline (only the separate bus route populates the registry
to 12, and even then "App" is absent, SH315/318). Honest: this confirms the MAIN-path registration
walk+lookup executing does NOT by itself produce the "App" entry — the SESSION-CTOR "App"
registration remains the standing unblock, unchanged but now byte-measured at the exact decision point.

## Honest

- No DM (DM-root 0, MH_* false); Route-B live-DM structural gate UNCHANGED. This cycle measures and
  resolves candidate #1's open status; it does not manufacture a DataModel.
- The guard is a diagnostic (opt-in, read-only); no behavior change on any existing run path.

## Verify

- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (lib 410/0, elfjit 154/0).
- New tests: `cargo test -p arm64jit --lib sh334` → 2 passed (inert-without-env, wrong-pc-miss).
- Repro: `bash runs/probe_sh334_reglive.sh` (reglive count=0, entries=[], DM-root 0, once-slot 0,
  fixidx0=0 tier2="", terminal run-variable 0x101dcab68/0x106240cb8).
- elfjit.rs byte-identical 1048570 B (unchanged, within 1MB hook). Commit local `dev` only.

Single-agent, default-inert, elfjit.rs unchanged.