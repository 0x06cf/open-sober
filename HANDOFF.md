# Open Sober — Agent Handoff

## SH346 (Sep 19, 2026, hermes-worker): futex test flake fixed, recon-v3 re-verified green, SH343 full ladder re-measured to 3 run-variable SH285-class app-start-region terminals (no Route-B advance)
Single-agent (cone suppressed). One production-code test-hardening change (arm64jit/src/jit.rs)
+ measurement only on the real binary (elfjit.rs unchanged, 1,048,479 B). Workspace green 594/0.

### Fixed this cycle
- `futex_requeue_actually_moves_waiter` flaked once on a full-workspace parallel run (the
  SH345 load-sensitive class): the single immediate `WAKE` syscall could catch a freshly-
  requeued waiter mid-transition and return 0. The REQUEUE side already spun; mirror that
  as a bounded WAKE-side spin that converges once the waiter is fully on dst and still
  asserts `woken>=1` after the 20s window — the SH133 fake-return-0 regression is NOT
  weakened (a broken handler exhausts the window and fails). Production passthrough
  unchanged. Workspace re-verified 594/0.

### Verified green
- recon-v3 frame artifact confirmed on attempt 1: 24 real task-driven frames (`present
  #0..#23 swap Ok(0x1)`), 196 node pops, 0 crash, 0 json abort, EXIT 124. Both recon-v3
  deliverables (type4_frame_thunk self-drive + JIT_JSON_ZERO_FIX) present + working.

### Re-measured (real libroblox.so, SH343 full-ladder env + KEYFIX, 4 strict-serial starts)
- The app-start factory (nativeAppBridgeAppStart 0x102338510) IS reached past the SH285
  insert-leaf on fresh runs — SH248d/e/f + SH259 assign-sites fire (cookie-jar 0x1021f47fc,
  once-cell 0x102339208, adapter 0x102339018, settings once-guard 0x102339d0c). SH344c's
  "continuation never reaches app-start" referred to the DMCONT +0x1f0 serialize body
  (which dies at SH285 pre-app-start); the full ladder's app-start factory body does run.
- The run then terminates in ONE of THREE run-variable arms, all SH285-class live-object
  walls (no DM-root, MH_* false in every arm):
  - Arm A (2/4): SIGSEGV guestpc=0x101db1b08 fault=0xff..ff — the SH284/285 LSM
    reader/pop read-back wall (0x1d9a15c backward byte-copy into an unconstructed output
    std::string; SH285 verdict stands: cause-not-symptom, do NOT repair-seed).
  - Arm B (1/4): SIGSEGV guestpc=0x10284cfa0 fault=0x0 — activity-lifecycle
    nativeOnDestroyed divergence (SH344b/345 class).
  - Arm C (1/4): guest `std::bad_function_call` propagated to HOST libc++abi (terminating),
    no JIT signal dump. `__cxa_throw` is not a host symbol (guest libc++ statically linked +
    JIT-translated), so the exact empty-std::function site wasn't extractable; it is an
    app-start-region live-object manifestation (empty std::function target a real session
    ctor populates), not a distinct route.

### Honest conclusion
Route-B live-DM structural gate UNCHANGED. No seed produces a live DataModel. The
reconciliation: the app-start factory body runs (a real forward-observation, past the
SH285 insert-leaf), but the LSM lane still terminates in the SH285-class live-object wall
family via run-variable SIGSEGV/bad_function_call arms. No seedable-forward product. SH174
capture-latch stays the single forward hook.

## Next (unchanged, authoritative)
SEP-17 SESSION-CTOR cause-level drive remains the primary forward. Every driven rung
(do-init -> DMCONT +0x1f0 -> app-start factory -> LSM) is measured; the terminal is the
SH285 live-object wall family — cause-not-symptom, not seedable (0x101db1b08 is a guest
std::string whose internal buffer pointer is uninitialized 0xff..ff; seeding it means
manufacturing full invariants, SH248h/SH256 class). Un-driven SESSION-CTOR candidates SH264
flagged (messageBus experience-launch receive, dataModel-bindings live-binder) remain wired
except onAppLuaWillStart (migration-gated class, not re-tread). All research subagents
Route-B-scoped; cone still suppressed.