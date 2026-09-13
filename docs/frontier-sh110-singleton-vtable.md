# SH110 — dispatch-singleton vtable: widening REVERTED (tracked as next gate)

## The three soft-return leak sites (root-caused, keep this)
V2Init/V2Start/V1 AppStart 'run_loop: pc 0x48... outside image' soft-returns
are NOT JNIEnv dispatches. They are C++ virtual dispatches through lazy-init
singleton objects at guest **0x106829a48** / **0x106829a68**:
- site A (file 0x62517c4):  `[vtable+0xf8]` (slot 31)
- site A2 (file 0x6251aa8): `[vtable+0x108]`
- site B (file 0x6260948):  `[vtable+0x548]` (slot 169)
The seeded vtable was 0x60 bytes, so those indexes read raw host x86 bytes past
the allocation -> blr to outside-image pc -> SOFT return (benign; the milestone
StartLuaAppDM still returns Ok). Global 0x10683d350 = version gate (dissected); it
chooses the code path but is not the leak.

## The widening (0x60->0x580) was REVERTED — here's why
Giving the two singletons a 0x580 flat vtable (every slot = the benign leaf
`routeb_singleton_leaf`, which returns a0) made nativeInitializeNativeFlags
(rung 2) hard-crash with a NULL+0x28 fault BEFORE StartLuaAppDM: a high-slot
virtual at one of those offsets returns a0 into guest x0 and the caller derefs
`x0+0x28` where x0 landed on a low value -> SIGSEGV. Net effect: the sound,
benchmarked milestone (EXIT 0, 0 crash, StartLuaAppDM returned Ok) regressed to
a hard abort.

## Next (differential fix, do NOT blanket-widen)
A flat all-leaf vtable is wrong for these two singletons because SOME high slots
are object/factory virtuals whose return value the caller derefs (must return a
real initialized object or NULL, not the identity a0). Reconcile per-slot:
(a) keep the 0x60 vtable (current, clean) for the benchmarked ladder, OR
(b) widen to 0x580 but make the +0xf8/+0x108/+0x548 slots return a stably-zeroed
guest-resident object record (so the caller's `+0x28` deref is a valid guest
object, not a fault), NOT the interaction a0. Slot-wise: identify at vtable+0xf8
and +0x548 what the caller reads from the return (x0+0x28) before deciding.
The 0x60 state is the committed baseline (b19b1c2 reverted by this doc's commit).

## Repro
product baseline: timeout 75 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe
  --renderframe-drive --renderframe-seedgles --persist-roundtrip --kicker 0x106863af8
full ladder (clean milestone): timeout 60 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000
  V2BOOT_WARMUP_MS=4500 JIT_ROUTEB_HASHFIX=1 [.../elfjit ... --v2boot --renderinit ...]

## Open thread-desync bonus finding
With the 0x580 vtable, nativeGameGlobalInit spawned its REAL worker thread
(thread population 477929:1 / 477930:2) which drove nativeInitializeNativeFlags —
so the engine genuinely self-constructs a worker. Its host OOB during a raw
syscall is the SH55/64 block-cache desync class (separate from the vtable).
