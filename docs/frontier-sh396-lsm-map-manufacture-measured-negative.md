# Frontier SH396 — MEASURED NEGATIVE: manufacturing the LSM map-global cannot cross the SH285 reader wall (the empty-map seeder already provides a coherent map and the reader still faults at 0x101db1b08)

Date: Sep 19-21, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green at
start (cargo test --workspace EXIT 0, arm64jit lib 446/0) and at end (reverted to HEAD —
see Honest Revert below; tests re-confirmed EXIT 0, 446/0).

## Why this cycle
The operator's MIGRATION directive is "keep manufacturing the genuine-vptr DM / PATH-B
reconstruction INSIDE this JIT until a concrete PROOF-of-dead-end (a gate provably
unprocessable), not an ROI judgement." SH384 manufactured the genuine LocalStorageManager
ctor and explicitly left "wiring the manufactured object into the lane so the SH285 reader
consumes it" as the open next-step; SH385 flagged the reader mechanism but no one had
attempted the root-cross: give the SH285 reader a COHERENT map-global so its reached
node's [+40] is a writable value instead of garbage 0xff..ff. SH396 implemented + measured
that manufacture hypothesis.

## The hypothesis under test
Reader 0x1d99e30 (disasm; guest=file+0x100000000): `adrp x8,726f000`@1d99e34 ->
`ldr x8,[x8,#2240]`@1d99e40 (=lsm_map_global [0x10726f8c0]) -> `add x8,x8,(key>>29)*8`@1d99e44
-> `ldar x8,[x8]`@1d99e4c (bucket) -> `ubfx x9,x0,#16,#13`@1d99e48 -> `ldr x0,[x8,idx*8]`@1d99e50
(node) -> `cbz`@1d99e54 -> `tbz [x0]#1`@1d99e5c (collision) -> `ldr x0,[x0,#40]`@1d99e60 (value) ->
ret. The pool-move append 0x1d9a15c then `add x10,x0,x1` and backward-`strb`-stores into the
value; a garbage value (unconstructed node) overflows the base and faults at 0x101db1b08
family (SH285, fault=0xffffffffffffffff). SH396's hypothesis: MANUFACTURE a coherent EMPTY
map-global into [0x10726f8c0] (mg_buf(key>>29) -> 8192-slot bucket(ubfx) -> one node,
[node+0]=0 so the collision tbnz is not taken, [node+40]=leaked writable value buffer)
installed at the ladder entry block 0x2173ff4, ONLY when the map-global is unconstructed (0)
— so ANY key the reader resolves returns a writable value and the append completes. This is
the "root-cross" of SH384's wire-in-lever, cause (manufacture the object) not symptom
(skip/ret).

## What MEASURED (real libroblox.so, full SH384 env + JIT_ROUTEB_LSM_MAP_MANUFACTURE=1, 4/4)
- **The manufacture never installs — the map-global is ALREADY coherent in the ladder env.**
  The existing elfjit `[lsm-map]` seeder (elfjit.rs ~6214-6310, the SH267 static-empty-map +
  per-node-cell machinery) fires FIRST and logs:
  `[lsm-map] seeded static empty LocalStorageManager map: global 0x10726f8c0 -> bucket array
  0x7febce9c0010 (4194304 buckets, shared zero sub @0x7febd09c0010, node_cells=true)`.
  So `cur != 0` at my guard's install point, and my guard CORRECTLY declined (`map-global
  already constructed — leave it, no manufacture`). No SH396 marker fires (mfg-lines=0).
- **And the ladder STILL faults at the SH285 reader wall.** Terminal every run:
  `[SIGSEGV] fault=0xffffffffffffffff guestpc=0x101db1b08` (sh285=1, 4/4).
- Route-B structural gate UNCHANGED: MH_FLAGS_LOADED=false MH_ENGINE_INITIALIZED=false
  MH_APP_READY=false AppBridgeV2[0x106a705e8]=0x0. The genuine LSM ctor drive (SH384) still
  manufactures its vtable-owning manager (vt 0x10635bf88) but that does not move the DM gate.

## Interpretation (the genuinely-new closure)
A coherent EMPTY map-global at [0x10726f8c0] (4194304 buckets + per-node 0x20 zeroed cells,
provided by the pre-existing [lsm-map] seeder) does NOT cross the SH285 reader fault. This
CLOSES the "map-global unconstructed → reader returns garbage" hypothesis with execution
evidence: the reader path that faults reaches a NODE that is NOT in the seeded empty map, with
[node+40] = 0xff..ff from a path-independent unconstructed object. Exactly SH385's verdict —
"the reached node is not the seeded zeroed cell — path-dependent; set only by a real LSM
session ctor" — now confirmed from a NEW angle (a fresh manufacture attempt, not a re-run of
SH385's composed crossings). The SH285 reader wall is genuinely NOT map-global-coherence-
fixable; manufacturing a static empty map cannot substitute for the real LocalStorageManager
session ctor that owns the reached node.

This is the operator's requested "keep grinding until a concrete PROOF-of-dead-end", not an
ROI judgement: manufacture of the LSM map structure (the object the reader reads) is MEASURED
unable to cross the wall, because the wall is upstream of the map (the session ctor that builds
the reached node/value), which no static manufacture reproduces.

## Honest Revert
The SH396 manufacture code was implemented + hermetic-tested (sh396, arm64jit lib 446->447,
workspace green) and MEASURED as redundant + refuted on the real binary, then REVERTED cleanly
(git checkout jit.rs to HEAD; capture script removed). No production Rust / JIT-hook default /
guest byte changed at the final commit — the tree is byte-identical to HEAD SH395 plus this doc
+ HANDOFF/STATUS. The evidence log /home/hermes-worker/runs/sh396-lsm-map-manufacture.txt is
outside the repo (per CLAUDE.md). Kept the run's novel datum (the seeder provides a coherent
map already) as documentation, not cruft.

## Do-not-re-tread (unchanged closures, all still stand)
LSM skips (SH349/350/358/373), keyfix/keytrace (SH341/343), EC reader-gate (SH355/356/374),
0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367), ALooper (SH365), governor
gates full-ladder (SH379), -9 string (SH380), map-header repair (SH248h), once-lambda store
seeding (SH381). SH396 ADDS: do NOT re-attempt manufacturing [0x10726f8c0]-style coherent
empty maps to cross the SH285 reader — the existing [lsm-map] seeder already provides it and
the reader still faults (measured 4/4). The LSM reader wall remains a session-ctor-only
object (SH385), consistent with the standing SESSION-CTOR live-DM wall.

## Files
- This doc.
- HANDOFF.md + runs/STATUS.md updated (SH396 entry).
- Evidence log /home/hermes-worker/runs/sh396-lsm-map-manufacture.txt (outside repo).
- No Rust / guest-byte / hook-default change at the final commit (SH396 reverted cleanly).