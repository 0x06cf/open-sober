# Frontier SH343 — CROSS: the SH341 LSM poison fencepost (persistence-lane terminal 0x101d9a528) is crossed; the full-ladder Route-B route advances one fencepost to the SH285 reader/pop terminal 0x101db1b08

## Session
Sep 2026, hermes-worker. Single-agent (cone suppressed). Recon-v3 deliverables
(type4_frame_thunk self-drive + JIT_JSON_ZERO_FIX) verified present+green this
cycle at HEAD. New production-relevant fix (opt-in env-gated), +3 hermetic unit
tests. Workspace green (arm64jit lib 414/0, all crates 0 failures).

## What SH341 left open
SH341 MEASURED that the LSM free-list pool-pop terminal (guestpc=0x101d9a528,
`mov x1,x0` + `str x8,[x1]`, write target = the KEY) faults on executable segment
because EXACTLY ONE of ~391 pops feeds a poisoned .text KEY (0x101d968e4, caller
LR=0x10626b6dc = the 0x626b6d0 pool-pop wrapper, the old "FMOD AAudio" label). BUT
it attributed the source without crossing it — the full-ladder Route-B session
route (STATUS candidate #2) still SIGABRT'd there, so "fix target = the FMOD-side
caller" was left as a recommendation, not a crossing.

## The change (SH343 crossing, opt-in JIT_ROUTEB_LSM_KEYFIX=1)
`routeb_lsm_keyfix_guard` (jit.rs, default-inert) fires at the SAME pop write-site
block entry pc==0x101d9a528. When the key classifies EXEC-segment-poisoned (.text,
write:off — never a legitimate LSM key), it substitutes the WRITE TARGET (s.x[0],
copied into x1 by the block's `mov x1,x0`) with a leaked zeroed host-heap cell, so
`str x8,[x1]` lands in real writable memory and the pop COMPLETES instead of ABRT.
Register-edit only. Valid host-heap keys (the other 390) and any non-anchor pc pass
through untouched. This is the persistence-lane crossing SH268/SH341 could not
reach; the ladder then proceeds INTO the post-persistence session-ctor rungs.

## MEASURED (real libroblox.so, runs/capture_sh343_lsm_keyfix.sh, full send-appevent
## + session ladder, pre/post identical env except KEYFIX)
- POST-fix keyfix fired: `[routeb-lsm-keyfix] ... poisoned .text KEY 0x101d968e4 at
  pop write-site ... -> write-target substituted to valid host-heap cell 0x7ff0...`
  (exactly ONCE, on the crashing iteration).
- The ladder now advances PAST the SH341 terminal: the crash moves from guestpc
  0x101d9a528 (pre-fix; the SH268/SH341 persistence-lane wall) to guestpc
  0x101db1b08 (post-fix) — SH285's LSM READER/POP terminal, fault=
  0xffffffffffffffff (an RBX-poisoned live-object pointer).
- Config unchanged otherwise; rungs 0..5 (NativeFlags/GlobalInit/UpdateAdapterInit/
  setTaskSchedulerBM/V2Init/StartLuaAppDM) all driven; AppBridgeV2 [0x106a705e8]=0,
  MH_* all false (no DM — the live-DM gate, not here).

## Interpretation (do-not-over-claim)
- The SH341 poison fencepost — the persistence-lane terminal wall that has blocked
  the full-ladder route since SH260/268 — is now a MEASURED CROSS. The single
  stale .text key no longer ABRTs the ladder; it lands in a valid cell and the LSM
  pop completes. This is the concrete crossing STATUS candidate #2 called for.
- One fencepost deeper the route hits a NEW (or rather SH285-documented) wall: the
  LSM reader/pop at 0x101db1b08, fault=0xffffffffffffffff — an RBX-poisoned
  live-object pointer (SH285 pinned this exact site as "reader/pop terminal, one
  fencepost past insert-leaf CROSSED"). That is the next forward hook.
- No DM (DM-root via manually-seeded holder only; MH_* all false); the Route-B
  live-DM gate is UNCHANGED. This is a persistence-lane advance, not a session
  crossing. Honest.

## Verify / files
- Hermetic tests: jit.rs `routeb_lsm_keyfix_guard_tests` (3: inert-without-env,
  valid-key-untouched+poisoned-redirected+writable-cell, wrong-pc-misses) — confirm
  the guard is default-inert, selective (only EXEC-poisoned keys), and edits only
  x0 at the exact anchor. Parallel-safe via an intra-module Mutex.
- Repro: runs/capture_sh343_lsm_keyfix.sh (full ladder + KEYFIX). Live capture
  /home/hermes-worker/runs/sh343-lsm-keyfix.txt (gitignored per CLAUDE.md).
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (arm64jit 414).
- Commit: local `dev` only (operator pushes). Doc + capture + guard + 3 tests.