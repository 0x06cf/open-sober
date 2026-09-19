# Open-Sober run status (hermes-worker)

Updated 2026-09-19, this session: SH350 — CROSSED the SH349+1 terminal (0x101d9a708) with a
BOUNDED single-caller skip (JIT_ROUTEB_LSM_PACK_SKIP). The completing ladder advances deep
into the LSM pool-pop continuation (175 pops) before terminating in the same run-variable
live-object family — a third fencepost of evidence for SH349's "LSM sub-call-whack-a-mole is
UNBOUNDED" verdict. recon-v3 plane stays green (24 frames). Workspace green.

## Current state

- `dev` HEAD: SH350 (default-inert opt-in JIT_ROUTEB_LSM_PACK_SKIP + sh350 hermetic +
  capture_sh350_pack_skip.sh + frontier-sh350). Workspace green (arm64jit lib 416/0; elfjit
  example 157/0 incl. sh350; recon-v3 frame plane re-verified 24 frames/0 crash).
- Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0, MH_* all false.

## What advanced this session

- Verified SH349's crossing holds on the full routeB ladder (SH285 terminal 0x101db1b08
  crossed with JIT_ROUTEB_LSM_APPEND_SKIP; new terminal 0x101d9a708).
- SH350 (measurement + implement): the new terminal 0x101d9a708 was a SINGLE-CALLER name-pack
  helper (verified whole-region BL scan), unlike the unbounded hundreds-of-callers bl 0x1d9d8b0
  family. Implemented `routeb_patch_lsm_pack_skip` (RET 0x101d9a708 -> caller takes benign
  index-0 tst/b.eq path). MEASURED 4/4: pack-skip fires, old terminal GONE, ladder advances to
  175 LSM pool-pops, then terminates run-variable (bad_function_call / 0x101d9a528 /
  0x102b9dee0 / 0x1021e40dc) in the SAME live-object family SH343/346 documented. No Route-B
  advance; DM-root 0, MH_* false. Committed.
- Also measured (confirmed, not implemented): the messageBus "experience-launch" topic used by
  SH347's publishRaw is a Java-side runtime string, not a hardcoded binary literal — so
  SH347's negative (publishRaw Ok but cb never fires) is NOT a topic-string bug; it stands
  confirmed. onAppLuaWillStart is an internal lambda (mangled Z-std-func), not an export.

## Honest status

- Route-B live-DM structural gate UNCHANGED. SH350 crossed one more LSM fencepost but the
  persistence lane is now at THREE depths of measured live-object-wall evidence — SH349's
  "unbounded, no bypass to app-start" verdict stands. The dataModel-bindings receive
  (onAppLuaWillStart) is an internal binder behind the same live-DM migration gate. SH174
  capture-latch stays the single forward hook. MessageBus publish being driveable-clean
  (SH347) remains the small live receive surface.

## Next-forward candidates

1. (PRIMARY, non-persistence Route-B) SH174 capture-latch remains the single forward hook;
   the Route-B live-DM wall is the standing gate. Next concrete path: the R1 synthetic
   CoreScript content path (stage a hand-authored ~20-line Luau ScreenGui module into the
   filesdir the rbxasset://scripts/CoreScripts resolver serves) so that the INSTANT do-init
   owns a live DM, the engine self-constructs real GuiObjects -> R+0x180/0x188 nodes with zero
   host layout. Content-path machinery (fsmap, --v2boot-set-filesdir, JIT_ASSET_TRACE
   rbxasset logging) already exists; the synthetic module + loader gate seeds are the omitted
   piece.
2. onAppLuaWillStart (dataModel-bindings live binder) stays migration-gated; not seedable.
3. Do NOT re-drive LSM sub-call skips — 3 fenceposts of measured evidence it is unbounded.