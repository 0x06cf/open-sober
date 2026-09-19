# Frontier SH346 — futex flake fix (WAKE-side convergence) + re-measured the SH343 full-ladder terminal: run-variable app-start-region arms, all SH285-class live-object walls; no Route-B advance

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace 594/0). One production-code test-hardening change
(arm64jit/src/jit.rs, futex_requeue test) + measurement only on the real
binary. elfjit.rs unchanged.

## What changed (production-adjacent)
`futex_requeue_actually_moves_waiter` flaked once on a full-workspace parallel
run (1/25-ish load-sensitive, same class SH345 documented). The REQUEUE side
already spun (bounded, keeping the SH133 fake-return-0 regression strong); the
immediate single `WAKE` syscall could catch a freshly-requeued waiter
mid-transition under parallel load and return 0. Fix mirrors the REQUEUE side:
a bounded spin that converges once the requeued waiter is fully on dst, and
still asserts `woken>=1` after the 20s window (a faked-0 handler exhausts it
and fails — the regression is NOT weakened). Production passthrough unchanged.
Workspace re-verified green (594/0) after the change.

## Re-measured (real libroblox.so, SH343 full-ladder env + KEYFIX, strict-serial)
The SH344c-recorded terminal and verdict were re-derived from 4 fresh runs. The
app-start factory (nativeAppBridgeAppStart 0x102338510) is now genuinely reached
past the SH285 insert-leaf on the full ladder — the SH248d/e/f + SH259 assign
sites fire (cookie-jar at pc=0x1021f47fc, once-cell 0x102339208, adapter
0x102339018, settings once-guard 0x102339d0c) — but the run then terminates in
one of THREE run-variable arms, all still SH285-class live-object walls, no
DM-root:
- Arm A (2/4 runs 2,3): SIGSEGV guestpc=0x101db1b08 fault=0xff..ff (the SH284/285
  LSM reader/pop live-object read-back wall, unchanged).
- Arm B (1/4 run 4): SIGSEGV guestpc=0x10284cfa0 fault=0x0 (the JNIActivityLifecycle
  nativeOnDestroyed divergence arm, SH344b/345 document it).
- Arm C (1/4 run 1): no host signal caught — guest `std::bad_function_call`
  propagates to the HOST libc++abi (terminating) before/without a JIT signal
  dump; same app-start-region factory fired the SH248d/e/f seeds. Attempted to
  pin the throw with a gdb `__cxa_throw` breakpoint: symbol not host-defined
  (guest libc++ is statically linked + JIT-translated; `__cxa_throw` resolves
  inside the guest, unwinding escapes to host bookkeeping), so the exact guest
  empty-std::function site was not extractable this cycle. It is a third
  app-start-region live-object manifestation (an empty `std::function` target a
  real session ctor would have populated), not a distinct route.

All three arms terminate in the SH285-class live-object wall family; DM-root
[0x106a68818]=0 and MH_* stay false across every arm. This is NOT a new
regression (SH344c already recorded the app-start factory + SH285 terminal
divergence) but re-confirms SH344c's honest reading: the app-start factory is
reached, the LSM lane is where the SESSION-CTOR world-build still dies.

## Honest conclusion
Route-B live-DM structural gate UNCHANGED. No seed produces a live DataModel.
The reconciliation with SH344c: SH344c's "continuation never reaches app-start"
referred to the DMCONT +0x1f0 serialize body (which dies at SH285 before the
bl app-start); the NEW observation this cycle is that the app-start factory
assign-sites fire on a fresh run (SH248d/e/f + SH259 seed writes execute), so
the full ladder does reach the app-start factory body, then dies in the LSM
lane. Both are the same live-object wall; the exact mechanism (SIGSEGV
0xff..ff vs SIGSEGV 0x10284cfa0 vs host-terminating bad_function_call) is
run-variable within ~25% bands. No seedable-forward product.

## Verify
- `cargo test --workspace` = 594 passed / 0 failed after the futex test change.
- 4 strict-serial full-ladder runs; terminal arms A/B/C above. Repro env/logs
  kept under /tmp/sh346-*.txt (gitignored).
- recon-v3 frame artifact (capture_taskv4_frame.sh) re-verified green this
  cycle (see STATUS).

## Files
- arm64jit/src/jit.rs: futex_requeue_actually_moves_waiter WAKE-side bounded
  spin (SH346).
- No elfjit.rs change. Frontier doc: docs/frontier-sh346-routeb-rerun.md.

## Next (unchanged, authoritative)
SEP-17 SESSION-CTOR cause-level drive remains the primary forward. Every driven
rung (do-init -> DMCONT +0x1f0 -> app-start factory -> LSM) is measured; the
terminal is the SH285 live-object wall family, which is cause-not-symptom, not
seedable. SH174 capture-latch stays the single forward hook. The app-start
factory reach (Arm A/C seeds firing) is a real forward-observation worth
tracking but does not change the structural gate. If the nested-bridge heap-issue
(SH151 class) reappears it is next to root-cause; it did not this cycle.