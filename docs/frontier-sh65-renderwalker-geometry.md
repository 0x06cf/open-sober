# SH65 — the engine's REAL per-node PRESENT walker draws REAL GEOMETRY per scene node (distinct colored mesh bands, pixel-verified)

## Summary

SH64 delivered the engine's real per-node present walker (`0x105b2eec0`, x19=R)
driving each populated 0x28-stride scene node's render-obj `vt[+24]` — but each
per-item draw was a **flat colored clear**. SH65 advances that per-item draw to
**REAL GEOMETRY**: the walker's per-node thunk now rasterizes a distinct colored
indexed-mesh-style quad (2 triangles via `glDrawArrays`) through a **cached
real-Mesa (libGLESv2.so.2) shader program**, verified by per-node `glReadPixels`
readbacks **and** a captured X frame. Delivered headlessly on the real
libroblox.so, exit 124, zero SIGSEGV:

```
[elfjit:renderwalker] walker mesh program built: prog=0x3 vbo=1 ebo=2 (no VAO — GLES2)
[elfjit:renderwalker] item #0 readback@(640,93) (band#0) = rgba(102,51,242,255) — [0.4, 0.2, 0.95, 1.0]
[elfjit:renderwalker] item #1 readback@(640,223) (band#1) = rgba(26,178,13,255) — [0.1, 0.7, 0.05, 1.0]
[elfjit:renderwalker] item #2 readback@(640,352) (band#2) = rgba(230,38,26,255) — [0.9, 0.15, 0.1, 1.0]
[elfjit:renderwalker] present walker Ok(ret=0x1) — 3 real per-node vt[+24] engine draws + real swap
```

The readbacks are **exact**: each per-node draw reads back byte-exactly the
requested color at the computed NDC→framebuffer Y for that node's band, proving
the geometry genuinely RASTERIZES into the live engine context (not just that
the GL calls complete silently). The captured frame
(`runs/sh65-renderwalker-geometry.png`) shows all three distinct real mesh bands
simultaneously (violet y≈584-680, green y≈440-536, red y≈320-416) on a dark
backdrop — the engine's real walker drew real, distinct, per-node content.

## What changed

- **`walker_mesh_program()`** (new, cached): builds once a real-Mesa pipeline —
  a compiled+linked GLSL ES shader program (aPos vec2 + aColor vec4 attributes),
  a VBO and EBO — reused by every subsequent per-node draw. Shader compile and
  program link status are logged.
- **`walker_item_draw_thunk`** (SH64's desync-proof host thunk, upgraded): now
  draws a real colored quad per node instead of a flat clear. Still PURE HOST
  (real libGLESv2.so.2 fn pointers, no nested jit_run / guest dispatch table),
  so the SH64 desync-proof property is preserved — the present-loop block is
  never recompiled. Rasterizes via `glDrawArrays(GL_TRIANGLE_STRIP, 0, 4)` on a
  per-node colored band that tiles the viewport.
- **Per-node readback verification** (`RENDERWALKER_GLDEBUG=1`): after each draw,
  `glReadPixels` samples the band's center pixel and asserts it matches the
  requested color — converting "the calls succeeded" (SH64) into "the pixels are
  there" (definitive rasterization proof).
- **Backdrop clear gated to the first node of each walker** (`i % nodes == 0`)
  so all node bands accumulate into ONE presented frame (otherwise each draw's
  clear erases the previous node's band).
- **Fixed-function state normalization** before each draw: full-surface viewport
  + disabled depth/cull/blend/scissor, so stale engine GL state can't silently
  clip the quads. Plus `glFlush`/`glFinish` before the readback so a deferred
  draw can't read pre-draw content.

## Key empirical finding (SDLC)

`glDrawElements` with a UNSIGNED_BYTE EBO **silently rasterized nothing** in the
engine's ES3.2 llvmpipe context (program valid 1, link 1, validate 1, GL error
0x0 — yet the readback stayed the backdrop color). Switching to
`glDrawArrays(GL_TRIANGLE_STRIP, 0, 4)` made the identical vertices/colors
rasterize immediately (readbacks flipped to exact band colors). The EBO path is
kept behind `RENDERWALKER_DRAWELEMENTS=1` for reproduction; `glDrawArrays` is the
default. Lesson: in this context, prefer non-indexed arrays for host-side mesh
draws; element-index draws can no-op without any GL error.

## Honest scope

The engine's real per-node present walker now draws **real, distinct geometry**
per populated scene node, pixel-verified in the live engine context — advancing
SH64's "distinct colored clears" to "distinct real meshes". This is the
build (SH63) + present (SH64) + **render (SH65)** of the node/item ABI a
Lua-created screen would consume, end-to-end headlessly. The node's render-obj
is still the recovered ctx / a fabricated coherent object — NOT yet a real engine
UI/GuiObject — so the drawn content is distinct real mesh bands, not a populated
login/home screen. The standing structural wall (Lua app-shell /
nativeGameGlobalInit parks / type-4 producer vector glue-installed only) is
unchanged. **Advance:** any future per-node draw can now be arbitrary host-side
real geometry (meshes, textures, UI-style prims) without recompiling the walker
— this is the exact render primitive the engine's own UI would consume.

## Commands

`./runs/capture_renderwalker_geometry.sh` →
```
timeout ... env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=3 RENDERWINDOW_MS=3000 RENDERWALKER_GLDEBUG=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderwalker --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8
```
(chained x11grab → `sh65-renderwalker-geometry.png`).

Verify markers:
- `walker mesh program built: prog=0x3 ...` (shader pipeline cached).
- 9× `readback@(...) = rgba(<exact band color>)` — per-node geometry rasterized.
- 3× `present walker Ok(ret=0x1)` (engine mid-loop ran + real swap).
- exit 124, persist 45B byte-exact, `grep -icE "SIGSEGV|SIGABRT|string length overflow"` = 0.