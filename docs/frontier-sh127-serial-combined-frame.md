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

## PREREQUISITE CONFIRMED (Sep 14, 2026, SH127 — runs/sh127-fwdump.txt)
Recon deleg_b193ac24 + the fwdump run established the precise mechanism and
confirmed the re-drive is VIABLE:
- **Mechanism:** the drain pop-loop 0x102856e40 is a CALLEE of the engine's
  idle-main-loop on guest thread 0 (the MAIN thread) — NOT a worker thread, NOT
  the render thread (cf. --deque-node-live comment "the LIVE drainer's deque
  (guest_tid 0 under --drain-poll)"). STANDALONE: StartApp's jit_run parks forever
  in the idle futex at lr 0x10284d134, so the drain keeps cycling (191 pops).
  COMBINED: StartApp RETURNS (the SH115-serial chain lets the engine complete its
  boot / exit its main loop instead of idling), so after "ladder done" NO guest
  thread is resident in the drain — that is the true cause of 0 frames, not a
  timing bug and not StartApp "parking".
- **Prerequisite check PASSED (fwdump):** after StartApp returns in the combined
  run, the drain's deque-maintenance forward-edges are STILL COHERENT + the type-4
  vector is STILL SEEDED:
    deque-fwd 0x1068262e8=0x10620db24  0x106826300=0x102176bfc
              0x106826308=0x1022199e0  | task-v4 [0x106829ea8]=0x7f00000001d8
  All in-image; the seeded type4_frame_thunk is still installed. So the drain
  state survives StartApp's return; only the resident DRIVER is missing.
- **Low-risk resolution (recommended):** re-enter the engine's idle-main-loop
  (NOT the drain body) as a BOUNDED top-level jit_run on a single thread AFTER
  the ladder joined + LADLED_DONE + RENDERCTX published, reusing --drain-poll's
  finite-timeout heartbeat (NO --drain-force-pop, NO block_cache_drop_region —
  that re-imports SH44). Because JIT_SERIALIZE_RENDER guarantees the ladder's
  jit_runs are done, exactly ONE top-level jit_run is in flight at re-drive time
  (the SH55/64 concurrency half is already gone). Remaining uncertainty: the
  idle-main-loop ENTRY pc is not yet disasm-pinned (lr 0x10284d134 is the futex
  inside the loop, and 0x102856e40 the drain body); and a bounded-exit discipline
  must be applied (TASKFRAME_WINDOW_MS/MAX_FRAMES) or the loop hangs (exit 124
  lost). A dedicated read-only disasm of the idle-loop entry is the next
  prerequisite before coding the re-drive.

Command: runs/capture_sh127.sh (serialized ladder + frame attempt);
runs/sh127-fwdump.txt (deque coherence proof).

## RE-DRIVE ENTRY PINNED (Sep 14, 2026 — deleg_591180f0, read-only disasm)
The top-level loop to re-drive is the **drain pop-loop guest 0x102856e40** (NOT
the wait-primitive 0x10284d014 — that is its park point). aarch64 objdump on the
real binary:
- **Entry:** jit_run(0x102856e40, x0=deque head/tail struct, x1=task-queue obj,
  x2=finite_timeout_ms). Loop header 0x102856f04; per-node pop-loop 0x102856f94.
- **Caller chain (REVERSED vs earlier assumption):** drain 0x102856e40 -> (loop
  header) `bl 0x284d014` wait-prim at guest 0x102856f44 -> `bl 0x62d62d0`
  (futex wrapper) at guest 0x10284d130 with lr 0x10284d134; and -> node-processor
  `bl 0x285682c` at guest 0x102857020 (dispatching type-4 via vt[40]).
- **Bounded exit recipe:** pass a FINITE x2 (timeout ms) so the wait-prim takes
  the timed-futex path (else parks forever = the exit-124 hang). The drain also
  exits on its seq-version guard (x21 = seq>>32; b.ne to exit returning w27&1).
  So `jit_run(0x102856e40, x0, x1, finite_x2)` terminates cleanly.
- **LAST OPEN QUESTION (blocks coding the re-drive):** the deque-root x0 and
  task-queue-obj x1 live pointers. In the STANDALONE path these are recovered
  from the running drainer's x20 (elfjit.rs:6366-6410, pc in [DRAIN_LO,DRAIN_HI)
  && is_ptr(x20)); after StartApp returns (combined path) no thread is in the
  drain, so x20 is unavailable. The re-drive therefore needs the deque root
  derived from a stable global (candidates: the deque-fwd globals
  0x1068262e8/300/308 confirmed coherent post-return) OR the root re-captured
  during the ladder before StartApp parks. Deriving this is the next
  prerequisite before the re-drive is codeable.