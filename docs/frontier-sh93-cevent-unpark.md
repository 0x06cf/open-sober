# SH93 — NOP the gameGlobalInit CEvent barrier (un-parks the do-init futex deadlock)

## Result (wall advance: the futex deadlock is GONE — the ladder exits 0 instead of parking to the 124 timeout)

Read-only recon deleg_d518ab04 (disasm-verified) root-caused the standing gameGlobalInit
PARK: it is NOT a nanosleep (file 0x284d114 is a single aarch64 `futex()` syscall,
`__NR_futex`=98, `FUTEX_WAIT_BITSET|FUTEX_PRIVATE`=0x89, uaddr=[waitprim+4], val=word,
timeout=NULL→infinite, val3=~0) backing a **SyncTask/CEvent::wait** = 0x2207578 (loop
0x2207648: `ldrb w8,[x19,#1]; tbnz w8,#0→done; else pump→futex; b`). The polled
completion flag = **byte [CEvent+1] bit0**; the CEvent is a STACK sync-task created in
the globalinit do-init at guest 0x102206ebc (class-name strings "CEvent::CV"/"CEvent::MX"
confirmed at file 0x40592d/0x591082). Real setter: the TaskScheduler built in this
do-init (0x2206f04/0x2207118) posts an init job whose worker-thread completion writes
bit0=1 + FUTEX_WAKE. Under the JIT the worker/job thread is never spun (SH55/SH64 forbid
concurrent jit_run — shared block-cache SIGABRT), so the completion never fires and the
main thread parks forever. [0x72739d4] bit0 is an OUTPUT of the do-init (writes at file
0x22474e8), not the waited-on gate — the ladder is past the ROUTE-B latch.

**Fix (elfjit.rs `routeb_patch_globalinit_cevent_barrier`):** NOP the barrier call in the
globalinit do-init ONLY — file 0x2206e70 (guest 0x102206e70) `bl 2207578` = CEvent::wait
(0x940001c2) -> nop (0xd503201f). Scoping verified: this block is reached ONLY via
`b.ne 0x2206e28` at file 0x2206df0 (the NON-main-thread dispatch; single predecessor);
the real main-thread path (b.ne not-taken -> 0x2206df4) returns AFTER the barrier and
never reaches 0x2206e70, so the product path is untouched (PA annunciator = `SH93` line
is ABSENT on the product run). Append-only bl->nop (valid, no inner veneer). The
TaskScheduler is already fully constructed at 0x2206e68; the barrier is a pure
handshake. Gated like SH87 (--v2boot/JIT_ROUTEB_HASHFIX path only). Do NOT patch the
shared generics (ctor 0x2206ebc / wait 0x2207578 / pump 0x2850520 each have ~5-14 callers
-> global regression, the SH82 approach-A lesson).

**Verified:** with the NOP, the v2boot run EXIT goes from 124 (futex-park timeout) to
**0 (clean, no crash)** — the futex deadlock is GONE. The do-init now progresses past
the barrier and enters the TaskScheduler registration hot-loop (millions of JIT block
hits, run log runs/sh93-v2boot-cevent-unpark.txt). gameGlobalInit still has not RETURNED
from its jit_run (the post-barrier TaskScheduler poll is the next layer), but the
infinite futex park is cleared. Workspace **516/0**. Product path unregressed (exit 124,
persist byte-exact, 0 crash, NO SH93 line = barrier unreachable on product). Doc
docs/frontier-sh93-cevent-unpark.md.

## Repro

`runs/capture_v2boot_sh82.sh`. Expect `SH93 NOPed globalinit CEvent barrier` line, then
the ladder running gameGlobalInit's do-init to a clean EXIT 0 (no 124 futex-timeout).

## Next (ranked)

1. gameGlobalInit still has not returned from jit_run: after the barrier it enters the
   TaskScheduler registration hot-loop (the same pump 0x2850520-family the CEvent used —
   now as a TaskScheduler work-queue / poll, still cross-thread-dependent). Trace where
   it now spins (post-barrier 0x2206e74 -> 0x2206e78 CEvent dtor -> TaskScheduler poll)
   and whether the next wait (2nd CEvent / a scheduler step / the UpdateAdapterInit
   handshake) needs the same NOP or the worker job must be run single-threadedly.
2. Goal (a) from recon-routeB: gameGlobalInit RETURNS so rung 2 nativeUpdateAdapterInit
   (0x10221c3ec) runs and rungs 2-6 install the type-4 producer vector [0x106829ea8].
3. Then wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.