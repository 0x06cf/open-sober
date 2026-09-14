# SH111 — singleton-vtable widening empirically dead-ended; NativeHelper callbacks wired

## Part A: the dispatch-singleton vtable is a dead end (do NOT re-attempt a widening)
The three V2Init/V2Start/V1-AppStart soft-returns ('run_loop: pc 0x48...' host-code
leaks) are vtable dispatches through the lazy-init singletons 0x106829a48 / 0x106829a68
at +0xf8 (site A, blr 0x62517c4), +0x108 (site A2, blr 0x6251aa8), +0x548 (site B,
blr 0x6260948). The committed vtable is 0x60 (12 identity-leaf slots).

THIS CYCLE the "differential widening" (0x60 -> 0x580) was tried TWO ways and BOTH
regress nativeInitializeNativeFlags to SEGV/abort (exit 134) BEFORE StartLuaAppDM:
1. b19b1c2: every slot = identity leaf (a0)  -> caller derefs NULL+0x28.
2. SH111 attempt: [0x60,0x580) = a stable-zeroed 0x40 guest object, gate slots
   +0xf8/+0x108/+0x548 identity, +0x558/+0x568 NULL -> SAME hard crash (only moved).

Root cause (confirmed by running the EXACT same combined --v2boot+render recipe on
PRISTINE HEAD = identical exit 139 from an unrelated `renderinit` thunk
`std::runtime_error: (bad-ptr)`): the widening is not the only variable. But the
narrow 0x60 vtable is the committed baseline that keeps nativeInitializeNativeFlags
clean — the shared vtable serves BOTH nativeInit (which stays clean ONLY because the
0x60 read-past into host bytes soft-returns benignly) AND V2Start (which needs wide
coverage). You cannot widen one without regressing the other. **The correct resolution
for the three soft-returns is a SCOPED patch at the three dispatch blr SITES
(0x62517c4/0x6251aa8/0x6260948) — redirect the `blr` to a fixed guest-resident leaf —
NOT a vtable growth.** Left as frontier-sh111 next gate.

Regression-locked: `sh111_tests::sh111_singleton_vtable_stays_0x60_baseline` asserts
the vtable stays at the narrow 0x60 len (<0xf8), so a future implementer cannot
re-introduce the widening accidentally.

## Part B: NativeHelper gameActivity_* callbacks wired (route-B step 2, first half)
The engine's StartLuaAppDM only ADVANCES its session when the NativeHelper
`gameActivity_*` JNI callbacks fire on the fake Java object. All five are invoked via
GetMethodID(name-string)+CallVoidMethod, and CALL_VOID_METHOD was the shared no-op
`ok` stub (returned 0 without dispatching on the method name), so the callbacks
silently no-op'd and the session never advanced.

New `jni_call_void_method` (jni.rs) dispatches on the method name and records a
per-callback milestone atomic, all returning 0 (void):
- gameActivity_onFlagsLoaded / onFlagsLoaded   -> MH_FLAGS_LOADED
- gameActivity_onEngineInitialized/onEngineInitialized -> MH_ENGINE_INITIALIZED
- gameActivity_onAppReady / onAppReady         -> MH_APP_READY
- gameActivity_onDidLogInReceived (VOID, String arg; NOT jboolean — subagent
  deleg_2e852a9c disasm-corrected)              -> 0, no milestone (login-payload gate)
- gameActivity_onGameLoaded / onGameLoaded     -> MH_GAME_LOADED
Registered at BOTH functions[CALL_VOID_METHOD] (=61) and functions[CALL_STATIC_VOID_METHOD]
(=141). The ADD-ONLY group: handful of atomic statics + getter fns, no JIT coupling.

NEW regression `nativehelper_game_activity_callbacks_dispatch_void_method` drives all
five through the OFFICIAL CALL_VOID_METHOD slot (build_jni + host_call_at) and asserts
each returns 0 (void) + fires its milestone + unrelated void calls are untouched.
arm64jit lib suite 341/0, elfjit examples 23/0.

NEXT (route-B step 2 tail): the milestone atoms are set-only — the boot glue does not
YET poll them to actually advance StartLuaAppDM's session, and the call sites are not
yet reached (the real ladder parks at nativeGameGlobalInit's renderinit thunk
`(bad-ptr)` runtime_error, which contradicts SH108's EXIT-0 claim — re-baseline the
no-render clean ladder this cycle). Wire the atoms into a session-advance, and either
clarify/fix the renderinit `(bad-ptr)` so the ladder runs to StartLuaAppDM Ok again.

## Repro / verification
- Unit: `cargo test -p arm64jit` (341/0 incl. nativehelper), `cargo test -p arm64jit --example elfjit` (23/0).
- Real binary combined recipe (v2boot+render): exit 139 = pre-existing `renderinit` thunk
  `std::runtime_error: (bad-ptr)`, IDENTICAL on pristine HEAD -> NOT a SH111 regression.
- Real binary no-render clean ladder: `timeout 60 env JIT_DRIVE_LIFECYCLE=1 V2BOOT_WARMUP_MS=4500
  JIT_ROUTEB_HASHFIX=1 ./target/debug/examples/elfjit <so> 0x2173ff4 --jni --startapp 0x258b144
  --v2boot --persist-roundtrip --kicker 0x106863af8`.