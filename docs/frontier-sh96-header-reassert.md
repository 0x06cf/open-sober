# SH96 — re-assert the coherent 0x400 map header on every INSERT entry (clears the post-growth index-overflow chain-walk)

## Result (wall advance: the INSERT chain-walk index-overflow fault is resolved; the ladder advances to a separate host-side concurrency class)

Read-only recon deleg_9e27d070 (disasm + run-log verified) corrected the SH91/95b
theory: the crash map (x19=0x7f43ec9ab350, +0x00=0x7f43ecdafdf0) IS a seeded family map
whose bucket array IS in TRUSTED_BUCKETS (SH90 registered it) — so the trust/scub layer
did NOT miss it. The real cause is the ENGINE'S OWN GROWTH: the growth path (file
0x29f3ec0, `str x0,[x19]`@0x29f3ee4) re-writes the map's numeric header (mask +0x40, div
+0x44, cap +0x3c, load +0x48) AFTER the once-per-map SH90 seed fired. A later insert
then computes `idx*8` that OVERFLOWS the owned 0x2000-byte array into image memory
(crash x22=0x1029b37f4, a code page in the registrar caller), `ldr x23,[x22]` reads
registrar insns (0x9401a599f941be80), `[x23+16]` faults. So the phantom slot VALUE was a
red herring — the slot ADDRESS overflowed.

**Fix (jit.rs, INSERT entry 0x1029f3e70):** on EVERY insert-entry visit for a trusted
FAMILY map, RE-ASSERT the coherent fixed-0x400 header (+0x3c=0x400, +0x40=0, +0x44=0x400,
+0x48=0x100, +0x60=0, +0x58=0) so the probe's `idx*8` always stays within the owned
1024-slot (0x2000-byte) array regardless of the engine's growth re-writes — plus keep
the SH91 in-image slot-VALUE scrub. Never repoints +0x00 (honors SH84/86b); a fixed
small header only increases collisions, never crashes. Family gate (h1 in
{SPAN 0x1029b4a84, STRING 0x102a25dec}) + trusted gate ensure we never touch a foreign
map or deref a garbage +0x00.

**Verified:** the old guestpc 0x1029f3f7c INSERT chain-walk fault (image-code as node)
is no longer the crashing site — the v2boot runs now fail at a DIFFERENT, host-side
fault (guestpc=0x0, fault==0xffffff80ffffffc8 or fault==rip==host-heap,
rbx_matches_gueststate=false, run-variable 134/139), i.e. the SH55/64 concurrent-thread
/ host-call concurrency class, now that gameGlobalInit runs much deeper. Workspace
**516/0**. Product path unregressed (exit 124, persist byte-exact, 0 crash). The reassert
is gated behind JIT_ROUTEB_HASHFIX (--v2boot).

## Repro

`runs/capture_v2boot_sh82.sh`. Expect the old `0x1029f3f7c` image-code-as-node fault to
be gone; the ladder faults at the host-side class (guestpc=0x0 / fault==rip==heap) or
runs to timeout.

## Next (ranked)

1. Resolve the host-side fault class (guestpc=0x0, fault==rip==heap / sign-extended
   addresses, run-variable): this is the SH55/64 concurrent-block-cache / a host-side
   indirect executing into heap now that the ladder + render/worker threads overlap
   heavily. Determine whether it is (a) a specific guest fn-ptr/vtable slot holding host
   garbage (seed to a real in-image fn / host-call bridge), or (b) the JIT block-cache
   desync from concurrent threads (serialize/gate the render thread during gameGlobalInit,
   or interpose the specific guest callback).
2. Goal: gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) ->
   rungs 2-6 install type-4 vector [0x106829ea8].
3. Wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.