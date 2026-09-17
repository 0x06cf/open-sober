# SH246 — scope-patch the continuation's `operator_new(0x28)` call site so the 0x28 closure boxes a REAL object, then MEASURE the continuation is allocation-walled (bad_alloc from other op_new sites, not this one)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(arm64jit lib + elfjit example sh246 1 passed). Route-B DMCONT focus.

## Context (do-not-re-tread)
SH245 made the REAL continueAfterFlagsLoaded_ (0x102bd1d68) execute its full body
headlessly (getter-tail b->ret + M+0x48 app-name seed), stopping one gate deeper:
the continuation's closure build at `bl operator_new` (0x102bd2128, w0=0x28)
returns NULL headlessly (allocator-activation byte [0x10727570c].bit0 clear,
size>0xa -> `mov x19,xzr` -> NULL) -> std::bad_alloc. SH245 already measured the
BROAD fix (seed bit0=1 so ALL op_new goes real-alloc) is a regression (3/3 early
SIGABRT). SH245's candidate (1) = SCOPE-materialize a leaked box at exactly this
call site. This cycle implements that lever AND measures whether it advances.

## The lever (scoped, opt-in JIT_ROUTEB_DM_CONT_OPNEW_BOX)
3-slot call-site patch at guest 0x102bd2120/24/28:
  file 0x2bd2120 = mov w0,#0x28   (0x52800500)
  file 0x2bd2124 = mov w1,#0x8    (0x52800101)
  file 0x2bd2128 = bl 0x1db1a38   (0x97c77e44)  operator_new
replaced (byte-guarded, idempotent) with materializing a leaked 0x40 zeroed box:
  movz x0,#hi.. ; movk x0,#..lsl16 ; movk x0,#..lsl32   (48-bit host heap ptr,
asserted < 2^48), then falls through to 0x2bd212c (`adrp x8,..`). The continuation
writes its closure into [x0]+0/0x10/0x20 -> a real leaked object, no NULL-box.
CRITICAL: `block_cache_drop_region` matches blocks by ENTRY pc, and this call site
is mid-straight-line-block (entry 0x102bd1dfc per region-watch). A window-only drop
left the stale compiled block calling op_new (measured: boxpatch fires but still
bad_alloc). Fix: drop the WHOLE continuation region [0x102bd1d68, 0x102bd2600).
With that, region-watch shows op_new 0x1021db1a38 = **0 hits** — the patched block
re-translates and never inlines op_new (the lever is real and engaged).

## Measured — the continuation is ALLOCATION-WALLED, not this-site-walled
Even with the closure op_new fully eliminated (0 inlined hits), the continuation
STILL bad_allocs every run (7+ runs, EXIT 134/139, off & on). The bad_alloc comes
from a DIFFERENT op_new site in the continuation's straight-line path — the
continuation PERVASIVELY allocates via both operator_new variants:
  - file 0x2bd2058 `bl 0x2338ef4` = nativeAppBridgeAppStart (app-controller) whose
    body uses the OTHER vari-operator_new `0x1d96768` at 0x28/0x20 (3 direct sites)
    — all NULL-returners headlessly;
  - plus std::string / JSON / app-start constructions throughout the body that
    call op_new 0x1db1a38 or 0x1d96768 with size>0xa (same gate, same NULL).
A diagnostic A/B (ret the `bl 0x2338ef4` @0x2bd2058) did NOT shift the observable
bad_alloc either — the alloc wall precedes/is-broader-than any one call site.
CONCLUSION: continueAfterFlagsLoaded_'s forward path is gated on headless REAL
allocations that this JIT cannot satisfy (every op_new fast-path NULLs; the
real-alloc path is the SH245-#4 regression). This is the AppBridge lifecycle /
live-world-build line SH174/SH204 already measured as the headless ceiling ("never
a DM factory, ZERO GuiObjects, do-not-chase").

## Honest (do-not-over-claim)
Implements SH245 candidate (1) correctly and PROVES it engages (0 op_new hits
after the widened drop). Does NOT advance Route B: the measured result is that the
continuation's next wall is not a single seedable op_new site but the pervasive
headless-allocation wall of the AppBridge/app-start construction — a dead-end under
the standing SH174/SH204 "AppBridge do-not-chase" doctrine, NOT a migration-gate
(no GPU needed; the allocator simply isn't headless-producible). The box patch is
inert by default (env-gated), byte-guarded, and safe; it stays as a forward
scoped-lever should a real session ever reach this code. The standing Route-B
live-DM structural gate is UNCHANGED; SH174 capture-latch stays the single forward
hook.

## Code / files
- crates/arm64jit/examples/elfjit.rs: `routeb_patch_cont_opnew_box()` (opt-in
  JIT_ROUTEB_DM_CONT_OPNEW_BOX, real-image byte-guard on 3 words, idempotent,
  <2^48 box assert, whole-continuation block-cache drop). Hooked into the --v2boot
  block next to routeb_patch_getter_fmod_tail_ret (self-guards on its own env).
- +hermetic `sh246_cont_opnew_closure_boxes_real_object_not_null`: pins the 3
  real-image window words, round-trips a 48-bit box ptr through movz/movk hw0-hw2,
  asserts opcode classes, verifies the real-image word guard passes and a drifted /
  already-applied site fails loudly. Examples batch green (87/0).
- Diagnostic removed after measurement (not shipped): skip-appbridge A/B
  (JIT_ROUTEB_DM_CONT_SKIP_APPBRIDGE) proved the alloc wall is broader than the
  bl 0x2338ef4 site.
- Repro runs/capture_sh246_cont_opnew_box.sh (off|on) + runs/batch_sh246.sh +
  runs/diag_sh246_skip_appbridge.sh.

## Next (honest, single-agent)
Route-B live-DM structural gate reconfirmed. The DMCONT continuation line is now
MEASURED to terminate in the pervasiveness headless-allocation wall (AppBridge
do-not-chase, SH174/SH204). Standing forward hook unchanged: SH174 capture-latch
arming *(0x106391908) at a real make_shared<DataModel>. Keep grinding the Route-B
line (operator's MIGRATION-IS-NOT-A-STOP directive), but this specific continuation
sub-line is provably allocator-gated on this JIT.