# Frontier SH295 — item-PROCESSOR [item+48] edge DRIVEN, measured to the live-object wall

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--v2boot-session-itemproc`).

## tl;dr

SH290-294 drove/closed the item-processor cone. SH293/294 left the LAST edge —
`[item+48] -> bl 0x22193a0` — as a STATIC "not driveable standalone" judgment.
This cycle DRIVES it and converts that judgment into a MEASURED runtime result:
the engine's OWN [item+48] continuation executes headlessly and deterministically
terminates at the nativeOnDestroyed live-object vt[+112] wall (guestpc=0x1028511f8,
fault=0x50), confirming SH294's second correction that the benign-cell subpaths are
UNREACHABLE (the dispatcher w2==0 arm faults before them).

## What was driven

In the `--v2boot-session-itemproc` rung, after the SH292 per-item dispatch, SH295
sets `item[+48]` to a benign continuation object (so the item-proc `cbz x0` at
0x22079d8 is NOT taken and `bl 0x22193a0` at 0x22079e0 EXECUTES), keeps item[+32]
on the benign leaf, and seeds [0x1068262e8] (the SH294-corrected registry cell) with
a benign leaf. Then drives item-proc again and observes where control lands.

## The mechanism (fresh disasm + measured)

- `0x22193a0` (the [item+48] continuation) does `mov w2,wzr` at 0x22193c4, then
  `bl 0x22076e8` at 0x22193d0 — a tail-TRAMPOLINE `mov x3,xzr; b 0x2850ef0` into the
  nativeOnDestroyed dispatcher (SH294's correction).
- Dispatcher `0x2850ef0`: `and w8,w2,#0xff; cmp w8,#1; b.gt` (w2>1 arm); `cbz w8 ->
  0x2850fa8` (w2==0 arm — the ONLY arm for this call, w2=0). 0x2850fa8: `mov x0,x1;
  bl 0x28511c4`.
- `0x28511c4` is the live-object body (sub sp,#0xe0) that derefs a real vt[+112]
  object. MEASURED: every run (3/3, runs/sh295-item48-*.txt) terminates
  `SIGSEGV fault=0x50 guestpc=0x1028511f8` — the `ldr x8,[x29,#24]` in 0x28511c4,
  a live-object class wall (SH273/SH174).
- The benign-cell subpaths 0x28506a4/0x28508a8 (which read [0x1068262e8] at
  0x28506d4/0x28508d8 via `cbz x8; blr x8`) are reached ONLY after the dispatcher
  returns — which it never does headlessly (w2==0 arm faults first). So the SH294
  benign-leaf CELL [0x1068262e8] is on this edge but UNREACHABLE: seeding it cannot
  unlock the edge. This is the measured proof of SH294's attribution.

## Measured (real libroblox.so, runs/sh295-item48-{1,2,3}.txt)

- Deterministic 3/3: SH295 drives item-proc with item[+48] set, region-watch fires
  0x1022193a0 (the continuation entry) + 0x1022079d8 (the [item+48] load), then
  `SIGSEGV fault=0x50 guestpc=0x1028511f8` (0x28511c4 body, live-object wall).
- The engine's OWN [item+48] per-item continuation runs headlessly for the first
  time and dies at the live-object gate — not at a seedable cell. SH293's
  "not driveable standalone" is now MEASURED, not merely judged.
- once-guard stays latched (0x101); no DM (MH_* false, DM-root 0).

## Honest (do-not-over-claim)

- Cause-not-symptom SESSION-CTOR — drives the engine's own [item+48] continuation
  to its real terminal; does NOT manufacture a DataModel (MH_* false, DM-root 0);
  Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
  forward hook.
- Item-proc state machine is now FULLY driven+measured: once-build (SH290),
  idempotent re-entry (SH291), [item+32] per-item dispatch (SH292), [item+48]
  continuation-to-wall (SH295).

## Verify

- `cargo test -p arm64jit --example elfjit sh295` = 1 passed.
- `cargo test -p arm64jit --examples` = green (123/0 baseline, +sh295).
- `cargo build --workspace` + `cargo test --workspace` EXIT 0.
- elfjit.rs + HANDOFF.md under the 1MB pre-commit hook (prose trimmed this cycle).

## Files

- `crates/arm64jit/examples/elfjit.rs` (+SH295 rung in `--v2boot-session-itemproc`,
  +hermetic sh295).
- `runs/capture_sh295_item48.sh`, `runs/sh295-item48-*.txt` (repro + captures).
- `HANDOFF.md`, `runs/STATUS.md` (ledger).
- `docs/frontier-sh295-item48-edge-driven-to-wall.md` (this).