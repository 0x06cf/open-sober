# SH68 — the engine's real emitter draws a LAYERED "login/home" frame with GL_BLEND alpha compositing, sized from the real scene list

## Result

The ENGINE's own geometry emitter `0x105b35288`, driven headlessly as ONE
top-level jit_run, now **rasterizes a 5-layer login/home-style frame** (dark
fullscreen backdrop, centered semi-transparent panel, green button bar, title
strip, field strip) through the engine's own GL stack, presented via the real
swap. The frame is **GL_BLEND alpha-composited**: the panel (straight alpha 0.55)
blends over the backdrop, and the overlap readback **matches the blend equation
`src*src.a + dst*(1-src.a)` to within float rounding** — genuine UI-style
compositing, the closest statically-reachable precursor to a real login/home
screen. Real libroblox.so exit 124, zero SIGSEGV/SIGABRT.

## Why this advances the frontier

Everyone prior landed solid (SH67d) and textured (SH67e) grids — distinct tiles
but no layering. SH68 upgrades the SAME engine path to a **layered, blended,
scene-shaped** frame: the layout is sized/placed from the REAL scene list the
engine's render path already carries (R+0x180/0x188 → SCENE_NODES), and the
overlap pixels verify the alpha blend equation. A real login/home UI is a stack
of composited, textured rects; the engine now reproduces that structure (as
fabricated geometry) headlessly through its own emitter + primitive_setup.

## What landed

- **`render_engine_emitter_home(ctx, iimg, ibase, isp, layer_override)`**
  (elfjit.rs) — 5 textured layers (backdrop -1..1 / panel alpha 0.55 / button
  bar / title / field strips) as ONE pre-uploaded VBO of 5*6=30 verts
  (stride 32, pos2@0 + color4@8 + uv2@24), one `jit_run 0x105b35288`
  (`x1=0` GL_TRIANGLES, `x2=0`, `x4=30`, `x5=0`), each layer's straight-alpha a
  separate texel of a 5-texel RGBA strip (FS outputs only `texture2D(uTex,vUV)`),
  `glEnable(GL_BLEND)` + `glBlendFuncSeparate(S_ALPHA, ONE_MINUS_SRC_ALPHA, ...)`
  before the emit, painter's-order triangle layout so later layers composite over
  earlier. Reads the real scene list to size the layout:
  `SCENE_NODES`/`RENDERSCENE_BASE` + re-derives head/tail from `R+0x180/0x188`
  (`tail-head)/0x28`), asserting `derived == n_scene`.
- Reuses `emitter_tex_program()` + the G/M/spec/BD scaffolding from SH67e.
- Env `RENDEREMITTER_LAYOUT=home` (overrides the grid; keeps RENDEREMITTER_QUADS/
  TEX for SH67d/e).
- Repro runs/capture_emitter_home.sh + artifacts.

## Empirical (real libroblox.so, runs/sh68-emitter-home.txt, exit 124)

```
scene list R=0x7f.. n_scene=3 head/tail-derived=3 match=true
LAYOUT=home layers=5 blend=enabled(SRC_ALPHA,ONE_MINUS_SRC_ALPHA) tex 40x1 RGBA uploaded uTex loc=0 prog=0x6
engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=30) layers=5 swap=Ok(1)   (x2)
blend probes (x2 frames, all 8 present=true):
  backdrop    (51,691) rgba(25,25,30,255) expect [26,26,31,255] diff=[1,1,1,0]
  panel-blend (640,360) rgba(44,47,58,255) expect [45,48,59,255] diff=[1,1,1,0]   <= the compositing proof
  button      (640,281) rgba(40,152,109,255) expect [41,152,109,255] diff=[1,0,0,0]
  title-bar   (640,472) rgba(192,188,175,255) expect [193,189,175,255] diff=[1,1,0,0]
walker still presents: 2x present walker Ok(ret=0x1)
crash/json-overflow: 0
```

The **panel-blend** probe is the key: rgba(44,47,58) at screen center is NEITHER
the backdrop (26,26,31) NOR the pure panel (61,66,82) — it equals the blended
result (panel src=(61,66,82) @ a=0.55 over dst=(26,26,31)):
r=61·.55+26·.45=45.2, g=66·.55+26·.45=48.0, b=82·.55+31·.45=59.1 ⇒ (45,48,59).
Readback (44,47,58) is within ±1 = float→u8 rounding (llvmpipe fixed-point blend).
If GL_BLEND were disabled, the panel would overwrite the backdrop (read ~(61,66,82));
if the alpha were ignored, it would read ~(61,66,82) opaque. It reads the
blend-equation value ⇒ **the engine's draw path genuinely alpha-composites.**
capture runs/sh68-emitter-home.png shows the modal/dialog layout (dark backdrop +
centered lighter panel + white title strip + gray field strip + green button bar).

## Honest scope

Proves the ENGINE's own geometry path rasterizes a single top-level multi-rect
frame with non-uniform per-layer geometry, per-layer texture sampling, and
GL_BLEND alpha compositing whose overlap readbacks match the blend equation — a
statically-fabricated login/home-precursor sized/placed from the real scene list
(R+0x180/0x188 → SCENE_NODES). It is NOT a real GuiObject, NOT a self-populated
render session: texture, program, and blend are host-set preconditions (the
emitter carries no such concepts); geometry is authored; the scene list only
drives the quad count (the engine never self-populates UI items). Standing
structural wall unchanged (Lua app-shell / nativeGameGlobalInit parks / type-4
producer vector .bss framework-glue-only). Next: ETC2/ASTC texture upgrade to
mirror real asset format (polish), per-node vt[+24] emitter integration is
desync-dead (SH64), so the real self-populated-session wall is out of reach
statically.

## Repro

`bash runs/capture_emitter_home.sh` (default 5 layers, scene-list sized).

Verify markers: `scene list ... n_scene=N head/tail-derived=N match=true`,
`LAYOUT=home layers=5 blend=enabled(SRC_ALPHA,ONE_MINUS_SRC_ALPHA)`,
`engine emitter Ok ... count=30 layers=5 swap=Ok(1)`, `panel-blend ... present=true`
(= the blend-equation proof), `present walker Ok(ret=0x1)`, exit 124,
`grep -icE "SIGSEGV|SIGABRT|string length overflow"` = 0.