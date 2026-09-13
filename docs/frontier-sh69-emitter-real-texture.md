# SH69 — the engine's real geometry emitter renders a REAL Roblox APK UI texture

## SH70 addendum (same cycle): decoders now handle PNG color-type 3 (palette)

The offline decoder `decode_png_rgba` now also supports **color type 3
(indexed/palette)** with optional `tRNS` per-index alpha (real UI sprites are
commonly palette PNGs — e.g. `ui/InGameMenu/BackgroundGlow@2x.png`). PLTE is
parsed into RGB, tRNS into per-index alpha (absent tRNS = opaque). Also added a
clear `WARN: RENDEREMITTER_REAL_TEX=1 but real UI texture failed to load/decode
— falling back to the palette strip` line so a bad/missing path is not silent.

Verified: 2 new hermetic ct-3 tests (palette-with-tRNS exact RGBA; palette
without tRNS opaque) pass; and the REAL binary decodes `BackgroundGlow@2x.png`
(512×512 ct=3) -> `RGBA8` and renders it through the engine emitter
(`decoded ... 512x512 RGBA8`, `tex 512x513`, emitter Ok, swap Ok, exit 124,
0 crash). The decoder now covers ct 0/2/3/4/6 — the full PNG surface the real
UI assets use.

## Result

The ENGINE's own geometry emitter `0x105b35288`, driven headlessly as one
top-level jit_run, now **rasterizes a real Roblox client texture** — the
loading-screen spinner (`ui/LoadingScreen/LoadingSpinner.png`, 100×100, RGBA,
real alpha, the iconic sky-blue `#29ABE2` arc) — through the engine's own GL
path, aspect-correct and alpha-composited over the layered login/home frame.
The **`real-arc-blue` probe reads `rgba(49,180,255)` byte-exact** at a screen
coordinate computed from the same aspect math — that color and pixel come from
the real APK PNG, NOT a host palette. Real libroblox.so exit 124, zero
SIGSEGV/SIGABRT.

## Why this advances the frontier

SH68 proved the engine's own draw path does genuine UI-style GL_BLEND alpha
compositing, but every pixel was a host-authored palette color. SH69 replaces
the layer-1 "panel" with **actual Roblox client content**: a real APK texture
is decoded offline and sampled through the engine's textured emitter path. The
result is the first time real Roblox UI pixels (not fabricated palette) render
through the engine's own render pipeline headlessly — the closest
statically-reachable step toward "the engine renders the real client's own
screens." A real login screen is a stack of composited, textured, transparent
rects; the engine now draws a stack that includes a real transparent Roblox
sprite.

## What landed (crates/arm64jit/examples/elfjit.rs)

- **`decode_png_rgba(data) -> (w,h,RGBA8)`** — minimal offline PNG→RGBA8 decoder
  (color types 0/2/4/6, 8-bit, non-interlaced; per-row filter unrolling incl.
  Paeth). **Zero new network dep**: rides `flate2` (already pinned in
  Cargo.lock via `zip`) — added `flate2 = "1.1"` to arm64jit's Cargo.toml.
- **`real_ui_texture()`** — cached loader of the real APK sprite (path
  overridable via `RENDEREMITTER_REAL_TEXTURE`; default the extracted
  `.../assets/content/textures/ui/LoadingScreen/LoadingSpinner.png`).
- **`imgpix_screen(ix,iy,imw,imh,hh,VW,VH)`** — image-pixel→window math matching
  the shader's linear vUV interpolation, so probe coords are exact.
- **`render_engine_emitter_home`** real-texture branch (env
  `RENDEREMITTER_REAL_TEX=1`, alongside `RENDEREMITTER_LAYOUT=home`): builds a
  2-D atlas `A_W×A_H = imw × (imh+1) = 100×101` — row 0 = the palette strip
  (solid blocks `[t/nq,(t+1)/nq)` for the backdrop/button/title/field layers),
  rows 1..=imh = the real image uploaded REVERSED (memory row M holds PNG
  top-first row (imh−M)) so it samples upright. Layer 1 becomes the real image
  in a centered aspect-correct box: `hh=0.5`, `half_w = hh·(W/H)·(VH/VW) =
  0.28125`, UV `u∈[0,1]`, `v∈[1/101,100/101]`. Texture set to GL_LINEAR +
  GL_CLAMP_TO_EDGE.
- **Real-mode probes** replace the panel-blend probes: `backdrop` (unchanged),
  `real-arc-blue` = an OPAQUE arc-body pixel img(33,88)→(49,180,255) [proof of
  real texture content], `transparent-center` = real alpha 0 → backdrop shows
  through [proof of the real alpha channel, NOT the old 0.55 host panel],
  `transparent-right` = arc is left-only [proof of correct orientation/aspect].

## Empirical (real libroblox.so, runs/sh69-emitter-real.txt, exit 124)

```
[elfjit:renderemitter-real] decoded real UI texture from .../LoadingSpinner.png: 100x100 RGBA8
scene list R=0x.. n_scene=3 head/tail-derived=3 match=true
LAYOUT=home layers=5 real_tex=true blend enabled tex 100x101 RGBA uploaded (glTexImage2D 0x1908) uTex loc=0 prog=0x6
engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=30) layers=5 swap=Ok(1)   (x2)
real probes: arc-blue@580,498 transparent-center@641,361 transparent-right@749,361
  backdrop          (51,691) rgba(25,25,30) expect [26,26,31] diff=[1,1,1,0] present=true
  real-arc-blue     (580,498) rgba(49,180,255) expect [49,180,255] diff=[0,0,0,0] present=true  <= REAL texture pixel
  transparent-center (641,361) rgba(25,25,30) expect [26,26,31] diff=[1,1,1,0] present=true   <= real alpha 0 -> backdrop
  transparent-right  (749,361) rgba(25,25,30) expect [26,26,31] diff=[1,1,1,0] present=true   <= arc left-only (orientation)
present walker Ok(ret=0x1) x2; crash/json-overflow: 0
```

`runs/sh69-emitter-real.png` shows the real sky-blue loading spinner (partial
~270° ring) centered, with the dark backdrop, green button bar, cream title
strip, and gray field strip composited — a recognizable Roblox loading/UI
frame.

## Honest scope

The texture, program, and blend are still host-set preconditions (the emitter
carries no such concepts); geometry is authored; and the scene list only drives
quad count (the engine never self-populates UI items). It is a real Roblox
texture, but a single static asset — the engine still does not self-populate a
session. Standing structural wall unchanged (Lua app-shell /
nativeGameGlobalInit parks / type-4 producer vector .bss framework-glue-only).

## Regression

4 hermetic tests in elfjit.rs (`sh69_tests::{type6_filter0_roundtrip,
gray_expands_opaque_alpha, imgpix_screen_centers, rejects_non_png}`), run via
`cargo test -p arm64jit --example elfjit`. (Binary-example test harnesses are
not included in `cargo test --workspace`; the real-binary artifact above is the
definitive verification.)

## Repro

`bash runs/capture_emitter_real.sh` (RENDEREMITTER_LAYOUT=home +
RENDEREMITTER_REAL_TEX=1 + explicit sprite path).