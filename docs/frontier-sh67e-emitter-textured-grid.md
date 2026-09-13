# SH67e — the engine's real emitter draws a TEXTURED populated frame (aTex 3rd attribute, host texture sampled)

## Result

The ENGINE's own geometry emitter `0x105b35288`, driven headlessly as ONE
top-level jit_run, now **rasterizes a populated 2×3 grid of 6 textured quads**
into the presented frame. The fragment shader's color comes **exclusively from
`texture2D(uTex, vUV)`** (the aColor vertex attribute is ignored), yet every tile
reads back exactly its distinct authored palette color — proving the engine's
primitive_setup wired a **3rd per-vertex texcoord attribute (aTex loc2)**, the
emitter interpolated per-tile UVs, and the sampler sampled the live pre-uploaded
host texture. Texture upload is safe re: SH67c (never touches GL_ARRAY_BUFFER).
Real libroblox.so exit 124, zero SIGSEGV/SIGABRT.

## Why this advances the frontier

SH67d proved the engine draws a populated solid-color grid; SH67e proves the SAME
engine path draws textured UI-layer-style quads. Real Roblox UI content is
textured (sprites, fonts, shader packs — served since SH57), so a populated
**textured** frame brought through the engine's own geometry emitter is the
closest statically-reachable precursor to a real login/home screen's rendering
path. The critical fact (pinned by read-only disasm, deleg_7604fc6a): the emitter
and primitive_setup dispatch ONLY vertex-attribute GL — texture binding is
host-set; the engine interpolates per-vertex UVs from the spec-wired aTex stream.

## What landed

- **`emitter_tex_program()`** (elfjit.rs) — a SEPARATE cached real-Mesa program
  (attributes aPos=0, aColor=1, aTex=2; uniform sampler2D uTex; FS outputs
  `texture2D(uTex,vUV)` only) so it never fights walker_mesh_program over shared
  GLES2-program state.
- **`render_engine_emitter_grid(..., tex)`** — when `tex`, adds `spec[2]`
  (attr 2 texcoord, offset 24, format idx 1 vec2 FLOAT, attrib-loc-enum 2 → loc
  2), spec end `spec+72`, `stride_tab[2]=32`, BD slot `G+0x68=bd0` (stride 24→32
  = pos2@0 + color4@8 + uv2@24), uploads a host palette-strip texture once,
  binds prog + sampler before the single emit, and `glDisableVertexAttribArray(2)`
  after (so the walker's next reuse of the VAO-less global context is clean).
- Env `RENDEREMITTER_TEX=1` (with RENDEREMITTER_QUADS); default keeps the solid
  grid (SH67d) / single quad (SH67b).
- Repro runs/capture_emitter_grid.sh + artifacts.

## Empirical (real libroblox.so, runs/sh67e-emitter-grid-tex.txt, exit 124)

```
textured program built prog=0x6 uTex loc=0
tex 48x1 RGBA uploaded (glTexImage2D 0x1908) uTex loc=0 prog=0x6
built engine geometry ctx G=.. M=.. VBO=3 quads=6 total_verts=36 GL_TRIANGLES stride=32 spec[3] aPos=0 aColor=1 aTex=2   (x2)
engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=36) quads=6 swap=Ok(1)   (x2)
per-tile readbacks: ALL 12 present=true, diff=[0,0,0,0] on EVERY tile
  tile#0 (234,558) rgba(102,51,242):0  tile#1 (640,558) rgba(25,178,12):0
  tile#2 (1045,558) rgba(229,38,25):0  tile#3 (234,234) rgba(12,153,229):0
  tile#4 (640,234) rgba(255,209,12):0  tile#5 (1045,234) rgba(102,51,242):0
walker still presents: 2x present walker Ok(ret=0x1) + 6 real colored quads
crash/json-overflow: 0
```

The 12 readbacks are `diff=[0,0,0,0]` — Byte-EXACT, better than SH67d's ≤1 —because the color now comes from a pre-computed 8-bit palette-strip texture
rather than float-interpolated vertex colors. capture runs/sh67e-emitter-grid-tex.png
shows the 2×3 grid (purple|green|red over blue|yellow|purple). Because the FS
ignores aColor and samples only uTex, any broken aTex wiring/UV/binding would have
yielded uniform (or black) tiles — the per-tile distinct exact colors are the
definitive "the textured path is live" proof.

## Honest scope

Proves the ENGINE's own geometry path rasterizes multiple fabricated, TEXTURED
quads whose UVs it interpolates from the spec-wired aTex stream, against a
pre-uploaded host texture, presented via the real swap. The texture bind, sampler
uniform, and program are host-set preconditions (the emitter has no texture
concept); geometry is authored, not a real scene. It does NOT prove a real
UI/GuiObject, the engine's own textured-asset pipeline, or engine-internal
glBindTexture — those remain behind the standing structural wall (no
self-populated render session). Next (SH67e §ranked): layered scene-shaped content
sized/placed from the real scene list (R+0x180/0x188) into a login/home-like
layout, + blend path for real UI compositing.

## Repro

`RENDEREMITTER_TEX=1 bash runs/capture_emitter_grid.sh` (default 6).

Verify markers: `textured program built ... uTex loc=0`, `tex 48x1 RGBA uploaded`,
`stride=32 spec[3] aPos=0 aColor=1 aTex=2`, `engine emitter Ok ... swap=Ok(1)`,
N distinct `present=true` with `diff=[0,0,0,0]`, `present walker Ok(ret=0x1)`,
exit 124, 0 crash/json-overflow.