# SH164 — Recon consolidation: wiring the INSTANT a live RBX::DataModel exists

Date: Sep 15, 2026, hermes-worker. Workspace green (538/0). 3 read-only Route-B
recon agents converged (deleg_632de38c task-0/1/2) on the DM-creator path the SH163
do-init tail cannot reach. No production code changed — this persists the corrected,
addr-pinned frontier so the loop stops re-deriving the same map.

## What the three agents mapped (convergent, authoritative)

### 1. The real DataModel creator is NOT a simple "ExperienceController"
- The primary first-login DM manager is `NativeDataModelManager`
  (`N3RBX22NativeDataModelManager`, heap `make_shared`), with the exact lifecycle of the
  startLuaAppDM path: `getFlagsFromEngine_ -> initEngine_ (StartupController::Stage cb)
  -> continueAfterFlagsLoaded_(std::string) -> initializeLuaApp_ -> startLuaApp_`.
- The first hard gate on `initEngine_` is a NETWORK feature-flag fetch whose completion
  callback is `continueAfterFlagsLoaded_(std::string)` — a data/comms gate (bypassable via
  the harness's identity-shim/flags layer), NOT a .bss seed.
- `UgcExperienceController` (`N3RBX23UgcExperienceControllerE`) / `LuaAppExperienceController`
  derive `BaseExperienceController`; `createDataModelForTeleport` exists but is a
  teleport/downstream branch, NOT the first-login creator.
- **Potential trap:** cone task-1 separately identified `UgcExperienceController::createDataModelForTeleport`
  lambdas at guest 0x102e1dc2c as "the" DM-build node — task-0 attributes primary first-login
  creation to NativeDataModelManager. RECONCILE at implementation time by disassembling BOTH
  callers; do not assume one before tracing which is driven by the governor tail.

### 2. DataModelServices::setDataModelToCurrent — fully mapped registry
- Layout: a contiguous run of `std::function<void(RBX::DataModel*)>` objects in `.data.rel.ro`
  guest 0x1063915a0 … 0x106392600. setDataModelToCurrent entry at guest 0x1063918f8, code ptr
  0x102dbcc10 (elf 0x2dbcc10, adrp 0x6391000+0x908), dispatch vt 0x635eec0.
  Raw-ptr overload at rodata 0x6e7562.
- All registry tails dispatch on **current-DM != NULL**; with NULL they tail into benign
  no-op stubs 0x101db2cf0/0x1021e96f8 (NOT faults) — consistent with SH163.
- `[0x106a68818]` is confirmed the app-bridge obj (0x106a687f8) +0x20 "DM-root" (SH156 seed
  target), a SEPARATE mechanism from DataModelServices singleton state — still not the holder.

### 3. Scene-walker / no-append-site (unchanged, honest)
- Walk sequence: do-init 0x102206df8 -> ctor 0x102207b50 (SH156, once-guard latches) ->
  ctor callees (global/telemetry/counter, benign) -> governor 0x102e9fa84 end-to-end
  (SH157-161) -> startAppWithParams 0x10258c6e4 -> 0x1023f00f8 (init3 NOP SH160) ->
  **createDataModelForTeleport / NativeDataModelManager initEngine_ [THE UNBLOCKED GATE]**
  -> setDataModelToCurrent arms registry -> Post-TTI DataModelPatcher (live-DM gate
  0x2d87b18) -> UniversalApp.rbxm (12MB present) -> CoreScripts -> GuiObjects -> scene nodes
  R+0x180/0x188 (walker 0x105b2ed48) -> emitter 0x105b35288.
- The GuiObject->scene-node append site is STILL not statically derivable (walker 0x105b2ed48
  has 0 direct bl/b xrefs, reached only by indirect dispatch). No shielding change.

## The single next implementable artifact (ranked, NOT a seed)
Step 5 is the only addr-pinned, non-seedable gate. `sizeof(RBX::DataModel)` is not static
(no `__shared_ptr_emplace<DataModel>` typeinfo in the .so; ctor/vtable targets hide inside
the packed ANDROID_RELA reloc stream). The live-trace cannot be produced by a read-only
recon — it needs a HARNESS DYNAMIC TRACE driving the create path. So the next cycle is a
harness-driven trace of the DM ctor (pin sizeof + Instance->WorldRoot->DataModel MI chain)
via the governor tail dispatch, NOT another static seed.

Do NOT re-chase: [0x10683cf38] no-op (SH163), synthetic loose CoreScript (SH163 dead),
DataModelPatch feeding (strictly downstream), do-init ctor callees.

## Files
- docs/frontier-sh164-recon-live-dm-wiring.md (this file)
- **Code change (this session):** `aassetmanager_open` gains a models→ExtraContent
  fallback (shims.rs) so the bare `models/UniversalApp/UniversalApp.rbxm` LocalAssetURI
  is served from `assets/ExtraContent/models/...`. Latently-correct (fires the instant a
  live DM drives DataModelPatcher::apply); pure fallback, models-scoped, never shadows.
  +1 hermetic test `aasset_fallback_re_roots_bare_models_to_extracontent`.
- **Code change (this session):** `routeb_tail_dispatch_capture` block-entry probe
  (jit.rs, env-gated `JIT_ROUTEB_DMTRACE`) fires at governor-tail block entry on every
  run, printing impl/slot/vt/vt[+0x30]+x0/x1/x2/x30 and flagging the DM-creator vt
  family. VERIFIED on a real healthy run (fires at 0x102e9fcc4/0x102e9fdc8,
  impl[+0x408]=0x106a72000 inert DISPATCH -> vt=0x106a72040 -> vt[+0x30]=host ptr, NOT
  DM family — the honest mechanism confirmation). +1 hermetic test
  `sh164_tail_dispatch_capture_reads_slot_and_is_env_gated`.
- Scratch that must NOT be committed (cleanup): /tmp/dmrecon, /tmp/sh163 (extracted .so).
- No production code changed in the recon phase. Workspace 538/0.

## Empirical verification (harness runs, this session) — the DM path is CONFIRMED un-reached
Two healthy real-binary ladder runs (env JIT_ROUTEB_DM_SEED=1 + HASHFIX + JSON_ZERO_FIX +
SETFIX + SH115, EXIT=0, ladder done, joined cleanly, 24 task frames) with JIT_REGION_WATCH:
- Governor-tail region 0x102e9fa80..0x102e9fb28: **1 hit** at pc=0x102e9fa84, and the SH161
  inert-DISPATCH seed fires at 0x102e9fcc4 (impl[+0x408]=0x106a72000). Positive control proves
  the region-watch mechanism works and that the tail executes.
- DM-creator region 0x102bd1a30..0x102bd1d08 (getFlagsFromEngine_/initEngine_): **0 hits**.
- CONCLUSION: the ladder reaches the governor tail but is DIVERTED by the inert DISPATCH
  before ever entering NativeDataModelManager. The real vt[+48] dispatch value is
  loader-relocated/runtime-built (not a static constant), so a simple address substitution is
  impossible; relaxing the guard restores the impl[+0x408]==0 SIGSEGV. A naive
  `--v2boot-dm-creator` rung that jit_runs initEngine_ is also NOT viable (initEngine_ needs a
  real NativeDataModelManager instance + engine-settings sub-object, and
  createDataModelForTeleport's 0x102e1dc2c was disproven as a static thunk). Reaching a live DM
  requires a harness DYNAMIC TRACE driving a real instance through the governor-tail dispatch —
  outside a static-seed reach. This is the standing Route-B structural wall, now empirically
  confirmed rather than inferred.