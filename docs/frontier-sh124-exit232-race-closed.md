# Frontier SH124/125 — exit-232 race closed, ladder completes

## Status
The pre-SH124 state: StartApp's jit_run now RETURNS Ok (post-SH123), so the main
thread ran off the end while the detached `--v2boot` ladder thread was still
building the StartLuaAppDM do-init. At the same time the guest's own libc
`exit(232)` (deep in the do-init construction) terminated the whole process via
the resolver-bound host glibc `exit` — NOT a guest `svc` exit_group (JIT_TRACE_SVC
verified no exit syscall). The log cut after the two json-fix clamps fired, before
"ladder done" + the session-advance probe printed. This was STATUS gate (a).

## SH124 — two-part fix (both under JIT_DRIVE_LIFECYCLE)
1. **Join the ladder** (examples/elfjit.rs): the `--v2boot` `std::thread::spawn`
   handle is now captured and, once StartApp's `jit_run` returns, JOINED (bounded
   300s) so main() no longer tears the detached ladder down mid-do-init. No-op
   while StartApp parks (the historical exit-124 timeout path — jit_run never
   returns there and the join is never reached).
2. **Exit-family intercept** (src/resolver.rs): the guest's libc
   `exit`/`_exit`/`_Exit`/`abort`/`exit_group` imports bind to a new
   `host_guest_exit` shim instead of host glibc exit. A SPAWNED-thread exit
   (`gettid() != getpid()`) logs the code and zeroes that thread's saved x30 so
   the run_loop's `s.pc = s.x[30]` lands pc=0 -> jit_run returns Err("outside
   image") and the caller (the ladder's rung loop) continues. A MAIN-thread exit
   still routes to real glibc `exit` (genuine termination preserved). Inert (no
   env) == the historical host-glibc binding; the default path is unchanged.

## SH125 — do-init flags-loaded latch seed (recon deleg_5e2c8480)
Recon disassembled the do-init region [0x102206000,0x102210000). The do-init
entry is guest 0x102206c40. Its flags-loaded getter (guest 0x10220671c, `ldrb
w0,[0x106a683e8]` at file 0x2206738) selects the "live DM" dispatch-object slot
`[x21+8]` when bit0==1, otherwise consumes host-garbage. Seed
`*0x106a683e8 |= 1` before the StartLuaAppDM rung (mirrors SH82/SH122 data-seed;
NO .text patch — a NOP on the getter's tbz would force the guard-init path).

## Verification (real libroblox.so, runs/sh124-run.txt)
EXIT 0; ladder completes end-to-end: nativeGameGlobalInit Ok ->
nativeUpdateAdapterInit Ok -> setTaskSchedulerBM Ok -> StartLuaAppDM Ok(0x3e8) ->
V2StartAppWithParams Ok(0x3e8) -> V1 AppStart__ Ok(0x3e8) -> "ladder done" ->
SH122 session-advance probe (MH_FLAGS_LOADED=ENGINE_INITIALIZED=APP_READY=false,
once-guard[0x106a68410]=0x1) -> "ladder thread joined cleanly". 2 json-fix
clamps, 3 setfix substitutions, SH125 seed + exit intercepts log, type-4 vector
[0x106829ea8]=host-thunk (harness seed), task frame present swap Ok(0x1). No more
exit-232 on the clean runs.

## Honest scope & downstream (recon §3)
The do-init region builds the app-data-model/app-config table [0x106dca000] and
telemetry registry — it does NOT construct GuiObjects / a LuaState. Real
login/home reachability still needs the Lua app-shell (StartLuaAppDM ->
ScriptContext -> DataModel -> scene-list = the standing structural wall) +
NativeHelper session-advance + type-4 [0x106829ea8] install. MH_* milestones
stay false: they are set-only (SH111) and are an output the harness must feed,
not a loop the do-init consumes.

## Run-variable note
The default ladder still shows the documented SH55/64 concurrent-thread flake:
2/3 clean EXIT 0 ladder-done, 1/3 crash at guestpc 0x106240318/0x106240b44 (tid 0,
near the documented site). Pre-SH124 this crash was MASKED by exit-232 killing
the whole process first; now it surfaces on that minority of runs. Both SH124
changes are env-gated; the crash is at rung 0 before any do-init/exit. This is
the real remaining blocker to flipping the opt-in chain default-ON (STATUS gate b).

## Characterization (after SH124, real libroblox.so)
WITHOUT the render pipeline (`--renderinit`/`--renderthunk`/`--renderframe`), the
full SH115-125 opt-in `--v2boot` ladder completes clean 3/3 (EXIT 0 / "ladder
done" / joined) — /tmp/norender-{1,2,3}.txt. The run-variable crash appears ONLY
when the render threads are ALSO driven concurrently with the ladder — i.e. it is
the render-thread-vs-ladder jit_run desync (the SH55/64 class), NOT the
session-construction path. The clean session-construction (ladder alone) is
stable; the concurrent render path is what remains run-variable.

## Regressions
+2 resolver tests: `sh124_exit_intercept_gated_on_drive_lifecycle` (exit family
intercepted only under JIT_DRIVE_LIFECYCLE; non-exit names never) and
`sh124_spawned_thread_is_not_main_thread` (a spawned host thread != main).
Workspace 25/25 + diff_battery 56/56; arm64jit example suite 33/0.