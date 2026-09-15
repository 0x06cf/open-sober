# SH178 — Route-B recon CORRECTION: the "ExperienceController::join make_shared<DataModel>" premise is MISATTRIBUTED (reloc-level proof; live-DM = law-level migration wall)

Date: Sep 15, 2026, hermes-worker. Workspace green (550/0). No production code change — this is a recon consolidation + premise correction that re-centers Route B on the TRUE addresses.

## Why this matters
The operator's Sep-15 directive returned the loop to Route B and specifically:
"re-attack the live-DataModel construction with a FRESH recon cone aimed at the
ExperienceController / initializeLuaAppWithDataModel / DataModelServices::setDataModelToCurrent
path, re-examining whether the declared 'not seedable' wall can be crossed via the
do-init/do-build completion or a dynamic DM-ctor trace (SH164's harness-trace artifact)
rather than a static seed. The parked 'not seedable' verdict was a JUDGMENT, not a law."

This cone (3 READ-ONLY Route-B-scoped subagents; 2 authoritative rounds plus an
independent packed-RELA decode on the main loop) DID re-derive it from scratch — and the
answer is a CORRECTION, not a tautology: the specific addresses the prior cones chased
for "the only make_shared<DataModel> is inlined at ExperienceController::join" are
mislabelled. The wall is real but for the CORRECT reason.

## The correction (decisive)
Prior cones (SH163/166/169 + many) cited these as the "ExperienceController::join inlined
make_shared<DataModel> / RTApp GetOrCreate" region:
- file 0x2206c40 / 0x2206d74  ->  WRONG. Disassembly of [0x2206c40, 0x2207600) (615 insns)
  shows this is the **GlobalInit do-init** (tail of exported JNI nativeGameGlobalInit): it
  consumes once-guard `ldar w9,[guest 0x106a68410]` -> builds two rodata addrs -> `bl 0x2173b3c`
  -> stores result at guest [0x106a68408]. The only operator-news are 0x28/0x40/0x100/0x1000
  (string/vector/dispatch-object sizes). **No DataModel-size allocation exists here.**
- file 0x2173b3c  ->  WRONG. It is a **string-intern GetOrCreate** (prologue `bl strcmp`,
  hash-bucket chain 0x2173c24..0x2173d48 with ldar reuse), NOT an RTApp app-registry and
  NOT a DM allocator. Its results land in flag-string cells.
- file 0x2206db8 (the do-init match dispatch: `ldr x0,[x19,#32]; cbz->0x2206ea4 benign, else
  ldr x8,[x0]; ldr x8,[x8,#48]; br x1`) — with [obj+#32]==0 it benign-continues; to take the
  dispatch it needs [obj+#32] = a REAL vtable'd object whose +0x30 is a real ctor — exactly
  the case SH156's JIT_ROUTEB_DM_SEED already drives to the GlobalInit installer -> governor
  dead-head. That ctor is NOT a DM maker.

So: the premise "the only make_shared<DataModel> is inlined at ExperienceController::join,
file 0x2206d74/0x2173b3c" does NOT hold against the binary (confidence 0.95). The genuine
DataModel allocation site was NEVER properly located — it is in a region not yet reached by
any cone.

## The law-level wall (reloc-level proof, main-loop decode)
Independent packed-ANDROID_RELA decode (568,272 relocs) of the DM registry region
[guest 0x1063915a0, 0x106392600):
- **490 relocations populate the region** — it is a large loader-built dispatch table.
- The single reloc writing guest **0x106391908** (= setDataModelToCurrent's getter return,
  SH172's "invokable __func vtable + current-DM holder" entry) is:
  `(r_offset 0x6391908, R_AARCH64_RELATIVE(1027), addend 0x6358d40)` -> at load writes
  **base + 0x6358d40**. The on-disk bytes at 0x6391908 are all-zero; the `0x6358d40`
  addend is loader-resolved into `.data.rel.ro`.
- RTTI: the `RBX::DataModel` typeinfo name string is at file VMA 0xcaca6b
  (`N3RBX9DataModelE`), inside the packed-RELA-populated .data.rel.ro — exactly the
  relocation+session-populated band (not a static rdata string address).

Conclusion: the DM holder/registry is **relocation- + session-populated**. Its semantic
content (a wired consumer std::function + a live current shared_ptr) exists only after a real
session constructs the DM and registers it — circular with the very construction we lack.
Cold, it is loader-zeroed then RELATIVE-filled. **No .bss/.data seed manufactures a live DM;
no dynamic jit_run of the misattributed join reaches one.** This is now LAW-level (relocation
mechanism), not merely a judgment. This is why ~30 prior cones failed and will keep failing
until a real app-launch runs the genuine make_shared<DataModel> (which the already-ready
SH169 JIT_DM_ALLOC_CAPTURE DELEGATE+VALIDATE latch would capture as `[validated]`).

## Honest route-B outcome of this re-attack
- **R1 (synthetic CoreScript):** reconfirmed DEAD on reachability, and this is now the
  *correct* reason: `rbxasset://scripts/CoreScripts` is consumed at exactly 3 sites
  (VMA 0x2588508, 0x2588530, 0x4345f88), all request-URI builders adjacent to StartApp —
  pull-only; the CoreScriptLoader 0x101f1d8ac has **ZERO direct bl callers** (dispatch/
  vt-registered only, reached only inside a live DM's ScriptContext); filesdir 0x10726d600
  never joins a CoreScripts Lua path. No boot path issues an rbxasset request pre-DM.
- **R2 (UniversalApp.rbxm real content):** reconfirmed blocked on the SAME live-DM wall
  ('DataModel expired' 0x229006 / 'deserializeInstance DataModel is null' 0x288090 /
  cache-miss gate [sp+160]!=NULL at 0x2d87b18).
- **Type-4 producer handoff [0x106829ea8]:** latent-only (SH173/175/176/177 proof). Correct
  mechanism; fires only once a genuine node source exists behind a live DM. MH_APP_READY
  stays false headlessly.
- **Present-walker 0x105b2ed48 node-list self-construction:** this is the already-done
  Route-A plane (SH152/153/154). Per operator directive it is NOT to be re-polished at
  Route-B's expense; it has zero DM/Luau dependency (renders whatever is in the R+0x180/0x188
  list) but fabricating those nodes is Route-A, not self-construction.

## Verdict
**Do NOT build the headless dynamic-DM-trace attempt.** It re-treads SH156/164c, targets a
misattributed address, and is expected to fault on the `.data.rel.ro` registry/session state
(SH169 next3 + SH164c anti-verdict + this cone agree). The only forward is a REAL app-launch
session (GPU host / real input / migration) where the engine's own make_shared<DataModel>
runs and the SH169 DELEGATE+VALIDATE latch captures it. If the main loop re-chases this wall,
it must first re-derive where the genuine join inline is from a fresh packed-RELA decode of
the DM vtable/ctor — NOT reuse the 0x2206d74/0x2173b3c addresses (misattributed).