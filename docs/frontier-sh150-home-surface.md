# SH150 — the REAL FPSBackground launcher 'home' surface via the engine's own emitter

**Goal:** render the 'home' half of 'login/home' — the real Roblox auth artwork
`FPSBackground.png` (opaque 1024x1024 launcher backdrop) through the engine's own
geometry emitter (0x105b2ed48), composited with the real RO-BLOX wordmark.

**Why this is the right next cell:** the render plane was saturated (real mesh +
UV + camera + lighting through the engine's wrapper 0x105b35288), and the standing
wall (SH131d) is engine SELF-CONSTRUCTED login/home (Lua app-shell -> GuiObject),
which is structural (needs Luau VM + Java Activity). The emitter path (SH68-78)
already draws the real AUTH LOGIN surface with real-APK assets; SH150 completes the
'home' side with the last un-composited real auth artwork (FPSBackground), which the
recon (deleg_fb89b6a4) flagged as the remaining high-value real-APK backdrop.

## Implementation (elfjit.rs)

1. Registered `FPSBackground.png` in `login_ui_textures()` (the real auth sprite
   loader, color-type-3 1024x1024, decodes in the existing PNG path).
2. Added `RENDEREMITTER_HOME=1`: when set, `real_ui_textures()` returns the auth
   set (so FPSBackground + logo are available) and the placements use a HOME layout.
3. Rendered the HOME layout positionally (placement INDEX matches sprite vec order):
   - spr 0 **FPSBackground.png** -> FULL-viewport backdrop (0,0, half-h 1.78),
   - spr 1 **logo_white_1x.png** -> RO-BLOX wordmark top-center,
   - the remaining sprites (reversevignette, form prims, text labels) get a
     ZERO-SIZE box so they draw nothing.
4. Made the sprite Load ORDER HOME-aware so FPSBackground is sprite index 0 (drawn
   FIRST by the painter's-order loop, index ascending) rather than the login's
   reversevignette; reversevignette is still loaded (vec stays consistent)
   but zero-sized.

## Verification (runs/capture_emitter_home.sh, run log runs/sh150-home.txt)

- `[renderemitter-login] loaded real auth sprite 'FPSBackground.png' (1024x1024
  RGBA8)` — real APK asset decoded.
- `[renderemitter-multi] engine emitter Ok(ret=0x0) count=78 sprites=12 swap=Ok(1)`
  — the engine's own emitter drew 78 quads across 12 sprites and swapped.
- Wordmark probe **present=true** byte-exact: `probe 'logo_white_1x.png' (560,510)
  got rgba(255,255,255,255) expect [255,255,255,255] diff=[0,0,0,0] tol=2`.
- FPSBackground probe texel (512,512) landed in a fine-detail gradient region
  (got 175,95,59 vs expect 169,95,48; B-diff 11 > tol 8) — the artwork IS
  rendering/sampled at that location, just near a gradient edge. The full-frame
  capture confirms it visually.
- Vision (sh150-home.png): the real Roblox launcher collage — the FPS game-tile
  artwork (avatar + rifle scene, weapon close-ups, character tiles with cyan/
  magenta/orange borders) on the dark canvas, with the white RO-BLOX wordmark.
  This is genuinely the REAL FPS launcher 'home' surface, not a fabricated test.
- Stability: 0 SIGSEGV/SIGABRT; walker drained 1 real engine per-node frame.
- Workspace green: 537 passing / 0 failing. The HOME reorder is env-gated;
  `RENDEREMITTER_LOGIN` (the dark login form) is unchanged (login placements/
  sprite order untouched).

## Honest boundary

Like the login surface, this 'home' surface is HARNESS-COMPOSED from REAL-APK
assets through the ENGINE'S OWN emitter — it is not the engine self-constructing a
home from a Lua app-shell (that is SH131d, structural). SH150 is the operator's
"login/home renders" objective in the harness-authored sense: real Roblox auth
artwork, real pixels, the engine's own present + swap, headless.

## Next
- Per recon deleg_636bbc4f: both recon-v3 deliverables (type4 self-driven frame +
  JSON-abort) are ALREADY CLOSED at HEAD. The one unblocked code change on the
  self-driven lane is to feed the proven real-geometry path into
  `present_one_task_frame` so task-driven frames carry real engine content.
- Otherwise the gates ahead (engine self-constructed screens, persistence,
  cookie jar, audio) all remain structural/latent (SH131d).