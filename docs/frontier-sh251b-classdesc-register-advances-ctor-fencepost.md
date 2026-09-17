# SH251b — class-desc-REGISTERED path drives the REAL PlayerGui+ScreenGui register getters
# headlessly and ADVANCES into a NEW ctor fencepost 0x102b9eca0 (NULL `this`, fault=0x10) —
# the farthest Route-B class-registry construction reached yet

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.
Repro: runs/capture_sh251b_consumer_dispatch.sh + runs/batch_sh251b_fencepost_repro.sh (3/3).
Hermetic: `sh251b_classdesc_register_advances_into_ctor_fencepost` (real-image guard, skip-if-absent).

## Why (a genuinely-forward re-attack on the operator's SEP-15 line)
SH164/SH189 recon fixed the Route-B "next-unsynthesized-object" as the **PlayerGui class
descriptor** in the global class-name registry — only after it resolves does the DM service
container `[dm + 0x68]` (singly-linked list of live service instances) get populated, which is
the structural pre-requisite for PlayerGui/CoreGui/ScreenGui instances that a GuiObject scene node
would live on. Prior runs reached the register getters only as part of the SH189 seed guard's
own fabricated drive. This cycle enables the class-desc-REGISTERED + genuine-DM CONSUMER+DISPATCH
combination (JIT_ROUTEB_DM_SERVICES=1 alongside the SH187c consumer) so the REAL register path runs
headlessly — and it does.

## Measured (real libroblox.so, 3/3 completing runs, stable)
- routeb-realctor GENUINE MATCH 3/3 + SH187b holder-plant 3/3 (genuine DM in current-DM holder).
- SH187c get-or-create consumer DROVE ok (ret x0 = a live heap consumer, DISPATCH mode) 3/3.
- **SH189 markers fire (NEW reachability):**
  - `PlayerGui class-register GETTER 0x10201fce0 DROVE ok ret x0=0x106c980b8`
  - `ScreenGui class-register GETTER 0x10201f42c DROVE ok ret x0=0x106c98a40`
  - `class-desc counter [0x106dca0e28] = 0 (want >0), cached desc [0x106c97f28] = 0x106c980b8,
    desc vtable [0x106c980b8] = 0x1067a6150 (want 0x1067a6150), PlayerGui vtable-family
    [0x106c980b8+0x230] = 0x106648908 (want 0x106648908)`
  - PlayerGui register once-body region [0x10201fda4..0x102020000) HITS (region-watch).
- Then a **stable NEW downstream fencepost**: the register continuation enters a large ctor
  guest **0x102b9eca0** (sub sp,#0x480 = 1152-byte frame; `stp x29,x30,[sp,#-32]!`; `mov x19,x0`
  = save this) whose first real deref `ldr x3,[x19,#16]` (0x102b9ecbc) faults with **this=x19=NULL
  (fault=0x10)**. Register dump confirms x0=x19=0, lr=0x102b9ecbc (translated leaf), host heap
  present for x23 (the DM/context) but the ctor's own receiver is NULL. Reproducible 3/3
  (EXIT 134/134/139; run 3's 139 = the known secondary-thread path, same site).
- The ctor is reached ONLY via indirect dispatch (no direct bl/b caller in .text; no
  .data.rel.ro relative-addend holds 0x2b9eca0) — a vtable/closure invocation whose receiver
  object was never constructed headlessly. That receiver is the next-unsynthesized-object.

## Verdict (forward, honest)
The SH189 class-registration line now genuinely executes on the real binary (PlayerGui+ScreenGui
class descriptors reachable headlessly for the first time) and lands on a NEW, pinned, reproducible
fencepost: the ctor at 0x102b9eca0 needs a live receiver (this != NULL). This is the same
fabricatable-object-graph / live-object class as SH174/SH204/SH248g — but at a location never before
measured (the register getters were previously only drivable via the fabricated-desc path, and the
register continuation had never been observed to reach a ctor). Route-B live-DM structural gate
UNCHANGED (a NULL-`this` ctor receiver still is not a live DataModel); SH174 capture-latch
(arm *(0x106391908) at a real session make_shared<DataModel>) stays the single forward hook. recon-v3
deliverables stay shipped + verified.

## Files
- runs/capture_sh251b_consumer_dispatch.sh, runs/sh251b-consumer-dispatch.txt
- runs/batch_sh251b_fencepost_repro.sh, runs/sh251b-consumer-dispatch-batch.txt
- docs/frontier-sh251b-classdesc-register-advances-ctor-fencepost.md (this file)
- crates/arm64jit/examples/elfjit.rs: +hermetic `sh251b_classdesc_register_advances_into_ctor_fencepost`
  (8 real-image byte-pins: PlayerGui getter 0x201fce0/0x201fce8, ScreenGui getter 0x201f42c, PlayerGui
  once-body 0x201fda4, new ctor 0x2b9eca0/0x2b9ecac/0x2b9ecb0 + fault insn 0x2b9ecbc).
- No default-config production path edited. Workspace green (cargo build + cargo test exit 0).

## SH251c CORRECTION (same session, hermes-worker) — 0x102b9eca0 is TEARDOWN, not a forward ctor
The SH251b "new forward ctor fencepost 0x102b9eca0 (indirect-dispatch-only, 0 direct callers,
next-unsynthesized-object)" classification is CORRECTED this cycle. The function's TRUE entry is
**0x2b9ec9c** (`paciasp`, 4 bytes before 0x2b9eca0), and the whole exec segment has **64,748 direct
`bl 0x2b9ec9c` callers and ZERO `bl 0x2b9eca0`** (measured by two independent BL-imm26 decoders —
my throwaway self-caller scan AND GNU objdump). Two of those callers are in the PlayerGui getter
0x10201fce0 and are EXCEPTION LANDING PADS (0x201fda0 / 0x201fe48, each preceded by a __cxa
call_once cleanup `bl 284cfbc` / `bl 5fb25e0`, saving the caught/unwinding context into x0 then
`bl 0x2b9ec9c`). So the observed `this=x19=NULL` (fault=0x10) is the NULL unwind context during the
spawned-thread SIGTRAP unwind (the run log's "SH131 spawned-thread raise(SIGTRAP) — unwinding this
jit_run" line immediately precedes the SIGSEGV). 0x2b9eca0 first calls 0x2ba2ba0, which saves ALL
GPRs+FP regs into the frame, then dispatches on `[x19+16]/[x19+24]` — a setjmp/ucontext-style
context-save trampoline used as the ubiquitous __cxa landing-pad cleanup. It is NOT a class-descriptor
ctor, NOT the register continuation, NOT a GuiObject producer, and there is NO fabricatable
receiver to seed. Do-not-re-tread: do NOT derive a "receiver for 0x2b9eca0" (64,748 callers; NULL-
this is expected-on-unwind). Genuine forward state UNCHANGED (descriptors registered; resolver map
0x106dca0e70 still empty, live-ctor-gated per SH193/194; SH174 capture-latch stays the forward hook).
recon-v3 stays green. +hermetic sh251c (byte-pins the paciasp entry + two landing-pad bl words +
their __cxa cleanup bls; asserts bl-imm26 targets 0x2b9ec9c NOT 0x2b9eca0). Workspace green 576/0,
examples 92/0. Full correction doc: docs/frontier-sh251b-corrected-teardown-helper.md.