# Frontier SH335 — timing-accurate closure on "App"-registration (bus route): the ctor lookup fires once per registration, "App" is NEVER among the entries, the fast-path matches "Execute" at the terminal count=12 lookup

Status: `dev`, single-agent (cone suppressed). Hermetic: existing `sh334_registry_live_guard_tests`
(2, now page-safe). elfjit.rs byte-identical (1048570 B, unchanged). New repro probe
`runs/probe_sh335_bus_reglive.sh`. Workspace green.

## Why this cycle

SH334 added `routeb_registry_live_guard` (JIT_ROUTEB_REG_LIVE=1) but it was **once-per-run** — a
single dump at the FIRST DM-ctor name->service lookup (0x2168798) entry. On the bus route that first
lookup sees registry count=0 (empty), so SH334 only ever produced the "MAIN-path empty" readout and
left the timing-accurate question open: **does "App" appear at ANY later call to the lookup on the
route that actually populates the registry?** SH316/318 had answered it only via a POST-RUN dump of
the final registry (12 task-scheduler services, no "App") — never at the live decision point.

## The change (minimal, read-only, default-inert)

Turned the once-per-run latch into a **count-transition** latch: dumps when the registry count
[0x106fe2f08] CHANGES since the last dump (naturally bounded by the number of distinct count
values). Also made the tier-2 fixidx/tier2-cell reads page-safe (guarded with `any_page_mapped`)
so the guard cannot SIGSEGV on an unmapped page in any test/run. No behavioral change to any
existing run path (env-gated, read-only).

## Measured (real libroblox.so, runs/probe_sh335_bus_reglive.sh, 3/3 deterministic)

The DM-ctor lookup now dumps **once per service registration** as the bus route populates the
registry, count 0 -> 1 -> ... -> 12:

```
count=0   DM-root=0x0 once-slot=0x0 tier2=""
count=1   DM-root=0x0 once-slot=0x0 tier2="Runtime0"
... (each count value dumped once) ...
count=11  entries=[Thread (BG)P, Thread (FG)P, Spawn (BG)P, Spawn (FG)P,
                   Yield (BG)P, Yield (FG)P, Close (BG)P, Close (FG)P,
                   SleepP, SchedP, UNKNOWN0]  DM-root=0x0 once-slot=0x0 tier2="Runtime0"
count=12  (only once-slot transitions to 0x400000b in the final readout)
```

- The task-scheduler family is the ONLY thing ever in the registry (Thread/Spawn/Yield/Close/
  Sleep/Sched/UNKNOWN — matching SH318's post-run dump).
- **"App" never appears as a registry entry at ANY lookup, including the terminal count=12.** The
  exact-token scan of every `entries=[...]` line across all four runs: NONE.
- DM-root [0x106a68818] stays 0x0 at every lookup.
- once-slot transitions 0x0 -> 0x400000b only at/after the count=12 lookup — the DM-ctor fast-path
  matches "Execute" (one of the two names it looks up), never "App". So the fast-path `cbnz x0`
  @0x61e3124 yields the task-scheduler "Execute" service handle, NOT a DM controller -> no DM.

## What this closes

SH332/333/334 left candidate #1 ("is 'App' registered by the time the DM-ctor looks it up?")
answerable only pre-crash (MAIN path, registry empty) or post-run (bus route, final 12 entries).
SH335 makes it timing-accurate on the registry-populating route: the lookup is called once per
registration and even incrementally "App" is never inserted. The ctor fast-path's ONLY match is
"Execute", so it can never return a live DM controller. Consistent with SH313/315/316/317/318's
closure, now at the live per-registration decision point.

Honest: does NOT manufacture a DM (DM-root 0, MH_* false); Route-B live-DM structural gate
UNCHANGED. This measures the "App"-registration gap with a tighter instrument; the SESSION-CTOR
"App"-registration (SH313/315/316/317/318) remains the standing unblock, unchanged but more
precisely pinned.

## Verify

- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (sh334 tests now page-safe,
  pass in parallel).
- Repro: `cargo test -p arm64jit --lib -- registry_live` -> 2 passed; `bash runs/probe_sh335_bus_reglive.sh`
  (count-seq 0..12, entries=task-scheduler only, "App" absent, once-slot 0x0->0x400000b, EXIT 124).
- elfjit.rs byte-identical 1048570 B (unchanged, within 1MB hook). Commit local `dev` only.

Single-agent, default-inert, read-only guard.