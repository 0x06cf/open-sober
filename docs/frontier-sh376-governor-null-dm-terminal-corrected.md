# Frontier SH376 — governor-NULL-DM terminal corrected (SH375's "SetInitParams abort" is a misread); GOVFLAG + the crossing-env now advances SendAppEventOnAppReady past governor+preload to the pack-helper

Session: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Two new probes
runs/capture_sh376_govflag_crossing.sh (SH373 crossing-env + GOVFLAG) +
runs/capture_sh376b_combined.sh (crossing-env + GOVFLAG + PRELOAD_VALUECELL) + one new
real-image hermetic `sh376_governor_predicate_flags_null_dm_and_session_advances`
(arm64jit lib 435->436). No production path edited (both probes use ONLY existing
default-inert guards: SH269 GOVFLAG + SH307 PRELOAD_VALUECELL + SH349 append-skip +
SH371 crossing seeds; the hermetic is read-only word-pins). Workspace green (cargo test
--workspace EXIT 0, 436/0 in arm64jit lib).

## The genuinely-new datum: SH375's terminal attribution is WRONG

SH375's ledger/doc recorded the terminal after SH285-crossing as a "SetInitParams
(0x102bcc814) SIGABRT." Re-running the exact SH375 env and reading the FULL live log
corrects this: **SetInitParams and V2InitWithParams both soft-RETURN benignly**
(`SetInitParams stopped: run_loop pc 0x3d0 outside image`, `V2InitWithParams stopped:
run_loop pc 0x4a0 outside image` — the SH331 shared-leaked-host-pc soft-return class).
The genuine SIGSEGV that ends the run is elsewhere:

- `[SIGSEGV] guestpc=0x102ea0b9c fault=0x0 x0=0x0 x20=0x0 x21=host-app-governor` —
  the **governor** (0x2ea0b48) dereferencing `[x21,#1032]=[controller+0x408]` = 0.
- `[SIGABRT] fault=0x3e90009ea23` follows (the engine's own abort after the fault).

So SH375's "the ladder aborts at SetInitParams — the 0x258b5d8 body is blocked by that
LSM consumer" conclusion was built on a MISREAD of a run-variable crash. The real
blocker the session drive hits after crossing SH285 is the **governor NULL-app-DM
controller deref at 0x102ea0b9c** — which was ALREADY provisioned-for by SH269's
`routeb_govflag_seed_guard` (predicate byte [0x106a64da0]) but never ARMED in the SH375
env (that env lacked `JIT_ROUTEB_APPSART_GOVFLAG`).

## Measured advance: arming GOVFLAG crosses the governor + preload walls

- **capture_sh376_govflag_crossing.sh (SH373 crossing-env + GOVFLAG, 4 runs):** GOVFLAG
  fires (3/4), `[elfjit:v2boot] SH269 seeded governor-predicate [0x106a64da0].bit0=1`,
  the 0x102ea0b9c NULL-deref is GONE, and the terminal ADVANCES to
  `guestpc=0x102bb803c` — the SH307/SH270 preload-overrides valuecell wall.
- **capture_sh376b_combined.sh (crossing-env + GOVFLAG + PRELOAD_VALUECELL, 4 runs):**
  the preload wall (0x102bb803c) is ALSO crossed (`SendAppEventOnAppReady returned`),
  and the terminal moves further to `guestpc=0x101d9a708` (SH350's name/version pack
  helper — the known-closed SH349/350 persistence family).

Interpretation: the crossing-env's SH285-cross is real, and once the governor flag and
preload valuecell are armed the session drive runs `SendAppEventOnAppReady` to
benign-return and only then re-enters the already-measured-closed LSM/pack persistence
lane (0x101d9a708, SH350 closure). This is a map-completion + attribution-correction:
the sequence of reachable terminals past the crossed SH285 is now governor NULL-DM
(SH269) -> preload valuecell (SH307) -> pack-helper (SH350, closed), NOT "SetInitParams
SIGABRT." No DataModel manufactured; the SH350 pack-helper re-entry is the standing
unbounded-whack-a-mole lane (do NOT re-drive LSM sub-call skips — SH349/350/358/373/375
stand).

## Honest

Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B
live-DM structural gate UNCHANGED. The advance is a corrected terminal sequence + a new
real-image hermetic; the furthest-session-forward the completed SendAppEventOnAppReady
still lands in the closed persistence lane, so no live-DM gate moved this cycle. The
corrected attribution MATTERS because SH362/SH375 used "SetInitParams abort" to justify
the 0x258b5d8 dispatch-body closure — that premise is now obsolete, but the corrected
blocker (governor NULL-DM) is itself SH269-armed and passes cleanly, so the 0x258b5d8
body's real (still-unreached) blocker is the SH350 pack-lane.

## Files

- runs/capture_sh376_govflag_crossing.sh, runs/sh376-govflag-crossing.txt
- runs/capture_sh376b_combined.sh, runs/sh376b-combined.txt
- sh376 hermetic (arm64jit lib 436)