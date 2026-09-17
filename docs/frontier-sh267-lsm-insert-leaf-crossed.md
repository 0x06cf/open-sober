# Frontier SH267 — LSM insert-leaf crossed via per-node cell seed (session-drive unblocks one gate)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed — no subagents).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. Default-inert: the new seed only fires under the opt-in env
`JIT_ROUTEB_APPSART_LSM_NODES=1`. Workspace green, one default-config-free
production path (a new option in the existing `seed_static_empty_map`).

## The gate (measured fresh at THIS HEAD f34ca66)

SH260 parked the LocalStorageManager insert-leaf `guestpc=0x101db1d04`
(SIGSEGV fault=0x0) as a "persistence detour live-object wall" and the loop
returned to Route B. But the SEP-17 directive relabeled the SESSION-DRIVE as
the PRIMARY lever, and SH264-266 measured their session rungs as all LATENT —
the --v2boot ladder self-terminates at this same LSM wall *before* any
post-ladder rung (MessageBus.subscribe, OnGameLoaded) can run. So the LSM
insert-leaf is not just the persistence detour: it is the first physical gate
between the ladder and the primary lever.

## The exact mechanism (fresh disasm, real libroblox.so)

The LSM static hash-map at global `[0x10726f8c0]` is two-level:
`*global` = bucket array (indexed by `key>>29`), each bucket slot = a `sub`
sub-array (indexed by `(key>>16)&0x1fff`), each `sub[idx]` = a per-key node
pointer (0 = empty).

- Reader `0x1d99e40`: `ldr x8,[0x726f8c0]; add x8,x8,key>>29<<3; ldar x8,[x8];
  ldr x0,[x8, idx<<3]; cbz x0,ret` — with all-zero sub slots it returns NULL
  ("not found"); correct with the old empty-map seed.
- INSERT `0x1db1cc8` (small-key path): `... ldr x1,[x9, x10<<3]`
  (`x1 = sub[idx]`), `mov w0,#2`, `bl 0x2b9ea40` = an atomic-OR `ldset
  x0,x0,[x1]` that ORs bit1 (the "present" flag) into the NODE that the slot
  points at. With slot=0 the atomic op targets address 0 -> `SIGSEGV
  fault=0x0 at guestpc=0x101db1d04`.

So the map genuinely requires a per-node (per-key) live allocation at `sub[idx]`
for an insert to land — exactly SH260's diagnosis.

## The seed (default-inert, opt-in `JIT_ROUTEB_APPSART_LSM_NODES=1`)

In `seed_static_empty_map`, when the env is set, fill each of the `0x2000`
sub-slots with its own leaked zeroed `0x60`-byte node cell (a real writable
address). Then:
- INSERT's `x1 = sub[idx]` is a real host cell; `ldset x0,x0,[x1]` ORs bit1 into
  valid memory (no addr-0 fault) and the insert completes.
- READER `0x1d99e50` now sees a non-null node; `cbz` falls through to
  `ldr x0,[x0,#40]` = node+40 = 0 (an empty stored value) rather than NULL.

Default keeps the proven all-zero empty-map behavior unchanged.

## Measured (real libroblox.so, full SH259 seed set + SETTINGS_ONCE, A/B 1 run each)

- **OFF** (baseline, unchanged): crash `guestpc=0x101db1d04`, `nodecells=0` —
  the exact parked LSM insert-leaf wall.
- **ON**: `nodecells=1` (seed fired), **`0x101db1d04` GONE**, run advances to
  a NEW terminal `guestpc=0x101d9a528`.

New terminal `0x101d9a528` (fn `sub_1d9a4e0`, the LSM free-list/pop path):
`x1 = 0x101d968e5`, fault addr `0x101d968e4`, x19=0x55db92146e50 (live host
heap), x20=0x101d968e4, lr=0x101d9a6b8. The trailing `str x8,[x1]` at
`0x1d9a568` writes a map link into the node at key `0x101d968e4` — file
`0x1d968e4`, inside the R-E exec segment `[file 0x0,0x62d8190)` prot write:off
. That is the **SH249/SH258 proven-unwritable live-object class** (a pointer
into execute-only code), hit one full fencepost deeper than SH260 parked it.
No seed/repair/count-clamp/dynamic-ctor lever can reach it (SH249 segment
proof). So the LSM lane now advances one gate further and dies at the standing
live-object gate — the same structural wall as Route B.

## Honest (do-not-over-claim)

- Does NOT manufacture a DataModel, does NOT lift the Route-B live-DM gate,
  does NOT get the session rungs (MessageBus.subscribe / OnGameLoaded) to run
  headlessly — the ladder still self-terminates at a live-object write-off wall
  (now `0x101d9a528`, one fencepost past the LSM insert leaf).
- What IS new and measured: the LSM insert-leaf that SH260 parked — and that
  was blocking the SEP-17 session-drive rungs — is now CROSSED by a genuine
  per-node-cell seed; the run reaches a strictly deeper terminal than every
  SH260-266 run. This is the persistence-adjacent gate the session lane needed
  cleared, not Route B itself.
- Default-inert; the seed is the first thing that deterministically moves the
  LSM lane past `0x101db1d04`.

## Code / verify / artefacts
- Code: `seed_static_empty_map` (crates/arm64jit/examples/elfjit.rs) — new
  `JIT_ROUTEB_APPSART_LSM_NODES=1` per-node-cell fill. Hermetic `sh267`
  (real-image guard, skip-if-absent) byte-pins the insert leaf 0x2b9ea40
  (bti/adrp/ldrb/cbz/ldset), the sub-slot read + bl at 0x1db1d40/44/48, the
  reader 0x1d99e50/54, and the writable map base.
- Verify: `cargo test -p arm64jit --example elfjit sh267` = 1 passed; full
  `cargo test --workspace` green; `cargo build --workspace` green.
- Repro: `runs/capture_sh267_lsm_nodes_ab.sh` (A/B OFF/ON).