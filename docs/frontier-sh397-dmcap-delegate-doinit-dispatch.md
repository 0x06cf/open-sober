# Frontier SH397 — never-run intersection closed: the SH174 DM-allocation capture latch WITH the working SH395 DELEGATE observer on the deepest do-init DISPATCH-REACHING ladder (no --v2boot-skip-appstart) records 0 validated make_shared<DataModel> — the "no live DM" verdict finalized AT the DM-ctor dispatch point via a trustworthy observer

Date: 2026-09-21 (this cycle), hermes-worker, single-agent (cone suppressed). One new probe script
`runs/capture_sh397_dmcap_delegate_doinit_dispatch.sh`. No production Rust / guest byte /
JIT-hook-default touched. Workspace green at start and end (cargo test --workspace EXIT 0). All
recon-v3 immediate-priority deliverables re-verified green at this exact HEAD this cycle
(capture_taskv4_frame.sh attempt 1: 24 real task-driven frames `present swap Ok(0x1)`, 197 node
pops, 0 json abort, 0 crash; capture_sh304: session-gated producer INERT 0 GATED; JSON len-clamp
JIT_JSON_ZERO_FIX at 0x102355d40 present).

## Why this cycle

SH395 established that the single forward observer (SH174 `routeb_dm_alloc_capture` latch) arms and
fires cleanly ONLY with `JIT_DM_ALLOC_CAPTURE_DELEGATE=1` (the safe-latch refuses to replace the
engine's real allocator hook 0x1021ebaf4 without it), and therefore future "0 validated" results are
trustworthy-by-construction. But SH395b ran the DELEGATE observer only on the `--v2boot-skip-appstart`
env, which SKIPS StartLuaAppDM/V2StartAppWithParams — so it never reached the do-init MAIN dispatch
(`br x1 @0x2206e24 -> vt[+48]=0x10258b5d8`) that SH361/SH362 measured as the real DM-ctor entry.
The furthest-reach + working-observer composition was never actually run under DELEGATE.

## The never-run intersection (SH397)

SH361's dyn-trace env (FULL ladder, no skip-appstart; the one that reaches the DM-ctor dispatch on
every run) + `JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1`.

## Measured (real libroblox.so, EXIT 139 terminal)

- **The do-init MAIN dispatch FIRES** (SH361 trace): `container=0x563bd8efd6c0 [container+32]=0x563bd94e5d70 (non-NULL) -> reach DM-ctor dispatch: [obj]vt=0x10635dde8 vt[+48]=0x10258b5d8 (br x1 @0x2206e24). once-guard=0x101 once-slot=0x400000b DM-root=0x0`. The furthest-advancing reach is real — this is NOT a skip-appstart run.
- **The WORKING DELEGATE observer installs** (trustworthy-by-construction, SH395): `routed CRT operator-new ACTIVE hook 0x1067daaf0 -> capture trail ... prev_hook 0x1021ebaf4`, then `FIRST call#1..#5` (all bytes=0x18 — 0x18-byte allocations, none DM-plausible) delegated through the engine's OWN real hook (prev_hook 0x1021ebaf4, delegation clean — no bad_function_call from delegation).
- **`[validated] make_shared<DataModel> = 0`** — the verdict, now measured with a working observer AT the DM-ctor dispatch point, not on a skip-appstart env.
- Terminal drains to the standing SH285 persistence-lane class (SIGSEGV/bad_function_call). Post-lifecycle: MH_FLAGS_LOADED=false MH_ENGINE_INITIALIZED=false MH_APP_READY=false AppBridgeV2[0x106a705e8]=0x0.

## Interpretation

The furthest-advancing dispatch-reaching ladder produces ZERO validated `make_shared<DataModel>`
under a WORKING observer. This finalizes SH394/SH395's "no live DM" verdict at the actual DM-ctor
dispatch junction (the deepest point any harness composition reaches headlessly). It re-confirms the
standing Route-B live-DM structural gate AT its most honest instrumentation depth: the do-init MAIN
dispatch brs to the StartAppWithParams body (0x10258b5d8) but the run faults in the SH285
persistence lane before any live DataModel allocation, so no make_shared happens. This is a
map-completion + observer-depth advance (never-run intersection), not a DM advance — honest.

Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0). The
session-gated producer + R1 content path remain latent-but-correct.

## Do-not-re-tread (all prior closures stand, unchanged)

SH393 cmd 1/13/15/17/18, LSM skips (SH349/350/358/373), EC reader-gate (SH355/356/374),
0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367), ALooper (SH365), governor gates
full-ladder (SH379), -9 string (SH380), map-header (SH248h), once-lambda store (SH381), SH267
node-cell (SH385), setDataModelToCurrent (SH388), LSM-manufactured wiring (SH385), and do NOT run
the SH174 latch WITHOUT DELEGATE expecting an install (safe-latch refusal — SH395; use
JIT_DM_ALLOC_CAPTURE_DELEGATE=1 to arm it). And do NOT re-manufacture [0x10726f8c0]-style coherent
empty maps to cross the SH285 reader (SH396, measured 4/4).

## Files

- runs/capture_sh397_dmcap_delegate_doinit_dispatch.sh (new, committed)
- Log /home/hermes-worker/runs/sh397-dmcap-delegate-doinit-dispatch.txt (outside repo)
- Commit: local dev only (operator pushes).