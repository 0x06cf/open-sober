# SH79 — TTF rasterizer now handles COMPOSITE glyphs (any Roblox UI label renders)

## Result

The SH77 pure-std TrueType rasterizer previously bailed (`None`, glyph skipped)
on any **composite glyph** (`numberOfContours < 0`). SourceSansPro-Bold has
652/1115 composite glyphs — all accented letters (À-Ï, é, ñ…), `%`, `:`, `;`,
`=`, quotes, `²³` — so an arbitrary Roblox UI label containing those chars
silently lost glyphs. SH79 decodes composite glyphs recursively into their
component glyphs' contours, each transformed by the component's offset / scale /
2x2, so the rasterizer now renders ANY font glyph.

Hermetic regression `sh79_composite_glyph_decodes_and_rasterizes` (real font,
no harness): `%`, `:`, `0` (all composite in SSPro-Bold) each resolve to
non-empty contour sets AND rasterize to >20 opaque glyph pixels — previously
they returned None (bailed). Example tests **22 pass** (was 21).

## What changed (elfjit.rs)

`font_glyph_contours` now delegates to a recursive `font_glyph_contours_rec`
(`depth<=16` guard against runaway nesting), which:
- on `numberOfContours >= 0`: decodes the simple glyph exactly as before;
- on `numberOfContours < 0`: walks the composite component records —
  flags/glyphIndex, then (per flags) arg1/arg2 (word vs byte, ARGS_ARE_XY_VALUES
  vs point-match — point-match falls back to 0 offset), then the transform
  (WE_HAVE_A_SCALE → uniform F2Dot14; AN_X_AND_Y_SCALE → sx/sy; TWO_BY_TWO →
  4-element); recursively fetches each component glyph's contours, maps every
  point `(x,y) -> (t11·x+t12·y+ox, t21·x+t22·y+oy)`, and appends the offset
  rings; honors MORE_COMPONENTS until the chain ends.

Long-loca (`indexToLocFormat=1`) was already handled in the simple arm and now
applies to composite glyphs too (a/b computed per gid, shared by both paths).

## Honest scope

Renders any single font glyph (composite or simple) into the atlas; the harder
CFF/`'OTTO'` and point-matching-args paths still fall back (component renders at
origin when args are point-match numbers — rare in modern TTFs). No
screen-level change yet: the shipped login labels use zero composite glyphs
(they were already simple), so this is latent robustness for arbitrary future
labels. Standing structural wall unchanged (engine still not self-constructing
a GuiObject/session).

## Repro

Any sh79 test run:
`cargo test -p arm64jit --example elfjit -- sh79_composite_glyph_decodes_and_rasterizes`.
Workspace **509/0**; example tests **22 pass**.