# SH220 — R1 content-path reachability: empirical 3-way cross-check (answers SH219b's delegated "name the R1 file" with a MEASURED no-op)

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed per operator
Sep-15). Date Sep 16 2026. Workspace green (arm64jit 388/0 + elfjit example 68/0 + all
crates, 0 failures). Repro: `runs/capture_sh220_r1_reachability.sh`. No production code
change (pure measurement of an already-shipped diagnostic).

## WHO (the operator's delegated task this answers)

SH219b shipped a default-inert `JIT_ASSET_TRACE` diagnostic on `aassetmanager_open`
specifically to answer the operator's delegated instruction: *"log the engine's first
rbxasset:// request to name the R1 CoreScript file to match"* (CONTENT PATH SYNTHESIS,
deleg_dbfc8eb2). The open thread: with a synthetic R1 CoreScript module staged at the
filesdir mirror, would the engine's own CoreScript loader (0x101f1d8ac, SH131d/SH186)
fire and pick it up, so the engine self-constructs a GuiObject (the Route-B marker)?

## MEASURED (3 arms, all on the canonical completing ladder, JIT_ASSET_TRACE=1)

JIT_REGION_WATCH = CoreScript loader region [0x101f1d8ac, 0x101f1db00) + rbxasset path
region [0x10232ed4, 0x10232f00) (the SH186 "3 refs, none load Lua" rbxasset literal site).

- **Arm A** (harness only, no staged file, assets NOT mounted): EXIT 124, **0 asset-trace
  lines, 0 CoreScript-loader region hits**, 0 crash.
- **Arm B** (staged synthetic R1 CoreScript at filesdir mirror `/data/user/0/.../files/
  scripts/CoreScripts/CoreScripts.lua` + cache mirror, SOBER_ANDROID_ROOT=/tmp/sh-r1-root):
  EXIT 124, **0 asset-trace lines, 0 region hits**, 0 crash.
- **Arm C** (arm B + REAL extracted assets mounted, SOBER_ASSETS_ROOT=
  `~/.cache/open-sober/android-env/assets`, which contains fonts/shaders/ExtraContent/
  models/UniversalApp/UniversalApp.rbxm — the R2 content): EXIT 124, **0 asset-trace
  lines, 0 region hits**, 0 crash.

All arms also confirmed the standing honest cell state (SH155): once-guard=0x1 → 0x0
(SH126 re-clears), DM-root = the SH156 host-heap seed (Box::leak, vtable 0x10635cce0),
once-slot=0x400000b (strcmp intern, NOT a live DM), mark_b(liveDM)=false.

## VERDICT — R1 content path = live-DM-gated (measured at 3 depths, not assumed)

1. With the diagnostic the operator built for exactly this question, the engine makes
   **ZERO** `assets/` (AAssetManager) and **ZERO** `rbxasset://scripts` requests on the
   completing ladder — even with real assets mounted. There is **no engine-requested R1
   filename to match**, because the loader never runs.
2. The CoreScript loader region [0x101f1d8ac] is never entered in any arm — consistent
   with SH186's mechanism-level law wall (loader mid-ctor instr, 0 bl callers; no live DM
   to parent a ScreenGui). Staging the file changes nothing because no code reads it.
3. Only the **host-driven** (Route-A/recon-v3) render plane executes; the engine's own
   asset/UI/content pipeline — AAsset, rbxasset, CoreScripts, R2 UniversalApp.rbxm — is
   fully inert at boot. This is a NEW measured fact: not only the R1 loader, the *entire
   Android asset surface* is unreached headlessly.

## HONEST BOUNDARY

This does NOT manufacture a DataModel. It converts the R1/R2 content thread from
"assumed reachability-blocked on live DM" (SH203/SH186) into a **measured 3-way negative
with the operator's own diagnostic deployed**. Do-not-re-tread: the common unlock stays a
self-built live DataModel (do-init completion + G1 XID + G2 onAppReady/"Home"), which
remains the Route-B structural gate (reconfirmed SH209/SH218/SH219). The forward hook
unchanged: SH174 capture-latch arming at a real `make_shared<DataModel>`; R1/R2 content
has nothing to load until then.

## STANDING (do-not-re-tread, unchanged)

Route-B live-DM = structural gate. recon-v3 deliverable re-verified green at this HEAD
(24 task-driven frames, 195 node pops, no json abort, 0 crash, EXIT 124). SH174 capture
latch ARMED + DELEGATING (FIRST-call allocs through engine's own hook 0x1021ebaf4).
Persistence fsmap roundtrip byte-exact (session.db on host disk). This cycle closes the
last delegated recon question with a measured result; no new seed is warranted.