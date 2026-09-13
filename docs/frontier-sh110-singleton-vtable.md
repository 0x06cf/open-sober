# SH110 — enlarge the dispatch-singleton vtable so high-slot virtuals resolve instead of leaking (SH103 class)

## Gate
V2InitWithParams (0x102365c54) / V2StartAppWithParams (0x10258b144) / V1
AppStart__ soft-return with `run_loop: pc 0x48... outside image` (host x86 code
bytes). Recon (deleg_0f55123e) corrected the JNIEnv hypothesis: these are NOT
JNIEnv dispatches. They are **C++ virtual dispatches through lazy-init singleton
objects** at guest **0x106829a48** and **0x106829a68** (their accessor graph
0x6249e9c/0x6249eb8 via globals 0x6829a48/0x6829a68):
- site A (file 0x62517c4):  `ldr x8,[x9]; ldr x8,[x8,#0xf8]; blr x8`  → vtable slot +0xf8 (slot 31)
- site A2 (file 0x6251aa8): `ldr x8,[x8,#0x108]; blr x8`             → vtable slot +0x108
- site B (file 0x6260948):  `ldr x8,[x9]; ldr x8,[x8,#0x548]; blr x8` → vtable slot +0x548 (slot 169)

Each reads `x8 = [[singleton]]` (the object's vtable word), then indexes
`[vtable + off]` and `blr`. The seeded singleton's vtable was only **0x60 bytes**
(every slot = the benign host-call leaf), so `[vtable+0xf8]`, `[vtable+0x108]`,
`[vtable+0x548]` **indexed past the leaked allocation** and pulled raw host x86
code bytes -> `blr` to an outside-image pc -> soft-return (SH103 class).
Global 0x10683d350 (version gate, low byte==6) chose the path but did NOT fix the
leak. 0x1067d1bb0 is a `.got.plt` JUMP_SLOT for pthread_getspecific (benign red
herring in x16).

## Fix (routeb_seed_task_singletons, elfjit.rs)
Enlarge the leaked vtable from 0x60 to **0x580 bytes**, every 8-byte slot =
the single benign host-call leaf (`routeb_singleton_leaf`). Covers vtable-offset
0x548 plus slop. Now `[[singleton]+0xf8]`, `[+0x108]`, `[+0x548]` all resolve to an
in-image/host-call leaf -> the virtual "calls" complete returning 0 -> the bridge
native proceeds past the dispatch instead of soft-returning.

This is Option A from the recon (guest-resident flat vtable table, every slot =
benign leaf) — the robust variant covering any slot offset boot may touch on
these two singletons.

## Result
- Product path (--renderinit/--renderthunk/--renderframe/--persist / no --v2boot):
  EXIT 124 (timeout), **0 crash, 0 stack-smash**, present #0 swap Ok(0x1), persist
  write=45B read_back byte_exact — unregressed.
- Full --v2boot ladder: with the enlarged vtable the engine now actually EXECUTES
  deeper — nativeGameGlobalInit spawns its REAL worker thread (thread population
  477674:1 / 477675:2) which drives nativeInitializeNativeFlags and then aborts in
  a raw-syscall host OOB (tid=477721, rip in libc syscall path, guestpc 0x0,
  in_jit_run). This is the NEW gate: the worker thread's concurrent guest execution
  desyncs the shared block cache (SH55/64 class) rather than soft-returning.
  Workspace **518/0**.

## Next
The worker thread (guest tid 477721) that nativeGameGlobalInit spawns now runs
real init code and hits a host OOB during a raw syscall. This is the shared-
block-cache / concurrent-jit_run desync (SH55/SH64) now surfaced as a hard fault
because the engine genuinely progresses. Investigate that worker thread's guest
context (its own guest stack? JTLS? the -z cvar) before it reaches the syscall,
or scope its jit_run further on the disasm. The enlarged vtable itself is correct
and should stay.

## Repro
timeout 60 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500
  JIT_ROUTEB_HASHFIX=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so
  0x2173ff4 --jni --startapp 0x258b144 --v2boot --renderinit 0x105b3a280 --renderthunk
  --renderframe --persist-roundtrip --kicker 0x106863af8
(worker-thread abort: grep 'SIGABRT.*guestpc=0x0' ; product baseline is the same
command WITHOUT --v2boot.)