# Frontier SH304 — SESSION-GATED type-4 producer (self-drive handoff), implemented + hermetic-tested

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--taskv4-seed session`). Completes the recon-v3 session-producer handoff
the operator named ("gate the type-4 producer on APP_READY + live DM to replace
the harness seed") — the one deliverable that was still genuinely absent from the
tree.

## What was already true at HEAD (do-not-re-tread)

- `type4_frame_thunk` (SH60, `--taskv4-seed frame`) is the harness task-driven-frame
  producer: it queues a present on EVERY w4=4 dispatch unconditionally (RENDERCTX
  self-guard only). Verified green: 24 task frames swap Ok(0x1), 197 pops, 0 json
  abort, EXIT 124.
- `[0x106829ea8]` (type-4 vector) is true .bss — external glue, no in-image writer;
  host install only.
- MH_* atoms (MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY/GAME_LOADED) are set-only
  observables (jni.rs CallVoidMethod). They stay false on bare-boot headless until
  a real do-init owns a live DM.
- Route-B live-DM structural gate UNCHANGED (no make_shared<DataModel>; DM-root 0).

## What SH304 adds (the missing session-gated producer)

Per recon-selfdrive-seed-jsonfix.md §B + the operator handoff, the producer must
ONLY emit task-driven frames once a REAL session owns a live DataModel — and it
must stay inert otherwise (the host reacts to a session, never fabricates one).
This is the "replace the harness seed when MH_APP_READY^live-DM" wiring:
exactly what was named but not implemented.

New, default-inert (elfjit.rs `examples/elfjit.rs`):

1. `session_producer_gate(mh_app_ready, live_dm) -> bool` — pure logic: `mh_app_ready
   && live_dm`. Hermetic-testable.
2. `session_live_dm()` — page-guarded read of the current-DM holder [0x106391908]
   (SH172 GETTER target, SH174 latch arm) OR do-init DM-root [0x106a68818].
3. `type4_session_gated_thunk(...)` — the host thunk; each dispatch reads MH_APP_READY
   + live-DM. GATED -> queues a real present exactly like `type4_frame_thunk`
   (engine make-current -> frame-fn -> swap on the recovered real ctx via the
   presenter). UNGATED -> bounded-log + return 0 (inert, no present). Producer-only,
   safe on the drain thread (no EGL), never re-enters the dispatcher/drain.
4. Install path: `--taskv4-seed session` registers and writes it into
   `[0x106829ea8]` (same write site as `frame`).

Behavior split vs `--taskv4-seed frame`:
- `frame`  = unconditional harness seed (recon-v3 deliverable, VERIFIED green, kept).
- `session`= session-gated: inert until MH_APP_READY && live-DM, then self-drives.
  This is the correct long-term wiring for when Route B produces a live session.

## Verify (repro)

- `cargo test -p arm64jit --example elfjit -- sh304_session_gate` = 3 passed.
- `cargo test -p arm64jit --examples` = 130/0 (127 baseline + 3 new sh304).
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0.
- elfjit.rs stays under the 1MB pre-commit hook (+7B margin) after back-filling the
  addition from verbose comment/prose (SH115/117/118/122/156/157/159/159c/177/223/
  226/236/255/264/272/276 + option-doc blocks) — all facts/addresses preserved, no
  behavior touched.
- Recon-v3 deliverable re-verified green at this HEAD first (24 frames swap Ok(0x1),
  197 pops, 0 json abort, EXIT 124) — `runs/capture_taskv4_frame.sh`.

## Measure on the real binary (session-gated path inert on bare-boot)

The gated producer is latent-but-correct: on a session-less bare boot MH_APP_READY
is false, so `--taskv4-seed session` dispatches log `UNGATED ... inert` and present
ZERO frames — the host no longer fabricates the session; it only reacts to one.
The gated branch fires only when a live DM exists (still the Route-B structural
gate). Repro: `runs/capture_sh304_session_producer.sh`.

## HONEST (do-not-over-claim)

- Implemented + hermetic-tested the wiring; on the REAL binary the gated branch
  cannot fire yet (MH_APP_READY stays false until do-init owns a live DM). This is
  latent-but-correct prep exactly as the operator described — it fires the instant
  a real session advances.
- Route-B live-DM structural gate UNCHANGED. SH174 capture-latch stays the single
  forward hook. No production path runs with the new thunk (opt-in flag only);
  default `--taskv4-seed frame` keeps the verified recon-v3 plane unchanged.
- elfjit.rs prose was condensed to hold the 1MB hook; no test was weakened, no
  behavior changed (pure comment/string trims + the new default-inert code).