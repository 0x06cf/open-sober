# SH182 — Host-DRIVE the manufactured DM through its REAL app-shell ctor (Route-B manufactured-DM line, first headless execution)

Date: Sep 15, 2026, hermes-worker. Workspace green (551/0 baseline; SH182 adds 1 hermetic test). Doc docs/frontier-sh182-dm-ctor-driver.md. Commit <SH182COMMIT>. Repro runs/capture_sh182_dm_ctor_driver.sh.

## TL;DR
SH181 installed a manufactured genuine-vptr RBX::DataModel (vt 0x1067162f0) into the
current-DM holder *(0x106391908). Empirically the DM app-shell ctor [V+0x30]
(guest 0x1057d6ef4) NEVER executed headlessly (region-watch = 0 'entered region'),
because no headless consumer dispatches the current-DM vtable. SH182 closes that
declared "next problem" by HOST-DRIVING the ctor: a default-inert, env-gated guard
fixes the ctor's one fault point (the stack-canary global) and runs the genuine
app-shell ctor on the manufactured DM via the existing NESTED `run_guest_callback`
(x0=manufactured DM, x1=zeroed descriptor). EMPIRICALLY the ctor now EXECUTES —
region-watch shows 1 'entered region' at exactly guest pc 0x1057d6ef4, the guard
logs `DROVE ok ret x0=… vt=0x1067162f0 entered AND returned through real code`,
clean ladder, EXIT 124, 0 SIGSEGV/SIGABRT/stack-smash.

## Cone (deleg_661626bb, READ-ONLY, 2 authoritative agents)
Agent-1 — full line-by-line decode of the app-shell ctor 0x1057d6ef4 ([V+0x30] of the
genuine primary DM vtable 0x1067162f0):
- The ctor reads only TWO things:
  (a) stack-canary global file 0x67d16f0 (guest 0x1067d16f0): VALUE 1 in a bare boot
      -> `ldr x21,[…]`->x21=1 -> `ldr x8,[x21]` at 0x57d6f10 derefs addr 1 -> SEGV.
      The ctor stores it to [x29,#-8] at prologue and re-reads it at the epilogue,
      so ANY stable value auto-passes — a standard canary, NOT a data seed.
  (b) DM field +0x38c (`ldrsw x3,[x19,#908]`) = 4 readable bytes (0 fine) — the only
      DM-object memory the ctor touches (4 bytes in PATH B, 0 in PATH A).
- The `[x1,#8]` deref (which the SH181 doc flagged as "a code-pointer -> faults") is
  NOT a code-pointer and is NOT called: x1 is a descriptor whose +8 holds a
  std::string. If [x1+8]==0 -> PATH A: clean zero-touch no-op + ret (a 0-byte DM
  touch survival proof). If [x1+8] == "ServerRestartScheduled" -> PATH B: runs a real
  init body (component ctor 0x2bc4f64, AppBridgeV2Init 0x238e0bc, post-client-
  settings 0x2286ed0, "placeVersion" vector append 0x22b737c — observable heap
  allocs + global writes). The ctor does NOT build the SceneGraph / render world /
  DM sub-objects — it is a sub-init, not a UI constructor.
- PATH B's SSO-byte layout vs the equality fn 0x2152f30 (which reads +0/+8/+16 =
  LONG form; the w9=0x2c=44 vs 22-char length discrepancy is UNRESOLVED) is NOT yet
  decoded. Do NOT lock an unverified byte model into code — shipped PATH A (the
  safe, honest first deliverable) and flagged PATH B as an empirical follow-up.

Agent-2 — the host-drive API + nesting rule:
- The harness has NO `run_guest_callback_with_args`; the mechanism is the
  `args: [u64; 8]` array of `run_guest_callback(fn_addr, args, tpidr)` (jit.rs:3802).
  `run_guest_callback_on` (jit.rs:3819) sets st.tpidr + st.x[..8] (args[0]=x0,
  args[1]=x1) + fresh aligned stack, then jit_run. Returns Ok(st.x[0]).
- NESTING is sane: same-thread nested run_guest_callback is the documented type4
  frame pattern (type4_frame_thunk calls it 3x nested) because `jit_run` detects
  nesting via IN_JIT_RUN (jit.rs:3037) and keeps the cache warm (no SH55/64 — that
  race is about OTHER threads taking a top-level run). SH151 is only a GL/Mesa
  reentrancy rule (no GLSL compile/GL-object alloc in the nested bridge); this ctor
  is pure engine .text, so it is safe to nest. Default-inert env-guard pattern:
  gate on `JIT_ROUTEB_DM_CTOR_DRIVER`, scope to StartLuaAppDM entry, OnceLock-drive
  (idempotent — the nested run's own entries re-enter the block hook).

## Code (arm64jit jit.rs, default-inert, +1 hermetic test)
- `routeb_dm_ctor_driver_guard` (env `JIT_ROUTEB_DM_CTOR_DRIVER=1`, scoped to
  [0x1023efe2c, 0x1023eff20]): (a) routeb_ensure_writable + seed the stack-canary
  global 0x1067d16f0 to a stable 8-byte word; (b) OnceLock-drive the genuine
  app-shell ctor guest 0x1057d6ef4 via run_guest_callback with
  x0=routeb_manufactured_dm(), x1=routeb_dm_ctor_arg_empty() (zeroed 0x28 descriptor
  -> PATH A empty-string survival no-op). Idempotent; env-off byte-identical.
- `routeb_dm_ctor_arg_empty` — the PATH A x1 descriptor: a leaked zeroed 0x28 buffer
  whose +8..+0x20 is a valid EMPTY libc++ std::string (SSO size 0) so the ctor's
  `ldr x8,[x1,#8]` gate sees 0 -> clean return.
- test `sh182_dm_ctor_arg_builds_sso_and_empty_plus_guard_env_gated` — descriptor is
  a stable non-zero zeroed buffer; env-off guard returns immediately; env-on + wrong
  pc region-gates immediately (no canary seed / no drive).

## Empirics (real libroblox.so, llvmpipe)
`JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_CTOR_DRIVER=1` + the canonical ladder +
`JIT_REGION_WATCH=0x1057d6ef4-0x1057d7100`:
- **[region-watch] entered region 0x1057d6ef4-0x1057d7100 at guest pc=0x1057d6ef4**
  (SH181 was 0 'entered region' — the DM vtable now DISPATCHES headlessly).
- **[routeb-dmctor] seeded stack-canary global 0x1067d16f0 ... for app-shell ctor**
- **[routeb-dmctor] manufactured-DM app-shell ctor 0x1057d6ef4 DROVE ok ret x0=<dm>
  (PATH A survival no-op) — DM <dm> vt=0x1067162f0 entered AND returned through real
  code**
- Ladder clean, EXIT 124 (timeout after completion), 0 SIGSEGV/SIGABRT/stack-smash.
Default env-off unregressed (the SH55/64 concurrent-thread flake is run-variable and
pre-existing — confirmed by a 2nd bare run EXIT 124, 0 crashes; SH182 code fires 0
times env-off).

## Honest scope + next
This is the FIRST headless execution of the engineered manufactured genuine-vptr DM
through its real relocated app-shell ctor — the missing half of SH181's lever. It is
still PATH A (empty-string survival no-op): the ctor RUNS real code but with a
zeroed descriptor it takes the clean no-op + ret, so it does NOT build DM sub-objects
/LICENSE or render. Route B's real self-constructed GuiObjects still need PATH B
(seed a genuine "ServerRestartScheduled" std::string to run the init body — its exact
SSO layout vs equality fn 0x2152f30 is the next decode), and beyond that the real
live-DataModel session (migration). SH182 proves the manufacture lever now DISPATCHES
and returns cleanly headlessly; PATH B is a concrete, grounded next step on the same
line. Standing: live-DM session = migration gate; manufacture lever live-correct.