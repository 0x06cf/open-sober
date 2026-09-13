# SH86 — clear the OTel/pb_defaults operator-new allocator-hook gate

## Result (wall ADVANCE: cleared the deepest Do-Init gate yet)

The `--v2boot` ladder now survives the entire OTel/pb_defaults descriptor
registration path up to the map-rehash copy. Read-only recon deleg_6c882d29
root-caused the guestpc 0x1029b43f0 SIGSEGV to the **CRT `operator new`
allocator-hook dispatch**, not the descriptor table:

- file 0x2a0d9b8 (the `new(size,line)` wrapper): `adrp x8,67da000; ldr x8,[x8,#2800]`
  = **active allocator-hook global [guest 0x1067daaf0]**; `adrp x9,67d0000;
  ldr x9,[x9,#2112]` = **default hook [guest 0x1067d0840]**; `cmp x8,x9; b.eq`
  takes the fast path (TLS allocator 0x1d96a40) when equal, else `blr x8`.
- Both cells are `0` in the file (correct load: equal -> fast path, no blr). Under
  the headless JIT the RW segment leaves `[0x1067daaf0]` as host-heap garbage
  (observed 0x7fcd98dd52e0), so the cmp differs -> `blr x8` jumps to the heap ->
  SIGSEGV (fault == heap addr == rip, the SH85 trailing fault).

**Fix (elfjit.rs v2boot ladder, before driving rung 1 nativeGameGlobalInit):**
`*(u64*)guest 0x1067daaf0 = 0; *(u64*)guest 0x1067d0840 = 0` — idempotent seed
mirroring SH82's main-thread-id pre-drive write; a real boot would install an
allocator override here, which never happens headlessly.

**Verified on real libroblox.so --v2boot:** with the seed the `cmp x8,x9; b.eq`
now takes the fast path, `bl 0x1d96a40` (TLS allocator) runs, the pb_defaults map
(new span-hash map) continues, and the ladder faults FURTHER into the map-rehash
copy (0x1029f4284) instead of the allocator hook.

## New (hermetic) regression

`qsort_interpose_hostside_u32_key_comparator_orders_and_is_named` (SH85) pins the
known comparator addr + +24-u32-key ordering; the map-family routeb_hashfix tests
(SH83/84) remain. Workspace **514/0**.

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the `SH86 seeded CRT
allocator-hook globals` line, the map `+0x18 -> 0` + `empty header` lines, the OLD
gates (0x1029f3f7c / 0x1029f3f84 / 0x1028bbfc0 / 0x1029b43f0) all GONE, and the
ladder faulting at 0x1029f4284 (rehash-copy map) — the next gate.

## Note on the hardening (SH86b/c/d/e/f)

Generalizing the SH84 empty-map force-seed to ALL map-op entries caused a glibc
"double free or corruption (out)" abort (it repointed +0x00/fill of a foreign,
rehashed map). Fixed: the destructive force-empty fires ONLY at the string-hash
insert entry (0x1029f3e70) guarded on +0x10 == 0x102a25dec + once-per-map SEEN set.
The universal +0x18 non-image repair was hardened: fire at the REAL JIT block
entries (0x1029f3e70 / 0x1029f424c / 0x1029f4284 / 0x1029f4310 / 0x1029f4088 /
0x1029f4348 — verified by JIT_REGION_WATCH; 0x1029f4258/0x1029f42d4 are mid-block
and never reached), check BOTH x0 and x19 candidates (the map can arrive live in
x19), skip non-map host-shaped objects via `map >= 0x100000000` AND `+0x10` in-image
guard. A temporary JIT_ROUTEB_HASHFIX_DEBUG trace proved the +0x18 slot of the
WATCHED map is 0x1029b4ae8 (a REAL in-image second hash — correctly left alone), and
the crash `+0x18=0x1800064` belongs to a DIFFERENT map instance created inside the
rehash loop, which the entry-set does not yet observe.

## Next (ranked)

The ladder has cleared: main-id park (SH82) -> intern table (SH82b) -> map batch
dispatch (SH83/84) -> qsort comparator (SH85) -> CRT operator-new allocator hook
(SH86). It now faults deterministically at 0x1029f4284 — the map-rehash COPY of a
NEW span/string map instance whose +0x18 holds 0x1800064 (stable across runs, but a
different map than the one the current hook watches). Next: (a) characterize the
map created INSIDE the rehash loop (its alloc site / when +0x18 is set to a
non-image value 0x1800064 — likely a count/offset being written into the hash-2
slot, or a second map whose +0x10 is ALSO non-image so the guard skips it) and seed
it; (b) get nativeGameGlobalInit to RETURN so rung 2 nativeUpdateAdapterInit
(0x10221c3ec) runs -> check rungs 2-6 install the type-4 producer vector
[0x106829ea8] (still 0); (c) wire NativeHelper callbacks. Standing structural wall
(real self-constructed session) otherwise unchanged.