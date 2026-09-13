# SH107 — Route-B standing wall broken: nativeGameGlobalInit RETURNS + rung-2 clears

## The win
The SH106 guard-GOT seed (making the pervasive `__stack_chk_guard` slot 0x67d16f0
stable) unblocked rung-1: **`nativeGameGlobalInit` now RETURNS** — the standing
wall since SH54/55 ("gameGlobalInit NOT RETURNED"). The ladder then advanced to
rung-2 `nativeUpdateAdapterInit` (0x10221c3ec).

## rung-2 gate (this cycle)
nativeUpdateAdapterInit read a global adapter-config RECORD pointer through BSS
global [guest 0x106ed7a18] (`adrp x9,6ed7000; ldr x9,[x9,#2584]; ldrb w10,[x9];
tbnz w10,#0`). Left 0 by the JIT -> `ldrb [x9]` NULL-deref SIGSEGV at 0x10221d7a8.
Seed a zeroed 0x20 record (bit0==0 -> takes the clean copy path
`ldr q0,[x9]; str q0,[x8]; ldr x10,[x9,#16]; str x10,[x8,#16]; ret`,
copying zeros, no change). rung-2 returns Ok(0x1).

## Verified (no-render --v2boot)
```
driving nativeGameGlobalInit ... seeded main-thread-id ... returned Ok
after nativeGameGlobalInit: [0x106829ea8] = 0x0
driving nativeUpdateAdapterInit @ 0x10221c3ec ... returned Ok(0x1)
after nativeUpdateAdapterInit: [0x106829ea8] = 0x0
driving setTaskSchedulerBM(false) -> stopped: run_loop: pc 0x88b8b515058595a outside image (garbage-blr, soft return)
driving V2InitWithParams @ 0x102365c54 (driven; run at timeout)
```
Three rungs execute where the ladder previously parked/aborted at rung-1.

## Open gates
- (a) Layer-2 canary-slot writer: ONLY with `--renderinit/--renderthunk` active, a
  nested-frame `stp x29,x30` saved-pair (stack ptr + return addr) is written onto
  the ladder frame's canary slot. WITHOUT render flags the ladder runs clean past
  gameGlobalInit. Render-reentry/cross-thread stack isolation — follow-up.
- (b) setTaskSchedulerBM blr's to garbage 0x88b8b515058595a — next gate.
- (c) V2InitWithParams — the big remaining init, now reachable.