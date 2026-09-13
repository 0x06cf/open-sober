# SH90 — seed the SPAN-hash map too (SH84 header+empty-array seed widened to 0x1029b4a84)

## Result (wall advance: the OTel span-map INSERT now completes thousands of insertions; the 13th-insert growth fault is CLEARED)

Read-only recon deleg_efda8475 (disasm-verified) root-caused the post-SH89 fault: the
13th insert into the OTel **SPAN map** faulted in the post-dispatch probe chain-walk
(`ldr x8,[x23,#16]` at file 0x29f3fe8). The span map (`+0x10 == 0x1029b4a84`, a real
in-image span hash) was NOT covered by SH84's once-per-map empty-header + zeroed-bucket
seed, which was gated on +0x10 == the STRING hash 0x102a25dec. With its numeric header
{+0x00 bucket, +0x3c cap, +0x40 mask, +0x44 divB, +0x48 load} unseeded host garbage,
after ~12 inserts the map grew in place (same object, INSERT reuses x19) with garbage,
so the post-growth probe's slot pointer landed in-image (x22=0x1029b37f4) and
`ldr x23,[x22]` returned the caller fn's own .text bytes (x23/x8=0x9401a599f941be80 =
ARM64 insns `ldr x0,[x20,#888]`+`bl 2a1ce5c` at file 0x29b37f4); `[x23+16]` derefs
unmapped -> SIGSEGV at 0x1029f3f7c.

**Fix (jit.rs routeb-hashfix, SH84 block):** widen the insert-entry seed gate from
`h1 == 0x102a25dec` to `h1 == 0x102a25dec || h1 == 0x1029b4a84`. The seed is already the
correct identical shape for the span map (same layout): +0x00 = fresh leaked 1024x8
zeroed array, +0x38=0x400, +0x3c=0x400, +0x40=0, +0x44=0x400, +0x48=0x100, +0x60=0,
+0x58(u64)=0. With load=0x100 vs the growth threshold (`b.hi 0x1029f3ea8`), growth is
skipped while <256 entries and the probe's idx stays in [0,1023] on the fresh array.
It fires on the map's FIRST insert (still empty -> no entries lost), once per map
(SEEN set), only at the INSERT-entry block boundary (0x1029f3e70 — the family's sole
JIT block entry per SH86b), and never touches a foreign object — fully honoring the
SH84/86b destructive-seed discipline (do NOT widen to other hashes or move the hook to
the mid-block growth/rehash sites 0x29f3ec0/30, which are not JIT block boundaries).

**Verified:** `empty header ... seeded` now fires for BOTH the string and the span map
(observed 2 distinct maps: 0x7fa0880284a0 + 0x7fa088ab1140). JIT_TRACE: the INSERT
block `0x1029f3f7c` executes **3439x** before the next fault (vs ~12 before SH90) —
thousands of OTel descriptor-registration keys now insert into the span map without
faulting. The growth gate is CLEARED. The ladder still faults at the SAME pc
(0x1029f3f7c) but only after deep insertion (a node-pointer-chain corruption
transient — x22/x23 again reading image code bytes as a bucket-node pointer, x21=
0xa0000a2, x10=0x28000, on a deeply-grown map) — a SEPARATE, much later manifestation
that needs its own recon. Workspace **515/0**. Product path unregressed (exit 124,
persist byte-exact, 0 crash). The seed is gated behind JIT_ROUTEB_HASHFIX (--v2boot path).

## Repro

`runs/capture_v2boot_sh82.sh`. Expect `string hash-map @ 0x… empty header + zeroed
bucket array … seeded` lines for ~2 maps (string + span), `SH88 substituted` lines,
and the ladder faulting at pc 0x1029f3f7c only after ~thousands of insert-block
executions (not the immediate 13th-insert crash of SH89).

## Next (ranked)

1. The remaining 0x1029f3f7c fault is now a deep map-node-pointer corruption (probe
   dereferences image-code bytes at x22=0x1029b37f4 / 0x1029b3830 as a bucket node
   after ~thousands of inserts, on a map that has grown large, x10=0x28000). Root-cause
   the chain-walk (`ldr x23,[x22]` at file 0x29f3fd8-ish reading a bucket/node whose
   next-pointer is a small int / image address): likely a bucket-entry node's `+8` (next)
   or key slot is itself the corrupting field, or a second distinct map instance reaches
   a grown state without its header being re-seeded. Next recon should pin which node
   field + which map, then decide (a) extend the per-map seed to fire on RE-growth, or
   (b) repair the specific chain pointer.
2. get nativeGameGlobalInit to RETURN so rung 2 nativeUpdateAdapterInit (0x10221c3ec)
   runs -> check rungs 2-6 install the type-4 producer vector [0x106829ea8].
3. Wire NativeHelper callbacks.
Standing structural wall (real self-constructed session) unchanged.