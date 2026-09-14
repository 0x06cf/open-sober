# SH161 — Route-B: the AppBridgeV2 governor TAIL now executes end-to-end

## Outcome
The governor's tail epilogue — the component-registration + session-start dispatch
block (guest 0x2e9fcc4..0x2e9fe3c) — now COMPLETES instead of SIGSEGV. The
`--v2boot` ladder runs to completion: nativeGameGlobalInit -> nativeUpdateAdapterInit
-> setTaskSchedulerBM -> V2Init -> StartLuaAppDM -> V2StartApp -> V1 AppStart ->
V2UpdateSurface -> **SendAppEventOnAppReady returned Ok** -> `ladder done`, on ONE
jit_run, **EXIT 0, 0 SIGSEGV/SIGABRT, reproducible 2/2**. This crosses the SH160
"governor continuation 0x102e9fcc4" frontier marker. Opt-in (JIT_ROUTEB_DM_SEED=1 +
JIT_ROUTEB_SETFIX=1); default path unregressed (EXIT 124 / 24 real task frames /
0 crashes / SH161 inert).

## Root cause (three read-only recon subagents, deleg_c94a8b2f)
The reported crash guestpc 0x102e9fcc4 (`ldr x22,[x19,#1096]`, x19=live host-heap
impl 0x7ff6...) cannot itself fault at address 0 — the SIGSEGV fault=0x0 with
x0=0x0 is the **NULL deref at the tail's dispatch**: the tail loads
`x0 = [x19,#1032]` = impl[+0x408] at guest 0x2e9fd78/0x2e9fda8 then runs
`ldr x8,[x0]; ldr x8,[x8,#48]; blr x8` (vt[+0x30]) at 0x2e9fd90/0x2e9fdb0.
Under the partial do-init impl is a live host-heap object (runtime address, not
static-seedable) and impl[+0x408]==0 -> `ldr x8,[x0]` faults. SH159d's earlier
patch covered only the governor's MODERN window (0x2e9fb44), not the TAIL, which
re-loads impl[+0x408] fresh and its block (starting 0x2e9fcc4) never sees SH159d's
x0 substitution.

## The two fixes
1. **tail-dispatch guard** (jit.rs `routeb_tail_dispatch_guard`, SH123 pattern,
   gated JIT_ROUTEB_SETFIX): at any block entry into the governor-tail region
   [0x102e9fcc4, 0x102e9fdc8], if `[x19+0x408]` is 0 or sub-image, write the inert
   DISPATCH (0x106a72000, all-leaf vt whose vt[+0x30]=leaf) into the impl slot.
   x19 (impl) is only known at runtime, hence the runtime hook — a static seed
   can't reach it. Idempotent. This makes the tail's `ldr x0,[x19,#1032]` load
   DISPATCH and the `vt[+0x30]` blr resolve benignly.
2. **tail-continuation NOP** (elfjit.rs `routeb_patch_gov_tail_cont`): after the
   dispatch, the continuation (guest 0x2e9fdf4) calls the device-display-handler
   shared_ptr/refcount helper **0x24c3768** with `x0=impl[+0x440]`==NULL under the
   partial do-init (fault `ldr [x0,#320]`, fault=0x140). 0x24c3768 is a
   conditional-release helper; its return is DISCARDED by the caller
   (`mov x0,x19` at 0x2e9fe00 right after). NOP'd the 3-instruction window
   (ldr/mov/bl -> nop,nop,nop), mirroring SH160's init3-gate NOP. Verified
   encodings (0xf9422260/0xaa1403e1/0x97d88e5b) with exact-match guard + block
   drop [0x102e9fdc8,0x102ea3b40].

## Verification (real libroblox.so, llvmpipe)
- Opt-in routeB ladder: EXIT 0, **0 SIGSEGV/SIGABRT** (was 134), full ladder to
  `SendAppEventOnAppReady returned Ok(0x106a72000)` + `ladder done`; repro 2/2.
  Log runs/repro-sh161c.txt / sh161d.txt.
- SH161 seeds fire: `impl[+0x408] -> 0x106a72000` + `tail refcount window -> nop`.
- governor-tail block walked: 0x102e9fcc4 -> ... -> 0x102e9fd90 dispatch (x0 now
  DISPATCH) -> 0x102e9fdc8 -> 0x102e9fe00 (post-NOP).
- Default env-off path: EXIT 124, 24 real task frames, 0 crashes, SH161 inert.

## Standing walls (unchanged)
MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY still false, DM-root has no live DM,
app-data-model count=1: the real GUI/self-constructed UI still sits behind the
structural CoreScripts/content + live-DataModel loader wall (SH131d-class) — the
governor's StartApp session now parameterizes/constructs, but a live DataModel
(needed for the Lua app-shell + rbx-storage.db remember-login) still requires
real content construction, not a seed.

## Next
The governor's app-start session epilogue continues past the NOP'd refcount helper
to 0x2e9fe04 `bl 23c12c0` (x0=x19, w1=2) and the tail teardown 0x2e9fe0c+. The
closest unblocked Route-B work is whatever the post-tail epilogue derefs next
(run it, observe the next structural gate), while keeping the ONE-jit_run ladder
discipline.