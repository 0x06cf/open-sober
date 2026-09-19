# SH466 — promote the SESSION PRODUCER HANDOFF gate core into the tested library

Single-agent (cone suppressed). The recon-v3 immediate-priority deliverables were
re-verified green at this HEAD FIRST (capture_taskv4_frame.sh attempt 1: 24 real
task-driven frames `present swap Ok(0x1)`, dispatch #2264000, 195 node pops, 0 json
abort, 0 crash, EXIT 0). Workspace green (cargo test --workspace EXIT 0; arm64jit
lib 677/0 incl. 1 new sh466 hermetic, was 676; cargo build --workspace +
--example elfjit OK). Production code in session.rs + the elfjit example.

## The gap closed

The recon-v3 §A END-STATE / SEP-17 "SESSION PRODUCER HANDOFF" gate — the logic that
lets type-4 self-drive emit REAL task frames only once a REAL session owns a live
DataModel — is load-bearing runtime logic, but it lived ONLY in the untested elfjit
example and was pinned ONLY under `cargo test --example elfjit`. It never ran in the
workspace suite, so a drift either way slipped through the `cargo test --workspace`
gate (the project's actual regression gate). SH466 promotes the two PURE pieces into
the library so they are hermetic-pinned where all other runtime contracts live.

## What changed

- `session.rs`: new `pub fn session_producer_gate(mh_app_ready, live_dm) -> bool`
  (the recon §B / SH303 gate: GATED only when app-ready AND live-DM) and
  `pub fn live_dm_cell_value_ok(v) -> bool` (the classifier that accepts only
  coherent guest-visible pointers `>= 2^32` with a clear top byte and non-zero, so
  it REJECTS the SH381 do-init once-lambda "Execute" service-handle sentinel
  `0x400000b` — the exact case where a naive producer would mistake the once-slot
  for a live DM). Both pure, deterministic, no env, no guest bytes.
- elfjit.rs: the two local copies are now thin wrappers over the library functions
  (single source of truth), and the SH304 example tests still pass. Net file size
  DROPPED 1,048,392 -> 1,048,152 B (under the 1MiB pre-commit hook).
- New hermetic `sh466_session_producer_handoff_gate_and_sentinel_rejection` pins the
  full 2x2 gate truth table + the sentinel / zero / boundary (2^32 edge, top-byte)
  rejection surface.

## Honest

NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root
[0x106a68818]=0 structural at the write site per SH462/463). This is
BUILD-THE-RUNTIME coverage completion: the self-drive handoff's pure decision
logic is now a tested library contract instead of example-only untested glue. The
gated producer remains latent-but-correct — it fires the instant a real session
advances (MH_APP_READY AND a live DM); the library functions are exactly what the
toolchain uses to classify that moment. No re-treads, not a render-plane visual.

Files: docs/frontier-sh466-session-producer-handoff-gate.md + crates/arm64jit/
src/session.rs (2 pub fns + hermetic) + crates/arm64jit/examples/elfjit.rs
(wrappers). Commit (SH466).