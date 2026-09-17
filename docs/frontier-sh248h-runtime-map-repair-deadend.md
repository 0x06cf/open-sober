# SH248h — runtime repair of the app-start live map header is also a dead-end
# (the "only a static seed is impossible" residual from SH248g is now closed)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## The gate left open by SH248g
SH248g determined the DMCONT continuation's nativeAppBridgeAppStart wall (SIGSEGV
guestpc=0x1021dde34) is a LIVE host-heap robin-hood hash-map insert whose
element-count field [map+8] is ctor-uninitialized garbage. It CONCLUDED "Seeding
an empty-map header is not possible at a stable address (ASLR)" — but that only
ruled out a STATIC .bss seed of the header. The runtime repair was never tried:
the map `this` is register-visible (x21) at the exact fault block-entry, so a
block-entry guard could substitute a coherent header IN PLACE the same way
routeb_setfix_empty_set (SH123) substitutes a coherent empty set for a dangling
live container. That was the genuinely-untried angle.

## This cycle: implement + A/B the runtime repair (default-inert)
Added `routeb_appstart_map_repair_guard` (opt-in JIT_ROUTEB_APPSART_MAP_SEED),
fires at pc==0x1021dde34, reads x21 from the live CPU state, and if the map's
header is incoherent (bucket base [map+0] unmapped OR count [map+8] not a sane
power-of-two), re-seeds a coherent EMPTY-8-BUCKET robin-hood header in place
(base=leaked 8-slot zeroed array, count=8, size=0, load-factor 0.75f). Idempotent
per map pointer (HashSet).

## MEASURED (real libroblox.so, full DMCONT env, 3 completing runs)
1. The guard FIRES and is idempotent: 3 map pointers get repaired per run
   (0x555f..0xe8 / ..0x388 / ..0x4d8, count was 0x2 each). Deterministic.
2. The wall is NOT crossed: every run still SIGSEGVs at guestpc=0x1021dde34
   (EXIT 134 / 3-run). Original fault addr 0x0 -> repaired-run fault addr 0x9
   (the header repair CHANGES the bucket walk but the fault persists).
3. WHY repair can't help (register evidence, both arms):
   - The 3 repaired maps are the SUCCESSFUL baseline iterations — they are
     genuinely host-heap (0x55f..) with coherent count=2; my repair was harmless
     to them because they were already fine.
   - The FATAL 4th-iteration map `this` is NOT host-heap: x21=0x100548ca9
     (baseline) / 0x1004a0373 (repaired) — an IN-IMAGE / low guest address
     (top 16 bits 0), not reachable/repairable by any header seed. The guard's
     `(map>>48)!=0` host-heap check correctly SKIPS it.

## Verdict (do-not-re-tread)
The map-header repair lever is a MEASURED dead-end, independent of static seeding.
The wall's fatal object is an under-allocated live array whose 4th stride
(0x2a0-step cursor, this+3*0x2a0) runs off the constructed region into
in-image/garbage memory — the app-start LiveObject graph that only a real
DataModel ctor would fully initialize. This is exactly the SH174/SH204
live-object structural gate, now confirmed from inside nativeAppBridgeAppStart
with BOTH the static-seed (SH248g) and runtime-repair (SH248h) levers closed.
Route-B live-DM structural gate UNCHANGED. SH174 capture-latch (arm
*(0x106391908) at a real make_shared<DataModel>) stays the single forward hook.

## What is new + measured this cycle
- Closes the residual SH248g deliberately left open ("only static seed ruled
  out"): runtime in-place header repair is also a dead-end at this wall.
- Confirms the DMCONT continuation's deepest reach (inside app-start) is the
  same live-object gate — app-start advances to it one gate deeper than before
  (SH248d-f reached it; SH248g+h characterize it unreachable-by-seed).

## Code / files
- Reverted the experimental guard (was default-inert, not shipped — destructive
  to nothing, but a lever provably unable to fire on the failing case adds no
  value). Clean tree, no default-config change. cargo build + cargo test
  --workspace green (lib 397/0).
- Doc: docs/frontier-sh248h-runtime-map-repair-deadend.md.
- repro: the SH248g confirm script (runs/batch_sh248f_adapter_seed.sh + same env
  with JIT_ROUTEB_APPSART_MAP_SEED=1).