# Frontier SH392 — arm the SH391 render-determinism guard on the canonical HARD-GATE artifact

Date: 2026-09-21. Single-agent (cone suppressed).

## Why this was the genuinely open in-scope defect

SH391 (previous cycle) shipped the deterministic fix for the SH345 render-plane flake: a
guard `routeb_render_memcpy16_guard` (jit.rs:1440) that, at the pinned memcpy16 leaf
guest 0x102859fd0, zeroes x1 when the leaf would perform a provably no-op SELF-COPY
(x0==x1, dest non-writable) so `cbz x1` short-circuits BEFORE the `str q0,[x0]` crash
store. A real walker copy (x0!=x1) is `b.ne`-exited and untouched, so the guard cannot
mask a genuine copy.

TEACHING GAP: the guard is opt-in via `JIT_ROUTEB_RENDER_MEMCPY16_GUARD`, and the audit
this cycle found the canonical recon-v3 artifact runbook `runs/capture_taskv4_frame.sh`
never set it. So every cycle the "24 real task frames / 24 node pops / 0 crash" line was
RE-verified green, it was green WITHOUT the deterministic fix — the ~1/25 SH345 SIGSEGV
remained live and the script's MAX_ATTEMPTS=6 loop was (still) hiding it on a retry. That
is exactly the retry-hide SH391 was meant to eliminate; the fix existed but was inert in
the artifact that produces the HARD-GATE evidence.

## The fix

Armed `JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1` in `run_once`'s env so the frame deliverable is
deterministic-by-construction. Rewrote the SH345 header to state the fix is not-retry-hide
and that the retry loop survives only for unrelated pre-existing run-variable walls
(SH353-class), not for this flake.

## Verification (real libroblox.so, this HEAD)

`DISPLAY=:599 ./runs/capture_taskv4_frame.sh` with the guard armed:
- confirmed-green attempt 1
- 24 real task-driven frames `present #N ... swap Ok(0x1)` (the deliverable marker)
- 0 SIGSEGV/ABRT, 0 json abort
- guard-firings = 0 on the clean run (correct: no divergence arm this run, guard inert)

Workspace: `cargo test --workspace` EXIT 0 (445 arm64jit lib tests + all bins, pre-existing
warnings only). No Rust production path / JIT hook default / guest byte touched (runbook +
docs only).

## Honest status

Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0).
SH174 capture-latch stays the single forward observer. Do-not-re-tread unchanged (SH388
setDataModelToCurrent, SH385 LSM crossings, SH355/356 EC reader-gate, SH362/375 0x258b5d8,
SH367 window-attach real, SH365 ALooper, SH379 governor gates full-ladder, SH380 -9 string,
SH248h map-header, SH381 once-lambda, SH267 node-cell).