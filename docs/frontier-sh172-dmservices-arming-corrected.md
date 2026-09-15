# SH172 — DataModelServices arming-spec CORRECTION (ABI-decoded, code-grounded)

Author: hermes-worker (autonomous loop) + SH171-cone (deleg_3bdfca3c task-0),
Sep 15 2026. Status: READ-ONLY recon correction; no production code change
(the SH169 arming spec in prior ledger entries was falsified at the ABI level).
Companion ledger: STATUS.md, HANDOFF.md.

## What the SH171-cone corrected (decisive, objectdump-verified)

Prior ledger (SH169, deleg_*fe92d2e1 task-0) claimed the post-migration arming
point for `DataModelServices::setDataModelToCurrent` was the spare std::function
capture word at **guest 0x106391918** (registry entry region
0x1063915a0..0x106392600). The SH171-cone's ABI-level re-read FALSIFIES this:

1. **`setDataModelToCurrent` (guest 0x2dbcc10) is a pure leaf GETTER, not a
   setter.** Verbatim objdump (file 0x2dbcc10-18):
   `adrp x0,6391000 / add x0,x0,#0x908 / ret` -> it returns the address
   **`&0x6391908`** (guest 0x106391908) and has ZERO direct `bl` callers
   (vtable-dispatch-only). Its code path never reads or writes 0x6391918, so
   "arming it via the capture word" is a category error.

2. **The registry std::function receives the DM as an INVOCATION ARGUMENT, never
   from a stored capture word.** Dispatch (guest 0x2dbcd18): x1 = a stack temp
   holding the DM argument, then `blr x21` (the invokable __func). Writing a DM*
   into the +0x18 inline slot cannot make the invokable fire with it.

3. **The correct post-migration arming target is the current-DM holder the
   getter RETURNS: guest 0x106391908** (the singleton slot whose address
   0x2dbcc10 yields). The registry fan-out tails dispatch on
   current-DM != NULL; NULL -> benign no-op stubs (0x101db2cf0 / 0x1021e96f8).
   Region verified mapped+in-LOAD#2 RW data ([0x62dc1c0,0x67d27c0) covers
   file 0x6391908).

4. **Honest limit:** the invokable __func vtable (0x106358d40) and the entry's
   `__f_` (0x6391900) are loader-runtime-built under packed ANDROID_RELA (all
   zero in file) — nothing proves boot constructs the entry. Arming (writing a
   live DM* into 0x106391908 AND letting the fan-out transition NULL->real) is
   a MIGRATION-TIME, in-process, post-migration host-call action. It cannot be
   implemented or verified headlessly; it MUST NOT be done as a static seed
   (hand-clobbering a relocated vt — project rule).

## Impact on the tree

- `crates/` references NONE of these addresses (repo-wide search: zero hits) —
  the SH169 capture latch (jit.rs routeb_dm_alloc_capture) is unchanged and
  confirmed wired + default-inert. No code edit is warranted.
- The captured DM base (when a real session's make_shared<DataModel> runs at
  migration) must be written to guest **0x106391908** (current-DM holder per
  the getter), NOT 0x106391918. This is the correct "arming" contract for the
  future migration-time host-call.

## Standby findings (same cone, consistent)

- **R1 synthetic CoreScript recon-v3 premise corrupted:** the literals ARE all
  present in the binary (`rbxasset://scripts/CoreScripts` @0x232ee7,
  `scripts/CoreScripts` @0x2f72df, `NoCoreScripts` @0x4b546b,
  `DebugDontLoadServerCoreScriptsNoMatterWhat` @0x4bfd3b,
  `rbxasset://models/UniversalApp/UniversalApp.rbxm` @0x6c9a40) — the earlier
  "absent" claim (deleg_dbdc8eb2 / SH169 R1) measured the WRONG premise. BUT the
  request is never issued headlessly (CoreScriptLoader 0x1f1d8ac is constructed
  downstream of a live DM; the only content hook is aassetmanager_open in
  shims.rs which is pull-only and never fires pre-DM). R1 remains dead pre-DM —
  now for the CORRECT reason (reachability, not literal absence).
- **Next-object recon (deleg task-2):** the next un-synthesized object is a live
  RBX::DataModel (inlined make_shared at ExperienceController::join), structural,
  seedable=false. Governor/app-data-model-counter/DM-root all advance but mount
  nothing without a live session.

## Standing (unchanged)

Live-DM = MIGRATION GATE (now ~28 recon angles). SH169 capture latch is the
ready observer for the instant a real session forms. Arming target corrected to
0x106391908.