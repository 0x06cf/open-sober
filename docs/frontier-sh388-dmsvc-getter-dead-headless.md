# Frontier SH388 — measure the SEP-15 setDataModelToCurrent cone door is DEAD headlessly (getter 0x102dbcc10 / body 0x102dbcc1c never EXECUTED), closing SH387's "cone door remains OPEN" with execution evidence

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start (cargo test --workspace EXIT 0, 622/0; arm64jit lib 442/0) and at end
(443/0 lib + full workspace gate re-run green).

## Why this cycle

SH387 byte-anchored the SEP-15 re-attack cone door (ExperienceController /
DataModelServices::setDataModelToCurrent / the current-DM GETTER 0x2dbcc10 leaf ->
returns &holder 0x106391908, and the BODY 0x2dbcc1c persistence-family state-setter) but
left its reachability UNMEASURED — the frontier doc's own "Honest" line said "the
experience-controller cone remains OPEN for a real re-attack armed with these pins."
SH387 pinned the BYTES hermetically; nobody had ever measured whether the engine actually
EXECUTES that accessor on a headless run — the exact "is the cone door live or dead"
question the operator's SEP-15 directive ("re-examine whether the declared 'not seedable'
wall can be crossed ... rather than a static seed") names. SH388 closes that gap with a
READ-ONLY reachability probe.

## The probe (default-inert, ZERO guest mutation)

- `routeb_dmsvc_getter_probe` (jit.rs, opt-in `JIT_ROUTEB_DMSVC_GETTER=1`): fires at
  block ENTRY of BOTH the GETTER (0x102dbcc10) and the BODY (0x102dbcc1c) on the live
  ladder, and reads back the current-DM holder [0x106391908] (vt + in-image check) + the
  app-data-model counter [0x106dca0e88] (EXECUTE-DO-INIT-GATES marker). Never mutates.
- Hermetic `sh388_dmsvc_getter_cone_door_reachability_pinned` (arm64jit lib 442->443):
  pins the getter/body block-entry words + the observation cells on the real libroblox.so.
- Probe run: `runs/capture_sh388_dmsvc_getter.sh` (full --v2boot reaching env + the
  SH385 guard set + `JIT_ROUTEB_DM_MANUFACTURE=1` so the holder may hold a planted DM).

## MEASURED (real libroblox.so, completing --v2boot ladder + DM-manufacture plant)

- **GETTER fires 0** and **BODY fires 0** — and the whole accessor region
  [0x102dbcc10,0x102dbcd40] gets **0 JIT region hits**. The engine NEVER executes
  setDataModelToCurrent (getter or body) headlessly on the full reaching env.
- The manufactured DM plant DID fire (`routeb-dmmanufacture` 1x): a genuine-vptr
  RBX::DataModel (vt 0x1067162e8) was planted into the current-DM holder [0x106391908],
  which already held an in-image object (0x106358d40). So the holder CAN hold an object —
  but no execution path ever reads it back through this accessor.
- Terminal: the run drains into the standing SH285 persistence-lane wall
  guestpc=0x101d9a528 (LSM pool-pop), EXIT 134 SIGSEGV — the same measured-closed family
  every Route-B ladder arm converges on (SH349/350/379/385). No live DM, MH_* false.

## Interpretation

The SEP-15 setDataModelToCurrent cone door is measured DEAD headlessly: its accessor is
never entered on the full reaching env, so neither "plant a manufactured DM in the holder"
nor "arm this cone" can advance Route B — control diverges to the persistence lane before
the accessor is ever reached (consistent with SH379's path-specific gate finding and every
do-init/EC/governor arm). This is a clean measured closure of the ONE cone the operator
explicitly wanted re-attacked, completing SH387's byte-pin with execution evidence. It does
NOT overturn the standing funnel; it confirms the setDataModelToCurrent arm is un-reachable
as well. The current-DM holder supporting a live object is necessary-but-not-sufficient:
nothing consumes it through this path headlessly.

## Honest

Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false, AppBridgeV2 [0x106a705e8]=0). This is a measured closure of
the last explicitly-named re-attack cone, arming it with the evidence that a real re-attack
would have to first make the accessor REACHABLE (a fixture the cone itself does not
provide). recon-v3 immediate-priority deliverables remain green (self-driven frame + JSON
fix + inert session-gated producer, unchanged this cycle). SH174 capture-latch stays the
single forward observer. Do-not-re-tread updated: do NOT re-attack the setDataModelToCurrent
cone expecting the accessor to fire — it is measured not-executed headlessly.

## Files

- crates/arm64jit/src/jit.rs: +`routeb_dmsvc_getter_probe` (opt-in) + hermetic
  `sh388_dmsvc_getter_cone_door_reachability_pinned` (arm64jit lib 442->443).
  jit.rs ~1,021,xxx B (<1MiB hook).
- Repro: `cargo test -p arm64jit --lib sh388`; probe `runs/capture_sh388_dmsvc_getter.sh`
  (log /home/hermes-worker/runs/sh388-dmsvc-getter.txt, outside repo).