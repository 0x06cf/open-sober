# Frontier SH292 — item-PROCESSOR per-item DISPATCH driven (doc's next gate past SH291)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(same opt-in `--v2boot-session-itemproc`). No production path edited.

## tl;dr

SH291's doc deferred "the NEXT gate past SH291 = item-proc's REAL per-item dispatch
(the two `blr x8` on a [sp]-constructed std::function)". Fresh disasm of item-proc
`0x102207950` corrects that attribution and then drives the real edge:

- The **actual** per-item dispatch sites are `[item+32] -> vt[+48] blr` (0x22079cc)
  and `[item+48] -> bl 0x22193a0` (0x22079e0) — NOT 0x2207cb0/0x207ce0 (those blrs
  live in a SEPARATE fn 0x2207bbc with its own once-guard, called only from
  0x2dadb90). SH290/291's zeroed item cbz-skipped BOTH real sites, so the engine's
  per-item dispatch had never run headlessly.
- SH292 sources `item[+32]` from `routeb_singleton_obj_addr()` (the SH159b benign
  dispatcher: a leaked object whose 0x60 vtable is ALL identity leaves), so the
  `vt[+48]` load resolves a registered host thunk and the `blr` **EXECUTES**;
  `item[+48]=0` keeps the 0x22193a0 branch skipped (it walks into the SH273
  lifecycle-notifier live-object wall). once-guard stays latched -> the run is pure
  per-item-dispatch + SH284 clock helper, isolating the NEW edge.

## Measured (real libroblox.so, same rung, 4/4 clean)

```
SH290 item-proc post: once-built=0x800000c once-guard=0x101            (once-body ran)
SH291 re-entry post:  once-built=0x800000c once-guard=0x101 IDEMPOTENT=true
SH292 per-item dispatch returned Ok(...) PER_ITEM_DISPATCH_EXECUTED=true
   once-built=0x800000c once-guard=0x101  (once still latched, cell stable)
   [item+32]->vt[+48] blr fired via benign host leaf
SH292 MH_FLAGS_LOADED=false MH_APP_READY=false
```

Deterministic 4/4. `dump` labels SH290/SH291/SH292 all print; the three-drive
sequence completes on a single ladder thread.

## Honest (do-not-over-claim)

- This is cause-not-symptom SESSION-CTOR: item-proc's **real per-item dispatch now
  executes** headlessly for the first time (the vt[+48] blr returns through a
  benign leaf), one state-construction gate past SH291's measured idempotency.
- It does NOT manufacture a DataModel: MH_* false, DM-root[0x106a68818]=0, Route-B
  live-DM structural gate UNCHANGED. The benign leaf returns identity (a live
  recorded-callback registration would be a real live object — SH248h/256 discipline;
  the [item+48]->0x22193a0 branch that reaches authentic per-item work is still gated
  on the SH273 lifecycle-notifier wall).
- SH174 capture-latch stays the single forward hook.
- Correction recorded: SH291's doc labeled 0x2207cb0/0x207ce0 the "per-item tail";
  those are fn 0x2207bbc's (a distinct once-guard), reached only from 0x2dadb90.

## Verify

- `cargo build --workspace` + `cargo test --workspace` EXIT 0.
- SH290/SH291/SH292 markers above (runs/sh292-itemproc-dispatch-{1,2,3,4}.txt, gitignored).
- recon-v3 plane re-verified green: 24 task frames swap Ok(0x1), 0 json abort, 0 crash.
- elfjit.rs (1,048,55xB) held under the 1MB pre-commit hook (condensed nearest prose).

## Files

- `crates/arm64jit/examples/elfjit.rs` (SH292 3rd drive in `--v2boot-session-itemproc` rung).
- `docs/frontier-sh292-itemproc-peritem-dispatch-drive.md` (this).