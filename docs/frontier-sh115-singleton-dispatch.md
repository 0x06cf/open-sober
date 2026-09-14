# SH115 + SH116 — singleton-dispatch scoped patch (soft-return resolution) + nativeInit lock-owner helper patch

(recon-v4 iteration of SH110/111 "next gate": make the three nullable-singleton
V2 bodies COMPLETE instead of soft-returning through the 0x60-vtable read-past.)

## The three soft-return sites (root cause, kept from SH110/111)
V2Init / V2Start / V1AppStart / SendAppEventOnAppReady each end a C++ virtual
dispatch through the seeded **0x60** vtable of a lazy-init singleton:
`ldr x8,[x8,#off]; blr x8` reads +0xf8/+0x108/+0x548 PAST 0x60 into host bytes,
blr's outside the image, and the enclosing body 'soft-returns' without finishing.
vtable widening was REVERTED (SH111: nativeInit faults); `blr->mov x0,xzr` was
REJECTED (SH114: return IS deref'd at +0x28 -> NULL+0x28). The differential fix:
**scoped .text patch at each SITE**, leaving nativeInit's own 0x60 reads + the
shared vtable untouched.

## SH115 — materialize-stable-object at 3 sites
Each accessor's 28-byte window (7 instr) is rewritten (guest addrs = file + 0x100000000):

| site | window (guest) | cmd: V2 |
|---|---|---|
| A | 0x1062517b8 | V2Init |
| A2 | 0x106251a9c | V2Start |
| B | 0x10626093c | V1AppStart |

- slot0-3 `movz/movk x8,#OBJ(lo16,16,32,48)` (build the stable object addr in x8)
- slot4 `mov x0,x8`  (x0 = OBJ)
- slot5 `ldr x9,[x19]`  (objA)
- slot6 `str x0,[x9]`   (objA[0] = OBJ) — overwrites the original `mov x0,xzr`

`OBJ = routeb_singleton_obj_addr()` — a stable zeroed 0x80 guest object with
`[OBJ+0]` = a leaked 0x60 vtable whose every slot is the identity leaf
`routeb_singleton_leaf` (virtual dispatch on OBJ -> benign leaf / soft-return).
Returns OBJ so the caller's `[ret+off]` reads resolve (never NULL+0x28). This is
the exact differential remedy SH110/111 mandate.

### Branch re-entrancy bug (OWN, fixed)
slot6 overwrites the shared `mov x0,xzr` tail, which is ALSO the target of the
accessor's early-return branches (`b.eq` when x0==x1, and `cbz x19`) — reached
BEFORE slot5's `ldr x9,[x19]` runs, so slot6 stored through stale guest x9
(first crash: fault=0x28 / x9=0x1). Fix: repoint all 6 such branches (2 per
site) one word forward to the epilogue (skip the store). `repoint_early_branch`
rewrites imm19 (B.cond/cbz): A `b.eq@0x106251784`(0x54000260->0x54000280),
`cbz@0x106251790`(0xb4000213->0xb4000233); A2 `b.eq@0x106251a5c`,`cbz@0x106251a68`;
B `b.eq@0x1062608fc`,`cbz@0x106260908` (-> epilogue).

## SH116 — nativeInit lock-owner helper patch
With the sites patched, nativeInit (rung 0) advances past its former soft-return
and reaches `pthread_mutex_lock(&*(0x1072739c0)+0x28)` (file 0x2320710
`adrp x8,7273000; ldr x0,[x8,#2480]` then 0x2320738 `add x0,x0,#0x28`). The
global's .bss page is **UNMAPPED at runtime** (mprotect RW -> ENOMEM; the engine
re-maps that page on boot), so it can't be seeded by a store.
Fix (file 0x2320710/0x2320714/0x2320718 = guest 0x102320710): replace the
`adrp; ldr` + dead `b` hop with `movz x0,#s_lo; movk x0,#s(16); movk x0,#s(32)`
materializing a stable all-zero 0x60 object s, so the +0x28 lock targets a valid
PTHREAD_MUTEX_INITIALIZER. VERIFIED: nativeInit clears the +0x28 gate and runs to
the NEXT gate (guestpc 0x10232090c, stack deref).

## Verified
- `cargo test --workspace` green (exit 0, no failures).
- `cargo test --example elfjit` 27/0 (sh111 baseline + sh115 x3 + sh116 + others).
- Encodings verified vs `aarch64-linux-gnu-as` (subagent).
- Default `--v2boot` ladder (no flag): EXIT 0, 0 crash, StartLuaAppDM returned Ok,
  taskv4 present #0 swap Ok(0x1), renderframe swap Ok(0x1) — **unregressed.**
- `JIT_SH115_SINGLETON_PATCH=1`: render pipeline runs (renderthunk real ctx,
  task frame present swap Ok(0x1)) AND nativeInit clears the +0x28 lock gate, but
  the ladder exits 134 at the next nativeInit gate (0x10232090c) -> stays OPT-IN.

## Repro (opt-in, will crash at the next nativeInit gate by design)
```
timeout 90 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=4000 JIT_ROUTEB_HASHFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  --renderinit 0x105b3a280 --renderthunk --renderframe --persist-roundtrip --kicker 0x106863af8
```
Watch: `patched site @0x1062517b8/9c/3c` + `repointed ...` + `SH116 patched nativeInit
lock-owner helper @0x102320710` + render pipeline markers, then the next nativeInit fault.

## NEXT (open])
1. Clear nativeInit guestpc 0x10232090c (stack deref) so SH115 flips default-ON.
2. nativeInit -> app-data-model -> getFilesDir fires -> engine opens rbx-storage.db
   in the fsmap store -> remembered session (SH114 data-dir getters already in).