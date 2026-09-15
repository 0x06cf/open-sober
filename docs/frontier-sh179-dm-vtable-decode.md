# SH179 — Workspace-flake fix (DM-capture test race) + fresh packed-RELA/RTTI decode of the genuine DataModel vtable family + capture-latch runbook audit

Date: Sep 15, 2026, hermes-worker. Workspace green (550/0). Commit <SH179COMMIT>.

## 1. Deterministic test-suite fix (code, arm64jit)

The workspace was NOT green at the session start: `cargo test --workspace` failed
(371 passed / 1 failed in the arm64jit lib suite) at `jit::tests::sh167_dm_alloc_capture_is_env_gated_and_never_clobbers_live_hook`.
Running the test alone passed 3/3; the full suite failed at DIFFERENT assert lines each time
(jit.rs:5068 in one run, 5085 in another) — the classic process-global shared-state race under
the Rust parallel test harness, not a production-logic regression.

Root cause: `sh167` (jit.rs:5016) and `sh169_delegating_trail_falls_back_safely_when_no_guest_image`
(jit.rs:5106) run on parallel test threads and both mutate the SAME shared, non-atomic state:
the process-global `PREV_DM_ALLOC_HOOK` static AND the `JIT_DM_ALLOC_CAPTURE(_DELEGATE)` process
env (set/remove via `unsafe { std::env::set_var/remove_var }`, which races a parallel thread's
`var_os` read in the guard — genuine UB in current Rust). So sh167 could read a PREV hook / DELEGATE
flag mid-mutation by sh169.

Fix (no test weakened, no new dependency): a process-global test-only `DM_CAPTURE_TEST_LOCK: Mutex<()>`
in the `jit::tests` module, acquired by BOTH the sh167 and sh169 tests. This serializes exactly the two
shared-state tests (the sh174-worth test is pure and untouched; production is a single-jit_run-thread
per run so the shared static stays correct there). Verifiable: arm64jit lib suite now 5/5 green
(previously ~1/2 flaky), full `cargo test --workspace` = 550 passed / 0 failed.

## 2. Fresh packed-RELA / RTTI decode — the genuine DataModel vtable family (recon, SH178's sanctioned forward)

Per SH178's mandate ("locate the genuine join via a fresh packed-RELA decode of the DM vtable/ctor,
never reuse 0x2206d74/0x2173b3c"), a bounded decode anchored on the RTTI typeinfo — not on the
misattributed `join` — advanced the hunt one structural step past SH178:

- The `N3RBX9DataModelE` mangled name (file VMA 0xcaca6b) is referenced by exactly ONE relocation:
  the slot at **fileVMA 0x6714e20** (guest 0x106714e20). Per Itanium RTTI, typeinfo `__si_class_type_info`
  `T` keeps its type-name pointer at `T+8`, so **RBX::DataModel typeinfo T = fileVMA 0x6714e18**.
- Three RELATIVE relocations store the address of `T` (= guest 0x106714e18) at the `-0x10` RTTI slot of
  three vtable hallmarks: locations **0x67162e0, 0x6716398, 0x67163f0** → candidate DataModel vtables
  **V = 0x67162f0, 0x67163a8, 0x6716400** (all on page 0x671000). My earlier probe mislabeled the
  page-0x671000 slot 0x67162e8 (offset-to-top region) as the vptr; the true vptr candidates are the
  three above.
- The DM object is HEAP-constructed `make_shared`, NOT a static singleton: ZERO relocations anywhere in
  the binary store any of the three vptr values as a static object's first word, and the vptr is not
  materialized via `adrp/add` or `movz/movk` as a full 64-bit constant anywhere in `.text` (0 matches
  for all three candidate values) — the compiler-induced vptr store reaches the object only through the
  runtime make_shared path.
- Plausible `operator-new` sites with sizes in the DataModel range 0x800–0x1200: **0x28f78a0 /
  0x2950b3c / 0x2950b5c (0x1000) and 0x2b37b6c (0x1108)** — 0x2b37b6c is followed by a ctor call with
  `(this=x0, arg=x19, 0x108=x2)`.

HONEST bound: the exact ctor / the exact make_shared "join" / the sizeof constant is NOT conclusively
pinned from static analysis in this cycle — the vptr materialization to those vtables wasn't found by
adrp/add or movz/movk (probable GOT/indirect or this-adjusted base store). This is now a WELL-SCOPED
residual (not a wall): the genuine DM vtable family is pinned (page 0x671000, T=0x6714e18), and the
next recon need only follow the object-vptr STORE from one of the three candidate vtables (e.g. in the
0x2b37b6c ctor) to close sizeof + the join. Even then, per SH178 + the audit below, a located join does
NOT unlock a headless session — the registry [0x1063915a0..0x106392600) is relocation+session-populated.

## 3. DM-capture latch audit (recon subagent deleg_4f69ac65 task-2, READ-ONLY, succeeded)

Verified-READY + two runbook corrections for the GPU-host migration (SH174 `docs/frontier-sh174-migration-runbook.md`):

- LATCH VERIFIED (0.97): `routeb_dm_alloc_capture`/`guard`/`dm_capture_worth` (jit.rs) constants and ABI
  match the sources; a genuine RBX::DataModel alloc (sizeof in [0x1000,0x40000] AND in-image vtable) is
  caught twice-over (base_ok forces capture regardless of size after SH174). Budget/first-8 logging
  non-blocking. Default-inert.
- CORRECTION 1 (runbook STEP A): BOTH `JIT_DM_ALLOC_CAPTURE=1` AND `JIT_DM_ALLOC_CAPTURE_DELEGATE=1`
  are required at the arming marker — without DELEGATE the guard silently refuses to arm on a live engine
  hook, so the runtime GOLD capture line would be absent.
- CORRECTION 2 (runbook STEP B): `prev_hook` MUST be verified non-zero at the arm marker before letting
  the migration run — if prev_hook reads 0, delegation is inert and every freed engine-pool allocation
  falls back to host-calloc and SIGABRTs (EXIT 134, the exact SH167 failure). Abort unless prev_hook != 0
  and ACTIVE != DEFAULT.
- Dynamic-trace feasibility RE-AFFIRMED impossible (~0.9): even a located join cannot be driven headlessly
  — a fabricated `this` cannot substitute the in-session engine object, and the relocated registry's
  semantic content (wired consumer std::function + live current shared_ptr) exists only post-session. The
  SH169 DELEGATE+VALIDATE latch at a REAL app-launch remains the only forward.

## Standing (unchanged)

Live-DM = MIGRATION GATE (~30 cones). Type-4 producer [0x106829ea8] latent-only. R1/R2 content dead on
reachability (downstream of a live DM). The headless dynamic-DM-trace is not built (SH178 verdict holds).
The genuine DataModel vtable family on page 0x671000 is the corrected anchor for any future Route-B work
and for the SH174 runbook's captured-`B` validation.