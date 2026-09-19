# Frontier SH351 — Stage the R1 synthetic CoreScript content path (Route-B marker prerequisite): a hand-authored Luau module + real loader gates so the engine SELF-constructs a GuiObject tree the instant do-init owns a live DataModel

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Additions: `pub fn
stage_r1_core_scripts` (arm64jit/src/jit.rs) + `pub fn fsmap::staging_root` +
`page_writable_rw` guard + 2 hermetic tests (sh351_*) + elffjit opt-in rung
`--v2boot-r1-stage`. Default-inert (rung opt-in). Workspace green (arm64jit lib 418/0 —
416 + 2 new sh351; elffjit example 157/0; cargo test --workspace exit 0).

## Why this is the standing Route-B forward (not a re-run)
SH349 returned the persistence lane (LSM sub-call-whack-a-mole measured UNBOUNDED, 3
fenceposts now incl. SH350). SESSION-CTOR/do-init live-DM remains the structural wall:
MH_* all false, DM-root [0x106a68818]=0 across every run. The operator's content-path
synthesis (deleg_dbfc8eb2) names the ONE deliverable that turns a COMPLETING do-init into
self-constructed UI with zero host layout: hand ONE synthetic CoreScript module whose Source
builds a ScreenGui, staged where the engine's rbxasset://scripts/CoreScripts resolver serves
it, plus the REAL loader gates. This is the exact Route-B marker (engine-authored GuiObject
-> R+0x180/0x188 scene nodes). It is latent-but-correct (fires the instant a live DM drives
the loader), matching SH131b's files-dir seed precedent.

## What landed
1. `stage_r1_core_scripts` (jit.rs): writes a ~20-line Luau module (`ScreenGui` named
   R1HostScreen + a `TextLabel`, under CoreGui) to
   `SOBER_ANDROID_ROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/<Name>.lua`
   for both inferred candidate names **AppShell.lua** and **CoreScripts.lua** (the desktop
   fastflags/name literals are ABSENT from this Android .so, SH-content-path measured — write
   both so whichever the engine first requests resolves). The path mirrors the files-dir
   global (0x10726d600, seeded by --v2boot-set-filesdir) that fsmap re-roots.
2. Real loader gates (content-path synthesis): flags-loaded [0x10672739d4].bit0=1,
   flags-latch [0x106a683e8].bit0=1, governor union-init guards [0x106a63da0]/[0x106a63d70]=0,
   loader settings slot [0x106ba3350]=0. Each guarded by `page_writable_rw` (a /proc/self/maps
   read-write check) so an unmapped/read-only cell (unit test, no image) cannot SIGSEGV.
3. `fsmap::staging_root()`: public alias of active_root so the same override `remap_path`
   honors also drives host staging (testable without SOBER_ANDROID_ROOT).
4. elfjit rung `--v2boot-r1-stage` (after the files-dir rung in the --v2boot block).
5. +2 hermetic tests: sh351_r1_stage_core_scripts_writes_both_candidates (both files land at
   the correct mirror path, body contains ScreenGui/R1HostScreen), sh351_page_writable_rw
   guard (guest .bss cell inert in a unit-test proc; a live heap page reported RW).

## VERIFIED
- cargo test --workspace exit 0; arm64jit lib 418/0 (2 new sh351). sh351 tests both ok.
- Staging path confirmed: fsmap re-roots `/data/user/0/com.roblox.client/files/...` under
  SOBER_ANDROID_ROOT (remap_path candidates table), matching the files-dir global seed.

## Honest (do-not-over-claim)
- The rung is LATENT: the completing ladder still terminates run-variable at the persistence
  lane (0x101d9a528 / pool-pop family) BEFORE reaching the post-ladder rung in the current
  full env — so staging is staged-and-tested, not yet exercised in a live ladder completion.
  This is exactly the SH131b-shaped prerequisite: it arms the content plane so the MOMENT a
  live DM owns a session the engine loads the module and self-constructs real GuiObjects.
- It does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED
  (DM-root 0, MH_* false). SH174 capture-latch stays the single forward observer; this adds
  the content half of the Route-B marker.

## Verify
Workspace green. Repro: cargo test -p arm64jit --lib sh351 -> 2 ok. In a live ladder:
`--v2boot --v2boot-set-filesdir --v2boot-r1-stage` with SOBER_ANDROID_ROOT armed -> on a
completing run the [r1] STAGED lines + the two files under
`$SOBER_ANDROID_ROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/`.