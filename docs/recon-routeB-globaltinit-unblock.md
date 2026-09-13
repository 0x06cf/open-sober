# Recon Route-B — unblock gameGlobalInit + self-constructing UI (deleg_39815cff)

Two read-only recon agents; guest = file vaddr + 0x100000000. Real binary:
/home/hermes-worker/.cache/open-sober/robbox/libroblox.so

## The gameGlobalInit park is a nanosleep poll, and its LATCH is one byte
`nativeGameGlobalInit` (0x102206404) body is short and RETURNS; the observed
"parks indefinitely" is the Main thread spinning in a re-armed **nanosleep** poll
(guest 0x10284d114/0x10284d134 — NOT futex as earlier docs said; w0=0x62=98).
It never leaves because the flags/scheduler gate is not satisfied.

**THE LATCH: byte global `0x72739d4` bit0 MUST = 1** ("flags have been loaded").
- gate read: 0x10224fa20 `ldrb [x8,#2516]` ([0x7273000+0x9d4]); `tbz #0` ->
  0x224fc80 -> fatal throw "Can't initialize the TaskScheduler before flags
  have been loaded" (0x100379b27) at 0x10224fc84.
- it's in .bss (base 0x6829e80) -> defaults 0 -> gate fails exactly at TaskScheduler init.
- engine's own flags-loaded setter writes it: file 0x22474cc, `strb #1,[0x72739d4]`
  at 0x22474e8 (helpers 0x224746c / callers 0x2baf6cc/0x2bbf744; nativeFlags
  apply chain 0x2320cec -> 0x2320f2c). nativeInitializeNativeFlags (0x10232048c)
  is on this path but its RETURN does NOT gate it; the check is the global byte.

## Minimal ordered unblock (ONE jit_run thread — do NOT spawn concurrent
## top-level jit_run entries, the shared JIT block cache SIGABRTs: SH55/SH64)
1. write byte [0x72739d4] = 0x01          (seed the flags gate; optional [0x7273990]=1)
2. nativeInitializeNativeFlags (0x10232048c) with FlagsInterface jobject whose
   GetFlagsCount (JNI slot 1368) returns >=1 + string getters return empty jstrings
3. nativeGameGlobalInit (0x102206404)     -> Main thread leaves the nanosleep poll
4. setTaskSchedulerBackgroundMode(false,"ASMA.start") (0x102bb2380 -> 0x10258aff0)
5. nativeAppBridgeV2InitWithParams(InitParams jobject, osVersion="33")
6. nativeAppBridgeStartLuaAppDM() (0x1023efe2c)
7. [surface] set native-window XID -> nativeAppBridgeV2StartAppWithParams(StartAppParams)
   + UpdateSurfaceAppWithPlatformParams + deliver onFlagsLoaded/onAppReady/etc.

## The NativeHelper callback contract (the actual missing harness surface)
`StartLuaAppDM` boots the Luau VM -> Lua app-shell -> GuiService -> builds UI, but
it advances ONLY when these NativeHelper `gameActivity_*` callbacks fire — and ZERO
are implemented in the Rust harness yet. Wire them into the existing RegisterNatives
registry (jni.rs has lookup/dispatch already). Order load-bearing:
- onFlagsLoaded() -> spinner/loading screen
- onEngineInitialized() -> session may create GuiObjects
- onAppReady() (surface attached) -> app-shell constructs first real login/home GuiObject tree
- onDidLogInReceived(isLoggedIn) -> gates login vs home screen (real session: false -> login)
- onGameLoaded() -> full home

## Self-constructed scene list (Ship-63 ABI already pinned, de-risks the tail)
Engine walks a 0x28-stride scene list at R+0x180/0x188 (one-past-end):
  node+0x08 = render-obj (load-bearing; present walker 0x105b2ed48 blr its vt[+24]
             which must resolve to geometry emitter 0x105b35288)
  node+0x18 = view ptr (must be non-NULL)
For a SELF-populated UI: the Lua session builds real GuiObjects -> SceneGraph ->
render-manager R -> real scene nodes; the HOST then does zero layout/geometry.
Marker: a scene-node count derived from real instances (no host render_scene_base).

## Ranked first-3 steps
1. Release the gameGlobalInit park: seed 0x72739d4.bit0 + drive ladder on one thread
   -> verify gameGlobalInit RETURNS (rung 2 nativeUpdateAdapterInit executes, no NULL-fault).
2. Wire NativeHelper onFlagsLoaded/onEngineInitialized/onAppReady (+onDidLogInReceived)
   as registered shims -> StartLuaAppDM advances the session.
3. Replace the fabricated node+0x08 render-obj with a genuine GuiObject instance
   (real vt[+64] dims + vt[+24] draw -> 0x105b35288) walked + presented by 0x105b2ed48.

Standing (honest): type-4 producer vector [0x106829ea8] remains framework-glue
seeded (Drive stays --taskv4-seed); no in-image GuiObject->scene-list writer exists
until step 2/3 create real instances.