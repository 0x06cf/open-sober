# SH67d — the engine's real emitter draws a POPULATED N-quad 2D frame (single top-level call, GL_TRIANGLES), closing the SH67c multi-tile blocker

## Result

The ENGINE's own geometry emitter `0x105b35288`, driven headlessly as ONE
top-level jit_run, now **rasterizes a populated 2×3 grid of 6 distinct colored
quads into the presented frame** through the engine's own GL stack
(primitive-setup 0x105b353d0 → glDrawArrays via @plt → real Mesa), every tile
pixel-verified at its grid position, presented via the real engine swap, exit
124, zero SIGSEGV/SIGABRT. This is the populated multi-element frame the SH67c
bisection had blocked.

## Why this shape (SH67c blocker, closed BY CONSTRUCTION)

SH67c's bisection proved: repeated engine-emitter jit_run rasterizes fine when
REUSING one already-uploaded VBO, but ANY `glBufferData`/`glBindBuffer` inside
the emitter drive (per-tile re-upload or fresh buffer object) silently drops
subsequent draws (variant SIGABRT at first=0). A research subagent
(deleg_ca280417, read-only disasm) pinned the mechanism: the engine's draw is a
stateless `glDrawArrays(first, count)` into whatever VBO primitive_setup just
bound to the VAO-less GLES2 global attrib-pointer state; re-issuing
host-side `glBufferData` on that buffer orphans its storage → the wired attrib
pointers dangle → silent drop / abort. The draw itself is textbook-correct.

Fix: **ONE emitter call with a single pre-uploaded VBO holding all N*6 verts
as GL_TRIANGLES (6 verts/quad), `mode_idx=0` (GL_TRIANGLES), `first=0`,
`count=6N`.** No per-tile re-drive, no inter-drive buffer change → the orphan
class cannot occur; GL_TRIANGLES keeps quads isolated (no TRIANGLE_STRIP
cross-tile fusion).

## What landed

- **`render_engine_emitter_grid(ctx, iimg, ibase, isp, nq)`** (elfjit.rs):
  uploads ONE VBO (once) with all nq*6 verts `[pos2+color4]` stride 24 in a 3×N
  grid (tile `t` at NDC cell with paletted color), builds the same fabricated
  engine geometry-context G (G+0x48/0x58 BD slots → shared VBO, G+0x78=0
  non-indexed, G+0x8e=0, spec/stride/format tables per SH66), normalizes
  fixed-function state + wires `glDrawBuffers(1,{GL_BACK})` + `glReadBuffer`
  (SH66b/SH67), then drives the emitter ONCE (`jit_run 0x105b35288` with
  `x1=0, x2=0, x4=6N, x5=0`), glFinish, engine bind+swap via ctx-vt[+24], and
  reads back each tile's center asserting per-channel ≤1 diff.
- Env `RENDEREMITTER_QUADS=N` (default unset → SH67b single quad unchanged).
- Repro `runs/capture_emitter_grid.sh` + artifact `runs/sh67d-emitter-grid.png`.

## Empirical (real libroblox.so, runs/sh67d-emitter-grid.txt, exit 124)

```
EXIT=124
engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=36) quads=6 swap=Ok(1)  (x2)
per-tile readbacks (2 frames = 12): ALL 12 present=true
  tile#0 (234,558) rgba(102,51,242) diff=[0,0,0,0]   purple
  tile#1 (640,558) rgba( 26,178,13) diff=[1,0,1,0]   green
  tile#2 (1045,558) rgba(230,38,26) diff=[1,0,1,0]   red
  tile#3 (234,234) rgba( 13,153,230) diff=[1,0,1,0]  blue
  tile#4 (640,234) rgba(255,209,13) diff=[0,0,1,0]   yellow
  tile#5 (1045,234) rgba(102,51,242) diff=[0,0,0,0]  purple
walker still presents: 2x `present walker Ok(ret=0x1)` + 6 real colored quads
crash/json-overflow: 0
```

capture runs/sh67d-emitter-grid.png shows the 2×3 grid (purple|green|red over
blue|yellow|purple) on the dark backdrop. Diff values are float→u8 rounding
(palette 0.10*255=25.5 may truncate 25 / Mesa round 26), never a real gap — a
missing tile would read the dark backdrop (13,13,20), a ≥50 delta on every
channel.

## Honest scope

Proves the ENGINE's own draw path rasterizes a genuinely **populated,
multi-element, isolated-quad frame** in the live engine context and presents it
via the real swap — advancing SH67b's single quad to a grid. It does NOT prove
a real GUI gesture: G is a fabricated coherent object, not a real UiObject; the
standing structural wall is unchanged (the engine never self-populates a real
render-manager/session; type-4 producer vector 0x106829ea8 is .bss
framework-glue-only). **Next (SH67d §4):** feed scene-shaped content — fabricate
N layered rect quads mirroring a real login/home layer, or size/place tiles from
the real scene list (R+0x180/0x188); textured version needs the same
one-time-pre-upload discipline for GL_TEXTURE.

## Repro

`bash runs/capture_emitter_grid.sh` (default 6) or `RENDEREMITTER_QUADS=N bash runs/capture_emitter_grid.sh`.

Verify markers: `renderemitter-grid] engine emitter Ok ... draw_mode=0x4(first=0,count=6N) quads=N swap=Ok(1)`, N distinct `present=true` per frame (via per-channel ≤1 diff), `present walker Ok(ret=0x1)`, exit 124, 0 crash/json-overflow.