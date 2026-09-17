# Frontier SH285 — the SESSION-CTOR settings-state self-drive crosses the LSM insert-leaf wall (A/B measured)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed — no subagents).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. No production code path edited (the LSM_NODES seed is SH267's,
already committed, default-inert under JIT_ROUTEB_APPSART_LSM_NODES=1).
Workspace green.

## Why (a genuinely new measurement, not a re-run)

The SEP-17 SESSION-CTOR line drove the engine's OWN initEngine_ settings-state
machine headlessly: the state dispatch `[this+16]` (==3/==5/==9) and all three
bodies now self-complete (SH277-284). SH280-284 consistently reported the
settings-state self-drive terminal as `guestpc=0x101db1d04` — **the same
LocalStorageManager INSERT-leaf wall SH260 parked** as the persistence detour.

Meanwhile SH267 (LSM_NODES) crossed that insert-leaf on the **full app-start
ladder** (StartLuaAppDM -> nativeAppBridgeAppStart), moving ITS terminal one
fencepost deeper to the LSM reader/free-list path. But nobody had ever combined
**LSM_NODES with the settings-state drive** — SH267 tested the ladder, SH284
tested the settings-state drive with LSM_NODES **off**. This cycle closes that
gap: does the engine's OWN session-state path, given live node cells, cross the
insert leaf too?

## The A/B (real libroblox.so, full SH284 seed set, 3x each)

- **A baseline (LSM_NODES off)** — exact SH284 parity:
  ```
  3/3 state=9 body direct returned Ok(0x0) + state→10
  3/3 SIGSEGV guestpc=0x101db1d04   (LSM insert-leaf: `ldset x0,x0,[x1]` atomic-OR
                                     of bit1 into sub[idx]=0 -> fault addr 0x0)
  ```
- **B (+LSM_NODES)** — the insert leaf is crossed:
  ```
  3/3 state=9 body direct returned Ok(0x0) + state→10
  3/3 SIGSEGV guestpc=0x101db1b08   (LSM reader/pop path, ONE fencepost deeper;
                                     fault=0xffffffffffffffff, x10=0xff..ff,
                                     lr=0x101db1b18 -> the node-cell-seeded map now
                                     returns a non-null node and the drive walks
                                     into its read-back)
  ```
`state9ok` fires 3/3 in both arms (the settings-state body completes either way —
the gate under test is the downstream LSM lane). Repro: `runs/capture_sh285_settings_lsmnodes_ab.sh`.

## What is genuinely new + measured

- The **LSM insert-leaf wall that parked the SESSION-CTOR drive at SH260/284 is
  now crossed by the engine's OWN initEngine_ settings-state path** when given
  live per-node cells — the exact cross SH267 proved on the app-start ladder,
  reproduced deterministically (3/3) on the settings-state self-drive.
- The SESSION-CTOR terminal advances one fencepost: 0x101db1d04 -> 0x101db1b08.
  This is the first time a SH280-284 settings-state run terminates past the
  insert leaf (all prior cycles parked AT it because LSM_NODES was off).

## Honest (do-not-over-claim)

- Does NOT manufacture a DataModel; DM-root[0x106a68818]=0; MH_* stay false;
  Route-B live-DM structural gate UNCHANGED. The new terminal 0x101db1b08 is
  still the LSM live-object read-back wall (SH268-class: the node's stored value
  is 0 / the free-list link target is not a valid container 0x-case under the JIT).
- What IS new: the engine's OWN session-state path now demonstrably advances past
  the insert leaf given node cells — cause-level SESSION-CTOR progress, one
  measured fencepost (LRM reader path) deeper than every SH260-284 run.
- The remaining wall is still the standing live-object class a real
  LocalStorageManager/session ctor owns; no seed manufacturing added.

## Disposition
`JIT_ROUTEB_APPSART_LSM_NODES=1` = SH267's committed seed, reused here (not new
code). Repro logged to runs/. LEDGER: STATUS.md + HANDOFF.md updated; tree clean.

## Verify
Workspace green (cargo build + cargo test --workspace exit 0). A/B repro above
(3x each arm, deterministic).