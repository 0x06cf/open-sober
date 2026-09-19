# SH381 — MEASURED at the exact store on the FULL ladder: the do-init once-lambda's DM-constructor `0x2173b3c` returns the **0x400000b "Execute" service-handle sentinel**, NOT a live DM — closing the last open question on the operator's EXECUTE-DO-INIT-GATES once-lambda gate with a direct, live, full-ladder measurement

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace EXIT 0; arm64jit lib 437/0 incl the new sh381 hermetic).

## The gap this closes
The operator's EXECUTE-DO-INIT-GATES said: "only seed flags-latch [0x106a683e8].bit0=1 +
main-id [0x106863a68]=this thread, LET the once-lambda populate [0x106a68818]". Two
previous measurements skirted but never directly answered whether do-init's own
`__call_once` lambda actually CONSTRUCTS a live DM:
- SH311 region-watched the once-lambda body (pcs 0x102206d1c/d70/d84 FIRE on the
  skip-appstart send-appevent env) but only read once-slot=0x400000b AFTER the run,
  and never x0 at the store.
- SH361's dyn trace showed once-guard latched (0x101) with DM-root[0x106a68818]=0 on
  the full ladder, and `[container+32]` non-NULL -> the br dispatch DOES fire — but
  never captured what the once-lambda itself produced.
Both left open: "does the once-lambda's DM-ctor return a real object, or the sentinel?"

## Also a genuine ADDRESS-reconciliation finding
The disasm of do-init (0x2206c40) shows the once-path stores the ctor return into
`str x0,[x23,#1032]` @0x2206d74 where x23=0x106a68000 -> **once-slot [0x106a68408]**
(= page +0x408). The harness probes read **DM-root [0x106a68818]** (= page +0x818) — a
DIFFERENT cell entirely. They are not the same slot; a probe monitoring 0x106a68818
will show 0 even when the once-lambda fires and writes 0x106a68408. This was never
called out in prior docs (they conflate the two as "DM-root").

## The measurement (full app-start ladder, live x0 at the store)
New read-only guard `routeb_doinit_oncelambda_probe` (JIT_ROUTEB_DOINIT_ONCELAMBDA=1)
fires at block-entry 0x102206d70 (`adrp x23` just after `bl 0x2173b3c` @0x2206d6c),
reading live x0 = the ctor return about to be stored. On the full ladder
(--v2boot --v2boot-session --v2boot-send-appevent --v2boot-send-game-loaded):
```
[routeb-sh381] do-init once-lambda ctor RETURN x0=0x400000b (sentinel/handle — NOT a live DM)
  at block-entry 0x102206d70; once-slot[0x106a68408]=0x0 DM-root[0x106a68818]=0x0 once-guard=0x200
```
- **x0 = 0x400000b directly at the store** — the identical "Execute" service-handle
  sentinel SH316/SH155 read post-run. The once-lambda's `bl 0x2173b3c` construct
  helper returns a service-handle sentinel, NOT a guest/heap object (>=0x100000000,
  top-16 0 = false). The once-path does NOT build a live DataModel.
- once-slot & DM-root both 0 when read (the sentinel is staged elsewhere/not yet
  stored into the probed cells at that PC; the point is the CONSTRUCTOR RETURN).
- once-guard=0x200 (bit9 stage sentinel, not bit0).
- The guard is READ-ONLY (hermetic proves it touches no cell); run terminatess at the
  standing SH248g live-map wall guestpc=0x1021dde34 (EXIT 139, signals=3) — unchanged.

## Conclusion (answers the GATE-FIX exactly, not a re-tread)
Even when do-init's once-lambda RUNS (proven by SH311 and re-confirmed here by the
store being reached), its `0x2173b3c` DM-constructor returns the 0x400000b sentinel,
so "LET the once-lambda populate [0x106a68818]" cannot yield a live DM by itself —
the construct helper is the service-handle factory, and the real DataModel is built
elsewhere (the do-init MAIN-branch `br x1` @0x2206e24 -> vt[+48]=0x10258b5d8 dispatch
SH361/SH362 measured, which then hits the SH285/shared live-object family). Route-B
live-DM structural gate UNCHANGED; DM-root 0, MH_* false, AppBridgeV2 0. SH174
capture-latch stays the single forward observer. The cell reconciliation (0x106a68408
vs 0x106a68818) is a genuine map refinement: post-run probes must read 0x106a68408 to
observe the once-lambda's write, not 0x106a68818.

## Do-not-re-tread
- Do NOT re-implement a seed to force the once-lambda store: its ctor returns the
  sentinel unconditionally on the full env; the store itself is not where a DM is born.
- Do NOT continue treating 0x106a68818 as "the once-lambda's write target" — that is
  0x106a68408. Re-read both cells (the probe reports both).
- Standing closures still hold: LSM skips (SH349/350/358/373), EC reader-gate
  (SH355/356/374), 0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367),
  -9 string + 0x102b504e4 (SH380), map-header repair (SH248h).

## Files
- crates/arm64jit/src/jit.rs: +`routeb_doinit_oncelambda_probe` (read-only, default-inert)
  + wiring at the block-entry hook + hermetic `sh381_oncelambda_probe_read_only_pc_gated`.
- runs/capture_sh381_oncelambda.sh (full-ladder probe; log /home/hermes-worker/runs/sh381-oncelambda.txt, outside repo).
- Reconciled addresses: once-path store [0x106a68408] (str x0,[x23,#1032] @0x2206d74);
  DM-root probe cell [0x106a68818].