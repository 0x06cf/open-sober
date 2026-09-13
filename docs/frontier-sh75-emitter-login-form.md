# SH75 — a COMPLETE Roblox login surface through the engine emitter

## Result

`RENDEREMITTER_LOGIN=1` (with `RENDEREMITTER_MULTI=1`) now draws a FULL login
/auth form headlessly through the engine's real geometry emitter
(`0x105b35288`): the real `reversevignette` backdrop + the real white RO-BLOX
wordmark + the real `noconnection` Wi-Fi-off chip, **plus** synthesized solid
UI prims — a near-white input field and a Roblox-green "Log In" bar — all
sampled from ONE shared vertical atlas in ONE top-level jit_run, GL_BLEND over
the dark backdrop.

Real libroblox.so, exit 124 (stable idle), zero crash:
```
[elfjit:renderemitter-login] loaded real auth sprite 'reversevignette.png' (1024x1024 RGBA8)
[elfjit:renderemitter-login] loaded real auth sprite 'logo_white_1x.png' (476x88 RGBA8)
[elfjit:renderemitter-login] loaded real auth sprite 'noconnection.png' (140x100 RGBA8)
[elfjit:renderemitter-login] synthesized solid auth sprite 'field.png' (256x16 rgba(224,224,230,255))
[elfjit:renderemitter-login] synthesized solid auth sprite 'loginbtn.png' (256x16 rgba(0,158,68,255))
[elfjit:renderemitter-multi] sprites=5 atlas 1024x1245 RGBA backdrop+6 quads total_verts=36
probe 'reversevignette.png' (641,359) got rgba(48,50,52,255) expect [47,49,51,255] present=true
probe 'logo_white_1x.png'  (560,485) got rgba(255,255,255,255) expect [255,255,255,255] present=true  diff=[0,0,0,0]
probe 'noconnection.png'   (640,306) got rgba(255,255,255,255) expect [255,255,255,255] present=true  diff=[0,0,0,0]
probe 'field.png'          (641,197) got rgba(224,224,230,255) expect [224,224,230,255] present=true  diff=[0,0,0,0]
probe 'loginbtn.png'       (641,136) got rgba(0,158,68,255)    expect [0,158,68,255]    present=true  diff=[0,0,0,0]
[elfjit:renderemitter-multi] engine emitter Ok(ret=0x0) count=36 sprites=5 swap=Ok(1)   x2
present walker Ok(ret=0x1)  x2
```
**10/10 probes present=true**; the wordmark, chip, field, and button are all
**Byte-EXACT** (diff=[0,0,0,0], each alpha=255 opaque so over any backdrop);
vignette diff=[1,1,1] tol6. Captured `runs/sh73-emitter-login.png` = a genuine
Roblox login/connection screen: wordmark → Wi-Fi status icon → input field →
green button, stacked and centered over the vignette.

## What changed (elfjit.rs)

1. `login_ui_textures()` now also loads the real `noconnection.png`
   (connection-loss chip, from the sibling `graphic/` root) AND synthesizes two
   solid UI prims (`field.png` 224,224,230 opaque; `loginbtn.png` Roblox green
   0,158,68 opaque), appended as uniform-filled atlas rows. Because the login FS
   outputs only `texture2D(uTex,vUV)`, a uniform row is drawn as a solid quad;
   the SH72 `ub = s.w/aw` fix keeps each row's u-extent local (256/1024 rows).
2. Login placements extended to the full form: vignette full-screen, wordmark
   upper-middle, chip below it, field (cy=-0.45, hh=0.045), green button
   (cy=-0.62, hh=0.05).
3. `+2` hermetic regressions (sh75_login_solid_rows_are_opaque...probe_is_
   byte_exact_over_any_dst; sh75_login_field_and_button_probe_land_inside_their_
   boxes). Workspace **509/0**; example tests now **11 pass** (was 9).

## Honest scope

This is the visual ceiling of the harness-authored surface: real login artwork
+ real-signal UI solid prims through the engine's own emitter, pixel-verified.
It is still NOT the engine self-constructing a login UI from its own session
(fabricated geometry + host texture/program/blend; the solid prims are
synthesized in the atlas). Standing structural wall unchanged: type-4 producer
vector [0x106829ea8] external glue; nativeGameGlobalInit parks; no in-image path
constructs a GuiObject and appends to the scene list. A true self-driven login
screen needs the Lua app-shell / game-activity session path.

## Repro

`runs/capture_emitter_login.sh` (RENDEREMITTER_LOGIN=1 sets the whole surface).
Log `runs/sh73-emitter-login.txt`. Docs differ from SH73/74 only in the added
solid prims.

## Next

(B) animate the login composite (spin/scroll the wordmark or drift a progress
bar) for a "live" login surface across present-walker frames — SH71's
`RENDEREMITTER_SPIN` machinery. After that, the standing structural wall is the
only thing between here and a self-populated login session.