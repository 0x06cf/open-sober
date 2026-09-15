# Recon SH165-fwd cone — the governor-tail engine-init path is a LOGGING warmer, not the UI path (deleg_7e5b7101)

Date: Sep 15, 2026, hermes-worker. READ-ONLY cone, all 3 subagents authoritative. This
CORRECTS two premises carried in prior handoffs and pins the real routable forward.

## ADDENDUM (same session): DMCONT continuation-routing is LATENT (empirically confirmed)
Implemented `JIT_ROUTEB_DMCONT=1` `routeb_dm_manager_cont` (routable per task-0): vt[+0x1f0]
routed to the REAL continueAfterFlagsLoaded_ (0x102bd1d68), M>=0x260, M+0x40=fabricated
flags-holder F, and seeds app-launched latch 0x683d920=0 + log mask 0x683d8f8=0. Hermetic test
passes; real-binary ladder CLEAN (EXIT 124, 0 SIGSEGV/ABORT, ladder done), seed fires
('continuation-routed manager ... +0x1f0=REAL continueAfterFlagsLoaded_'), BUT
**JIT_REGION_WATCH=0x102bd1d68-0x102bd2490 shows 0 entries** — the real continuation does NOT
execute. Root cause: the pipeline's vt[+0xf8] (network feature-flag fetch) returns via our leaf
without completing, so the engine never synchronously invokes the +0x1f0 (continueAfterFlagsLoaded_)
completion callback. DMCONT is LATENT-BUT-CORRECT (routes the right slot; trigger requires a
synchronous-completing +0xf8 fetch), the SH161b honest class. Consistent with task-1: even if it
ran, it only warms logging / sets base-url -- no self-constructed UI.

## Task-1 correction (DECISIVE): "app-shell ctor 0x102207b50" is a FastLog warmer, NOT a UI builder
- guest 0x102207b50 (file 0x2207b50) is `b 0x2207b54`, a `__cxa_guard` one-time-init
  (guard global guest 0x102a640d70 / file 0x6a640d70). Its guarded body only WARMS FastLog /
  exception-log namespaces (callees 0x2212b74..0x2212d6c are tiny register-stubs; it stows
  fn-ptrs into guests 0x10267cdf28/0x10267ce5f0). It new-allocates NO object and never touches
  DataModel/Lua. `[FLog::NativeDM] initEngine_:` / `continueAfterFlagsLoaded_:` strings at file
  0x244e71/0x244e8f confirm the manager is pure state/flag logging.
- The real exported JNI is `nativeAppBridgeAppStart` at file 0x2338510 (guest 0x102338510); its
  internal `AppStart` (file 0x2338ef4) invokes that guard at 0x2338ff8, sets base-url
  (0x21f47fc), JNIAppLifecycle setActive (0x21f5f80), dispatches virtual hooks. A 0x78 config
  object (new 0x1d96768, vt 0x635d980) is destroyed in-place in the same call.
- **The scene walker 0x105b2ed48 has NO direct `bl` caller in .text** (reached only via a
  function pointer / render-advance path); the app-start/ctor path NEVER produces GuiObjects or
  scene nodes.
- Real login/home UI requires a live DataModel + Luau VM + ScriptContext/CoreScriptLoader
  (file 0x1f1d8ac) + ContentProvider, built by the SEPARATE engine boot (nativeGameGlobalInit /
  service-provider + game-document load), NOT the governor-tail path.

## Task-0: continueAfterFlagsLoaded_ (0x2bd1d68) IS routable — the concrete recipe
AAPCS64 this-call: x0=M (manager), x1/x2 are DEAD (never read). To drive it to the real
`bl 0x2338ef4` (nativeAppBridgeAppStart):
- M must be >= 0x260 bytes real allocation (the harness's current 0x20 shell is ~19x too small;
  every M+0x48..0x240 store overruns). M+0x0/M+0x8 may be NULL (M+0x8 cbz-gated); M+0x48..0x240
  AUTO-FILL from F's blob (empty ok) — only the space must exist.
- **M+0x40 must be a fabricated-but-structural flags-holder F** (>=0x310 zeroed block: F+0x300 =
  zeroed pthread_mutex = PTHREAD_MUTEX_INITIALIZER so lock/unlock succeed; F+0xf0 = zeroed
  inline std::strings -> valid empty SSO; F+0x18 must point to a valid object whose vtable+16 is
  callable to survive past app-start's return at 0x2bd2080-94; allocates a 0x28 closure at
  0x2bd2128).
- **Global 0x683d920 (app-launched latch) MUST be 0**, else `tbnz w8,#0` at 0x2bd1fe8 jumps to
  0x2bd2080 and SKIPS the app-start entirely. (Also seed 0x683d8f8 log/mask=0 to skip logging.)
- nativeAppBridgeAppStart call args (0x2bd203c-58): x0=1, x1=&baseUrl string (copied from
  M+0x48), x2=&str2 string (from M+0x78), x3=&empty string, w4=0.
- **ROUTABLE, NOT a dead-end** — routing the manager vt+0x1f0 slot to the REAL guest 0x102bd1d68
  (instead of a leaf) is the correct minimal forward. Caveat: the routine does not exit cleanly
  with a bare F (post-app-start needs *(*F+0x18)+16 callable + the 0x28 closure), and the
  app-start's own virtual hooks / game-document load are downstream.

## Task-2: no static headless seed yields a live DataModel (STRICTLY BLOCKED as pure static harness)
- createDataModelForTeleport: getter 0x2e1dc2c is a vtable-gated ICF resolver (ZERO direct bl);
  real fn 0x2e1dc38 is an INIT-OVER-PREALLOCATED-MEMORY ctor (never mallocs/operator-new).
- **sizeof(RBX::DataModel) NOT derivable statically** — stripped, packed ANDROID_RELA typeinfo,
  no RTTI size table; the make_shared allocation size lives only as an inlined immediate inside
  the engine's own factory/ExperienceController instantiation sites (vtable-gated).
- setDataModelToCurrent IS trivially harness-callable once you hold a live DM*: DataModelServices
  singleton getter 0x2dbcc10 (instance file 0x35d5908), fan-out std::function<void(DataModel*)>
  dispatch (0x2dbcc98) vs vt 0x635eec0 (registry guests 0x1063915a0..0x106392600).
- VERDICT: realistic path to a live DM is engine-internal synthesis — jit-run the engine IN-PROCESS
  so ITS OWN make_shared/ExperienceController path allocates a genuine DataModel on the real heap,
  capture the returned shared_ptr at RUNTIME (the only place sizeof is resolved), then feed it to
  setDataModelToCurrent. That is a live-engine probe/synthesis, not a harness-seed sequence.

## What this means for Route B
The governor-tail engine-init path (fnB -> continueAfterFlagsLoaded_ -> nativeAppBridgeAppStart)
that SH165 + SH165-fwd drive is a LOGGING/flag warmer — routing continueAfterFlagsLoaded_ to run
will execute real engine code end-to-end (a coherence milestone, base-url set, lifecycle fired)
but yields NO self-constructed GuiObjects. Real self-constructed UI remains strictly behind a LIVE
DataModel (engine-internal make_shared, headlessly only via an in-process jit-run probe) + Luau VM +
CoreScript/content. That standing structural wall is UNCHANGED and confirmed from a third angle.