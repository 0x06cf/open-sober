# Frontier SH320 — cross the do-init DONE-path dispatcher thread-match gate to the MAIN
# (DM-ctor) branch via a JIT block-entry guard (SESSION-CTOR, candidate (b))

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh320` (crate-side env/pc-gated unit
test in jit.rs) + `sh320_doinit...` (elfjit real-image pin of the bind-dispatch words).
Workspace green (arm64jit lib 406/0 = 405 + sh320; elfjit examples 145/0 = 144 + sh320 elfjit pin).
elfjit.rs held 54 B under the 1MB pre-commit hook (after condensing ~3KB of SH-prose comments;
facts/addresses preserved). Route-B live-DM gate UNCHANGED (no DM; DM-root [0x106a68818]=0, MH_* false).

## 1. The problem (measured)

SH319 established the do-init DONE-path dispatcher 0x2206db8 EXECUTES on the plain SESSION-CTOR
bus run (`--v2boot-skip-appstart --v2boot-session-bus`, see my-own earlier bullet): once latches
(once-guard=1, once-slot=0x400000b, registry=12), and the dispatcher forks at

    `ldr x20,[x8,#2664]` @0x206de4   ; x20 = main-id [0x106863a68]
    `bl pthread_self` @0x206de8      ; x0  = current thread's pthread_self
    `cmp x0,x20` @0x206dec
    `b.ne 0x2206e28` @0x206df0       ; TAKEN when main-id != self -> non-main box-build

SH319 measured the box-build branch (mov w0,#0x40 @0x2206e34) firing every run. The MAIN
(binder-dispatch) side — 0x206df4 `ldr x0,[x19,#32]` -> [x0]=vt -> `ldr x1,[x8,#48]`(vt+0x30)
-> epilogue -> `br x1` @0x206e24 = DM-ctor dispatch entry — stayed latent.

## 2. Why a harness-side main-id seed CANNOT flip it (measured)

The `--v2boot` ladder rungs seed main-id [0x106863a68] to the harness thread's pthread_self
(elfjit.rs, SH82/SH126/SH315 pattern) but RESTORE it to orig (0) after each rung. Demonstrating
I added `--v2boot-bus-mainid` (a bus-rung re-seed) AND A/B'd it (ab_sh320_mainbranch.sh):
SEED fires, once latches, registry=12 — yet the SH319-exact box-build marker 0x102206e34 STILL
fires (1 hit) on both arms. Root cause: under JIT_DRIVE_LIFECYCLE the done-path dispatcher runs
on a SPAWNED CLONE WORKER (guest thread 3, pids in SIGABRT dumps) whose pthread_self != the
harness ladder thread, so no harness-side seed can ever match it. Only a JIT block-entry guard
running ON that worker can seed the tag of whichever thread actually executes the dispatcher.

## 3. The fix (crate-side JIT block-entry guard, default-inert)

`routeb_donepath_main_branch_guard` (jit.rs, opt-in `JIT_ROUTEB_DONEPATH_MAIN=1`), registered in
the per-block-entry guard dispatch (fires at real block entry 0x2206db8 — NOT the mid-block
0x206df4, which the SH301 lesson says JIT_DUMP_PC cannot sample). Body: at pc==0x102206db8 it
writes [0x106863a68] = libc::pthread_self() of the CURRENT jit thread (the same id the guest's
pthread_self resolves to), only when the cell != me (idempotent, and it does NOT fight the run's
own real-main-thread id when the main thread happens to run the dispatcher). The research-level
result is a real cross of the thread-match gate: on the crate-guard arm the dispatcher's b.eq is
NOT taken (box-build 0-hit) and the run proceeds toward the MAIN binder-dispatch.

## 4. Measured outcome (real libroblox.so, runs/ab2_sh320_fork_flip.sh)

- BASELINE (guard OFF): box-build marker 0x102206e34 fires (1 hit, SH319 parity).
- CRATE-GUARD ON (JIT_ROUTEB_DONEPATH_MAIN=1): the guard's eprintln logs the seed
  (`[routeb-sh320] seeded main-id=... (this jit thread) at dispatcher 0x2206db8 ...
  -> b.eq NOT taken -> MAIN binder-dispatch`), and the box-build marker 0x102206e34 is
  **0-hit** (2/2 clean A/B runs). Run-variable downstream (spawned-exit abort vs SIGSEGV
  0x1021f3748 fault=0x50 — the SH273 shared lifecycle-notifier live-object wall).
HONEST: does NOT manufacture a DM (DM-root 0, MH_* false). The structural Route-B live-DM gate
is UNCHANGED. What is new + measured: the dispatcher's thread-match fork is provably flippable
to its MAIN (DM-ctor binder-dispatch) side headlessly — SH319 measured the box-build side,
SH320 measures the far side reachable. Next forward work (as the MAIN binder-dispatch now
runs): the vt+0x30 br target of [x19+32] must resolve to a live controller — i.e. a real
once-slot/DM-root — which remains the SESSION-CTOR do-init/once-lambda line, not a seed.

## 5. Verify

- `cargo test -p arm64jit --lib sh320` (1 passed; arm64jit lib 406/0).
- `cargo test -p arm64jit --example elfjit -- sh320` (the elfjit real-image pin, 1 passed; 145/0).
- `cargo test --workspace` EXIT 0 (586 passed, 0 fail).
- Repro A/B: `runs/ab2_sh320_fork_flip.sh` (baseline vs crate-guard, box-build marker).
- Default-inert (guard is opt-in env only); no production code path altered.

Single-agent, cone suppressed. Workspace green; elfjit.rs under the 1MB hook.