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

## (C) Animation (real progressed spinner)

The robust loader spinner is conceptually a rotating tween; the emitter path
can redraw each frame with an updated quad (a cheap screen-space rotation of
the arc). That is painter's-order geometry on a per-frame jit_run — the SH67d
multi-frame precedent. Turn RENDEREMITTER_REAL_TEX across the present-walker's
per-frame draw into a visibly rotating spinner = a real loading-screen
animation headlessly. Medium effort, high exhibit value.

## Standing wall (unchanged, out of reach statically)

The engine never self-populates a session: the type-4 producer vector
[0x106829ea8] is external framework glue (no in-code store — SH46/53), and
nativeGameGlobalInit 0x102206404 runs real init then parks (SH54/55), so no
in-image path constructs a GuiObject and appends to the scene list. A true
self-driven login screen requires the Lua app-shell / game-activity path beyond
the park.

## Recommendation

Land (B) default-self-load + fallback (trivial, makes the feature real), then
(A) with the ct=3 decoder extension using a login-representative asset — the
two together remove the last "real asset surface" limitations and let any real
Roblox UI PNG render through the engine. (C) is a satisfying but optional
polish. Do NOT attempt per-node emitter vt[+24] (SH64 desync) or the
self-populated wall (out of reach statically).