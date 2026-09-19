# Frontier SH405 — arming SH320 (JIT_ROUTEB_DONEPATH_MAIN) flips the do-init onto its MAIN dispatch: the never-executed 0x10258b5d8 app-start body finally RUNS deep, then drains into the standing SH285/LSM persistence lane from a SECOND (MAIN-arm) entry point
# (do-init world-build; next-gate after SH404)

Date: 2026-09-21, hermes-worker, single-agent. Workspace green before/after
(`cargo test --workspace` EXIT 0, arm64jit lib 453->454 with the new sh405 hermetic;
elfjit.rs untouched; jit.rs added the hermetic + 0 production-path change).

## Why this cycle

SH404 measured that the do-init MAIN dispatch `br x1` @0x2206e24 is BYPASSED (0
block-entry hits) — the run takes the FALL-THROUGH arm (0x102206e30 box-build -> nested
worker 0x102206fac -> app-shell ctor band). It left open: *"can a seed turn the park into
a clean RETURN that advances do-init to completion?"* and named the MAIN dispatch + DMCONT
as the two un-reached gates. SH405 answers the dispatch half.

## The key datum SH404's env missed

The do-init thread-dispatch at 0x2206de0-0x2206df0 is `pthread_self() vs [0x106863a68]`
(stored main-id): `b.ne @0x2206df0` TAKEN -> fall-through box-build (SH404's arm); NOT taken
(ids match) -> falls to `ldr x0,[x19,#32]` -> MAIN branch -> `br x1` @0x2206e24. The SH320
guard (`JIT_ROUTEB_DONEPATH_MAIN=1`) exists to re-seed [0x106863a68] to the EXECUTING thread
at block-entry 0x2206db8, forcing the match. SH404's env did NOT arm it. SH405 arms it.

## What SH405 measured (real libroblox.so, @0x2173ff4, SH404 env + DONEPATH_MAIN + EARLYRET + SSO_SEED, 2/2 reproducible)

- **SH320 seed fires**: `seeded do-init done-path main-id [0x106863a68]=0x... (this jit
  thread)` at 0x2206db8 (was 0x3) -> the thread-match b.eq is NOT taken -> **MAIN branch**.
- **The never-executed 0x10258b5d8 dispatch body finally ENTERS** (region-watch:
  guest pcs 0x10258b5d8, 644, 6b8, 724, 744, 77c, 7c8, 830, 850, bbb0 all hit once) —
  the function every cycle from SH362-404 labeled unreachable now runs deep headlessly.
- **It crosses the SH322 lifecycle wall** (0x21f3748; guard seeds caller pair [x1] -> the
  early-ret obj, tbnz taken) **and the SH323 SSO fencepost** (0x21f5078 + 0x1025f36ac both
  seeded empty SSO strings) — both seeds fire on this path.
- **Then drains into the standing SH285/LSM persistence lane**: the SH341 pool-pop
  write-site 0x101d9a528 fires repeatedly immediately before SIGSEGV at guestpc 0x1025f501c
  (fault=0x0; EXIT 134-139). 0x1025f501c is the app-start body's continuation just after
  `bl 0x25f52b4 @0x25f5018` (the app-obj build, x0=[x19,#1088]).

## Interpretation (map refinement, not a DM)

- **The MAIN dispatch IS reachable** — it was never a law that the do-init must fall
  through; SH404's fall-through was the absence of the SH320 thread-match seed. Arming it
  makes the do-init take its real app-start MAIN arm, which executes the 0x10258b5d8 body.
- **The persistence lane is ARM-RELATIVE, not path-fixed**: it swallows BOTH the
  fall-through arm (SH404 -> app-shell band -> park) AND the MAIN app-start arm (SH405 ->
  0x10258b5d8 body deep -> drain). This is a second, independent entry into the same
  measured-closed SH285 family (SH372's path-independence verdict strengthened, now from
  the MAIN arm rather than the continuation lanes).
- Honest: does NOT manufacture a DataModel. DM-root [0x106a68818]=0, MH_* false, AppBridgeV2
  genuine vt 0x1063a3410 unchanged. The advance is that the do-init's MAIN world-build path
  is now reachable and the app-start body executes — the closest unblocked construction line
  — and the next wall on it is the same persistence lane.

## Files

- New real-image hermetic `sh405_donepath_main_flips_doinit_to_main_dispatch_into_appstart`
  (arm64jit lib 453->454) byte-pins br x1 @0x2206e24, the 0x10258b5d8 prologue, the
  app-start `bl 0x25f52b4 @0x25f5018` (0x940000a7), and the callee prologue (0xa9bb7bfd).
- Capture: runs/capture_sh405_donepath_main.sh (SH404 env + DONEPATH_MAIN + EARLYRET +
  SSO_SEED). Log (outside repo): /home/hermes-worker/runs/sh405-donepath-main.txt.

## Next

On the SH405 MAIN arm, the next wall is the SH285 persistence lane drain (0x1025f501c after
app-start). Per the standing do-not-re-tread, the LSM family is measured-returned vs a real
session ctor, so do NOT re-drive LSM skips. The forward is now the do-init MAIN arm's
world-build continuation past the persistence drain (does the app-start body's persistence
access eventually move to the DMCONT/do-init-completion gates or is it the same
measured-unbounded lane?), and DMCONT 0x102bd1d68 remains the standing next construction
gate (still 0 hits on this env). Do-not-re-tread unchanged (SH285/385/395-398,
setDataModelToCurrent SH388, EC reader-gate, window-attach real SH367, ALooper SH365,
governor-gates full-ladder SH379, LSM crossings SH393/396).