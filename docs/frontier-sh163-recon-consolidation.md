# SH163 — Route-B consolidation: do-init path is seed-EXHAUSTED (6-agent cone convergence)

## Date / author
Sep 14, 2026, hermes-worker. Workspace green (538/0). No code change — this commit
persists the corrected, convergent recon state so the loop stops re-chasing dead
seeds. Baseline real-binary ladder re-verified this session: EXIT 124 (outer
timeout 300s on a CPU-busy box), **0 SIGSEGV/SIGABRT**, governor tail + SH161
DISPATCH seed fire, SH158 AppBridgeV2 probe solid (once-guard=1, singleton
0x1063a705e8 vt0x1063a3410, vt[+0x18]=0x102e9fa84 governor).

## What six read-only recon agents converged on (two 3-wide waves)

### 1. The do-init provider gate `[0x10683cf38]` is a NO-OP, not an unblock
Recon task-0 proposed seeding the global-init's provider cell
`ldr x8,[x8,#3896]` at file 0x2dadbb4 (`adrp 683c000`), CLAIMING
guest `0x10683cf98`. **Verified correction: 0x683c000 + 3896(0xF38) = 0x683cf38,
so the guest cell is `0x10683cf38`** (the recon's 0x...cf98 is a 0x60 offset
slip). More importantly the NULL case is NOT a gate: after `0x2212e44` =
`sysconf(_SC_LEVEL2_CACHE_SIZE)` → x20 (huge, ≥3 on any host),
- provider==NULL → `0x2dadc04`: `cmp x20,#3; b.cc 0x2dae0ec` — since x20≥3 the
  b.cc is not taken -> `mov x25,xzr` -> fallback string-build block `0x2dadc10..`
  -> `cbz x25 0x2dadcf8 -> 0x2daddc0` -> loop -> `cbz x25 0x2dadfd8 -> 0x2dae0a4`
  -> `b 0x2dae0ec`.
- So NULL reaches the SAME init continuation `0x2dae0ec` the non-NULL path does.
  Seeding `[0x10683cf38]` is a no-op (or worse, a garbage-object blr). **Do not
  seed it.** Do not seed the ctor body's four callees either (0x2207d6c /
  0x2207df8 / 0x22082d8 / 0x2208354 app-data-model register) — task-0 verified
  they all complete cleanly by design and never construct a DataModel.

### 2. Synthetic loose CoreScript is a DEAD END on this Android build
Recon task-1 (exhaustive ADRP+ADD scan): the literals `rbxasset://scripts/CoreScripts`
(0x232f34) and `scripts/CoreScripts` (0x232d5f, 0x2f7157) have **zero code
references**. The `.so` has no `.rela.dyn` (only `.rela.plt`), so ADRP+ADD is the
only pointer mechanism. A host-placed `files/scripts/CoreScripts/*.lua` is never
read. `LoadCoreScriptsFromPatchOnly` flag (0x53087d) confirms the patch-model
path. **Do not build the R1 synthetic-CoreScript lever.**

### 3. Real content is patch-models, and they are strictly DOWNSTREAM of a live DM
The engine consumes home/login UI via `rbxasset://models/UniversalApp/UniversalApp.rbxm`
(file 0x6c9a40, ref at 0x20ec890, 12MB PRESENT in APK) and the OTA patch
`rbxasset://models/DataModelPatch/DataModelPatch.rbxm` (0x461b3c, ref 0x23a2dc4,
ABSENT). The DataModelPatcher (`DataModelPatchManager::apply`) is Post-TTI and
hard-gates on a LIVE non-expired DataModel: `[sp+160]!=NULL` cache-miss gate
0x2d87b18 ("cache miss too, no content" 0x245964), `DataModel expired` 0x229006,
`deserializeInstance DataModel is null` 0x288090. It runs AFTER a live DM; it can
be fed staged local .rbxm (backup-cache skip-download + signature blake3/N3RBX)
but is **purely downstream** — not the app-shell enabler.

### 4. Session-driven producer is ABI-correct but STRICTLY LATENT
Recon task-3 (verify + decide): engine producer 0x10285682c is a clean 4-arg
lock-free push (x0=deque `[drain+112]&~0x3f`, x1=node, w2=steal, x3=notify); deque
version=active-head `[node+8]`, futex latch `[node+0xc]=[Q]+4`; the drain's own
re-enqueue does the exact `atomic_add([node+8],0x1_0000_0000)` then
FUTEX_WAKE **0x8a** (not 0x81) on `[node+0xc]`. **COHERENT but gate
`MH_APP_READY ∧ live-DM` is structurally unreachable headlessly** — MH_APP_READY
is a host set-only atom only fired by real engine callbacks, and there is no
in-image GuiObject→scene-list writer without the Lua app-shell. Mounting it
ungated self-drives a garbage loop (`strlen`-faults). **This wiring is the end-state,
NOT the next step. Do not make it the next work item.**

### 5. How a real DataModel is actually created — the honest dependency chain
`RBX::DataModel` (mangled N3RBX9DataModel) is created by `ExperienceController
::createDataModelForTeleport` and by AppBridge `startLuaAppDM →
initializeLuaAppWithDataModel`; it is wired as "current" via
`DataModelServices::setDataModelToCurrent(std::shared_ptr<RBX::DataModel>)`
(rodata 0x6cf890); consumers hold `weak_ptr<RBX::DataModel>`. The harness's
`[0x106a68818]` seed is NOT the real holder (zero ADRP/reloc refs; it's a static
string-init region on bss page 0x6a68000). Host-planting a bare `RBX::DataModel`
ctor is NOT minimal: `sizeof(RBX::DataModel)` is unresolved, the ctor chains
Instance→WorldRoot→DataModel with MI layout + vtable writes, and it registers with
DataModelServices + spawns the governor. The recommended recipe (least invasive)
is to drive the real engine creation path, not plant a ctor.

## Dependency chain (single clearest)
```
GlobalInit do-init (0x2206c40) -> match dispatch (0x2206df4, main-id-seeded SH82/122)
  -> app-shell/global-init ctor 0x2207b50 (__call_once completes, once-guard latches, SH156)
  -> governor 0x102e9fa84 (executes end-to-end, SH157-161) -> startAppWithParams (0x258c6e4)
  -> [MISSING: real ExperienceController / initializeLuaAppWithDataModel creating a live DataModel]
  -> DataModelServices::setDataModelToCurrent (live-DM holder)
  -> Post-TTI DataModelPatcher::apply (live-DM gate) -> UniversalApp.rbxm / DataModelPatch.rbxm
  -> ScriptContext runs CoreScripts -> app-shell -> GuiObjects -> scene nodes at R+0x180/0x188
```

## The honest next step (ranked)
1. The common structural unblock is a **real live DataModel** created by the engine's
   own ExperienceController / initializeLuaAppWithDataModel path. There is no
   remaining .bss seed on the do-init path (this commit proves the last candidate,
   `[0x10683cf38]`, is a no-op). Advancing past this needs either:
   (a) a dynamic trace that pins `sizeof(RBX::DataModel)` + drives one real ctor
       through the engine path, wiring DataModelServices, or
   (b) content-delivery wiring (UniversalApp.rbxm deserialization) staged to fire
       the instant a live DM exists.
2. Everything else (session producer, R1 synthetic CoreScript, DataModelPatch
   feeding) is latent behind (1) and must not be implemented first.
3. The next IMPLEMENTABLE seed must be found on the 
   `ExperienceController::createDataModelForTeleport` / `initializeLuaAppWithDataModel`
   path — NOT the do-init tail. All research subagents must stay Route-B-scoped and
   target that path specifically.

## Files
- docs/frontier-sh163-recon-consolidation.md (this file)
- No production code changed. Workspace 538/0.