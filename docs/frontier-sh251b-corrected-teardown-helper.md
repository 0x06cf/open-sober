# SH251b CORRECTION — 0x102b9eca0 is a SHARED RUNTIME TEARDOWN HELPER, not the
# next-unsynthesized Route-B object (the crash is NULL-this unwind cleanup, not forward construction)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.
Correction to: docs/frontier-sh251b-classdesc-register-advances-ctor-fencepost.md (SH251b) and
its +1 hermetic run.

## What SH251b claimed
SH251b reported that after the PlayerGui/ScreenGui class-register getters drove ok, the register
"continuation" entered "a large ctor 0x102b9eca0 (sub sp,#0x480)" whose `ldr x3,[x19,#16]`
(0x102b9ecbc) faults with this=NULL, and classified it as A NEW STABLE FORWARD FENCEPOST —
"reached ONLY via indirect dispatch (0 direct bl/b callers in .text; 0 .data.rel.ro relative-addend
holds it)" — i.e. the next-unsynthesized-object (live-object graft class, SH174/204/248g), a
genuine forward Route-B construction site at a never-before-measured location.

## What is MEASURED (this cycle, authoritative, two independent decoders agree)
The function's TRUE entry is **0x2b9ec9c** (`paciasp`), 4 bytes BEFORE 0x2b9eca0. The whole image
(the R-X .text/exec load segment, file [0x0, 0x62d8190)) contains:

- **64,748 direct `bl 0x2b9ec9c` call sites** (BL-imm26 decode, verified); **ZERO `bl 0x2b9eca0`**.
- Every call targets the paciasp entry 0x2b9ec9c, which begins the same function body that lands
  at 0x2b9eca0. So "0 direct bl/b callers" is factually WRONG by ~64,747.

Two of those callers are IN the PlayerGui getter 0x10201fce0, and BOTH are **exception landing
pads**, not forward-control-flow construction:
- 0x201fda0: `mov x19,x0; ... bl 284cfbc; mov x0,x19; bl 2b9ec9c`  (call_once __cxa cleanup)
- 0x201fe48: `mov x19,x0; ... bl 5fb25e0; mov x0,x19; bl 2b9ec9c`  (call_once __cxa cleanup)
Both save the caught/unwinding context into x0 and invoke the helper with it. The observed
`this=x19=NULL` (fault=0x10) is the unwind context being NULL during the spawned-thread SIGTRAP
unwind (the run log's "resolver SH131 spawned-thread guest raise(SIGTRAP=5) — unwinding this
jit_run" immediately precedes the SIGSEGV). This is TEARDOWN, not a Route-B construction gate.

## What the helper actually is
0x2b9eca0 (`sub sp,#0x480`, `mov x19,x0`, `add x0,sp,#0x270`, `bl 2ba2ba0`) first calls
0x2ba2ba0, which SAVES ALL GPRs (x0..x30) AND ALL FP regs (d0..d31) into the stack frame
(movi/stp pairs), then reads `[x19+16]`/`[x19+24]` and dispatches on them — a
setjmp/ucontext/jmp_buf-style context-save trampoline (fiber/stack-save helper), invoked
ubiquitously (64,748 sites) as the __cxa/unwind landing-pad cleanup path. It is NOT a class
descriptor ctor, NOT a GuiObject producer, and NOT the register continuation itself.

## Verdict (do-not-re-tread)
SH251b's "new ctor fencepost 0x102b9eca0 = next-unsynthesized-object, indirect-dispatch-only,
reached by the class-register continuation" is a +4-entry mislabel put on a shared teardown helper.
The crash it pinned is the NULL-unwind-context landing-pad cleaning up during the SIGTRAP unwind —
a benign/unwind artifact, NOT forward Route-B progress and NOT a fabricatable receiver gate.
A future cone must NOT seed/derive a "receiver for 0x102b9eca0": there is no unique receiver;
the site is reachable from 64,748 direct callers and its NULL-this is expected-on-unwind.
The genuine forward state is UNCHANGED: PlayerGui+ScreenGui class descriptors ARE registered
(headless, vtable+vtable-family markers match) and the name->classid resolver map at 0x106dca0e70
stays empty (per SH193/194 the resolver is live-ctor-gated). SH174 capture-latch
(arm *(0x106391908) at a real make_shared<DataModel>) remains the single forward hook.
recon-v3 deliverables stay shipped + verified.

## Verified
- Two independent BL-imm26 decoders (a new throwaway self-caller scan + GNU objdump) both count
  64,748 `bl 0x2b9ec9c` and 0 `bl 0x2b9eca0` over the exec segment.
- The two getter callers decoded to `bl 0x2b9ec9c` (words 0x942dfbbf / 0x942dfb95, targets
  0x2b9ec9c) and are preceded by `bl 284cfbc` / `bl 5fb25e0` __cxa cleanup calls.
- The PlayerGui getter's landing-pad blocks (0x201fd8c, 0x201fe30) are only reached on exception
  unwind, which the run's SH131 SIGTRAP-unwind line confirms is the active path at the crash.
- Existing sh251b byte-pins are all still correct bytes (the site bytes are real); only the
  DOCUMENTED INTERPRETATION ("unique indirect-only ctor" / "next-unsynthesized receiver") is
  corrected. Add a supplementary hermetic (sh251c) pinning the true paciasp entry 0x2b9ec9c and
  the two landing-pad callers so the correction is guarded against future drift.

## Files
- docs/frontier-sh251b-classdesc-register-advances-ctor-fencepost.md (addendum appended)
- crates/arm64jit/examples/elfjit.rs: +hermetic `sh251c_getter_landingpad_cleanup_not_forward_ctor`
  (byte-pins the paciasp entry 0x2b9ec9c, the two landing-pad `bl 0x2b9ec9c` caller words at
  0x201fda0/0x201fe48, and the __cxa cleanup bls that precede them; asserts bl-imm26 decodes to
  0x2b9ec9c, not 0x2b9eca0 — i.e. a +4-entry drift fails loudly).
- HANDOFF.md, STATUS.md ledgers updated.
- No production code path edited. Workspace green (cargo build + cargo test --workspace exit 0).