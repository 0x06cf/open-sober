# SH283 — the engine5 state=5 continuation park is a REAL cross-thread contended mutex (falsifies SH282's "static-init cannot block")

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed), default-inert.

## tl;dr

`--v2boot-session-engine5` (SH280-282) drives the initEngine_ settings-state machine's
state=5 body (0x2bd24b4) headlessly. SH281/282 reached the GlobalInit-reentry
continuation 0x2207118 and measured a clean timeout park (EXIT 124). SH282 **guessed**
the park was a "cant-block zeroed .bss static-init pthread_mutex_t at 0x106863aa0" and
recommended the wrong next step. **This cycle proves that premise WRONG with a JIT_TRACE
register/owner dump and crosses the park for the first time.**

## The measured mechanism (what the park really is)

The state=5 body's config-dispatch (bl 0x2bce0d4, w2=1) tail-branches to the
GlobalInit-reentry 0x275a0c4, which bl's the callee 0x275a23c (`operator_new(0x40)` box),
bl 0x2206f04 (intermediate), then tail-`b 0x2207118` (the continuation). The continuation:

- locks the fixed session-global mutex **0x106863aa0** (`bl 0x2b53a68`)
- enqueues onto the fixed queue object **0x106863a70** (`bl 0x2d9713c`)
- the enqueue, when `[queue+32]==0`, constructs the queue via **0x22071ac**
  (SH174/204 world-build)

A bounded JIT_TRACE run shows the engine5 rung reaches the continuation and the **last block
is `hostcall@pthread_mutex_lock pc=... x0=0x106863aa0` with `[mutex_lock] 0x106863aa0
bionic_word=0x00000002 state=0x2 type=0x0 g_owner_tid=0x3ab673`** — i.e. the mutex is
**LOCKED_CONTENDED (state=2) and already OWNED by a spawned worker thread** (pthread_create
at 0x2d97d70, start routine 0x1022076f8), which never runs to release headlessly, so the
runner futex-waits forever → EXIT 124.

So the park is a **real cross-thread contention** on a mutex a never-scheduled worker holds.
It is NOT a "zeroed static-init lock that can't block" (0x106863aa0 IS in zeroed .bss —
readelf -lW last RW LOAD memsz 0xb5d47c ends 0x7333c3c, filesz ends 0x6829e58 — but the
worker already wrote state=2 into it). SH282's "supply a properly-sized box" / "DEAD END"
was chasing the wrong mechanism.

## The gate (default-inert, opt-in JIT_ROUTEB_ENG5_QMUTEX_FREE=1)

In `bionic_mutex_lock` (resolver.rs), before the bionic CAS fast path: if
`JIT_ROUTEB_ENG5_QMUTEX_FREE` is set AND `m == 0x106863aa0`, force-clear the bionic 16-bit
state word to 0 (steal the contended lock from the never-running worker) so the CAS fast
path acquires it and the continuation proceeds into the enqueue-construct. Exposed as a
pure predicate `sh283_should_steal_session_mutex` (env + address scoped) for the hermetic
test. Never touches any other mutex, nor glibc-formatted / recursive / PI mutexes (only hit
on the NORMAL bionic path, after the glibc-init + !normal guards).

## A/B (real libroblox.so, full SH281/282 seed set)

- **A baseline (gate off):** EXIT 124; region hits stop at the mutex lock
  (0x102b53a68/78); enqueue 0x2d9713c = 0 hits, construct 0x22071ac = 0 hits. Park.
- **B (gate on):** the engine5 rung's **`SH280 state=5 body direct returned Ok(0x0)`** —
  the state=5 body COMPLETES for the first time headlessly; **`[this+16](state)=7`** — the
  settings-state machine advances 5→7 through the config-dispatch→reentry→continuation→
  enqueue→construct chain. **enqueue 0x2d9713c enters (1) and construct 0x1022071ac enters
  (1)** with the last region hit deep inside the enqueue at 0x102d97160.
- Repro 3/3 (runs/sh283-repro-{1,2,3}.txt): every run prints state=5 body returned Ok and
  state=7. (Run 1 subsequently self-drove into app-start via the engine3 continuation and
  died at the SH260 LSM map wall 0x101db1d04 — the parked SEP-15 persistence detour, NOT a
  regression of this gate; run variability 124/134 is the long-standing ladder flake.)

## Honest

- This is cause-not-symptom SESSION-CTOR progress: the engine's OWN settings-state machine
  now self-completes its state=5 body through the reentry continuation, crossing the
  SH281/282 park gate. The state=7 completion is a measured new milestone (was never
  reached before).
- Does NOT manufacture a DataModel (MH_* still false, DM-root[0x106a68818]=0); Route-B
  live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
- The steal is safe because the worker that owns 0x106863aa0 is never scheduled headlessly.
- Corrects SH282's premise (contended-vs-static-init) and its "dead-end" next step.

## Verify

- `cargo build --workspace` EXIT 0; `cargo test --workspace` green; arm64jit lib 400/0
  (+1 sh283); examples 115/0. sh283 filtered = 1 passed.
- Repro: `bash runs/capture_sh283_repro.sh` (3/3 state=7); A/B `bash runs/capture_sh283_ab.sh`.

## Discipline

Default-inert (env-gated), single-agent, no production path edited off (the only change is
the env-gated steal in the bionic mutex path + the scoped predicate + hermetic test),
trimmed for the 1MB pre-commit hook.