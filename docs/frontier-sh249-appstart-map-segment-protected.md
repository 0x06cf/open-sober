# SH249 — the app-start map wall's fatal pointer is SEGMENT-PROTECTED R-X text
# (closes SH248h's "maybe a fixed .bss holder at a stable address" residual)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## The residual SH248h deliberately left open
SH248h closed the RUNTIME in-place map-header repair lever at the DMCONT
continuation's nativeAppBridgeAppStart wall (SIGSEGV guestpc=0x1021dde34), but its
fact-base had one hole: the FATAL 4th-iteration map `this` (x21 = 0x100548ca9 /
0x1004a0373 across runs) was only characterized as "top 16 bits 0 = low guest
addr" and dismissed by the repair guard's `(map>>48)!=0` host-heap check. That left
un-answered whether it was a *seedable fixed .bss cell at a stable writeable
address* — which would have made a STATIC header seed possible (the SH248g lever
restated).

## This cycle: segment-level determination (real libroblox.so, authoritative)
Ran a throwaway probe (reuses libloader::elf::load_elf_image -> host_addr_of) on
the two observed fatal addresses:

    guest 0x100548ca9 -> host Some(...) seg[guest 0x100000000..0x1062d8190]
                        prot { read: true, write: false, execute: true }   (R-X)
    guest 0x1004a0373 -> host Some(...) seg[guest 0x100000000..0x1062d8190]
                        prot { read: true, write: false, execute: true }   (R-X)

Both land inside the single big R-X EXECUTE LOAD segment. That segment is
`write:false`. So the fatal map-`this` is:
- NOT host-heap ASLR (it maps into the image), and
- NOT a writable .bss/.data / .data.rel.ro cell (the segment is execute-and-read-only).

There is therefore NO address the JIT (or any runtime repair) can write a coherent
map header into for this object. Both prior levers are closed by construction:
- SH248g static seed: impossible (not a fixed writable cell; its address is a
  garbage element value read from an under-allocated live collection).
- SH248h runtime in-place repair: impossible (the repair's `(map>>48)!=0` check
  correctly skips it, but even forcing a write would fault on the R-X segment).

## Confirmation the wall is stable
Fresh 3-run batch (full DMCONT env, runs/batch_sh248f_adapter_seed.sh args): all 3
terminate EXIT 134 (SIGABRT->SIGSEGV) at guestpc=0x1021dde34, region-watch shows
the continuation advancing through nativeAppBridgeAppStart (0x10233901x/0x10233920x,
dispatcher 0x102bd8ce8, continueAfterFlagsLoaded_ 0x102bd1d68, app-name guard
0x102bd1f64) — reproducible. (Earlier single-run diversion to the type-4 re-enqueue
0x10285682c path is the known run-variable lane; the batch confirms the app-start
map wall is the dominant terminus.)

## Structural meaning (do-not-re-tread)
The 4th stride of the 0x2a0-stride slot-array walk lands on an in-image text
address — i.e. the app-start is iterating a live collection whose element count
slots are under-allocated (the guest allocates+walks its OWN host-heap maps for the
first 3 inserts, SH248g, then element[3] of the outer slot array is stale garbage).
The outer slot-array base is ASLR host-heap and its ctor never ran headlessly;
clamping the outer collection count is not currently anchored to a fixed global.
This is deterministically the SH174/SH204 live-object structural gate (a real
make_shared<DataModel> ctor would fully initialize the collection), confirmed now
with all three levers measured/proven closed at the SAME site:
  static-header-seed (SH248g), runtime-header-repair (SH248h), segment-protected
  (SH249, this cycle).

## SH250 (same session, follow-up): the "clamp the outer collection count" lever
## A/B-FALSIFIED with a full register dump
While tracing the caller chain this cycle, an apparently-open alternative to the
map-header repairs was considered: SH248g/h both attacked the MAP object's header
fields, but the wall is really an under-allocated outer SLOT-ARRAY of stride 0x2a0 —
so maybe the *iteration count* that bounds the walk is a seedable fixed global (a
levier neither prior session ever tried). A full JIT_DUMP_PC register dump at the
wall (runs/batch_sh248f_adapter_seed.sh + JIT_DUMP_PC=0x1021dde34) FALSIFIES it with
three clean captures then the fault:
  - x19 (per-insert hash) DIFFERS each iteration: 0x6bfdfab08a4be46a /
    0x3e66f7c296aef7de / 0x40c29c7e746e86a1 (distinct keys, 3 real inserts).
  - x24 = STABLE map identity 0x55cd0fcf02d0 across all 3 inserts AND === the value
    stored at the caller-side global [0x1067d16f0] (guardGOT reads it) — the map
    root lives at an ASLR host-heap global, count/capacity are live fields of that
    heap object, re-set by each insert. There is NO fixed .bss count to clamp.
  - x21 (bucket cursor) advances per-iteration (host-heap), then iteration-4 reads
    an element computed from a ctor-uninitialized field -> faults 0x0.
The register dump independently reproduces SH248g's finding and extends it: the
outer collection root is a host-heap map object whose integer fields are set live
at runtime — so "clamping the iteration count" is not a seedable lever either. The
SH249 segment-proof closes the remaining static-write angle. No further lever at
this wall without a real DataModel ctor.

## Code / files
- crates/arm64jit/examples/elfjit.rs: extended the sh248g real-image guard
  `sh248g_appstart_hashfind_wall_anchored` with a SH249 segment-membership
  assertion: both observed fatal addrs must lie in [0x100000000, 0x1062d8190)
  (the R-X text segment). Verified 1 passed filtered; full example suite 90/0.
- Throwaway probe example (sh249_probe.rs) removed after deriving the pin.
- Doc: docs/frontier-sh249-appstart-map-segment-protected.md.
- repro: runs/batch_sh248f_adapter_seed.sh (same DMCONT env).

## Honest bottom line
This is pure regression hardening + a proof-closure of a residual — it does NOT
manufacture a DataModel and does NOT lift the Route-B live-DM structural gate.
Standing forward hook unchanged: SH174 capture-latch arming *(0x106391908) at a
real make_shared<DataModel>. recon-v3 deliverables stay shipped + verified (fresh
green at HEAD this session: 90/0 examples, cargo build + cargo test --workspace
exit 0).