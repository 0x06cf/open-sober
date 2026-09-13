# SH78 — thicker, crisper login-form text labels (2x supersample + atlas guard rows)

## Result

`RENDEREMITTER_LOGIN=1` now renders the login form's text labels at 2x the
rasterized resolution with **transparent atlas guard rows**, so the glyphs read
thicker and steadier at 720p while the on-screen quad geometry, probe
screen-coordinates, and byte-exactness are all unchanged.

Real libroblox.so, exit clean (drained), 0 crash:
```
[elfjit:renderemitter-login] rasterized text label 'Log In' (512x80)
[elfjit:renderemitter-login] rasterized text label 'Email address' (512x64)
[elfjit:renderemitter-login] rasterized text label 'Password' (512x64)
[elfjit:renderemitter-multi] engine emitter Ok(ret=0x0) count=60 sprites=9 swap=Ok(1)
[elfjit:renderemitter-multi] probe 'Log In'           (627,138) got rgba(255,255,255,255) diff=[0,0,0,0] present=true
[elfjit:renderemitter-multi] probe 'Email address'    (697,196) got rgba(96,96,110,255)   diff=[0,0,0,0] present=true
[elfjit:renderemitter-multi] probe 'Password'         (672,166) got rgba(96,96,110,255)   diff=[0,0,0,0] present=true
```
**9/9 probes present=true**, the three text probes byte-exact. Stroke-thickness
proxy (opaque glyph pixels in the presented frame) rose across all three labels:
Log In 254->328, Email address 758->878, Password 524->596; the ASCII ridge-map
of the button now shows solid `#####`/`######` strokes (L-o-g-I-n) instead of
faint thin columns. Persist roundtrip 45B byte-exact; walker drained 1 frame;
swap Ok(0x1); zero SIGSEGV/SIGABRT/string-overflow. Captured
runs/sh78-emitter-login-text-2x.png.

## What changed (elfjit.rs)

Subagent-ranked this cycle's frontier B (login-text polish) > C (TTF hardening)
> A (standing wall). A was re-confirmed still statically impenetrable (the
file-0x5b2c828/0x5b2eb7c/0x5b2ec1c "node builders" only *consume* the
pre-populated R+0x180 scene list inside the already-driven scene renderer /
present walker; nativeGameGlobalInit is a thin park wrapper; real GuiObject
construction needs the unreachable StartLuaAppDM layer). Three changes:

1. **`rasterize_login_label` parameterized** (`w`/`h`/`pu` passed in instead of
   hardcoded 256/40/0.034), baseline pad `h/10`, RealSprite uses `w`.
2. **2x text table** in `login_ui_textures`: Log In 512x80 pu=0.068; Email
   address 512x64 pu=0.060; Password 512x64 pu=0.060. Because `w` and `h` both
   double, `w:h` is preserved (512/80 == 256/40), so `half_w`, quad height, and
   every probe screen-coordinate are bit-identical — only texel density and
   stroke thickness double.
3. **Atlas guard rows** (`GUARD=1` added to each sprite's memory stride in
   `render_engine_emitter_multi`): a transparent row after every sprite so
   GL_LINEAR sampling near a sub-row boundary never bleeds a NEIGHBOR sprite's
   color (the text labels sat directly under the opaque field/button rows —
   the source of the faint look). `vlo/vhi` span only the data rows, `px` is
   zero-initialized so guards stay transparent.

## Regression (4 new sh78b tests + 2 updated sh77 font tests; example 21 pass)

`sh78b_aspect_invariant_geometry` (512/80==256/40, 512/64==256/32),
`sh78b_atlas_guard_rows_pad_blocks` (memlo advances by h+GUARD; guard row stays
below the sprite's v-window), `sh78b_stroke_thickens_at_2x` (max opaque run
> 1.4x the old pu), `sh78b_3x3_probe_interior_opaque` (each probe texel has a
full 3x3 opaque neighborhood -> robust vs GL_LINEAR). Updated
`sh77_text_grid_runs_for_each_label` + `sh77_login_label_probes_are_opaque
_glyph_interior` to the 2x table + new probe pivots. Workspace **509/0**.

## Honest scope

Thicker/crisper glyphs still host-rasterized (real APK font) fed through the
engine's own emitter; NOT engine self-constructed GuiObject/session. Standing
structural wall confirmed unchanged (A closed this cycle).

## Repro + next

`runs/capture_emitter_login_text.sh` (RENDEREMITTER_LOGIN=1 + LIVE=1). Log/view
runs/sh78-emitter-login-text-2x.txt/.png. Next: opaque "bare wood" hardening
(the C path — composite glyph + long-loca for arbitrary labels) or keep
advancing toward the standing wall as new levers appear.