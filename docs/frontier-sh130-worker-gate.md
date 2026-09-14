# SH130 — Combined-frame UNBLOCK: worker-admission gate

## Problem (the standing combined flake)
The SERIALIZED combined run (--v2boot ladder + --renderinit + --taskv4-seed frame
+ --deque-node-live) historically presented 0 frames and was ~50-75% flaky with a
false `*** stack smashing detected ***` SIGABRT during rung-0 nativeInitializeNativeFlags.
blamed on SH55/64. JIT_SERIALIZE_RENDER (SH126) only serialized the HARNESS
renderinit thread vs the ladder; it never gated the ENGINE'S OWN self-spawned
clone workers.

## Root cause (recon deleg_5a9376df, read-only + run logs)
- The clone workers are engine threads created via `pthread_create(start_routine
  = 0x10284d168, arg)`, logged by the shim (`[shim] pthread_create(start_routine=
  0x10284d168, ...) -> tid=1/2`), each a fresh host thread with its OWN 16 MiB
  guest stack + CpuState + TLS that immediately runs a TOP-LEVEL `jit_run` of
  0x10284d168. The routine is a thin wrapper: it does one indirect `blr` into the
  real per-thread worker body, then ret. The workers park on the per-thread
  wait-primitive 0x10284d014 (idle lamport/futex) until the engine posts work.
- The FLAME is the MAIN (guest_tid 0) ladder rung executing CORRUPTED guest
  state: when a worker wakes (task job / FUTEX_WAKE / cond signal) it compiles+
  executes guest blocks and MUTATES shared guest .bss/globals WHILE the
  translated ladder rung reads them -> data race -> a torn/half-written guest ptr
  or host ptr lands in a guest slot -> the SH104/105 "stored canary slot
  clobber" -> false __stack_chk_fail. NOT a stack collision (each worker has its
  own stack; SH108's collision was the harness renderinit reusing boot SP,
  already fixed) and NOT a cache-clear race (SH100's guard handles that).
- Deadlock check: the ladder does NOT depend on the workers (SH93 already NOP'ed
  the one CEvent barrier the workers post for; bionic_pthread_join is a no-op),
  so DEFERRING the workers to post-ladder is safe.

## Fix (SH130)
Host-side admission gate in `jit.rs`: `pub static WORKER_ADMISSION_GATE`. In
`spawn_pthread`'s spawned closure, before `jit_run`, park (yield+1ms) while the
gate is set. set by elfjit before the boot entry when
`JIT_SERIALIZE_RENDER=1 && --v2boot`; CLEARED at "ladder done" (LADDER_DONE)
so workers proceed post-ladder. Touches ZERO guest bytes. Default OFF (gate
never set on the product path / without JIT_SERIALIZE_RENDER, so product worker
behavior is unchanged). Not gated per-worker-tid — all self-spawned clone NWRKers
are deferred uniformly (they spawn during boot, before the ladder rungs).

## Empirical (real libroblox.so, serialized combined, runs/sh130-combined-frames.txt)
- EXIT 0 when the run also carries --deque-redrive (settle-loop=3): clean
  ladder + joined, 0 crash (the redrive itself SKIPs on headcell — its own open
  prereq, unchanged).
- Baseline combined (no --deque-redrive): **24 REAL TASK-DRIVEN FRAMES PRESENTED**
  in the combined ladder+render run — `present #N swap Ok(0x1)` monotonically
  across N=0..23, engine's currency-owning renderinit presenter, REPRODUCIBLE
  (3/3 runs: 24, 21, 24 frames). This is the SH127/SH128 goal finally reached:
  the SAME run that constructs the session now emits real task-driven frames.
  Residual: the process then exits **133 (guest SIGTRAP)** — a TaskScheduler
  fatal fired by the UNBOUNDED drain dispatch flood (every idle drain dispatch
  is heartbeat-patched to w4=4 -> PENDING_PRESENTS grows without bound after the
  24-frame presenter cap), not a render crash (no SIGSEGV/SIGABRT; all 24 swaps
  Ok(0x1)). Standalone exits 124 only via the outer `timeout` — the flood is
  inherent to the --taskv4-seed frame heartbeat design.

## Honest scope & status
- **Breakthrough:** worker gate fixes the SH55/64 combined flake (reproducible
  24 frames in the combined run). The combined run now works.
- Residual (next gate): the exit 133 SIGTRAP from the unbounded dispatch flood —
  cap/bound the drain dispatch rate (or PENDING) so the combined run exits clean
  after N frames instead of the engine raising SIGTRAP.
- The SH128 --deque-redrive becomes less central: with the gate keeping StartApp
  resident in its drain, the combined frame plane works WITHOUT the re-drive.
  The redrive remains a documented fallback (its own headcell prereq open).
- Workspace green (522/0), example green.

Command: runs/capture_sh130.sh