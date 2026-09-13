# SH88 — substitute the OTel/pb_defaults registry map for sub-image map/this tags

## Result (wall advance: cleared the OTel FIND dispatch gate; ladder faults one gate deeper)

Read-only recon deleg_ba208bc6 (disasm-verified) root-caused the post-SH87 fault:
after the SH87 `blr x8`->`blr x1` map-family dispatch patch, the ladder still
SIGSEGVs at the FIND op (file 0x29f424c, guest 0x1029f424c) because a CALLER hands
it a static `.data.rel.ro` protobuf FIELD-TAG constant as the map/this argument:

- guest 0x1800064 (file 0x62f5110) is ONE ENTRY of a 16-byte-strided array of
  protobuf descriptor field-tag constants
  (Table B @0x62f5110..260: 0x1800064,65,72,76,66,67,68,69,6e,6a,73,77,6f,6b,6c,79,
   74,78,75,70,71,6d,..., term 0 @0x62f5270; each record [tag,0], tag=0x0180·0x10000|0x64).
- 0x1800064 is NOT a map: it is `< 0x100000000` (image base = 0x100000000, so never a
  pointer/image), and [0x1800064+0x10] is unmapped -> the FIND's `ldp x1,x8,[x19,#16]`
  reads [0x1800064+16] = 0x1800074 -> SIGSEGV fault=0x1800074 (matched exactly).
- Why: the upstream map the FIND should operate on — the OTel/pb_defaults BSS registry
  slots 0x106838368/0x106838378/0x106838380 — is never constructed under the JIT, so
  the registration passes the descriptor TAG constant as `this`.

**Fix (elfjit.rs `routeb_seed_pb_registry_map` + jit.rs `routeb_substitute_map`):**
seed a coherent empty span-hash map once (same shape SH84 builds: +0x00 zeroed 1024x8
bucket array, +0x10=0x1029b4a84 real in-image span hash, +0x18=0, +0x38/0x3c/0x44=0x400,
+0x40=0, +0x48=0x100, +0x58=0, +0x60=0), install it into the three BSS registry slots,
register it with the dispatch hook, and at the FIND op entries (0x1029f424c, 0x1029f4284)
overwrite ANY non-zero sub-image (tag) x0/x19 map candidate with the seeded pointer. A
tag is deterministically sub-image, so substituting is safe and makes the FIND terminate
cleanly (empty map, no match) instead of faulting.

**Verified:** JIT_TRACE + run: SH88 substitution fires 492x; the OTel FIND dispatch
gate (0x1029f4284 blr-into-0x1800064) is GONE. The ladder now faults at the NEXT gate
(`blr x8` at the INSERT op 0x1029f3f7c, a rehashed/new map whose +0x18 carries fresh
garbage 0x9401a599f941be80, x0=0x40000d5) — i.e. the recon's predicted "watch the
rehash/erase path next" has surfaced. Workspace **515/0** (new regression
`routeb_hashfix_substitutes_subimage_map_candidate`). Product path unregressed (exit
124, persist byte-exact, 0 crash; patch + seed are --v2boot/JIT_ROUTEB_HASHFIX-gated).

## New regression

`routeb_hashfix_substitutes_subimage_map_candidate` (jit.rs) — pins the FIND entries,
the tag constant, and the substitution predicate (sub-image -> seeded in-image-hash map).

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the `SH88 seeded coherent
empty pb_defaults registry map` + `SH88 substituted` lines, and the ladder faulting at
the INSERT-op gate (0x1029f3f7c blr x8 through a fresh +0x18) instead of the FIND gate.

## Next (ranked)

1. The INSERT op at 0x1029f3f7c now blr's through a RECENTLY-REHASHED map's +0x18
   holding fresh garbage (0x9401a599f941be80) — the +0x18 non-image repair fires at
   block entry, but the rehash writes +0x18 mid-block, after the hook. Determine whether
   forcing `blr x8`->`blr x1` at the INSERT dispatch (file 0x29f3f78, like SH87 did for
   FIND) is safe here too (it should be — same family), or extend the +0x18
   non-image zeroing to cover the map-creation/rehash writers.
2. get nativeGameGlobalInit to RETURN so rung 2 nativeUpdateAdapterInit (0x10221c3ec)
   runs -> check rungs 2-6 install the type-4 producer vector [0x106829ea8].
3. Wire NativeHelper callbacks (onFlagsLoaded -> onEngineInitialized -> onAppReady ->
   onDidLogInReceived -> onGameLoaded) so StartLuaAppDM advances the session.
Standing structural wall (real self-constructed session) unchanged.