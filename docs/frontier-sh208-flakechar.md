# SH208 — Ladder flake characterization: run-variable dispatch flakes are NOT the SH202 on-demand patcher (do-not-chase, no seed warranted)

**Session type:** verification + characterization. Workspace green (cargo test
--workspace EXIT 0). Both recon-v3 immediate-priority deliverables re-verified
green at HEAD. No production code change (measurement shows do-not-chase).

## 1. Immediate-priority recon-v3 deliverables RE-VERIFIED at HEAD
`runs/capture_taskv4_frame.sh` at HEAD ffb9920 (after the SH207 resolve-guard
edit moved the three-map probe to the top of the guard):

```
EXIT=124  0 SIGSEGV/0 SIGABRT  (no json abort)
[elfjit:taskv4] type4_frame_thunk registered at 0x7f00000001d0
[elfjit:taskv4] seeded dispatcher type-4 vector [0x106829ea8] = 0x7f00000001d0
[elfjit:taskv4-frame] patched heartbeat w4#2 0x102856f24 / w4#3 0x102856f68 -> 52800084
[elfjit:renderthunk] published RENDERCTX 0x7f498c110a00
[elfjit:taskv4-frame] present #19..#23 swap Ok(0x1)  <- distinct colors
=== present count: 24 ===
=== node pops: 197 ===
```
(1) SELF-DRIVEN FRAMES and (2) JSON-ABORT both still green. JIT_JSON_ZERO_FIX
confirmed firing in the ladder runs (`[json-fix] append check 0x102355d40 would
overflow (len=0x.. cap=0) -> forcing len=0`).

## 2. The residual ladder flake is the run-variable host-pointer/translated-block
class — the SH202 on-demand patcher is NOT the writer

The operator's Route-B directive lists SH55/64 run-variable faults as a standing
nuisance that keeps the ladder from deterministic-clean. This cycle tested the
one untested hypothesis: that the SH202 V2 on-demand patcher (which edits `.text`
mid-run at the outside-image stop, then rewinds pc) is the corruption writer
(guest frame unwinding over an edited block).

**On-demand patcher classification** — 2 independent batches:
- mode=on (V2_ONDEMAND=1): batch 7/8 clean, batch 6/6 -> 1 fault at 0x1021e34b0
- mode=off (SH200-only, SH207-style baseline): 12/14 clean, 2 faults

The faulting sites are DIFFERENT between modes and between runs:
- ON: 0x1021e34b0 (`ldr x8,[x8,#168]` — reads an object ptr threaded from the
  caller frame via helper 0x21e3960 `ldr x0,[x0,#8]; ret`; fault=0xa8 = NULL+0xa8
  vtable-slot deref; enclosing obj 0x561ebc69b970 host-heap, run-variable)
- OFF: 0x1062412dc (fault=0x7f0000000020 — a host-pointer slot), 0x1021deaac
  (fault=0x20)

All three are run-variable live-object dispatches (host-heap obj whose own
dispatch slot is NULL/stale), NOT fixed `.bss` pointers — the documented
SH176/SH103/SH109 singleton-vtable host-pointer class. Disassembly
confirms the faulting object comes from a caller-frame member, not a global.

**VERDICT:** the on-demand patcher is exonerated (removing it does not lower the
flake rate; the fault site just moves). These are parallel-universe run-variable
JIT-cache/dispatch flakes — the SH104/105 stack-smash + SH55/64 class. They are
non-deterministic, non-seedable, and do-not-chase (consistent with ~30 prior
cycles). A production fix would require pinning a specific hostcall-return leak
(SH104/105's named-but-unimplementable bridge-sanitize — proven wrong by SH105
because guest stack pointers are ALSO 0x55-range, so a range-based zero would
corrupt legit self-pointers). Not pursuable this cycle.

## 3. Route-B structural gate UNAFFECTED
No ladder-clean-progress sample produced a live-DM (once-slot stays an intern,
resolver maps stay EMPTY {0,0}, app-data-model counter at baseline). Route-B
live-DM world-build = structural gate unchanged (SH174/193/194/196/203/204/206/207).

## 4. Shipped
- Two characterization scripts (default, gitignored logs):
  `runs/capture_sh208_compare.sh` (on/off flake-rate A/B) +
  `runs/capture_sh208_flakechar.sh` (multi-run fault-signature sampler).
- This doc.

## Repro
```
./runs/capture_sh208_compare.sh on  6   # V2_ONDEMAND=1 ladder sample
./runs/capture_sh208_compare.sh off 14  # SH200-only baseline sample
```
## Verdict
No new seed, no production change. Immediate-priority deliverables stay green.
Standing wall unchanged. Do NOT re-tread the on-demand-vs-flake hypothesis (closed
above). Next genuine lever (unchanged): a live-DM session (SH174 latch) or a
fresh out-of-band Route-B derivation.