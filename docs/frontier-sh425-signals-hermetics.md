# SH425 — hermetic coverage of the guest signal-delivery core (`signals.rs`)

**Single-agent (cone suppressed).** Recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort,
0 crash). Workspace green (arm64jit lib 490/0 incl. 5 new sh425 hermetics;
`cargo test --workspace` EXIT 0, 671 passed/0 failed; `cargo build --workspace`
OK). Production code ONLY in `crates/arm64jit/src/signals.rs` (off the 1MiB
hooks — jit.rs/elfjit.rs untouched, byte-unchanged); the edit is a pure
`#[cfg(test)]` addition (5 hermetics + a SIG_TEST_LOCK to serialize the
process-global SIG_ACTIONS table against parallel runs), no production path /
guest byte / JIT-hook-default changed.

- `signals.rs` serves the Linux signal contract to translated AArch64 guest
  code: `rt_sigaction` (134) records SIG_DFL/1=SIG_IGN/a guest handler;
  `sigprocmask` (135) applies SIG_BLOCK/UNBLOCK/SETMASK while silently dropping
  unblockable SIGKILL(9)/SIGSTOP(19) bits; `mark_pending`/`take_deliverable_pending`
  hold a blocked signal pending and deliver it once unblocked (ascending
  signal-number order); `dispatch_current_thread` runs an installed handler via
  the aarch64 signal ABI (x0=signo, x1=siginfo*, x2=ucontext*, x30=SIGRET) and
  `sigreturn` restores the interrupted context. This is the surface a real
  client relies on for fatal-path diagnostics (SIGTRAP=5 default-terminate exit
  133) and cross-thread cooperative signal pickup.
- **Genuine gap closed:** `signals.rs` had **ZERO tests** of its own logic — the
  masking/pending/ordering/ABI-contract correctness was only ever exercised via
  SIGTRAP/fault paths on a full boot. SH425 delivers the module's own hermetic
  pair, exactly mirroring the SH423 (fsmap) + SH424 (boot) coverage pattern — a
  runtime-surface module that is load-bearing but had no self-tests.
- **+5 hermetics** (deterministic, no real binary, no env; each takes
  SIG_TEST_LOCK so parallel hermetics never race the global SIG_ACTIONS table):
  1. `sigprocmask_block_unblock_setmask_drops_unblockable_and_osets` — SIG_BLOCK
     from empty → mask set; oset carries prior mask; SIG_UNBLOCK clears; an
     attempt to block SIGKILL/SIGSTOP is silently dropped; SIG_SETMASK replaces
     wholesale.
  2. `sigprocmask_einval_paths` — sigsetsize<8, how>2, and SIG_SETMASK-on-NULL-set
     are all -EINVAL; a NULL-set/NULL-oset SIG_BLOCK query returns 0.
  3. `pending_deliverable_ordering_and_blocked_hold` — pending 10,12 deliver in
     ascending order; a blocked signal stays in pending_mask and is not
     delivered until unblocked.
  4. `sigaction_install_query_roundtrip_and_einval` — out-of-range signal is
     -EINVAL; install handler+flags+mask, query back via oact, byte-exact.
  5. `handler_frame_sigreturn_restores_context` — a real installed handler makes
     `dispatch_current_thread` enter `begin_handler` (x0=signo, x1/x2 aligned
     siginfo/ucontext ptrs, x30=SIGRET, redirect_request queued); `sigreturn`
     restores the full interrupted context (pc resumes after the svc, sp/tpidr/
     nzcv/x-regs exact, x0 forced to syscall-return 0); a stray second sigreturn
     returns false.
- **Honest:** NOT a DM (DM-root [0x106a68818]=0 under the complete SH415
  substrate probe — once-guard bit0=1, once-lambda let to run, AppBridgeV2
  genuine vt 0x1063a3410, yet DM-root stays 0). Route-B live-DM structural gate
  UNCHANGED. This is BUILD-THE-RUNTIME coverage completion in the SEP-18
  direction — hardening the host-side signal substrate the engine runs on top
  of — NOT a re-tread and NOT a DM-seed. recon-v3 deliverables unchanged-green.

Files: `crates/arm64jit/src/signals.rs` (+5 hermetics). Commit (pending).