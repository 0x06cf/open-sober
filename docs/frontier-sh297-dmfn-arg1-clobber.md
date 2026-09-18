# Frontier SH297 — DM-construction fn: correct SH296's seed (arg1 clobbers this+136/144) — construction body EXECUTES

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--v2boot-session-dmfn` + `JIT_ROUTEB_DMFN_FIELDS=1`).

## tl;dr

SH296 MEASURED that the DM-construction handler **0x1023f03b4** executes headlessly
FIRST time but benign-returned `Ok(0x0)` even with `this+136/144` seeded to the
routeb singleton, and concluded "the deep body needs GENUINE distinct live
sub-objects". **That verdict rested on a mis-seeded test.** Fresh disasm + a new
stage-2 drive with a coherent **arg1** shows the construction body (0x23f0484)
EXECUTES past its gues ordering and advances ~1 hop to the app-server
registration fn 0x21e45c8, faulting at 0x1021e460c — **one BL before
nativeAppBridgeAppStart (0x23f05f8)**. The live-DM gate is NOT a blanket-singleton
impossibility; the specific seed was wrong.

## Why SH296's S2 measured "no advance" (the correction)

The construction body's entry does:

```
23f03e8: mov x22, x1          ; x22 = ARG1 = x1 (NOT this)
23f0484: ldp x21, x24, [x22, #8]   ; x21 = arg1[8], x24 = arg1[16]
23f049c: stp x21, x24, [x20, #136] ; this+136 = arg1[8], this+144 = arg1[16]  (CLOBBER)
23f04d0: cbz x21 → 0x23f0680        ; if arg1[8]==0 -> SAFETY EPILOGUE
```

It reads the app-request's `[arg1+8]` / `[arg1+16]` and **writes them over**
`this+136/144`. SH296 S2 seeded `this+136/144` directly — those are immediately
overwritten from a **zeroed arg1 buffer** (`[arg1+8]=0`), so `cbz x21` fired the
safety epilogue (benign-return) before ANY `this` field was even read,
including `this+128` (the refcount-factory 0x2b4ea48 input). The "blanket
singleton insufficient" conclusion never actually exercised the gates.

## What SH297 drives instead

New stage-2 (same opt-in flag) seeds the REAL gates:
- **arg1** (x1) = coherent struct with `[arg1+8]=[arg1+16]=routeb singleton`
  → these become `this+136/144` after the clobber-store → `cbz x21` passes;
- `this+120` = singleton (x28, stored into the built app-server box);
- `this+128` = singleton → factory **0x2b4ea48** `ldar x20,[x0+8]; cmn #-1;
  bl 0x2b9e760` refcount-increments and returns it nonzero → `cbz x0` passes.

Then the body runs: builds the app-server box x26 (0x21e4414 op 0x40), registers
via **0x21e45c8** (bl @0x23f05b8), then would reach the app-router
**0x21e45c8's** tail and **nativeAppBridgeAppStart 0x2362e98** (bl @0x23f05f8).

## Measured (real libroblox.so, full SH296 base env + JIT_ROUTEB_DMFN_FIELDS=1)

Region-watch on the construction body `0x1023f0484-0x1023f0f40` + app-start +
marshaler + EC world **fires at guest 0x1023f0544 / 0x1023f0550 / 0x1023f055c**
— the body's app-server box-build block — confirming the construction body
actually EXECUTES (SH296 S2: 0 hits, benign-return). Then SIGSEGV at
**guestpc=0x1021e460c** (`ldr x0,[x21,#8]` inside fn 0x21e45c8, the app-server
registration called from 0x23f05b8) — one BL before app-start. Fault value
0x7f00000001e8 (host-thunk region) indicates the fabricated registration object's
[+8] field is not a valid host object — a live/app-registry-object wall one hop
shy of the app-start threshold. Deterministic 3/3 EXIT 134/139, all faulting at
0x1021e460c.

## VERDICT (do-not-re-tread SH296's premise)

- SH296's "needs genuine distinct live sub-objects, blanket singleton
  insufficient" is REVISED: the singleton IS sufficient to pass the arg1 +
  factory gates; the correct fix is where the seed goes (arg1 + this+120/128),
  not more distinctness.
- The construction body is no longer benign-returning; it builds the app-server
  box and dies in 0x21e45c8. The **next gate** is a coherent 0x21e45c8
  registration object (its `[+8]` must be a valid host obj — see 0x21e45d8/0x21e460c
  field layout), after which nativeAppBridgeAppStart 0x2362e98 is reachable.
  That app-start line is the standing live-object wall (SH248-260 family).
- Route-B structural gate UNCHANGED (no DM); SH174 capture-latch stays the
  single forward hook. This is cause-not-symptom SESSION-CTOR: the DM-construction
  fn's gate chain is now corrected + measured, one measurable hop closer to
  actually reaching app-start via this front-door.

## Verify

- `cargo test -p arm64jit --example elfjit -- sh297` = 1 passed.
- `cargo test --workspace` EXIT 0 (25 test-result-ok groups); examples green.
- elfjit.rs 1,048,302 + HANDOFF.md under the 1MB pre-commit hook (prose trimmed).

## Files

- `crates/arm64jit/examples/elfjit.rs` (SH297 stage-2: arg1[8]/arg1[16]=singleton
  + this+120/128; +hermetic sh297).
- `runs/capture_sh297_dmfn.sh`, `runs/capture_sh297b_dmfn_region.sh`,
  `runs/sh297-dmfn-s2-*.txt`, `runs/sh297-dmfn-region-*.txt`.
- `HANDOFF.md`, `runs/STATUS.md` (ledger). `docs/frontier-sh297-dmfn-arg1-clobber.md` (this).