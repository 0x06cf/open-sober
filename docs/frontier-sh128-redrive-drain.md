# SH128 — Combined re-drive of the drain pop-loop (opt-in)

## Context
Recon-v3 deliverable (1) presents self-driven frames in the STANDALONE path
(`--taskv4-seed frame`, StartApp stays resident in the drain, 24 frames). But in
the SERIALIZED combined run (JIT_SERIALIZE_RENDER + `--v2boot`) StartApp RETURNS,
so no guest thread is resident in the engine's drain pop-loop `0x102856e40` and
the combined run presents 0 frames (frontier-sh127: structural drain-not-driven).

The drain is normally driven by StartApp's jit_run on guest thread 0 (MAIN); its
ABI (disasm) is `jit_run(0x102856e40, x0=deque-root{headcell@+0,tag@+8}, x1=
task-queue obj, x2=finite timeout ms)`. The deque root is recoverable
standalone from the running drainer's x20 (`mov x20,x0` at 0x2856e84) but that is
unavailable after StartApp returns (no resident thread).

## Search-subagent finding (deleg_f139e286, read-only)
x0 IS synthesizable: headcell is a stable image .bss cell (0x10682a638 / 0x10682b338),
tag = [headcell]>>48; the drain reads [x0] read-only so a 16-byte guest root is safe.
x1 (task-queue obj) is NOT a flat global — it is `*( TLS-getter(0x67d67c0) + 0x410 )`
via the pump's getter 0x102b9dee0 (pthread_getspecific), valid on the MAIN thread
whose TLS the scheduler was built on during StartApp. x1 must therefore be resolved
ON the main thread after the ladder joins.

## Implementation (`--deque-redrive`, opt-in, default-unregressed)
Three parties coordinate around one window via atomics REDRIVE_X1/REDRIVE_ACTIVE/
REDRIVE_DONE:
1. **Main thread** (after the ladder joins): resolves x1 via
   `run_guest_callback(0x102b9dee0,[0x67d67c0..])]` then `[+0x410]`; picks a coherent
   headcell; allocates a 16-byte root `{headcell, tag}`; sets REDRIVE_ACTIVE; runs
   `jit_run(0x102856e40, root, x1, finite=TASKV4_REDRIVE_MS default 6000)`; clears
   REDRIVE_ACTIVE; waits ~10s for the presenter to flush.
2. **renderinit presenter** (currency thread): holds its PENDING_PRESENTS drain loop
   open for the whole REDRIVE_ACTIVE window (rather than the default 2s), then marks
   REDRIVE_DONE.
3. **--deque-node-live injector**: waits for REDRIVE_ACTIVE before starting its inject
   budget, so injected type-4 nodes dispatch INTO the live re-driver.

The settle loop is reduced to 3 iters under redrive so main reaches the re-drive
promptly (the re-drive IS the live main-loop work then).

Regressions (hermetic, sh128_tests): the headcell-coherence predicate
`sh128_packed_has_coherent_node` (low48 in-guest node + non-zero 16-bit tag).

## Empirical (runs/sh128-redrive-frame.txt, this session)
The re-drive mechanism is IMPLEMENTED but its runtime verification was BLOCKED: the
serialized combined run remains run-variable at the documented SH55/64 structural
class — rung-0 `nativeInitializeNativeFlags` hits a false `*** stack smashing ***`
(`terminated`, SIGABRT) on ~2/3-3/4 of runs no matter what, driven by the engine's own
clone-worker threads (tids 1,2) racing the ladder's jit_run. JIT_SERIALIZE_RENDER only
serializes render↔ladder, not the engine clones (gating them risks deadlock). So no
clean combined run was obtained this session to reach the `--deque-redrive` drain
re-entry and observe frames. The flag is provably inert without `--deque-redrive`
(all edits gated; baseline capture_sh127.sh re-verified run-variable, not a regression;
workspace 522/0 + elfjit example 36/0).

## Honest scope & status
- Default path unregressed (all SH128 edits gated on `--deque-redrive`).
- Product artifact for task-frames remains the STANDALONE plane (24 frames, clean)
  + the ladder-only session construction (3/3 clean with SH126-r0) — the frontier-sh127
  recommended resolution (1).
- --deque-redrive stays as a documented opt-in for a future clean combined run
  (e.g. once the engine-clone flake is serialized).

Command: runs/capture_sh128.sh (single) or runs/loop_sh128.sh (retry loop).