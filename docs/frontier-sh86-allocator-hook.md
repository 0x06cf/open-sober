# SH86 — clear the OTel/pb_defaults operator-new allocator-hook gate

## Result (wall ADVANCE: cleared the deepest Do-Init gate yet)

The `--v2boot` ladder now survives the entire OTel/pb_defaults descriptor
registration path. Read-only recon deleg_6c882d29 root-caused the guestpc
0x1029b43f0 SIGSEGV to the **CRT `operator new` allocator-hook dispatch**, not the
descriptor table:

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
(new span-hash map) continues, and the ladder faults FURTHER away from the
registration body (next gates are run-variable/nondeterministic; the deterministic
0x1029b43f0 blr-through-heap is gone on every run). The map-family fix was also
generalized this cycle to check BOTH x0 and x19 candidates at map-op entries (the
span-hash map arrives live in x19 while x0 holds the .data registry map).

## New (hermetic) regression

`qsort_interpose_hostside_u32_key_comparator_orders_and_is_named` (SH85) pins the
known comparator addr + +24-u32-key ordering; the map-family routeb_hashfix tests
(SH83/84) remain. Workspace **514/0**.

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the `SH86 seeded CRT
allocator-hook globals` line, the map `+0x18 -> 0` + `empty header` lines, the OLD
gates (0x1029f3f7c / 0x1029f3f84 / 0x1028bbfc0 / 0x1029b43f0) all GONE, and the
ladder faulting at a run-variable deeper pc (next gate).

## Next (ranked)

The ladder has cleared: main-id park (SH82) -> intern table (SH82b) -> map
batch-dispatch +13/84 (SH83/84) -> qsort comparator (SH85) -> CRT operator-new
allocator hook (SH86). `nativeGameGlobalInit` STILL does not return (ladder never
prints "nativeGameGlobalInit returned" / rung 2 never runs). Next: (a) the remaining
faults are now run-variable (e.g. 0x106241148 / 0x7f0000002198) — chase whichever
new uninitialised-heap fn-ptr/data gate surfaces (mirror the seed/hook pattern);
(b) get `nativeGameGlobalInit` to RETURN so rung 2 nativeUpdateAdapterInit
(0x10221c3ec) runs -> check rungs 2-6 install the type-4 producer vector
[0x106829ea8] (still 0); (c) wire NativeHelper callbacks. Standing structural wall
(real self-constructed session) otherwise unchanged.