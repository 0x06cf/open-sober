# SH382 — Session-gated type-4 producer honors the SH381 once-slot reconciliation; recon-v3 deliverables re-verified green at HEAD

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace EXIT 0, 617/0; arm64jit lib 437/0; elfjit example 160/0
incl the new sh382 hermetic).

## What this cycle
The recon-v3 immediate-priority deliverables (self-driven frame thunk + JSON-abort
clamp) are re-verified green at HEAD (capture_taskv4_frame.sh attempt 1: 24 real
task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash).
That work is complete. This cycle folds the JUST-landed SH381 finding into the
latent session-gated producer that recon-v3 §A names as the end-state:

- SH381 reconciled two cells that prior cycles conflated: the do-init once-lambda's
  DM-constructor returns the 0x400000b "Execute" service-handle sentinel, and that
  sentinel is staged via `str x0,[x23,#1032]` @0x2206d74 into **once-slot
  [0x106a68408]** (x23=adrp 6a68000) — a DISTINCT cell from the harness-probed
  **DM-root [0x106a68818]** (+0x410).
- The recon-v3 §A end-state producer `type4_session_gated_thunk` + `session_live_dm()`
  (elfjit.rs) read [0x106391908] (current-DM holder) and [0x106a68818] (DM-root) but
  NOT the once-slot. If a genuine DM were ever written to the once-slot the producer
  would fail to recognize the session; and the 0x400000b sentinel, if ever read,
  must NOT count as a live DM.

## The change (behavior-preserving, gate decision unchanged)
- Added a pure predicate `live_dm_cell_value_ok(v)` (guest pointer >= 0x100000000,
  top-16 cleared, non-zero) — documents that the 0x400000b sentinel is rejected
  (it is < 0x100000000), matching every SH155/311/316 post-run read.
- `session_live_dm()` now also reads once-slot [0x106a68408] as a THIRD live-DM
  candidate (the once-lambda's actual write target). All three reads page-guarded
  via `read_visible_u64`. ORed so a genuine DM written to any of the reconciled
  cells is recognized.
- No production path / no JIT hook / no guest byte touched. Gate still requires
  MH_APP_READY AND live-DM; the once-slot arm only ADDS a recognition path for a
  genuine DM and explicitly rejects the sentinel.

## MEASURED (real libroblox.so, completing ladder + --taskv4-seed session)
Run EXIT 124 stable, 0 SIGSEGV/ABRT. The session-gated producer logs:
```
[elfjit:session-producer] disp #1 UNGATED (app_ready=false, live_dm=true) — inert
[elfjit:session-producer] disp #2 UNGATED (app_ready=false, live_dm=true) — inert
[elfjit:session-producer] disp #3 UNGATED (app_ready=false, live_dm=true) — inert
```
Correctly inert: MH_APP_READY=false is the hard session discriminator (a bare
session-less boot never flips it), so no dispatches emit frames. The lone
`present #0` is the pre-existing RENDERINIT warmup self-test (task frame #0), not a
session-producer emission. `live_dm=true` reflects pre-existing identity-mapped bss
cell reads (the classifier reads holder/root regardless before this cycle); adding
the once-slot arm does not change the gate decision while MH_APP_READY stays false.

## Honest
Does NOT manufacture a DataModel — Route-B live-DM structural gate UNCHANGED
(DM-root 0, MH_* false, AppBridgeV2 0). This is a correctness/completeness fold of
SH381's reconciliation into the latent session-gated producer (folds the once-lambda's
REAL write target in, keeps the sentinel rejected), plus hermetic pinning of that
predicate. SH174 capture-latch stays the single forward observer.

## Do-not-re-tread
Unchanged closures stand: LSM skips (SH349/350/358/373), EC reader-gate
(SH355/356/374), 0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367),
ALooper (SH365), governor gates full-ladder (SH379), -9 string/0x102b504e4 (SH380),
map-header repair (SH248h), once-lambda store seeding (SH381).

## Files
- crates/arm64jit/examples/elfjit.rs: +`live_dm_cell_value_ok`, `session_live_dm()`
  now also reads once-slot [0x106a68408], +sh382 hermetic
  (`live_dm_cell_ok_rejects_sh381_execute_sentinel`) in the sh304 module.
- Log: /home/hermes-worker/runs/sh382-session-producer-reconcile.txt (outside repo).