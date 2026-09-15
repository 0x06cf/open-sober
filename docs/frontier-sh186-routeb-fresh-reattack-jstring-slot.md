# SH186 — Fresh 3-agent Route-B re-attack (operator Sep-15 directive) + slot-170 jstring-path closure

Date: Sep 15, 2026, hermes-worker. Workspace green (arm64jit jni 16/0 + workspace target
374/0 verified at session start). One production-adjacent edit this cycle (a hermetic
regression assertion), plus empirical re-verification of the SH182 manufacture lever.
Doc captures three independent recon answers + the code heuristic.

## Cone (3 fresh Route-B agents, READ-ONLY; the two 429-dropped twos re-batched)

### 1. ExperienceController / live-DM dynamic-trace re-attack (deleg_99108c2b task-0)
Operator's SEP-15 ask was to re-examine whether the declared 'not seedable' wall is
crossable via do-init/do-build completion or a DYNAMIC DM-ctor trace (SH164 harness-trace
artifact) rather than a static seed. Findings (raw-bytes + objdump, fresh disasm):
- `createDataModelForTeleport` guest 0x2e1dc38 (thunk 0x2e1dc2c): ZERO bl callers, zero
  adrp/addr-taken refs, zero data/vtable 8-byte pointer refs whole-file. Never called,
  never address-taken, and it is itself a non-constructor (returns global &0x6398000+0x600 =
  a teleport-guard helper, calls 0x2b4ea48). DEAD leaf.
- `setDataModelToCurrent` guest 0x2dbcc10: getter verified (adrp/add/ret -> &0x106391908),
  ZERO bl callers, ZERO pointer refs. Leaf, unused. (Re-confirms SH184-2.)
- The one reachable DM-touching path = JNI export nativeAppBridgeStartLuaAppDM
  (guest 0x1023efe2c) -> dispatcher 0x2baeeec -> do-init 0x2206c40 -> 0x2206db8/0x221942c
  builds a std::function closure and blrs through a captured vtable; it reads a prebuilt
  appbridge/binder [0x106a68818] + a pre-registered scheduler runnable at
  0x6a68000+1032 (guarded by init flag 0x6a68000+0x410). DM construction is buried inside
  that scheduled app-start; no entry point makes a DM in isolation.
- operator-new hook (0x102a0d9b8 / active 0x1067daaf0) would intercept the make_shared
  IF it ran, but it never executes headlessly (make is gated behind the scheduler/binder
  preconditions). Necessary, not sufficient.
- DECISIVE: a live RBX::DataModel is NOT constructable headlessly via a dynamic trace.
  The only hypothetical is fabricating the binder + scheduler runnable in-memory first
  (engineering boot-strapping, not a dynamic trace), which requires the app-start scheduler
  to run full DM/Lua init = the same difficulty as the static-seed route. Strictly migration.
NOTE: this agent ALSO claimed the prior 'DM vtable family 0x671000 / app-shell ctor
0x1057d6ef4' anchors are FABRICATED (read .data.rel.ro on-disk zero bytes -> 'crypto
S-boxes'; and 0x57d6ef4 appears absent from .text which starts at file 0x1d95980). This
claim is a MISATTRIBUTION, falsified empirically below: .data.rel.ro vtable slots are
RELATIVE-reloc-zero in file BY DESIGN (SH178/179), and the SH182 capture DRIVES guest
0x1057d6ef4 to actually execute. The low-VMA '0x1057d6ef4 is data' framing wrongly maps
guest=file_vaddr+0x100000000 onto the wrong segment base; the harness enters it for real.

### 2. SendAppEventOnAppReady / fabricated-jstring step-2 ABI (deleg_70c84ea5 task-0)
- 0x102bb463c JNI export: parses 4 jstrings via helper 0x21e1fec, discriminator reads the
  4th (x5) string -> 'Home' (0x656d6f48) -> w19=4 at 0x2bb47c4 (also Chat=1, More=5,
  Games=3, AvatarEditor=2). Packs event into a 0x58-byte heap obj (operator new 0x58),
  dispatch 0x2baeeec with x0=&stack_event, w1=0.
- NO success token: JNI method is void (0x2bb4a08 ret, no x0), dispatch 0x2baeeec is void
  (0x2baef94). 0x3e8 nowhere in the path. Dispatch reads governor 0x683d000+0x10/+8,
  branches on w20 bit0 -> w1=0 -> calls 0x2206c40 (x0=global,x1=event,x2=0).
- 0x2206c40 -> 0x2206db8 = main-thread marshalled event delivery (compares pthread_self vs
  0x6863000+2664 stash; off-main posts continuation + returns; on-main derefs [event+32] and
  MessageBus_getLastRaw 0x2baaa60 or vtable slot [x8,#48]). Touches message-bus/governor only
  to POST/DELIVER the AppReady event; never reads back a DM/registry holder or returns a
  gate-able status.
- jstring helper 0x21e1fec: only JNIEnv derefs are vtable slot 169 (GetStringUTFChars,
  offset 1352) AND slot 170 (ReleaseStringUTFChars, offset 1360). Builds RBX copy via
  0x1d9d074 (strlen+alloc+memmove, NUL-terminated). **CRITICAL PATH**: the fabricated-jstring
  path derefs BOTH slot 169 AND slot 170 — 170 (ReleaseStringUTFChars) must also be a
  non-null stub or the engine invokes a garbage vtable slot.
- DECISIVE: SendAppEventOnAppReady is purely telemetry / one-way event injection, NOT a
  load-bearing OK gate (both it and dispatch are void; no 0x3e8). It IS driveable headlessly
  clean (fabricated NUL-terminated 'Home' jstring -> w19=4 -> dispatch) provided BOTH slots
  169 and 170 are stubbed; cannot gate a session advance. Route B gains only 'calls succeed
  cleanly', no observable DM advance.

### 3. Type-4 producer handoff + R1 synthetic-CoreScript path (deleg_70c84ea5 task-1)
- R1 (synthetic CoreScript Lua -> ScreenGui) = LAW wall, not judgment: `rbxasset://scripts/
  CoreScripts` at guest 0x10232ed4 has exactly 3 code refs (0x102588508/0x102588530/
  0x104345f88), all URI/string-build feeding content-check/app-bridge-init; NONE loads+executes
  Lua. filesdir obj 0x10726d600 has 10 code refs (writer nativeSetFilesDirectory, crashpad x5,
  fastlog x3) — ZERO joins CoreScripts or any Lua filesystem path. CoreScriptLoader 0x1f1d8ac
  is a mid-constructor (ADRP+STORE vtable ptr), not an entry; ZERO direct bl callers. Even a
  perfectly-seeded Lua cannot self-construct a ScreenGui (ScreenGui lives in PlayerGui/CoreGui,
  both live-DM instances). UniversalApp.rbxm @0x1006c9a40 has 2 refs -> 0x2bf7354 (passive
  string-array seed, not load+execute).
- Type-4 vector [0x6829ea8]: ZERO native store sites, single reader = dispatcher w4==4 arm
  0x2853784 (ldr/cbz/br) -> host-install-only CONFIRMED; plain func-ptr (=0 default). Producer
  ABI confirmed (this,node,w2=[node+32]&1,x3=[node+32]&~1); pump = CAS + FUTEX_WAKE_BITSET_-
  PRIVATE 0x8a @0x2856744 (DM-agnostic, self-sustains mechanically).
- MH_APP_READY gate: `MH_APP_READY && live-DM` conjunction is correct; the FIRST conjunct
  (MH_APP_READY) is judgment (forcible in principle by a host SendAppEventOnAppReady call),
  the SECOND (live-DM) is the irrefutable law wall. "strictly post-live-DM" overstates the
  first term but the conjunction is load-bearing (ungated self-push = idle livelock).

## CODE (this cycle)
arm64jit/src/jni.rs: extend the hermetic `jni_table_has_official_abi_slots_nonnull` test to
also assert slot 170 (ReleaseStringUTFChars) is a non-null host thunk — the slot the
fabricated-jstring / SendAppEventOnAppReady step-2 path (0x21e1fec) derefs in addition to
slot 169. Production table already stubs it (build_jni line 1092 `functions[
RELEASE_STRING_UTF_CHARS] = ok`); the assertion locks it against regression. Verified:
`cargo test -p arm64jit jni_` = 16 passed / 0 failed.

## Empirical re-verification (this box)
Re-ran runs/capture_sh182_dm_ctor_driver.sh (the manufacture-lever repro) fresh:
- `[routeb-dmctor] SH182: seeded stack-canary 0x1067d16f0 ...`
- `[routeb-dmctor] SH182: manufactured-DM app-shell ctor 0x1057d6ef4 DROVE ok ret x0=0x7f.. 
  vt=0x1067162f0 entered AND returned through real code`   <- the lever EXECUTES real relocated code
- `entered region` count = 1 (the app-shell ctor region really ran; SH181 was 0)
- EXIT 124, 0 SIGSEGV/SIGABRT/stack-smash.
This falsifies deleg_99108c2b task-0's 'vtable fabricated / 0x1057d6ef4 absent' claim at the
empirical level: the guest pc 0x1057d6ef4 IS reached and executed by the JIT. The
manufacture lever remains the one real headless Route-B artifact.

## STANDING
- Route-B live DM = MIGRATION GATE, re-confirmed at the strongest level yet by a fresh
  3-agent cone + empirical manufacture-lever run. R1 = LAW wall (no Lua load path nor a DM to
  parent a ScreenGui). SendAppEventOnAppReady = telemetry-only (no 0x3e8 gate), now hermetic-
  safe (both 169+170 stubbed+asserted).
- The manufactured-DM headless line is exhausted at this evidentiary depth. Do NOT re-derive
  holder/DMCONT/live-binder/CreateDataModelForTeleport (authoritative negatives, SH184/185/186).
- NEXT (honest): live-DM session = migration (GPU-host/real-input), SH174 capture latch =
  validated observer. Any new Route-B push MUST first locate a live-DM-construction event
  OUTSIDE the five closed gates, else strictly migration work. Cone kept armed per discipline.