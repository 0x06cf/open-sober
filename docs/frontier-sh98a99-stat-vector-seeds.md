# SH98–SH99d — stat struct-size marshaler + gameGlobalInit probe-vector / dispatch-object seeds

## SH98 (fully cleared a gate): bionic `struct stat` marshaler

`*** stack smashing detected ***` (glibc `__stack_chk_fail`) was the deterministic
post-SH97 crash. gdb + JIT_TRACE pinned it: the deep gameGlobalInit walk stat()s a
guest STACK-LOCAL bionic `struct stat` (128 B); `stat` was unshimmed, so raw host
glibc wrote its x86-64 144 B struct -> 16-byte overrun clobbered the adjacent
`__stack_chk_guard` canary. New `bionic_stat/fstat/lstat` (shims.rs) run real
`libc::stat` into a host buffer and marshal into the bionic aarch64 128-B layout
(UAPI offsets, hard-capped 0x80). **`stack smashing` count = 0**, ladder advanced to
guestpc 0x10220847c. +1 hermetic regression. Commit 676108b.

## SH99–SH99d (advanced, not fully cleared): the probe vector + dispatch object

The 0x10220847c `fault=0x0` was the walker (fn entry 0x102208354) deref'ing the
global 0x10-element-stride vector's BEGIN slot [0x106dcae10]=0 UNCONDITIONALLY
(`ldrb [x8]` @0x10220845c) before any empty check, then virtual-dispatching on
[0x106dcae20]. Seeds (elfjit `routeb_seed_game_global_vector`, --v2boot gated):
- vector end/begin/cap 0x106dcae08/10/18 -> one non-null zeroed node (empty span);
- dispatch obj 0x106dcae20 -> a 0x70 obj whose every slot = a leaf-filled vtab;
- second 8-byte vector 0x106dcaEA8/B0 -> node (empty, skips `ldrb [x23+8]`);
- getter once-guard 0x106846ba0 bit0=1 (cached path), and 0x106846970 shaped as
  BOTH vector (end/begin=node) AND vtable (+0x10/+0x18=leaf) — recon deleg_af1196d9
  showed the constructor (bl 0x10220890c) stores the once-guarded getter 0x101dc4418's
  result as obj+0's vtable, so pre-seeding [0x106dcae20] alone cannot survive.

Fault advanced deterministically 0x0 -> 0x10 -> 0x18 across these; BOTH virtual
`blr` dispatches (0x102208500 and 0x102208580) now execute and the once-guard cached
path is exercised. **The ladder still faults** at reported guestpc 0x102208504
(fault 0x18) on a guest WORKER thread — and the crash site is RUN-VARIABLE across
0x102208450 / 0x102208504 / 0x1021db144 (see runs/sh99{i,j}*.txt), which is the
SH55/64 concurrent-thread/block-cache signature, not a stable unseeded content slot.

## Next (ranked)

1. Decide whether the residual 0x102208504 fault-0x18 is (a) one more unseeded
   sub-object (seed it) or (b) SH55/64 concurrency (fix the worker-thread block-cache
   / cached-stack reuse instead). The run-variability across sites argues for (b).
2. Goal: gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) ->
   rungs 2-6 install type-4 vector [0x106829ea8].
3. Wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.