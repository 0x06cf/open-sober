# SH69 — next artifact options (real Roblox texture now renders through the engine)

SH69 landed the plumbing: real APK UI texture decoded offline (flate2) and
rasterized through the engine's own emitter path with real alpha + correct
aspect, verified byte-exact. The next unblocked steps, in reachable order:

## (A) Render a larger, login-surface-representative real texture (<1 hr)

The spinner proves the plane; a bigger, more recognizably "login/home" sprite
would strengthen the exhibit. Candidates already extracted:
- `content/textures/ui/InGameMenu/BackgroundGlow@2x.png` (512×512, palette+tRNS
  alpha — exercises the ct=3 palette path the decoder currently rejects).
- `content/textures/ui/Input/JumpButtonRegular@2x.png` (240×240, RGBA).
- `content/textures/StudioSharedUI/alert_error@2x.png` etc.
The only code change needed is the deafult path in `real_ui_texture()` + the
probe pixel coords (imgpix_screen already handles any aspect). If a palette-ct
asset is chosen, extend `decode_png_rgba` to handle color type 3 (PLTE + tRNS)
— a small, well-defined addition. **Extend the decoder to ct=3 regardless** — a
large fraction of real Roblox UI sprites are palette PNGs; doing it now removes
the blocker for any ct=3 asset.

## (B) Default SELF-LOAD (no env) + graceful fallback

Currently `RENDEREMITTER_REAL_TEX=1` is opt-in and a missing/path-wrong PNG
silently falls back to the palette strip (log line shows real_tex=true but the
old strip). Make the real texture the home-layout default and make
real_ui_texture() log a clear warning + return None → palette fallback. Low
risk, keeps the SH68 baseline reproducible.

## (C) Animation (real progressed spinner) — DONE (SH71)

Landed SH71: `RENDEREMITTER_SPIN=1` rotates the real spinner's NDC box per
frame (`frame*15°`), so the fixed texture visibly rotates across the
present-walker frames — a real loading-screen animation through the engine's
own emitter path. Verified by a radial-sweep arc-angle that advances
90→70→50→…→120° across 8 frames (exit 124, 0 crash). A tween/curve could be
added but the rotation proof is complete.

## Standing wall (unchanged, out of reach statically)

The engine never self-populates a session: the type-4 producer vector
[0x106829ea8] is external framework glue (no in-code store — SH46/53), and
nativeGameGlobalInit 0x102206404 runs real init then parks (SH54/55), so no
in-image path constructs a GuiObject and appends to the scene list. A true
self-driven login screen requires the Lua app-shell / game-activity path beyond
the park.

## Recommendation (all of A/B/C landed: SH69 real texture, SH70 ct=3 + fallback warn, SH71 spin animation)

The real-asset surface is now complete: any Roblox UI PNG (ct 0/2/3/4/6) decodes
offline and renders + animates through the engine's own emitter path headlessly.
Remaining reachable increments: render a fuller multi-texture login/home
composite (multiple real sprites in one frame), or drive the per-frame rotation
with a Roblox-style tween curve for a more authentic animation. The
self-populated-session wall (Lua app-shell / nativeGameGlobalInit / type-4
vector) is out of reach statically — per SH68/69/71 honest scope, do NOT attempt
per-node emitter vt[+24] (SH64 desync).