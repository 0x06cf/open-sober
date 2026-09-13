# SH72 — the engine renders a FULL MULTI-SPRITE login/composite of real Roblox UI textures

## Result

The ENGINE's own geometry emitter `0x105b35288` now draws **THREE REAL Roblox UI
sprites in ONE frame** — the loading spinner (`LoadingScreen/LoadingSpinner.png`
100x100, ct6), the Robux icon (`InspectMenu/ico_robux@3x.png` 54x54, ct4), and
the jump button (`Input/JumpButtonRegular@2x.png` 240x240, ct6) — each in its
own aspect-correct box, all sampled from a **single shared vertical atlas** in a
**single top-level jit_run**, alpha-composited over a dark backdrop with
GL_BLEND. Real libroblox.so exit 124, 2x engine-emitter Ok + 2x present-walker
Ok, **8/8 pixel probes present=true**, zero crash. This is the first time the
engine renders a *populated multi-element frame made of several real Roblox UI
assets simultaneously*.

## Why it advances the frontier

SH69/SH70/SH71 proved ONE real texture through the engine path; SH72 proves the
runtime can composite MULTIPLE real Roblox sprites at distinct positions in one
frame — the structural property a real login/home screen's render-loop needs
(many UI images, one draw pass, correct placement + alpha). The captured frame
(`runs/sh72-emitter-multi.png`) shows the recognizable Roblox loading surface:
dark backdrop, blue spinner arc upper-middle, R$ robux icon top-right, jump
button bottom-center.

## What landed (elfjit.rs)

- **`RENDEREMITTER_MULTI=1`** (with `RENDEREMITTER_LAYOUT=home`): drives a new
  `render_engine_emitter_multi()` — one VBO of (1 backdrop + N sprite) quads
  (24 verts, GL_TRIANGLES, stride 32), one atlas texture (aw = max sprite width,
  ah = 1 strip row + Σ sprite heights), one emitter `jit_run`.
- **Shared atlas**: row 0 = opaque dark backdrop strip; each sprite's rows
  [memlo, memlo+h-1] hold its RGBA, top-first via the SH69 reversal (memory row
  m = k holds PNG row h-1-k) so v=high → image top. Per-sprite v = [memlo/ah,
  (memlo+h-1)/ah].
- **`ub = s.w/aw` (SH72 bug):** the sprite's u must map image columns [0, s.w)
  — NOT the full atlas row `[0,1]`. Using ub=1 squished the image into the left
  s.w/aw of the box (the right part sampled empty atlas → transparent).
- **Probe math matches the emitter's projection** (SH72 finding): the emitter
  maps NDC Y un-inverted into glReadPixels y = (1+y_ndc)/2*VH; the probe follows
  exactly (image pixel iy → y_ndc = (cy-hh) + (1-(iy+.5)/h)*2hh). (imgpix_screen
  is a top-down/centered-symmetric form that happens to agree only for centered
  H-symmetric boxes; the multi probes use the direct emitter mapping.)
- **Expected = GL_BLEND composite** (SH72 finding): the sprite texels are not
  all opaque (robux center a=0, jump-white a=0x66), so the expected probe color
  is `out = src*a + 26/26/31*(1-a)` over the backdrop — a plain rgb==src compare
  false-fails on semi-transparent sprites. This is also why the SH69/71
  byte-exact spinner (opaque arc) still works: a=1 reduces to src.
- **Diagnostics** (all env-gated `RENDEREMITTER_GLTRAP=1` / `RENDEREMITTER_MULTI_INDEX=N`
  — default flows untouched): GL error + link + vertex dump, full-frame dump to
  /tmp, and single-sprite isolation.

## Empirical (real libroblox.so, runs/sh72-emitter-multi.txt, exit 124)

```
engine emitter Ok(ret=0x0) count=24 sprites=3 swap=Ok(1)   x2
probe 'LoadingSpinner.png' (592,400) img(33,88)  got rgba(49,180,255,255) exp [49,180,255,255] diff=[0,0,0,0]    present=true  (BYTE-EXACT arc)
probe 'ico_robux@3x.png'    (1079,663) img(10,10) got rgba(254,254,254,255) exp [255,255,255,255]   diff=[1,1,1,0]   present=true  (opaque white outline)
probe 'JumpButtonRegular@2x.png' (623,137) img(84,120) got rgba(118,118,121,255) exp [117,117,120,255] diff=[1,1,1,0] present=true  (semi-transparent white-over-backdrop blend)
probe 'spinner-transparent-center' (641,510) got rgba(26,26,31,255) exp backdrop(26,26,31,255) diff=[0,0,0,0] present=true  (real alpha 0 → backdrop shows through)
```

8/8 across 2 frames; walker 2x `present walker Ok(ret=0x1)`; zero
SIGSEGV/SIGABRT/json-overflow. Captured frame `runs/sh72-emitter-multi.png`.

## Honest scope

Same standing wall: the box positions, atlas, and texture program are host-set;
the engine emits the multi-quad geometry and samples the real textures through
its own GL path. The composite is a real array of real Roblox UI sprites, but
it is authored (not populated by the engine's own UI scene from a session). The
engine never self-populates a render-manager/session (Lua app-shell /
nativeGameGlobalInit parks / type-4 vector .bss framework-glue-only — all
unchanged). This is the strongest static composite of real UI yet, not a
self-driven login screen.

## Repro

`env RENDEREMITTER_LAYOUT=home RENDEREMITTER_MULTI=1 RENDERWALKER_MAX_FRAMES=2
RENDERWALKER_WINDOW_MS=2800 RENDERWALKER_NODES=3` + the standard elfjit play
recipe. Success markers: `loads real sprite ... xN`, `engine emitter Ok ...
sprites=3 ... swap=Ok(1)`, one `present=true` per sprite + the transparent-center
probe, `present walker Ok(ret=0x1)`, exit 124, zero crash. Repro script:
`runs/capture_emitter_multi.sh`.

## Hermetic tests

`cargo test -p arm64jit --example elfjit` (9 pass), +3 SH72:
`sh72_imgpix_rect_centered_reduces_to_imgpix_screen`,
`sh72_imgpix_rect_offset_box_shifts_probe_screen_coords`,
`sh72_atlas_uv_layout_is_disjoint_within_unit_range`. Workspace 509/0.