# Open-Sober run status (hermes-worker)

Updated 2026-09-19, this session: SH334 — LIVE answer to the standing "App"-registration question
(candidate #1) via a default-inert block-entry registry readout at the DM-ctor lookup, surviving the
FMOD/LSM crash; workspace green with 2 new hermetic tests.

## Current state

- `dev` HEAD: to be committed (SH334). Clean after commit, NOT pushed (operator pushes).
- `cargo test --workspace` green (arm64jit lib 410/0 incl. 2 new sh334 tests; elfjit examples 154/0; all crates 0 fail). elfjit.rs byte-identical at 1048570 B (under 1MB pre-commit hook — not touched this cycle).
- New repro probe: `runs/probe_sh334_reglive.sh`. New frontier doc: `docs/frontier-sh334-reglive.md`.

## What advanced this session

- **SH334 (measured 3/3 deterministic):** added `routeb_registry_live_guard` (crates/arm64jit/src/jit.rs,
  opt-in `JIT_ROUTEB_REG_LIVE=1`, read-only, fires once) that snapshots the service-registry count +
  entry names + DM-root + once-slot + tier-2 controller cell at the EXACT moment the DM-controller
  ctor's name->service lookup (fn 0x2168798, entry 0x102168798) runs — BEFORE the run-variable
  FMOD/LSM crash that makes the post-ladder dump() unreachable.
- **Decisive result:** at the ctor lookup, `service-registry-count[0x106fe2f08]=0 entries=[] DM-root=0
  once-slot=0 fixidx0=0 tier2-cell=""` — the registry is EMPTY when the lookup runs on the SH332-style
  MAIN path, so "App" is NOT registered before the crash. This resolves candidate #1's long-standing
  "open but UNCHANGED" status with a live measurement: the registration-walk+lookup execute, but they
  do not produce the "App" entry before the run dies. Consistent with SH315/318 (only the bus route
  populates the registry, and even then no "App").
- Terminal remains run-variable (FMOD 0x106240cb8 / LSM reader 0x101dcab68 / leaked-host-pc) — the
  SH332/SH330-class known downstream, no new stable gate.

## Honest status

- No DM (DM-root [0x106a68818]=0, MH_* false); Route-B live-DM structural gate UNCHANGED.
- SH334 measures and resolves candidate #1's open status (registry empty at the ctor lookup on the
  MAIN path); it does not manufacture a DataModel. The SESSION-CTOR "App"-registration remains the
  standing unblock.

## Next-forward candidates

1. (SESSION-CTOR) The "App" service registration so the DM-ctor fast-path resolves a live controller.
   SH334 pins the decisive measurement point (empty registry at the lookup); the remaining work is to
   get a REAL app-start session to register "App" (SH313/315/316/317/318 line) — live-state work.
2. Clear the FMOD/AAudio + LSM reader run-variable walls (0x106240cb8 / 0x101dcab68) only if a stable
   gate can be pulled from them — note SH212/213/132/SH332 class them run-variable/non-seedable
   (do-not-re-derive); SH334 now gives a live readout that survives them, so the dump-gap is closed.