# Frontier SH370 — SH357-consolidation completion: lock the sh323 cookie-jar/settings guard test (determinism hardening)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One-line test-harness determinism
fix (arm64jit/src/jit.rs: `sh323_settings_sso_seed_guard` now holds ROUTEB_PROC_TEST_LOCK). No
assertion weakened; no production path edited. Route-B live-DM structural gate UNCHANGED.

## Why
While confirming recon-v3 deliverables green at HEAD, `cargo test --workspace` intermittently
failed. Isolated-arm64jit ran clean 10/10, but `--test-threads=16` reproduced two failure types:
(a) `futex_requeue_actually_moves_waiter` (the documented SH345/346 load-sensitive real-kernel
timing class, seen ~1/20-1/25 under parallel full-suite load — canonical gate is deterministically
green), and (b) `sh248d`/`sh248e`/`sh323` shared-fixed-page races (the SH357 class). The SH357
consolidation locked every routeb-family cookie-jar/adapter/once test EXCEPT `sh323` — which writes
the SAME fixed cookie-jar page (CELL_A=0x106ed7a18, CELL_B=0x106ed7a28 == sh248d's B) with zero
shared-lock serialization, so it raced the locked siblings mid-assert and the sibling re-zero landed
("cookie-jar slot A/B must be seeded" left:0).

## The fix
Add the consolidated lock (`let _mgr_guard = ROUTEB_PROC_TEST_LOCK.lock()...`) at the top of
`sh323_settings_sso_seed_guard_is_env_pc_gated_and_seeds_cell`, exactly like sh248d/sh248e/sh273/
sh175. Behavior-neutral; no assertion removed or relaxed.

## Measured
- `sh323` passes (jit.rs 1 passed/0 failed).
- sh323/248d/248e isolated `--test-threads=16`: 50/50 clean.
- `cargo test --workspace`: EXIT 0, 612 passed/0 failed (6/6 consecutive canonical runs + earlier
  10/10 + 8/8). Deterministic-green canonical gate.
- RouteB-family parallel-stress failure set drops from two types (sh248d-family + futex) to only
  the documented futex/sharded-page load-sensitive class — which only appears under artificial
  `--test-threads=16` and is the accepted SH345/346 residual, not the canonical gate.

## Honest
Test-harness determinism only — no Route-B forward (DM-root [0x106a68818]=0, MH_* false) and no
production-path change. The residual futex flake under extreme parallelism is pre-existing and
documented (SH345/346/357); the canonical `cargo test --workspace` gate is reproducible-green.

## Files
crates/arm64jit/src/jit.rs (+6 loc lock); docs/frontier-sh370-*.md (this); HANDOFF.md; STATUS.md
(runs/STATUS.md). Commit local dev only (operator pushes).