# Frontier SH361 — dynamic DM-ctor trace: the do-init MAIN-branch dispatch is now MEASURED, not assumed (read-only observation guard)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace exit 0; arm64jit lib 423->424). One new READ-ONLY
observation guard `routeb_doinit_dyn_trace_guard` (jit.rs, opt-in
`JIT_ROUTEB_DOINIT_DYN_TRACE=1`, default-inert, fires once per run, ZERO guest
mutation) + one hermetic sh361 test. elfjit.rs only rebuilds (no product
change). Capture probe runs/capture_sh361_doinit_dyn_trace.sh.

## Why this cycle
The operator's EXECUTE-DO-INIT-GATES directive explicitly calls for a "dynamic
DM-ctor trace (SH164's harness-trace artifact) rather than a static seed" — measure
whether the ladder's do-init actually reaches the DM/app-shell dispatch, not assume
it. SH320's docs described the do-init MAIN branch binder-dispatch (0x206df4 ->
vt+0x30 -> br x1 @0x206e24) as a "DM-ctor entry" WITHOUT ever reading the container's
DM-holder field. That is the measured-versus-assumed gap this cycle closes.

## The decision point (disasm-verified on real libroblox.so)
do-init worker 0x2206db8 (true block entry): `mov x19,x1` (@0x2206dd0, x19 =
container arg) -> MAIN branch `ldr x0,[x19,#32]` @0x2206df4 -> `cbz x0, 0x2206ea4`
@0x2206df8:
- NULL container field -> bails to MessageBus_getLastRaw 0x2206ea4 (DM-ctor `br x1`
  NOT attempted);
- non-NULL -> `ldr x8,[x0]; ldr x1,[x8,#48]` @0x2206e00 -> `br x1` @0x2206e24 = the
  DM/app-shell dispatch target.

## MEASURED (real libroblox.so, full app-start ladder, SH344-style env):
```
[routeb-doinit-dyn] SH361 DM-ctor trace @0x2206db8: container=0x5632067bfb70
  [container+32]=0x563206dab800 (non-NULL) -> reach DM-ctor dispatch:
  [obj]vt=0x10635dde8 vt[+48]=0x10258b5d8 (br x1 @0x2206e24).
  once-guard=0x101 once-slot=0x400000b DM-root=0x0
```
- The do-init MAIN-branch container holds a **non-NULL** DM-holder field
  ([obj]vt=0x10635dde8), so control does NOT bail to MessageBus-getLastRaw — it
  reaches the `br x1` dispatch @0x2206e24. The do-init MAIN path's dispatch DOES fire.
- **The dispatch target is 0x10258b5d8 = nativeAppBridgeV2StartAppWithParams+0x494**
  (mid-body of the app-bridge StartApp path; disasm `sub sp,#0x170` prologue @0x258b5d8,
  reads `ldrb [x8,#3488]` @0x258b604 then `bl 0x2dae640` nativePreloadFlagOverrides) —
  NOT a DataModel/MakeDataModel constructor. So SH320's "DM-ctor entry" characterization
  is REFINED: the MAIN dispatch converges on the StartAppWithParams app-bridge region,
  consistent with SH340/SH308's region-level "app-bridge pipe converges on the real
  StartApp path."
- Run then ABRTs (EXIT 134) at the standing SH285 persistence-lane live-object wall
  guestpc=0x101db1b08; DM-root [0x106a68818]=0; MH_* false. Route-B live-DM structural
  gate UNCHANGED.

## Honest
Does NOT manufacture a DataModel (DM-root 0, MH_* false). The trace is READ-ONLY —
it seeds nothing, mutates no guest state (the sh361 test asserts [container+32] is
left untouched even when the guard fires). It answers the operator's explicit
"dynamic trace rather than static seed" ask: the do-init MAIN branch reaches its br
dispatch and that dispatch lands in StartAppWithParams, then the run dies at the
SH285 live-object wall. No single CONFIRMED DM-constructor reach; the route-B structural
gate stands measured one level deeper (dispatch target pinned, not just region).

## Files / verify
- crates/arm64jit/src/jit.rs: +routeb_doinit_dyn_trace_guard (read-only, opt-in,
  once-per-run) + sh361 hermetic (arm64jit lib 423->424).
- runs/capture_sh361_doinit_dyn_trace.sh (probe; live captures gitignored).
- Workspace green (cargo test --workspace EXIT 0, 0 failures).
- Commit: local `dev` only (operator pushes).