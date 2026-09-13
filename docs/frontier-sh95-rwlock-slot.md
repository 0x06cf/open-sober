# SH95 — don't seed the pb_defaults rwlock-ptr slot (0x106838378) with the map pointer

## Result (wall advance: the pthread_rwlock_unlock heap-crash is GONE; the OTel registrar now completes and the do-init's tail unlock path runs cleanly)

When the registrar loop finally completed (SH94 killed the x19-iterator spin), the
do-init's tail (`mov w0,#1; ret` at file 0x29b39d4) ran and called
`pthread_rwlock_unlock` (guest 0x102a1ce5c, `stp x29,x30; bl pthread_rwlock_unlock@plt`)
with `x0 = [x20,#888] = [0x106838000+888] = [0x106838378]` — and that slot held the
SH88-seeded MAP pointer, not a valid pthread_rwlock. `pthread_rwlock_unlock` on a
garbage lock word does an indirect elision/decoupled jump -> SIGSEGV
(fault==rip==host-heap, guestpc=0x102a1ce5c, rbx_matches_gueststate=false; the trace's
last block before crash was `block@0x1029f4034 -> pc=0x1029b39c4`, the registrar's
post-INSERT advance).

**Fix (elfjit.rs `routeb_seed_pb_registry_map`):** stop seeding `0x106838378`. The
SH88 seed list was `[0x106838368, 0x106838378, 0x106838380]` but only 0x106838368 and
0x106838380 are the REGISTRY-MAP slots the registrar/find ops read
(0x106838380 = `ldr x0,[x21,#896]`+896, the INSERT map; 0x106838368 used by the
find-op registration). 0x106838378 (offset 888) is a pthread_rwlock POINTER slot the
do-init tail unlocks. .bss is zeroed so 0x378 holds a valid UNLOCKED rwlock natively;
seeding it with the map pointer corrupted it. Now only 0x106838368 + 0x106838380 get
the map. Idempotent, --v2boot-gated, product-safe (never reached on product).

**Verified:** the guestpc 0x102a1ce5c rwlock crash is GONE on every run. The registrar
loop completes and the ladder consistently advances to the do-init tail; the run then
faults at a different, native site (guestpc 0x1029f4034 INSERT-tail,
fault==rip==host-heap, rbx_matches_gueststate=false — the SH55/64 concurrent-thread /
block-cache class now that gameGlobalInit runs much deeper). Workspace **516/0**.
Product path unregressed (exit 124, persist byte-exact, 0 crash). Doc
docs/frontier-sh95-rwlock-slot.md.

## Repro

`runs/capture_v2boot_sh82.sh`. Expect NO `0x102a1ce5c` crash; the registrar completes
and the ladder faults at the deeper native INSERT-tail site (0x1029f4034).

## Next (ranked)

1. The remaining 0x1029f4034 fault==rip==host-heap crash is the SH55/64 class
   (concurrent-block-cache / a host-side indirect that lands in heap). gameGlobalInit
   now gets far enough that the render/worker threads and the ladder overlap heavily;
   determine whether this is (a) a new guest fn-ptr/vtable slot holding host garbage
   (seed to a real in-image fn / host-call, SH81/SH86/SH95b-style), or (b) the concurrency
   desync itself (the ladder and render threads sharing the JIT block cache) — in which
   case interpose/serialize the specific guest callback or gate the render thread during
   gameGlobalInit's do-init.
2. Goal: gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) ->
   rungs 2-6 install type-4 vector [0x106829ea8].
3. Wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.