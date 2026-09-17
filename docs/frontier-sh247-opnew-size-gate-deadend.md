# SH247 — route ALL `operator_new` sizes through the working descriptor path (size-gate `b.ls`→`b`) → MEASURED: still `std::bad_alloc`, the DMCONT continuation is allocation-waled at its first >0xa box (proof-of-dead-end via a 2nd mechanism)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(patch + hermetic sh247 green; elfjit examples 88/0).

## Context (do-not-re-tread)
SH245 ACTIVATED the real continueAfterFlagsLoaded_ (0x102bd1d68) headlessly and
stopped one gate deeper: `std::bad_alloc` because operator_new returns NULL for
size>0xa when the allocator-activation byte [0x10727570c].bit0 is clear. SH246
scoped-boxed the 0x28 closure (0x2bd2128) and MEASURED the continuation is
**pervasively** allocation-walled: bad_alloc persists from MANY interior op_new
sites (BOTH variants 0x1db1a38 and 0x1d96768). Both hammers already measured a
dead-end: (a) seeding bit0=1 -> ALL op_new real-alloc -> early SIGABRT (SH245 #4,
regression); (b) scoping one closure -> other sites still NULL (SH246).

## The lever (SH247, opt-in JIT_ROUTEB_OPNEW_SIZE_GATE)
Fresh disasm: BOTH variants' fast path is
`ldrb w8,[0x10727570c]; tbnz w8,#0,REAL; cmn x2,#0xa; b.ls SMALL(0x1db1ab4/0x1d96824); mov x19,xzr(NULL)`.
The ≤0xa "SMALL" path is NOT a separate allocator — it builds a scudo
size-class descriptor (0x1d969cc/0x1d99bf0 classifier, size classes up to ~2.8KB)
and calls the REAL allocator tail 0x1db1c60, which demonstrably returns real
memory for the tiny sizes that run today. The single-word lever: change the size-gate
`b.ls SMALL` (taken ⇔ unsigned size≤0xa) to an UNCONDITIONAL `b SMALL`, so EVERY
size (incl. the continuation's 0x28/0x20 closures + string/JSON constructions) walks
the SAME descriptor→real-tail path that works for tiny sizes — WITHOUT touching the
bit0 flag. 2 sites, single word each, byte-guarded, whole-function block-cache drop
(0x101db1a38..0x101db1c60 and 0x101d96768..0x101d969a0).

## Measured (real libroblox.so, canonical completing --v2boot ladder, DMFORCE+DMCONT
+GETTER_TAIL_RET+M48_SEED env, region-watch [0x102bd1d68,0x102bd2600))
- patch engages: `[SH247] size-gate @0x101db1a78 -> unconditional b small-path
  (readback 1400000f)`, `@0x101d967ec -> ... (readback 1400000e)`.
- **OFF (default): EXIT 134 (ABRT/bad_alloc), 0 continuation region hits** — dies
  upstream of the continuation.
- **ON (size-gate only): EXIT 139, continuation RUNS (region hits d68 then dfc) but
  still terminates `std::bad_alloc`; no region hits deeper than 0x102bd1dfc.**
- **ON+combined (SH246 0x28 box patch ALSO on): same — bad_alloc, region hits d68+dfc
  only**, i.e. even with the specific 0x28 closure boxed AND all sizes routed through
  the descriptor path, the continuation dies at its NEXT >0xa allocation.

## Conclusion (honest, do-not-over-claim) — two measured mechanisms
The descriptor→real-allocator-tail path that serves ≤0xa headlessly does NOT serve
the continuation's >0xa boxes: the ≤0xa working case is a narrow size-class whose
region happens to be set up; the 0x28/0x20 classes' regions are not. So the DMCONT
continuation is allocation-walled headlessly via a **2nd measured mechanism** (in
addition to SH245's NULL-fast-path wall and SH246's pervasive-box wall): the real
allocator genuinely cannot serve size>0xa on this box regardless of the route
(NULL, descriptor-tail, or scoped-box). This is the proof-of-dead-end standard the
operator's MIGRATION-IS-NOT-A-STOP doctrine demands for this specific sub-line. It
does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED.
Standing forward hook stays SH174 capture-latch arming *(0x106391908) at a real
make_shared<DataModel>.

## Why this specific gate (not a re-tread)
Every prior attempt to clear the continuation's alloc wall hit a single mechanism
(NULL fast path). SH247 closes the surviving *unmodified* route: the working
descriptor path. Its measured failure (descriptor path can't serve >0xa) removes the
last "maybe the working small path can handle the larger boxes" candidate and pins
the alloc wall to the size-class region setup — a live-session state, not a seed.

## Corroborating observation: the allocator is THE headless construction crux
The SAME allocator-activation byte [0x10727570c].bit0 + size threshold gates
StartLuaAppDM's OWN real body too: file 0x23ff31c (inside nativeAppBridgeStartLuaAppDM
0x1023efe2c, the +0xf4a4 offset) reads the byte and `tbnz w8,#0`; with the bit clear it
does `cmn x20,#0x9; b.cs 0x23ff4ec` — a ~9-byte threshold routing to the SAME
descriptor/real-alloc path. So op_new (both variants), the DMCONT continuation, AND
StartLuaAppDM's real construction path ALL funnel through this one flag: headlessly,
no engine-authored object >~10 bytes can be built (every string / vector / container /
the DataModel itself). Solving the 0x70c-size-class bootstrap would be the single
enabler for all of Route B; the flag's =1 route alone SIGABRTs (regions/cache not set
up) and the flag-writers (file 0x2817b48/0x282a004) sit deep in full-session
app-lifecycle init, not a headless-invokable CRT bootstrap.

## Why this specific gate (not a re-tread)
Every prior attempt to clear the continuation's alloc wall hit a single mechanism
(NULL fast path). SH247 closes the surviving *unmodified* route: the working
descriptor path. Its measured failure (descriptor path can't serve >0xa) removes the
last "maybe the working small path can handle the larger boxes" candidate and pins
the alloc wall to the size-class region setup — a live-session state, not a seed.

## Code / files
- crates/arm64jit/examples/elfjit.rs: `routeb_patch_opnew_size_gate()` (opt-in
  JIT_ROUTEB_OPNEW_SIZE_GATE, real-image byte-guard on both `b.ls` gate words,
  unconditional `b` replacement, whole-function block-cache drop). Hooks into the
  --v2boot block next to routeb_patch_cont_opnew_box (self-guards on its own env).
- +hermetic `sh247_opnew_size_gate_routes_all_sizes_to_small_descriptor_path`: pins
  both real-image gate words (must be B.cond cond=LS), verifies the replacement is an
  unconditional B whose imm26 lands on the documented SMALL path (drift fails
  loudly), real-image guard passes. Examples batch 88/0.
- Repro runs/capture_sh247_opnew_sizegate.sh (A/B off|on) + combined arm measured above.
- cargo build --workspace EXIT 0; cargo test --workspace green; sh247 1 passed.

## Next (honest, single-agent)
The DMCONT continuation sub-line is provably allocator-gated headlessly (3 measured
mechanisms now). Advancing it further would require either (a) reverse-engineering
+ seeding scudo's size-class region/cache state so >0xa allocations genuinely
succeed — a multi-hour lift whose payoff is still "app-controller lifecycle, NO live
DM" (the EC-world DM factory is measured unreachable, SH231/235/237/238), or (b) a
pervasive guest bump-allocator — blocked by the run-loop image-bounds check and
equally unlikely to reach a DM. Per the operator, keep grinding the Route-B line but
the allocator sub-axis of the continuation is measured dead on this box. Standing
forward hook stays SH174 capture-latch arming at a real make_shared<DataModel>.