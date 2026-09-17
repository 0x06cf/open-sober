# Frontier SH260 — LocalStorageManager insert leaf: the new terminal after the settings-once seed (measured, parked)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed — no subagents).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. Workspace green, no default-config production path edited
(pure regression-pin + measurement, default-inert).

## The gate (measured this cycle, fresh at THIS HEAD d323ef6)

SH259 (f422cfc/d323ef6) cleared the settings/registry factory once-guard
[0x106a6f430].bit0=1 and the app-start orchestrator's standing 0x1021dde34
map wall is GONE — the orchestrator now walks its OWN ~93-block app-start
registration body. This cycle re-runs the fresh SH259 repro (real libroblox.so,
full SH248c-f + SH259 seed set) and captures the NEW terminal with a
JIT_DUMP_PC register dump (evidence runs/sh259-settings-once.txt) to decide
whether the persistence/data-store wall is seedable or a live-object gate.

## Measured result

- **New terminal confirmed**: SIGSEGV guestpc=0x101db1d04 = fn 0x1db1cc8
  (small-key map-insert, LocalStorageManager init), after the settings-once
  seed; 93 distinct app-start region pcs walked. Ladder runs StartLuaAppDM ->
  nativeAppBridgeStartAppWithParams -> nativeAppBridgeAppStart.
- **The map global is WRITABLE and the seed IS intact**: `lsm_map_global =
  0x7f9c7c51a010` (== the harness `seed_static_empty_map` bucket-array base,
  readback confirmed at elfjit.rs:6741). [0x10726f8c0] is a writable .bss cell.
- **The fault ADVANCED one fencepost past the NULL-bucket read** (SH248c
  pattern): the reader path (0x1d99e30) that previously NULL-deref'd is now
  serviced by the seeded empty map; the crash is now in the INSERT leaf:
  fn 0x1db1cc8 reads the map base (0x1db1d08 adrp/0x1db1d14 ldr [x8,#2240]),
  computes the bucket slot (0x1db1d20 add x9,x8,x9,lsl#3), `ldar x9,[x9]`
  (0x1db1d2c) reads the seeded bucket->0 (shared zero sub) fine, then
  `bl 0x2b9ea40` (0x1db1d44, x30=0x101db1d48 at crash) — the atomic-claim
  insert leaf: `ldr x1,[sub,idx*8]`=0 then `ldset x0,x0,[x1]` on addr 0 ->
  fault=0x0.
- **Insert-leaf opens a REAL GAP beyond the read path**: 0x2b9ea40 is
  `bti c; adrp x16,683b000; ldrb w16,[x16,#2648]` (an atomic-or refcount flag
  dispatching between ldset/ldxr-stxr). The insert wants to claim a per-node
  slot in a real bucket (sub-element), i.e. GENUINE per-node live allocation,
  not a read-only "not found" map. That is the SH174/SH204 live-object class.

## Decision (operator SEP-15 directive)

The LocalStorageManager init is the persistence/data-store line (objective 2b),
which the operator's Sep-15 hard directive explicitly flags as a DETOUR — the
loop is required to RETURN to Route B (live-DM construction) as TOP priority,
not to grind the cookie/persistence cone. So this wall is MEASURED and PARKED:

- The reader path is already seed-advanced (SH248d-259 progress is real).
- The insert leaf needs genuine per-node live allocation (`ldset` claim) that a
  fixed seed cannot supply without fabricating a whole live hash-map with real
  buckets — the SAME structural gate as SH174/SH204, at the persistence layer.
- No further levers churned here. Not because it's "hard" but because per the
  operator's standing re-attack directive this specific line is a detour: the
  forward hook for Route B is unchanged and it is the SH174 capture-latch
  (arm *(0x106391908) at a real make_shared<DataModel>).

## Route-B re-measurement at the new state

Fresh at THIS HEAD with the app-start registration body executing (SH259), a
region-watch on the genuine-DM surface shows **0 hits** for EC-world
[0x102e1c650,0x102e25200), the marshaler 0x1023f03b4–0x1023f1300, and
DataModelServices 0x102dbcc10–0x102dbcf00. So SH259's newly-unlocked app-start
body does NOT shift Route-B reachability — the live-DM structural gate is
UNCHANGED even with ~93 new app-registration blocks executing headlessly.

## Honest (do-not-over-claim)

- Does NOT manufacture a DataModel, does NOT lift the Route-B live-DM gate.
- The SH259 value holds: deepest app-start gate cleared + a real app-registration
  body (0x233a804..0x233ae38, 0x102346f2..) executes headlessly that 9 prior
  cycles never reached; the persistence line is reached for the first time.
- The LSM wall is confirmed (measured, not judged) as the SH174 live-object
  gate at the persistence layer, and is PARKED per the operator directive.

## Artefacts

- Code: none beyond a hermetic test. `sh260_lsm_insert_wall_anchored`
  (crates/arm64jit/examples/elfjit.rs, real-image guard family, skip-if-absent)
  byte-pins the LSM insert-fn entry + map-base read + bucket deref +
  `bl 0x2b9ea40` + the insert-leaf 3-word prologue + writable map-global
  membership. Drift fails loudly.
- Verify: cargo test --workspace green; cargo test -p arm64jit --example elfjit
  sh260 = 1 passed; recon-v3 plane re-verified green this cycle (24 task-driven
  frames swap Ok(0x1), 197 pops, 0 json abort, EXIT 124).
- Repro: runs/capture_sh259_settings_once.sh (fresh SH259 measurement).

## Route-B standing (unchanged)

Route-B live-DM = structural gate. SH174 capture-latch (arm *(0x106391908) at a
real make_shared<DataModel>) stays the single forward hook. recon-v3
deliverables (type4 self-driven frames + json zero-fix) stay shipped + verified.