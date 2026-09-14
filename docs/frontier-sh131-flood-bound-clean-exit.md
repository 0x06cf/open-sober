# SH131 — Combined-run clean exit (bound the post-frame dispatch flood)

## Goal
The SH130 worker-admission gate presented 24 REAL task-driven frames in the
same ladder+render run, but the process always exited **133** (a silent guest
SIGTRAP) instead of a clean 0/124. STATUS.md's standing next gate: *"bound the
post-frame dispatch flood so the combined run exits clean after N frames."*

## Root cause (two independent mechanisms, both fixed)
### 1. The dispatch flood was unbounded
`--deque-node-live`'s injector ran a 400-tick x 50ms = **20s** re-injection
loop, keeping the engine drain dispatching through the seeded type-4 frame thunk
long after the presenter's 24-frame / 2000ms budget finished. PENDING_PRESENTS
grew unbounded (seen 6659→9491→13512 pending across runs; dispatch #16000+ in
the log well after default "presenter drained"). That state alone was flaky.

### 2. The worker SIGTRAP was a RAW HOST signal — invisible to guest dispatch
Even with the flood bounded, a released clone worker hit the engine's fatal
`raise(SIGTRAP)` at the *other* SH121-class sites (SH121 only NOP'ed the
TaskScheduler-ctor one). The guest's `raise` import binds to **host glibc
`raise`**, delivering a REAL host SIGTRAP that terminates the whole process
(128+5=133) — it never re-enters our guest signal dispatch, so a
signals.rs/bound-signal diagnostic could not have caught it (confirmed:
`[signals] SIGTRAP default-terminate` never printed). This is exactly the same
class as SH124's exit-232 (guest libc exit bound to host glibc exit killing the
whole process), but through `raise` instead of `exit`.

## Fix (2 parts, both opt-in via existing env/JIT_DRIVE_LIFECYCLE)
### Part A — flood bound (`combined_frame_capture`, elfjit.rs)
In the combined serialized-capture mode (JIT_SERIALIZE_RENDER=1 + --v2boot +
`--taskv4-seed frame`) the presenter, after draining its frame budget, now:
- sets `TASKFRAME_HALT=1` (new static), and
- nulls the type-4 vector `[0x106829ea8]` back to **0** — the boot-idle no-op
  state the dispatcher's `ldr x3,[x8,#3752]; br x3` returns-through doing
  nothing (the exact pre-SH60 idle).
The injector's 400-tick loop checks `TASKFRAME_HALT` each tick and `break`s on
it, so it stops re-injecting foreign nodes the moment the frame budget is met.
Verified by log: `injector stopped after 2 ticks (post-frame budget)`, no more
dispatch traffic after the bound.

### Part B — `raise` intercept (resolver.rs, mirrors SH124)
New `host_guest_raise` shim + a `sh131_raise_is_intercepted()` gate: under
`JIT_DRIVE_LIFECYCLE` the guest's `raise` import is routed to the shim, which
- a **spawned-thread** `raise(SIGTRAP)` → logs + unwinds just that jit_run
  (zeroes saved x30 → run_loop pc=0 → returns), so the released clone worker's
  fatal no longer kills the process;
- a **main-thread** raise (or any genuinely-delivered signal) → still calls the
  real glibc `raise` (genuine termination preserved).
Inert without the env == historical host-glibc `raise` binding.

## Empirical result (real libroblox.so, runs/capture_sh130.sh)
**Before:** EXIT=133, 21 frames (log cut mid-present at #20, no "presenter
drained", no `[signals]` line — the raw-host-SIGTRAP exit).
**After:** **EXIT=0** on **3/3 sequential runs**, 24 full REAL task-driven
frames each (`present #N swap Ok(0x1)`), 0 SIGSEGV/SIGABRT, order:
`presenter drained: 24` → `SH131 flood bound ... nulled type-4 vector=0` →
`injector stopped after 2 ticks` → `SH131 spawned-thread guest raise(SIGTRAP=5)
— unwinding this jit_run (process survives)` → `StartApp returned — joining --
v2boot ladder thread` → `ladder thread joined cleanly` → **exit 0**.

## Tests / regression
- `sh124_exit_intercept_gated_on_drive_lifecycle` extended to also assert the
  SH131 `raise` gate (same JIT_DRIVE_LIFECYCLE condition, folded into ONE test
  so the two env-mutating probes never race on the global var in parallel —
  a standalone sh131 env test raced with sh124 on the shared env).
- elfjit example 39/0, workspace 522/0.
- Default/product paths (flags/env off) are bit-identical (all new code is
  gated on `combined_frame_capture()` / `JIT_DRIVE_LIFECYCLE`).

## Scope (honest)
This makes the SH130 combined-run artifact **clean-exiting and reproducible**
(24 real task-driven frames in the same run that constructs the ladder/session,
EXIT 0). It does NOT advance the standing structural walls: engine
self-constructed login/home GuiObjects and the cookie-jar/login-persist contract
still sit behind the Lua app-shell + nativeInit 0x10232090c wall; the 24-frame
plane is still the seeded type-4 thunk, not a real session-driven producer.