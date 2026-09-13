# SH73 — the engine's REAL geometry emitter renders a REAL Roblox LOGIN/auth surface

## Result

`RENDEREMITTER_LOGIN=1` makes the engine's real geometry emitter
(`0x105b35288`) draw an actual Roblox **auth/log-in surface** headlessly — the
dark `reversevignette` backdrop + the white Roblox wordmark
(`logo_white_1x.png`), both decoded OFF LINE from the **real APK Auth artwork**
(`ExtraContent/textures/ui/LuaApp/graphic/Auth/`), sampled from a shared
vertical atlas in ONE top-level jit_run, composited with GL_BLEND over the dark
backdrop.

Real libroblox.so, exit 124 (stable idle), zero crash:
```
[elfjit:renderemitter-login] loaded real auth sprite 'reversevignette.png' (1024x1024 RGBA8)
[elfjit:renderemitter-login] loaded real auth sprite 'logo_white_1x.png' (476x88 RGBA8)
[elfjit:renderemitter-multi] sprites=2 atlas 1024x1113 RGBA backdrop+3 quads total_verts=18
[elfjit:renderemitter-multi] probe 'reversevignette.png' (641,359) got rgba(48,50,52,255) expect [47,49,51,255] diff=[1,1,1,0] present=true
[elfjit:renderemitter-multi] probe 'logo_white_1x.png'  (560,485) got rgba(255,255,255,255) expect [255,255,255,255] diff=[0,0,0,0] present=true
[elfjit:renderemitter-multi] engine emitter Ok(ret=0x0) count=18 sprites=2 swap=Ok(1)   x2
present walker Ok(ret=0x1)  x2
EXIT=124
```
**All 4/4 probes present=true**; the wordmark probe is **Byte-EXACT**
(diff=[0,0,0,0], alpha=255 white over any dst). Captured frame
`runs/sh73-emitter-login.png` = recognizable Roblox splash: RO-BLOX wordmark
(with the iconic tilted-square "O" / rhombus-mark) centered upper-middle against
a dark vignette (lighter center glow, darker edges).

## What changed (elfjit.rs)

1. **`real_ui_textures()` now honors absolute paths** (an entry starting with
   `/` loads directly; otherwise relative to the `content/textures/ui` root).
   The real auth assets under `ExtraContent/.../Auth/` leave the original root.
2. **New `login_ui_textures()`** — loads the real Auth surface
   (`reversevignette.png` + `logo_white_1x.png`) behind its own OnceLock.
3. **`real_ui_textures()` returns the login set when `RENDEREMITTER_LOGIN=1`.**
4. **Login placements slice** — reversevignette full-screen backdrop
   (hh=1.78 → full-viewport square), wordmark upper-middle (cy=0.35, hh=0.22);
   the SH72 spinner-transparent-center probe is gated off in login mode (no
   spinner on the auth surface).

The atlas/probe/emit machinery is untouched — SH73 is a data/placement change on
the SH72 function, exactly as the SH72 next-artifact doc recommended (option A:
point the same machinery at real Roblox LOGIN ui sprites).

## Honest scope

This is real login-screen ARTWORK rendered through the engine's own emitter
path headlessly — a decisive visual step toward "the engine renders its own
login screen". But the surface is **harness-authored**: fabricated geometry +
host texture/program/blend, sized/placed by the harness. It is NOT the engine
self-constructing a GuiObject or a login UI from its own session. Standing
structural wall unchanged: type-4 producer vector [0x106829ea8] external-glue
only; nativeGameGlobalInit parks; no in-image path constructs a GuiObject and
appends to the scene list (a true self-driven login screen needs the Lua
app-shell / game-activity session path).

## Repro

`runs/capture_emitter_login.sh` (RENDEREMITTER_LOGIN=1 + RENDEREMITTER_MULTI=1
+ RENDEREMITTER_LAYOUT=home on the full productized boot recipe).
Log `runs/sh73-emitter-login.txt`. Workspace **509/0** (unchanged).

## Next

Options from SH72 next-artifact still open: (B) animate the login composite
(spin/move the wordmark) for a "live" auth surface; or (A') pull an even more
login-complete set (the `Auth/` folder also has `classind_16.png`,
`FPSBackground.png`, `noconnection.png` — a connection-loss chip worth placing
to mimic the offline-login state). Standing wall (real self-populated session)
unchanged.