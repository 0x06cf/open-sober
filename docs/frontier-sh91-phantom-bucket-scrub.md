# SH91 — scrub phantom image-range bucket slots in the insert chain-walk

## Result (wall advance: the 0x1029f3f7c node-pointer fault is CLEARED; ladder faults much deeper at the OTel registration loop 0x1029b3828)

Read-only recon deleg_f77df07a (disasm-verified) root-caused the SH90-residual fault:
the map header/array were valid and never reallocated (crash-map +0x00 was still the
seeded array; +0x44 divisor still 0x400; x10=0x28000 is just the `udiv` quotient
hash/0x400, not an array size), so it was NOT growth/rehash overflow. Instead a single
BUCKET SLOT held the IMAGE address **0x1029b37ec** (a `.text` thunk the descriptor
registration wrote into a slot as a phantom head link — present only after ~thousands
of inserts into the never-grown 1024-slot map, i.e. deep hash collisions make long
chains). The insert chain-walk `ldr x23,[x22]` (head, file 0x29f3fb4) picked it up and
`ldr x8,[x23,#16]` (file 0x29f3fe4) deref'd unmapped image+16 -> SIGSEGV (guestpc
0x1029f3f7c). Bucket-node layout (24 B): +0x00 key/value (OTel descriptor ptr), +0x08
next (chain), +0x10 hash — NO +0x18 field.

**Fix (jit.rs routeb-hashfix):** at the INSERT entry (0x1029f3e70), scrub any bucket slot
whose 64-bit value is in the guest image range `[base, base+len)` — a valid managed-heap
node pointer or host metadata is NEVER in the image range, so any image-range slot value
is a phantom head link -> NULL it (chain sees empty -> alloc a fresh node). The scrub
only addresses arrays WE own (registered in a shared TRUSTED_BUCKETS set at seed time),
so it never derefs a foreign/unseeded map's invalid +0x00 (fixed a Rust panic from
deref'ing a garbage bbase), and never touches the map header or a valid node pointer —
honoring the SH84/86b discipline (no +0x00 repoint of a populated map).

**Verified:** the SIGSEGV at guestpc 0x1029f3f7c (image-code-as-node-pointer) is GONE.
The ladder now faults DEEPER at **guestpc 0x1029b3828** — file 0x29b3808-0x29b3830, the
OTel/pb_defaults registration CALLER loop: it iterates a `.data` table at guest
0x1067da308 (`adrp 67da000; add x19,#0x308`) inserting each entry into the registry map
at [0x106838380] (x21=0x106838000, `ldr x0,[x21,#896]`), advancing `ldr x8,[x19,#16]!;
cbnz` — and the new fault is `fault==rip==host-heap` (execution jumped INTO a heap
pointer — the same blr-into-heap class as SH86). This is one register-table loop deeper
than the map itself. So SH91 cleared the map-internal chain fault; next is this caller's
dispatch. Workspace **515/0**. Product path unregressed (exit 124, persist byte-exact, 0
crash). The scrub is gated behind JIT_ROUTEB_HASHFIX (--v2boot path).

## Repro

`runs/capture_v2boot_sh82.sh`. Expect `SH91 scrubbed N phantom image-range bucket
slot` + the `SH88 substituted` line, and the ladder faulting at
`guestpc 0x1029b3828` (fault==rip==heap) instead of the map chain-walk.

## Next (ranked)

1. Clear the OTel registration-loop gate at 0x1029b3828: the loop reads the registry
   map slot [0x106838380] (a slot the SH88 substitute-slot seeding left = the seeded
   empty map, so the map is likely fine) and dispatches `blr x8` through a heap ptr
   (fault==rip==host-heap). Trace which `blr` inside the block starting at 0x1029b3828
   jumps to the heap and whether the target is the same operator-new/alloc hook class
   (SH86 seeded 0x1067daaf0/0x1067d0840 — but this may be a DIFFERENT function-pointer
   slot, e.g. the map's insert-hook or vtable entry). Determine the specific slot and
   the fix (extend the SH86 operator-new seed OR the SH81 vtable/host-call seeding).
2. get nativeGameGlobalInit to RETURN so rung 2 nativeUpdateAdapterInit (0x10221c3ec)
   runs -> check rungs 2-6 install the type-4 producer vector [0x106829ea8].
3. Wire NativeHelper callbacks.
Standing structural wall (real self-constructed session) unchanged.