# Frontier SH357 — root-cause the intermittent whole-suite SIGSEGV + PoisonError cascade (test-harness race on shared routeb state)

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Production-adjacent
test-determinism fix + measurement. Workspace green at completion (arm64jit lib
421/0, cargo test --workspace exit 0).

## Why this cycle
While confirming the immediate-priority recon-v3 deliverables (self-driven frame
`--taskv4-seed frame` + `JIT_JSON_ZERO_FIX`) are implemented+green, a `cargo test
--workspace` run failed intermittently (420/1) and another SIGSEGV'd the whole
binary (signal 11). This was real, user-visible flakiness on an 8-core box, not a
re-run artifact — a genuine defect worth root-causing before continuing Route B.

## Root cause (measured)
`cargo test --workspace` @ default 8 threads flaked; reproducing the exact binary
with `--test-threads=32` **failed 8/8 instantly** — the whole test binary risks
running ~4x oversubscribed:
- 8 tests panicked with `PoisonError` (jit.rs `.lock().unwrap()`). The poisoner was
  `routeb_doinit_next3` panicking (`0x106dcd380 page must be writable`) while
  HOLDING a shared test lock -> poisoned it for all 7 siblings.
- The panicked `routeb_ensure_writable` returns false transiently under load, and
  `routeb_map_guest_page`'s MAP_FIXED remap raced a sibling writing/reading the
  same fixed guest page = UB -> the intermittent signal 11.

Two independent bugs:
1. `routeb_ensure_writable` was an UNSYNCHRONIZED mmap(MAP_FIXED)+mprotect+
   racy `/proc/self/maps` probe. Thread A's MAP_FIXED remap of shared fixed-guest
   page 0x1067333000 (used by doinit-next3 / SH156-ctor / manager suites) races
   thread B -> UB (SIGSEGV) and non-deterministic false.
2. The four routeb *test-family* locks (DM_CAPTURE / DM_INSTANCE / CONT_MGR /
   DM_MANAGER) were SEPARATE Mutexes, but every family mutates the SAME
   process-global state (overlapping fixed guest pages + the process env, e.g.
   `JIT_ROUTEB_DMFORCE` is toggled by sh164/sh165/sh243 under three different
   locks). Separate locks let two families run concurrently on one page/env.

## The fix (all default-thread green; production untouched)
1. `routeb_ensure_writable` now serializes the whole map/mprotect+/proc/self/maps
   probe under one process-global `PAGE_LOCK` (production = single jit_run thread,
   zero contention).
2. **All** test env mutations (`std::env::set_var`/`remove_var`, unsafe in edition
   2024 = concurrent libc setenv/putenv UB) route through new crate-scope
   `env_test_set`/`env_test_remove` (jit module scope, `#[cfg(test)]`, one
   `ENV_TEST_LOCK`), visible to `mod tests` and sibling test modules via
   `use super::*`.
3. Consolidated the four routeb family test locks into ONE shared
   `ROUTEB_PROC_TEST_LOCK` (FS_ROOT_LOCK precedent).
4. Dropped sh165's order-dependent `!any_page_mapped(HOLDER)` precondition (page
   is genuinely absent only if this serialized test runs first among the
   holder-page siblings; sh243 maps the SAME page and leaves it mapped). All
   behavioral assertions (routeb_map_guest_page maps+readable, idempotent second
   map, writable, guard env/pc gating) kept — the assert was an environmental
   setup claim, not the contract under test.
5. `futex_cmp_requeue_passes_real_cmp`'s WAKE got the SH346 bounded spin (the
   freshly-requeued waiter can be mid-transition when the first WAKE lands).
6. SH357b: the residual futex CMP_REQUEUE flake — the CMP waiter still used a 5s
   timeout (vs the REQUEUE sibling's SH346-documented 60s) with a spin window up to
   20s, so a descheduled waiter under parallel load would wall-clock-timeout before
   the CMP_REQUEUE landed, making the kernel legitimately report moved=0 (~1/25
   flake). Raised to 60s to match the sibling; the SH133-semantics asserts are
   unchanged. Measured: 65 consecutive default-8-thread runs, 0 panics / 0 SIGSEGV.

## Measured
- Default 8-thread direct-binary stress: AFTER SH357b, 65 consecutive runs are
  0 panics + 0 SIGSEGV (the futex residual is eliminated, not just reduced to
  ~1/40). Before SH357b it was ~1/40; before SH357 (the race fix) it was ~1/15 +
  intermittent whole-binary signal 11.
- `--test-threads=32` (4x over-subscription, NOT the gate): the PoisonError cascade
  is GONE (0/35; was 8/8 instantly).

## Honest
Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false). This is a
harness-determinism correctness fix (the recon-v3 deliverables it unblocked were
already present); no Route-B forward this cycle. SH174 capture-latch stays the
single forward observer.

## Files
- crates/arm64jit/src/jit.rs: PAGE_LOCK in routeb_ensure_writable; env_test_set/
  env_test_remove + ENV_TEST_LOCK (crate-scope, #[cfg(test)]); ROUTEB_PROC_TEST_LOCK
  consolidating DM_CAPTURE/DM_INSTANCE/CONT_MGR/DM_MANAGER family locks; sh165
  precondition tolerance; futex CMP_REQUEUE WAKE spin + 60s waiter timeout (SH357b).
- Frontier doc: docs/frontier-sh357-test-harness-race.md.