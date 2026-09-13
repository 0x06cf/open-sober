# SH87 — patch the map-family dispatch blr x8 -> blr x1 (hash2 slot can't be branched into)

## Result (wall advance + new caller-side gate surfaced)

Read-only recon deleg_48b60f95 (disasm-verified) root-caused the deterministic
0x1029f4284 SIGSEGV to the hash-map family's generic dispatch at file 0x29f427c:

    29f4274: ldp x1, x8, [x19, #16]   ; x1=[+0x10] primary hash, x8=[+0x18] optional hash2
    29f427c: cbz x8, <l1>             ; hash2==0 -> use primary
    29f4280: blr x8                   ; else branch through hash2   <-- crash
    <l1>:   blr x1

The OTel/pb_defaults rehash-copy creates a NEW map whose +0x18 carries the STABLE
constant 0x1800064 (=0x0180·0x10000|0x64, decoded from the static `.data.rel.ro`
protobuf table at file 0x62f5110) instead of 0, so `blr x8` jumps to 0x1800064
(unmapped) -> SIGSEGV fault=0x1800074 (=0x1800064+0x10). Crucially 0x1800064 is a
deterministic copied constant, not random heap garbage — which is why it's stable
across every run.

**Fix (elfjit.rs `routeb_patch_map_dispatch`):** patch file 0x29f4280 (guest
0x1029f4280) `blr x8` (0xd63f0100) -> `blr x1` (0xd63f0020), cache-drop the map
family block range. Safe because:
- hash2 is REDUNDANT in this family — the observed real value 0x1029b4ae8 is just
  `br x1` (aliases the primary hash);
- the hash only SELECTS a bucket probe; correctness comes from the key-eq
  comparator at map+0x08, so always using hash1 cannot reorder/break lookups;
- it is a key-eq-validated dispatch, immune to JIT block-entry coverage gaps (the
  JIT_ROUTEB_HASHFIX entry-hook misses the rehash-copied map because +0x18 is set
  mid-loop).
Applied ONLY on the --v2boot path (like routeb_patch_dispatch_gate), so the product
path is untouched.

**Verified:** JIT_TRACE shows `block@0x1029f4284 -> pc=0x1029b4a84` — the patched
`blr x1` now fires the primary span-hash. The old blr-x8-into-0x1800064 dispatch
gate is gone. BUT the ladder now faults at a NEW caller-side gate: the rehash fn is
entered with **x19 = 0x1800064** (a constant, not a heap map) at some call site, so
the fn reads [0x1800064+16] and faults — i.e. a caller is passing the protobuf
constant table entry as the map/this argument, independent of the dispatch. That is
the next gate.

## New (hermetic) regression

None added (the fix is a .text opcode patch exercised only on the real binary's
--v2boot path — same as SH81's gate force). Workspace **514/0**.

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the `SH87 patched
map-family dispatch 'blr x8' 0x1029f4280` line, and the ladder faulting at the NEW
caller-side gate (`x19=0x1800064` passed as the map) instead of blr-x8-into-0x1800064.

## Next (ranked)

The 0x1029f4284 dispatch gate is cleared by the opcode patch. The ladder now faults
because a caller passes the constant entry 0x1800064 (from `.data.rel.ro` file
0x62f5110) where a map/this pointer is expected. Next: (a) find that call site (the
rehash-copy loop that writes +0x18=0x1800064) and determine the real map/object it
should pass — likely a per-OTel-resource-schema descriptor whose registration index
is being misread as the map base; (b) get nativeGameGlobalInit to RETURN so rung 2
nativeUpdateAdapterInit (0x10221c3ec) runs -> check rungs 2-6 install the type-4
producer vector [0x106829ea8] (still 0); (c) wire NativeHelper callbacks. Standing
structural wall (real self-constructed session) otherwise unchanged.