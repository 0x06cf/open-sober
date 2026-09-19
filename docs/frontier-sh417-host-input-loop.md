# Frontier SH417 — the persistent-tracker host input LOOP (STATUS next-forward #3)

Date: 2026-09-19, hermes-worker, single-agent (cone suppressed). Workspace green
before (cargo test --workspace EXIT 0, 651/0) and after (arm64jit lib 470/0 incl.
the new SH417 hermetic; full workspace re-verified). Capture
runs/capture_sh417_host_input_loop.sh, log /home/hermes-worker/runs/sh417-input-loop.txt
(outside repo). Production code only in session.rs + elfjit.rs rung (both off / at
the 1MiB hooks, no guest byte changed); jit.rs untouched.

## Why this cycle

STATUS.md next-forward #3 named the closest unblocked, non-re-tread surface: "Hang
`drive_host_input_poll` on a real host loop (poll repeatedly while a session owns a
screen) — it is currently one-shot opt-in, proven on real binary." The SH416 one-shot
poll built a FRESH `PointerTracker` every call, so a press's DOWN in poll i and its
MOVE in poll i+1 were translated as two different pointers — the pointer-down /
multi-touch state that is exactly the input axis's job (a real finger stays down
across X motion events) was being thrown away between polls. A real host loop must
own ONE tracker for the whole session.

## What landed

- **session.rs**: `drive_host_input_loop(iimg, ib, tpidr, boot_sp, iterations) ->
  usize` — the persistent-tracker host input loop. Checks the SAME three guards as
  the SH414 pump / SH416 poll ONCE up front (bridge env armed + a real registered
  window XID + a live input image), then drains N non-blocking batches of the
  registered ANativeWindow through ONE `PointerTracker` (pointer state survives
  across iterations), marshalling each translated MotionEvent into the guest
  `nativePassInput` via the exact SH413/414 ABI. Bounded by `iterations` so a
  harness never spins. A mid-loop X error (server closed the window) logs + stops
  the loop, keeping events already delivered — it never aborts the session.
- **elfjit.rs**: `--v2boot-input-loop` rung (INPUT_LOOP_ITERS env, default 8) —
  the SH416 one-shot becomes a bounded real host loop.
- New hermetic `sh417_host_input_loop_inert_without_all_guards` (arm64jit lib
  469 -> 470): all three guard trips (bridge off, no XID, empty image) return 0
  BEFORE `pump_registered_window` is reached (no X connect), and a zero-iteration
  loop is bounded (no hang).

## Measured (real libroblox.so, SH416 env + INPUT_LOOP_ITERS=4)

- Window wired: `[elfjit:anativewindow] wired real X11 window XID=0x200000 on :308
  as the guest ANativeWindow`.
- `[session-drive] host-input loop: polling real window 0x200000 on :308 for 4
  iterations (persistent tracker, bridge armed)` — the loop ran all 4 iterations,
  each draining `0 raw -> 0 translated -> 0 delivered`, `running total 0`,
  then `done 4 iterations, 0 real events delivered`.
- `[elfjit:v2boot-input-loop] SH417 persistent-tracker host-input loop over 4
  iterations delivered 0 events`.
- No input-path crash. The run's terminal SIGSEGV/SIGABRT is the KNOWN pre-existing
  Route-B lane at guestpc 0x1029f3f7c (nativeInit "outside image", SH416 documented
  it identically), NOT this change — it happens in nativeInitializeNativeFlags before
  the loop's delivery point is even meaningful, and the loop completes before it.

## Honest

Inert-by-construction exactly like every runtime axis: 0 real events on a boot with
no live session — there is no constructed login/home screen to deliver to until a
completed do-init owns a live DataModel, so `drive_host_input_loop` returns 0 and
moves nothing. It is the cause-not-symptom input runtime surface (BUILD-THE-RUNTIME):
the moment a live DM advances, the same bounded loop delivers real desktop pointer
input to the guest's constructed screen, with correct persisting pointer state across
frames. Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0,
MH_GAME_LOADED false, no make_shared). recon-v3 deliverables re-verified green at
HEAD. No re-treads.

## Files

- crates/arm64jit/src/session.rs (+drive_host_input_loop, +sh417 hermetic, doc on
  the one-shot poll corrected to point at the loop)
- crates/arm64jit/examples/elfjit.rs (+--v2boot-input-loop rung)
- runs/capture_sh417_host_input_loop.sh
- docs/frontier-sh417-host-input-loop.md

## Next (standing)

Route B / do-init completeness stays the top front (STATUS #1): the substrate now
REPORTS the completion markers, the marker stays non-live until the engine's own
session ctor builds the DM world. Do-not-re-tread unchanged: LSM skips/rebuilds
(SH349/350/358/373/375/377/378/385/396), setDataModelToCurrent (SH388), EC reader
(SH355/356/374), window-attach real (SH367), ALooper (SH365), governor gates
(SH379). Continue the SEP-18 runtime build (real Activity/AppBridge/JNI-lifecycle/
GLES/session drive) as the one cause-not-symptom front.