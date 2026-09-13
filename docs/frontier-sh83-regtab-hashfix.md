# SH83 — registration string-hash-map: repair garbage hash-fn-2 slot (+0x18)

## Result (wall ADVANCE: SH82b gate cleared; ladder faults FURTHER at bucket probe)

The SH82b fault (guestpc 0x1029f3f7c — `blr x8` through an invalid per-entry
callback) is cleared. The `--v2boot` ladder now advances past the hash dispatch
and faults FURTHER at the map insert's bucket lookup (guest 0x1029f3f84, the
`ldp w9,w8,[x19,#64]` / udiv / `ldr x23,[x22]` bucket-probe) on 3 clean runs.
Same one-gate-per-SH stepwise pattern.

## Mechanism (disasm-verified on the real binary)

`nativeGameGlobalInit`'s do-init builds a string-keyed hash-map (generic
string-hash-map, header at the heap map). Its insert function `0x29f3e70` ends
in the dispatch:

    29f3f6c: ldp x1, x8, [x19, #16]   ; x1 = [+0x10] = primary hash fn
                                       ; x8 = [+0x18] = optional hash fn 2
    29f3f74: cbz x8, 29f3f80          ; if hash2 == 0 -> use x1
    29f3f78: blr x8                    ; else call hash2  <-- SH82b crash site
    29f3f80: blr x1                    ; single-hash path (real string hash)

On the headless JIT path the `+0x10` slot holds the engine's REAL string hash
(0x102a25dec, a plain string-hash leaf at file 0x2a25dec) and the `+0x18`
(optional hash-fn-2) slot holds LEFTOVER HOST HEAP GARBAGE —
0x4741495241003635 = ASCII "56\0ARAIG" (repeated in x8 AND x23 at both faulting
runs) — instead of the engine's default 0. Non-zero hash2 -> `cbz x8` NOT
taken -> `blr x8` jumps into unmapped memory -> SIGSEGV fault=0x0.

The engine map is SINGLE-hash: it only ever needs `+0x10`; `+0x18` must be 0 so
every op falls back to `blr x1`. This is exactly the same "uninitialised
heap field that should be 0" class as every SH80-82 gate (the guest heap is a
host allocation not history-zeroed).

## Fix (JIT_ROUTEB_HASHFIX=1, env-gated opt-in; off by default)

Add a block-entry hook (arm64jit/src/jit.rs run_loop, mirroring json_zero_fix):

- When armed AND pc == 0x1029f3e70 (the map insert ENTRY — a real block boundary;
  note: the dispatch at 0x1029f3f6c is MID-block, not a JIT block entry, so the
  hook must fire at fn entry, not the dispatch):
  - x0 at entry is the map header (the fn does `mov x19,x0` at 0x29f3e98).
  - If [map+0x10] == 0x102a25dec (this is the string hash-map, not a foreign
    two-hash map) AND [map+0x18] != 0, write 0 to [map+0x18].
- Idempotent; the correct value is 0 so a real two-hash map (which would have a
  non-string-hash at +0x10, or actually uses +0x18) is never touched. The
  +0x10 match guards against clobbering a legitimate second hash.

Empirically the +0x18 slot's value is run-variable (this run it happened to be 0
at the logged entries while previously garbage) — the hook only *repairs* when
non-zero, and the observed ladder advance (SH82b crash -> bucket-probe crash) is
consistent: the insert now takes `blr x1` and the REAL string-hash runs.

## New (hermetic) regression

`routeb_hashfix_repairs_garbage_hash_fn2_slot` pins:
- insert entry guest 0x1029f3e70 (file 0x29f3e70) and crash 0x1029f3f7c (file
  0x29f3f7c),
- the +0x10 primary single string-hash 0x102a25dec (file 0x2a25dec),
- the repair: garbage non-zero +0x18 -> written 0, +0x10 untouched.

Workspace **512/0** (+1). Product path (no --v2boot, hook off) unregressed:
exit 124, persist 45B byte-exact, present #0 swap Ok(0x1), 0 crash.

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the `seeded
main-thread-id` line, then nativeGameGlobalInit's real do-init running the
string-hash insert, and the OLD SH82b fault `guestpc=0x1029f3f7c` GONE (advances
to the next gate ~0x1029f3f84 bucket probe).

## Next (ranked)

The ladder clears the hash-dispatch gate and now faults at the map insert's
bucket lookup (guest 0x1029f3f84: `ldp w9,w8,[x19,#64]` mask/count cols ->
udiv/msub -> `ldr x9,[x19]` bucket array -> `ldr x23,[x22]`). The map's
size/count/mask fields (+0x58 size, +0x38 count, +0x3c/40/44 masks, +0x00 bucket
array) are uninitialised host heap -> wild bucket read. Next: (a) disassemble the
map's ctor/empty-init to seed a coherent empty map (bucket array + size/mask so
the probe reads bucket->0 and takes the "not found -> insert new" path); (b) once
the insert completes, continue toward nativeGameGlobalInit returning -> rung 2
nativeUpdateAdapterInit (0x10221c3ec) and check whether rungs 2-6 install the
type-4 producer vector [0x106829ea8]; (c) wire NativeHelper callbacks. Standing
structural wall otherwise unchanged.