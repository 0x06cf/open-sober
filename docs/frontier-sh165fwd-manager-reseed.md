# SH165-fwd — scoped NativeDataModelManager singleton re-seed (manager guard)

Date: Sep 15, 2026, hermes-worker. Workspace green. Commit: f801a34.

## EMPIRICAL VERIFICATION (the deliverable)
`routeb_dm_manager_guard` fires on real fnB entry and the whole ladder benign-completes:
- `[routeb-dmforce] SH165 manager singleton holder 0x102727550 -> fabricated all-leaf-vtable
  manager 0x7f... (vt[+0x30]=write-leaf, +0xf8/+0x108/+0x1f0=leaf, vt[+0x720]==0) at
  pc=0x102bd1b98 -> ... (was host JavaVM* 0xaa0003f31400000c)` — the holder was holding a
  host JavaVM* (confirming the recon), now re-seeded with M.
- `[region-watch] entered region 0x102bd1a30-0x102bd1d08 at guest pc=0x102bd1b98` (real
  engine-init fnB entered) + the SH164 shell (DMSFORCE force) fires.
- **EXIT 0, 0 SIGSEGV/SIGABRT/stack-smashing, ladder done, SendAppEventOnAppReady returned
  Ok(0x3e8)** — the whole engine-init pipeline (fnB -> 0x102bd8ce8 -> manager vt +0xf8/
  +0x108/+0x1f0 -> vt+0x720) now benign-completes instead of faulting.
- Reproducible (capture_sh165fwd.sh; the crash-on-unwritable-page was deterministic, this fix
  removes it).

### The crash the fix had to clear
First attempt found `page_is_mapped(HOLDER)=true` but the guard still SIGSEGV'd writing
0x102727550. Root cause: **the holder page is mapped READ-ONLY** (file-backed .data), NOT
unmapped — so `page_is_mapped` (readable) passed, `routeb_map_guest_page` (which only maps a
genuinely-**absent** page) never fired, and the guard's own `write_unaligned` faulted on
the RO page. (The engine's getter 0x2174c04 reads the holder via ldar; the recon's "unmapped"
premise was wrong for this global — it is present-but-RO.) Fix: `routeb_ensure_writable`
(+ `page_is_writable`), which maps anon RW when the page is genuinely absent, else
`mprotect(PROT_READ|PROT_WRITE)`s the file-backed RO page (private mapping COWs safely), then
seeds. Default-inert (DMFORCE-only), idempotent.

## What shipped
`routeb_dm_manager_guard` (jit.rs, gated `JIT_ROUTEB_DMFORCE=1` — the SAME flag that
forces fnB via SH165's `routeb_dm_force_guard`). It re-seeds the **NativeDataModelManager
singleton holder at guest 0x102727550** with a fabricated all-leaf-vtable manager M,
SCOPED to entry into the fnB engine-init region [0x102bd1a30, 0x102bd1d08].

Why: fnB (0x102bd1b98, real engine-init) does `bl 0x102bd8ce8`, which reads its arg's
+0x18 C-string and delivers it to the manager's vtable slots +0xf8/+0x108/+0x1f0
(continueAfterFlagsLoaded_ family). The manager singleton comes from getter 0x102174c04
-> guest holder **0x102727550**, but JNI_OnLoad (boot entry 0x102173ff4) already `stlr`'d
a host `JavaVM*` into that global — so without a re-seed the getter returns a host
`JNINativeInterface*` and the engine-init's `blr vt[+0xf8]` would dispatch into RAW host
JNI. Recon (deleg_62a86bcd task-0, authoritative) mandated the SCOPED re-seed to avoid
ever blanket-clobbering the JNI-critical global.

Fabricated M: `vt[+0x30]=write-leaf` (`str x0,[x1]; mov w0,#0; ret` — the getter fills its
out-field with `this`), `vt[+0xf8]=vt[+0x108]=vt[+0x1f0]=leaf`, every other slot 0 so
`vt[+0x720]==0` -> the post-FFI continuation 0x102bd9058 soft-returns benignly;
`M[+0]=vt`, `M[+8]=0` (getter tail-helper cbz-cleans). Built once via
`routeb_dm_manager_fabricated` (OnceLock); leaf registered via `register_host_call_auto`
(the proven host-call dispatch mechanism used by the SH147/149 shims). Guard is
idempotent (skips if already == M) and never touches the holder when its page is unmapped
`page_is_mapped` probe — a bare unit test / non-ladder process never faults.

## Hermetic regression
`sh165_dm_manager_guard_is_scoped_leaf_vtable_and_env_gated` (jit.rs): env-off -> holder
untouched; env-on + non-fnB pc (governor tail 0x102e9fcc4) -> holder untouched (scoped);
env-on + fnB region -> holder re-seeded with M, vt[+0x30] is the write-leaf (NEVER
engine-init 0x102bd1b98 -> no recursion), +0xf8/+0x108/+0x1f0 leaves live, vt[+0x720]==0,
M[+8]==0; idempotent on a second call.

## Recon forward (this cycle's cone, all 3 authoritative)
- deleg_0b8468d7 task-0: **fnB is a void init leaf** — after `bl 0x102bd8ce8` (file
  0x2bd1c10) it runs only the stack-canary epilogue + `ret` at 0x102bd1c30, touching NO
  manager vtable slot and never checking the manager return value. The +0xf8/+0x108/+0x1f0
  dispatches are ALL inside 0x102bd8ce8, not in fnB. So with the holder seeded non-NULL +
  all-leaf vtable, 0x102bd8ce8 completes, fnB completes to its ret, and the run
  benign-completes. CRITICAL: if the holder were NULL, the getter returns immediately
  leaving x20=[sp+16]==0 -> 0x102bd8d20 `ldr x8,[x20]` faults BEFORE +0xf8 — the guard's
  non-NULL seed is exactly what prevents this. No +0x720 soft-return gate exists in fnB.
- deleg_0b8468d7 task-1 (live-DM path): HONEST — **no .bss/.data seed produces a live
  DataModel.** getFlagsFromEngine_(0x2bd1b98)->continueAfterFlagsLoaded_(0x2bd1d68)->app-start
  0x2338ef4->app-shell ctor 0x2207b54 is the routable SESSION forward, but
  initializeLuaApp_(0x2bd24b4)/startLuaApp_(0x2bd2668) are thin state-advancers that DON'T
  build a DataModel. The real DataModel is a heap `make_shared` from a factory with ZERO
  direct bl xrefs (0x2e1dc38), reachable only via indirect vtable dispatch; ctor/sizeof
  hidden in packed ANDROID_RELA. A real companion factory must be driven (harness dynamic
  trace) — the standing structural wall.
- deleg_0b8468d7 task-2 (content/offline): rbxasset://models/UniversalApp/UniversalApp.rbxm
  is a LOCAL bundled URI read OFFLINE via AssetReaderAndroidImpl->AAssetManager (fn
  0x21f5f94, AAsset_open@plt 0x21f6944) — the SH164 aasset ExtraContent re-root is the
  correct offline path. But the DataModelPatcher deserializer hard-gates on a LIVE DataModel
  ([sp+160]!=NULL at 0x2d87b18 <- [x0,#136] from a live app context; abort 'DataModel is null'
  0x288090), so staged UniversalApp.rbxm consumption is STRICTLY DOWNSTREAM of a live DM.

## Honest residual
SH165-fwd makes the engine-init fnB path benign-complete (previously the getter returned a
host JavaVM* and vt[+0xf8] would dispatch into raw host JNI). It does NOT yet advance to the
app-shell: the fabricated manager's +0xf8/+0x108/+0x1f0 are benign leaves, so the session
does not reach `continueAfterFlagsLoaded_` (0x2bd1d68)->app-start. Routing the manager's
flag-completion slot (+0x1f0 per recon) to the REAL 0x2bd1d68 is the next session-forward
candidate, but continueAfterFlagsLoaded_ writes a flags blob into M+0x48..0x240 (our M is
0x20 bytes) and needs the app-bridge/app-shell object — the risky structural territory
task-1 flagged; a live DataModel is NOT producible by seeds at all.

## Repro
- `runs/capture_sh165fwd.sh`, log `runs/sh165fwd-manager-reseed.txt` (the deliverable
  probes: the `SH165 manager singleton holder` line fires, fnB region-watch hits, ladder
  done, 0 SIGSEGV/SIGABRT).
- hermetic: `sh165_dm_manager_guard_is_scoped_leaf_vtable_and_env_gated`.

## Next
1. Verify empirically (this script) that the manager guard fires + run stays clean, then
   commit SH165-fwd.
2. Decide whether to route the manager flag-completion slot to real continueAfterFlagsLoaded_
   (0x2bd1d68) with a sized-up M — a session-forward that reaches the app-shell ctor path
   (0x2207b54), vs the honest structural DataModel wall (task-1: seeds cannot produce a live
   DataModel; a real companion factory must be driven).