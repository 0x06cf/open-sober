# Frontier SH133 — host futex now hits the real kernel for REQUEUE/CMP_REQUEUE

## Status
SH133 (reachable-path hardening, direction a per SH131d). Workspace green
(533/0, elfjit example 41/0). Commit: SH133. Default bit-identical on the boot
path (only genuinely-missed futex ops changed behavior); re-verified on the real
binary (EXIT 0, 24 real task frames, 0 crash).

## Why
The guest `futex` syscall (AArch64 nr 98) handler in `jit.rs` forwarded only
WAIT(0) / WAKE(1) / WAIT_BITSET(9) to the real host futex and returned a fake
**0** for everything else. The engine's rebindable threads + bionic
`pthread_cond` broadcast / rwlock path uses **FUTEX_REQUEUE (4)** and
**FUTEX_CMP_REQUEUE (6)** to re-park wakees onto the condvar's own futex.
Returning a fabricated success there stranded the parked waiter forever: the
wakee's wait never resolved because the requeue (the move) never happened. That
is a silent lost-wakeup on exactly the multithreaded plane the real client is
now reaching.

Additionally, a truly-unknown futex op (WAKE_OP/PI/WAIT_REQUEUE_PI/timed) also
returned 0, letting a condvar/rwlock believe its waiter had been parked or
woken when nothing happened.

## Fix
Extracted the futex dispatch from the giant `guest_svc` `match` into a free,
hermetic-testable `fn handle_futex(a: &[u64; 6]) -> libc::c_long` (jit.rs), and
extended coverage to:
- FUTEX_WAIT(0) / FUTEX_WAKE(1) / FUTEX_WAIT_BITSET(9) — unchanged (real kernel).
- **FUTEX_REQUEUE(4)** -> real kernel `futex(uaddr, op, nr_wake, nr_requeue,
  uaddr2)` (moves parked waiters onto `uaddr2` genuinely).
- **FUTEX_CMP_REQUEUE(6)** -> real kernel `futex(uaddr, op, nr_wake,
  nr_requeue, uaddr2, cmp)` (the `*uaddr == cmp` gate happens in the kernel, so
  -EAGAIN is reported truthfully on mismatch, never a fabricated 0).
- Unknown ops return **-ENOSYS** (a real error) instead of a fake 0, so a futex
  user sees the failure rather than assuming its waiter was handled.

The `om = op & 0x7f` mask (handles the PRIVATE/FD/C_PRIVATE flags in the high
bits) is unchanged, so the existing `FUTEX_WAIT_BITSET_PRIVATE` (0x89) barrier
still routes correctly.

## Verification
- 3 new hermetic tests in jit.rs run REAL host futexes through `handle_futex`:
  - `futex_requeue_actually_moves_waiter`: a waiter parks on a source futex;
    `handle_futex(REQUEUE, nreq=1 -> dst)` genuinely moves it (kernel returns
    the moved count), then a `WAKE` on dst releases it. Under the OLD fake-0
    handler this would strand the waiter until its 5s timeout.
  - `futex_cmp_requeue_passes_real_cmp`: mismatched `cmp` returns -EAGAIN (not
    0); matching `cmp` moves the waiter (wake on dst releases it).
  - `futex_unknown_op_returns_enosys_not_zero`: an unhandled op returns < 0.
- `cargo test --workspace`: 533/0 (was 530, +3). elfjit example 41/0.
- Real libroblox.so combined capture re-run: EXIT 0, 24 real task-driven frames,
  0 crashes, ladder done + joined cleanly (boot-path futex WAIT_BITSET
  unaffected).

## Next
Continue reachable-path hardening: the auth-datastore / fs / network shims the
real client touches (SH114 getters latent; rbx-storage.db behind the SH126 wall;
cookie ingress SH129 behind the jar-init wall). Engine's own GLES plane + FMOD
audio (aaudio SH132) remain the active correctness capabilities awaiting a
session advance.