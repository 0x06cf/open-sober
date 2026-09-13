# SH89 — patch the INSERT-op dispatch blr x8 -> blr x1 (same family as SH87)

## Result (wall advance: the OTel span-map INSERT now completes 12 entries; ladder faults one gate deeper at the map GROWTH/rehash path)

After SH88 cleared the FIND dispatch gate, the ladder faulted at the INSERT op's own
optional-hash-2 dispatch: file 0x29f3f78 (guest 0x1029f3f78) is byte-identical to the
SH87 FIND site:

    29f3f6c: ldp x1, x8, [x19, #16]   ; x1=[+0x10] primary hash, x8=[+0x18] optional hash2
    29f3f74: cbz x8, 29f3f80          ; hash2==0 -> use primary
    29f3f78: blr x8                   ; else branch through hash2   <-- crash (SIGSEGV 0x1029f3f7c)
    29f3f80: blr x1

When the map REHASHES/GROWS, a freshly-moved/copied map's +0x18 momentarily holds
leftover garbage (observed 0x9401a599f941be80) rather than 0, so `blr x8` jumps into
unmapped memory -> fault 0x0. Same family, same fix as SH87: patch 0x1029f3f78
`blr x8` -> `blr x1` (0xd63f0100 -> 0xd63f0020) + block-cache-drop the INSERT op range.

**Fix (elfjit.rs `routeb_patch_map_dispatch`, extended):** now patches BOTH dispatch
sites (FIND/grow 0x1029f4280 + INSERT 0x1029f3f78), both `blr x8`->`blr x1`, and drops
the JIT block cache over both ranges (0x1029f4240..0x1029f4360 + 0x1029f3e70..0x1029f3f90).

**Verified (JIT_TRACE):** the INSERT dispatch now fires `blr x1` to the primary span
hash (`block@0x1029f3e70 -> pc=0x1029b4a84`), the map completes **12 consecutive
inserts** (x19=0x7fd6ac9b0c60 persists across all 12), then faults at the 13th entry
through the map-GROWTH/rehash path (guestpc 0x1029f3f7c, x8=0x9401a599f941be80 garbage,
fault=0x0) — i.e. the opcode patch cleared the immediate blr-into-garbage gate and the
ladder advanced INTO the growth/resize machinery. Workspace **515/0**. Product path
unregressed (exit 124, persist byte-exact, 0 crash). Both patches are --v2boot-GATED
(routeb_patch_map_dispatch only runs on the v2boot ladder path) so the product path is
untouched.

## Repro

`runs/capture_v2boot_sh82.sh`. Expect both `SH87/89 patched map-family dispatch 'blr x8'`
lines (0x1029f4280 + 0x1029f3f78), `SH88 substituted` lines, and the ladder faulting at
the map-growth gate (13th insert, guestpc 0x1029f3f7c) instead of the INSERT dispatch.

## Next (ranked)

1. The 13th insert triggers map GROWTH: the size/load counters hit the growth threshold
   and the grow/rehash reallocates, producing a map whose intermediate +0x18 carries
   garbage. Trace the growth entry (rehash loop ~0x1029f4310 / find-a-new-bucket path)
   to find whether (a) the new grown map needs the empty-header/coherent-number seeding
   the SH84 insert fix applies (it may be a SECOND map instance the INSERT-only seeding
   misses), or (b) the growth's own map copy is the culprit. Likely: extend the SEEN-set
   force-empty/header seed to ALSO fire on the growth (rehash) entry when the newly
   grown map carries garbage +0x18, mirroring SH84.
2. get nativeGameGlobalInit to RETURN so rung 2 nativeUpdateAdapterInit (0x10221c3ec)
   runs -> check rungs 2-6 install the type-4 producer vector [0x106829ea8].
3. Wire NativeHelper callbacks.
Standing structural wall (real self-constructed session) unchanged.