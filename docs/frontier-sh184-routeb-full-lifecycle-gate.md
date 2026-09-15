# SH184 — Route-B re-examination (operator Sep-15 directive), 3-agent fresh cone:
# NativeDM path MIGRATION-GATE confirmed at full-lifecycle level; current-DM holder
# *0x106391908 has NO data consumer (orphaned address-forming getter, ref-level closure);
# type-4 producer vector host-install-only + session-driven swap gate correctness confirmed.

Date: Sep 15, 2026, hermes-worker. Workspace green (374/0 arm64jit + all crates).
No production code change — the two recon-v3 deliverables (type4_frame_thunk self-drive,
JIT_JSON_ZERO_FIX) are already implemented and empirically verified; the fresh cone this
cycle answers the operator's 'return to Route B / re-examine the wall' directive at a
stronger, partly-corrected evidentiary level. Commit <SH184COMMIT>.

## What the 3-agent fresh Route-B cone converged on (all READ-ONLY, authoritative)

### 1. NativeDataModelManager full lifecycle = MIGRATION-GATE (task-0, deleg_4350aec4)
Address correction (the given 0x102bd1d68 was the tail of bootstrapTheApp_'s dispatcher,
NOT continueAfterFlagsLoaded_):
  method                file      guest
  initialize_           0x2bd1948 0x102bd1948
  getFlagsFromEngine_   0x2bd1a84 0x102bd1a84
  bootstrapTheApp_      0x2bd1be0 0x102bd1be0  (state dispatcher; states 3/5/9, state4=idle wait)
  initEngine_           0x2bd1dc0 0x102bd1dc0
  continueAfterFlagsLoaded_ 0x2bd3bb0 0x102bd3bb0
  initializeLuaApp_     0x2bd21d4 0x102bd21d4
  startLuaApp_          0x2bd2504 0x102bd2504
  continueAfterLuaAppStarted_ 0x2bd6f68 0x102bd6f68
  initializeLuaApp_ listen  0x2bd6678 0x102bd6678
  messageBus exp-launch cb  0x2bd76e8 0x102bd76e8
Chain: network flags fetch (never synchronous headless) -> continueAfterFlagsLoaded_(
REAL fetched payload string, moved to x20 — so a host flags-sentinel alone cannot invoke it)
-> live-only init: JNIAppLifecycleNativeAdapter_setActive (0x102bd3c10), GL/client-settings
(227be54/227cb24/2281234), initAppShellReporter (21f5408), nativeAppBridgeAppStart (2362e98)
-> initializeLuaApp_ (configures then LISTENS for inbound messageBus 'experience-launch
request', does NOT construct DM) -> DM/place load via dataModelBindings_onGameLoaded(live
binder) + dataModelLifeCycle_onAppLuaWillStart. initEngine_ hard-asserts
'*** FATAL: initEngine_: Engine settings is null' + needs nativeActivity_onEngineSettingsReceived
(window/GL surface APP_CMD_INIT_WINDOW). NO single cell write short-circuits any stage.
VERDICT: MIGRATION-GATE, independently re-confirmed at full-lifecycle this cycle.

### 2. current-DM holder *0x106391908 = NO DATA CONSUMER, ref-level closure (task-1)
Exhaustive decoder (all PC-relative classes + GOT/RELATIVE relocs + movz/movk + data ptrs,
register-copy tracked, branch-reset): *0x106391908 has exactly ONE static reference —
an orphaned 3-instruction getter 0x2dbcc10 (adrp 6391000/add #0x908/ret) that forms the
address but NEVER dereferences the stored value. 0x1063918f8: ZERO references. The getter
has no `bl` callers, no address-taken entry, is unexported (sits inside exported
Java_..._nativePreloadFlagOverrides range but is itself dead). Live session: also NO consumer
(joint/registry code never reads *0x6391908). -> the SH181/182 manufacture lever plants a
genuine-vptr DM into a slot NOTHING reads. This FALSIFIES the prior 'join reads the holder'
premise at the reference level (SH172/178 attributed registry consumers to session gating;
that attribution is refuted — there is no reader at all). Manufacture lever = inert into an
inert slot; its headless value was already BOTH latent (SH181) and live-ctor-dispatching
(SH182) but is now closed even weaker than believed: not 'fires only on live session' but
'fires into a slot never dereferenced'. Do NOT re-derive. (Both task-0 and task-2 independently
corroborated: holder page 0x6391000 zero data consumers.)

### 3. type-4 producer vector host-install-only + session-driven swap gate (task-2)
0x102829ea8 (.bss+0x28) confirmed host-install-only: whole-ELF streamed scan = exactly ONE
access, the dispatcher w4==4 arm (2853784 adrp 6829000 / 2853788 ldr x3,[x8,#3752] /
28537b8 br x3). Zero other readers, zero guest writers, zero relocs. Engine producer
0x10285682c ABI reversed: leaf producer(this=x0, node=x1, flags=w2=[node+32]&1, aux=x3=
[node+32]&~1); optional this->[24] pre-hook bit0=skip; per-CPU deque this->[8]+(getcpu()&0xf)
*0x4a140, head@slot+0x10, tagged CAS. SELF-SUSTAINING LOOP CONFIRMED: producer pushes the same
intrusive deque the drain pop-loop 0x102856e40 drains; drain dispatches frame->[40] (blr) w4 type
codes; w4==4 -> dispatcher reads seeded vector -> br host thunk -> thunk can re-push via producer
= closed loop. Real futex WAVE_PRIVATE(0x81... actually FUTEX_WAKE_PRIVATE=138,1) syscall at
0x2856744-60 (and WAIT_PRIVATE=0x81 on same word). MH_APP_READY is STRICTLY post-live-DM:
SendAppEventOnAppReady 0x102bb463c is a JVM-exported export (Java_...V2SendAppEventOnAppReady)
with zero internal bl callers, no DM check, runs only when the Android activity lifecycle invokes
it after game load (preceded by sibling ...OnGameLoaded); headless has no Java/bridge event -> the
host NativeHelper.onAppReady (JNI slot 61) NEVER fires -> MH_APP_READY cannot be set headlessly.
VERDICT: the derived `MH_APP_READY && live-DM` session-driven producer swap gate is CORRECT and
provably can't fire headlessly; the harness keeps its inert seeded vector while ungated (a live-DM
proxy is the only reliable gate; the DM holder cannot gate — no consumer). Self-sustaining push
before a live DM = idle-spin livelock burn, so the gate is load-bearing, keep it.

## STANDING (strengthened closure)
- Route-B live DM = MIGRATION GATE at full-lifecycle level (task-0) — NOT seedable.
- Manufacture-lever line CLOSED WEAKER still: holder *0x106391908 has no data consumer at all
  (task-1). Do not re-dispatch carve-out cones on it.
- Type-4 producer host-install-only confirmed; session-swap gate (MH_APP_READY && live-DM)
  correct + strictly post-live-DM (task-2). Harness seed stays inert-but-correct as-is.
- recon-v3 deliverables re-verified this session's baseline (self-drive present markers in
  capture_taskv4_frame.sh sh60 log; JIT_JSON_ZERO_FIX forces len=0 at 0x102355d40 in
  sh122-run-jsonfix.txt). Both default-inert/env-gated; workspace 374/0 + all crates.
- NEXT (unchanged, honest): live-DM session is the single migration gate. The headless
  manufacture/carve-out line is exhausted at stronger-than-ever closure. Any further Route-B
  cone MUST first re-derive whether a subscriber to the DM-created/loaded event
  (dataModelBindings_onGameLoaded / dataModelLifeCycle onAppLuaWillStart) could expose a
  construction — else strictly a GPU-host/real-app-launch migration item with the SH174 capture
  latch as observer.