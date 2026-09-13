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

`routeb_hashfix_repairs_garbage_hash_fn2_slot` pins (SH83):
- insert entry guest 0x1029f3e70 (file 0x29f3e70) and crash 0x1029f3f7c (file
  0x29f3f7c),
- the +0x10 primary single string-hash 0x102a25dec (file 0x2a25dec),
- the repair: garbage non-zero +0x18 -> written 0, +0x10 untouched.
`routeb_hashfix_seeds_coherent_empty_map_header` pins (SH84):
- the bucket-probe crash site 0x1029f3f84 (file 0x29f3f84),
- the coherent empty-map numeric header (count/cap +0x38, divisors +0x3c/+0x44 =
  0x400, mask +0x40=0, load +0x48=0x100, size u64 +0x58=0, err +0x60=0),
- idx = hash mod 0x400 stays in [0,1023] for ANY hash, and empty bucket slots read
  sentinel 0 (the "not found -> insert new" path).

Workspace **513/0** (+2). Product path (no --v2boot, hook off) unregressed:
exit 124, persist 45B byte-exact, present #0 swap Ok(0x1), 0 crash.

## SH84 addendum (same hook, cleared the bucket-probe gate)

The insert now runs its REAL string-hash (0x102a25dec) and faulted FURTHER at the
bucket probe (guest 0x1029f3f84: `ldp w9,w8,[x19,#64]` (mask/divisor) -> udiv/msub ->
`ldr x9,[x19]` bucket base -> `add x22,x9,x8,asr#29` -> `ldr x23,[x22]` wild bucket).
Root cause: map numeric header (+0x38..+0x60) uninitialised host-heap garbage -> idx =
hash mod garbage -> wild slot read -> `ldr x23,[x22]` reads garbage (x23 still showed
0x4741495241003635 in the fault dump). Fix extends the same hook at insert entry
0x1029f3e70 (guarded on +0x10==0x102a25dec): force +0x00 bucket array to a fresh
zeroed 1024x8 array ONCE per map (host-side SEEN set; repeat inserts keep the entries
the insert-new path writes) + set count/divs/mask/load/err/size. Empirically (real
libroblox.so --v2boot, 2 clean runs): the probe now computes idx in [0,1023], reads
bucket sentinel 0, and the first entry INSERT COMPLETES — the ladder faults FURTHER at
a NEW deeper region, file 0x28bbfc0 (a qsort comparator of the enum-registration
path). Note: this run the +0x18 slot held 0x102a25ee0 (a near-hash value, not the SH83
"56\0ARAIG" garbage) — the +0x18-zero repair is still required and works.

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the `seeded
main-thread-id` line, then the run's `+0x00 forced to zeroed 1024x8 bucket array` +
`EMPTY header seeded` lines, the OLD SH82b fault `0x1029f3f7c` AND the bucket-probe
`0x1029f3f84` both GONE, and the ladder faulting further at `rip=0x1028bbfc0` (the
qsort comparator gate).

## Next (ranked)

SH83 cleared the hash-dispatch gate (+0x18 garbage); SH84 cleared the bucket-probe
gate (the do-init's string-hash-map insert now COMPLETES its first entry headlessly).
The ladder faults FURTHER at a NEW deeper region — file 0x28bbfc0, a qsort
COMPARATOR (`ldr w8,[x0,#24]`, comparing 0x50-byte record [+24] fields), reached
from the enum-registration path at file 0x28bbf80 (3× qsort@plt on record arrays).
That is the next gate. (a) identify the array being sorted + where it should come
from (the comparator crash reads a bad element ptr / the base/mask fields of the
record array are garbage) and seed/repair so the sort completes; (b) continue
toward nativeGameGlobalInit returning -> rung 2 nativeUpdateAdapterInit
(0x10221c3ec) -> check rungs 2-6 install the type-4 producer vector [0x106829ea8];
(c) wire NativeHelper callbacks. Standing structural wall otherwise unchanged.