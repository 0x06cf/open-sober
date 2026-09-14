# SH127 — Serialized combined-run task frames: diagnostic isolation

## Context
Recon-v3 (docs/recon-selfdrive-seed-jsonfix.md) deliverable (1) — self-driven
frames — is implemented and re-verified: `type4_frame_thunk` installed into the
dispatcher's type-4 vector [0x106829ea8] via `--taskv4-seed frame`; a w4=4 drain
dispatch marshals to the currency-owning renderinit presenter which drives the
engine's real frame machinery (make-current 0x105b3b358 -> frame-fn 0x105b32c00
-> swap 0x105b3b408) on the recovered real EGL ctx (RENDERCTX). The standalone
repro (runs/capture_taskv4_frame.sh) presents 24 real task-driven frames
(`present #N swap Ok(0x1)`), 191 real type-4 node pops, EXIT 124, 0 crash.

## Problem (this doc)
In the SERIALIZED combined run (JIT_SERIALIZE_RENDER=1 + `--v2boot` ladder +
render + taskv4), the presenter drained 0 frames (pending=0) even though the
ladder completes and RENDERCTX is published. Pre-SH127 the `--deque-node-live`
producer thread started its 400-tick (x50ms = 20s) live-drain capture budget at
t=0, burned it entirely in the pre-recovery window, and gave up before
RENDERCTX published (runs/sh126-serial-s1.txt).

## SH127 change
Added a JIT_SERIALIZE_RENDER=1 + `--v2boot` gate at the TOP of the
`--deque-node-live` producer closure (elfjit.rs ~6306): it now waits for
LADDER_DONE, then for RENDERCTX (300s bound, mirroring the renderinit gate at
elfjit.rs:6945), BEFORE starting the drain-capture budget. Mirrors the existing
serial predicate (`std::env::var("JIT_SERIALIZE_RENDER")==Some("1")` &&
`std::env::args().any(|a| a=="--v2boot")`).

## Result — blocker ISOLATED as structural
With the gate (runs/sh127-serial-ladder-frame.txt): "serialized gate passed:
LADDER_DONE=true RENDERCTX=0x7fcee81109c0", but STILL 0 frames / 0 node pops /
"gave up after 400 ticks". The engine's live drain pop-loop pc NEVER lands in
[0x102856e40, 0x1028570a4) after the ladder completes — StartApp is parked by
the ladder's do-init and its idle-main-loop (which normally sustains the task-
deque drain) never resumes. Contrast the standalone path where StartApp's drain
stays live (191 pops): there the drain drives itself; under the ladder StartApp
never re-enters it.

This is NOT a timing bug the gate can fix; it is the documented SH55/64/structural
class (engine-side task loop not driven once StartApp parks). No regression: opt-in,
default product path unchanged and green.

## Candidate resolutions (untried, ordered by risk)
1. Accept ladder-only + the standalone self-driven-frame plane as the
   reproducible product artifact — BOTH proven clean (ladder-only 3/3; standalone
   24 frames). Lowest risk.
2. Drive the drain pop-loop 0x102856e40 as a top-level jit_run on the
   renderinit thread AFTER LADDER_DONE + RENDERCTX, where currency holds, with
   deque-node-live injecting nodes. Would present frames in the combined run but
   risks the SH44 block-re-eviction class (the drain body recompiles while
   running) and could hang (drain has no natural exit) — high risk.

Command: runs/capture_sh127.sh (serialized ladder + frame attempt).