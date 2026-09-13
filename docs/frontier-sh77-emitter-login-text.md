# SH77 — real ROBLOX login-form TEXT LABELS rendered by the engine's own emitter

## Result

`RENDEREMITTER_LOGIN=1` now renders a COMPLETE Roblox login form with real
**text labels** on it — white "Log In" over the green button, dark-slate
"Email address" and "Password" placeholders over the two input fields —
rasterized from the APK's OWN font (`SourceSansPro-Bold.ttf`) by a new
**pure-std TrueType outline rasterizer** and fed through the engine's real
geometry emitter as transparent 256-wide RGBA8 atlas rows.

Real libroblox.so, exit 124 (stable idle), zero crash:
```
[elfjit:renderemitter-login] rasterized text label 'Log In' (256x40)
[elfjit:renderemitter-login] rasterized text label 'Email address' (256x32)
[elfjit:renderemitter-login] rasterized text label 'Password' (256x32)
[elfjit:renderemitter-multi] engine emitter Ok(ret=0x0) count=60 sprites=9 swap=Ok(1)
[elfjit:renderemitter-multi] probe 'Log In'          (642,138) got rgba(255,255,255,255) diff=[0,0,0,0] present=true
[elfjit:renderemitter-multi] probe 'Email address'   (588,192) got rgba(96,96,110,255)   diff=[0,0,0,0] present=true
[elfjit:renderemitter-multi] probe 'Password'        (669,169) got rgba(96,96,110,255)   diff=[0,0,0,0] present=true
```
**9/9 probes present=true** — real reversevignette (diff <=1), wordmark + chip
byte-exact white, field/loginbtn/field2 solid rows byte-exact, and all THREE
text labels byte-exact at glyph-interior texels (the glyph color is baked into
the atlas texel, so the GL_BLEND composite equals the glyph color over ANY dst).
Captured runs/sh77-emitter-login-text.png = a recognizable login form. ASCII
ridge-maps of the presented frame confirm the labels read as real letterforms
(e.g. L-o-g-I-n, E-m-a-i-l a-d-d-r-e-s-s, P-a-s-s-w-o-r-d), not smears.

## What changed (elfjit.rs)

A self-contained SH77 block (next to `decode_png_rgba`):
- **`sfnt_find_table`** — tag-indexed table lookup, rejects CFF/`'OTTO'` heads.
- **`font_cmap4` + `Cmap4::gid`** — Format-4 subtable (Windows BMP preferred),
  honoring BOTH the idDelta and idRangeOffset segment encodings (idRangeOffset
  addresses are subtable-relative).
- **`font_advance_width`** (hmtx), **`font_glyph_contours`** (head/loca short+
  long, glyf simple contours incl. repeat flags + x/y delta decoding; bails on
  composite/`numberOfContours<0`).
- **`flatten_ring`** — TrueType quadratic flattening: on-on → line, on-off-on →
  quadratic (4-point sampling), consecutive off-curves → implied on-curve at
  midpoint; outputs a closed image-space ring map (non-zero winding).
- **`covered`** — non-zero-winding point-in-polygon (handles nested counters).
- **`rasterize_text_row`** — lays out a string, flattens every glyph into one
  image-space ring set, 4x4-supersampled coverage into an RGBA8 row (opaque
  glyph interiors + smooth α edges, transparent background).
- **`login_font_bytes` / `rasterize_login_label`** — cache the real font, bake a
  centered 256-wide label row.
- `login_ui_textures()` appends `field2.png` (2nd input field) + the three text
  rows (`Log In` white 256x40, `Email address`/`Password` dark-slate 256x32).
- `placements` (login) adds the matching boxes; field/loginbtn probes moved to
  the box edge (probe pixel 24,8) because the centered text overlays the old
  center-probe pixels (painter's-order - expected).

Zero new deps / zero network (pure std; extract the real APK font, already in
`assets/fonts/`).

## Regression (6 new hermetic sh77_tests, example 17 pass total)

`sh77_flatten_ring_emits_closed_quad_polygon`, `sh77_quadratic_offs_flatten
_cover_interior`, `sh77_covered_uses_nonzero_winding_for_counter`,
`sh77_cmap4_maps_ascii_log_in_to_nonzero_gid`, `sh77_text_grid_runs_for_each
_label` (opaque>50, glyph rgb baked, bg transparent), `sh77_login_label_probes
_are_opaque_glyph_interior` (the exact probe texels are opaque glyph interiors).
Workspace **509/0**.

## Honest scope

Text labels are good glyphs but authored/host-rasterized (real APK font,
engine's own emitter path, correct painter's-order compositing over the form).
NOT the engine self-constructing a GuiObject/session; standing wall unchanged
(type-4 producer vector glue-only; nativeGameGlobalInit parks; no in-image
GuiObject->scene-list path).

## Repro

`runs/capture_emitter_login_text.sh` (sets RENDEREMITTER_LOGIN=1 + LIVE=1).
Log runs/sh77-emitter-login-text.txt, captured runs/sh77-emitter-login-text.png.

## Next

Standing structural wall (self-populated session behind the Lua app-shell), or
polish: thicker glyphs at the current box size (GL_LINEAR atop tightly-packed
atlas rows + thin strokes read faint at 720p — bump each text row's
rasterized size / pad atlas rows); or sub-256 rows to relax strict aw=256.