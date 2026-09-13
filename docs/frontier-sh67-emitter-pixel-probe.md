# SH67 — emitter pixel gap: GL_NONE / draw-buffer hypothesis DISPROVEN; correct ES3 wire

## What landed (crates/arm64jit/examples/elfjit.rs, --renderemitter path)

1. **SH66b's GL_BACK wire was a silent no-op — fixed with the real ES3 symbol.**
   The SH66b probe called `glDrawBuffer` (singular) to wire the default
   framebuffer to GL_BACK. `nm -D libGLESv2.so.2` proves Mesa exports only the
   ES3 plural `glDrawBuffers` — the singular name is desktop-only, so
   `mesa_fn(h, b"glDrawBuffer\0")` returned None and the wire never ran
   (draw_buffer stayed 0x0 all along). Replaced with
   `glDrawBuffers(1, {GL_BACK})` + `glReadBuffer(GL_BACK)`, both real ES3
   symbols present in the lib.

2. **New env-gated `RENDEREMITTER_GLTRAP=1` post-emit GL-state dump** — checks
   glGetError, GL_CURRENT_PROGRAM, GL_DRAW_FRAMEBUFFER_BINDING, GL_DRAW_BUFFER,
   and the current program's aPos/aColor attrib locations + link status, right
   after the emitter's jit_run. Off by default (no hot-path impact).

## Empirical finding (real libroblox.so, exit 124, zero crash) — runs/sh67-gltrap.txt

`GLTRAP err=0x0 cur_prog=0x3 draw_fbo=0 draw_buffer=0x0 aPos_loc=0 aColor_loc=1 link_status=1`

Every state the emitter's draw depends on is CORRECT after the emit:
- `err=0x0` — GL_NO_ERROR; the draw did not fault or reject.
- `cur_prog=0x3` — the walker_mesh_program is still current (the emitter did
  NOT rebind/replace its own program, and 0x3 == walker program, which links).
- `aPos_loc=0 / aColor_loc=1 / link_status=1` — the program still has our two
  attributes at loc 0/1 and links cleanly.
- `draw_fbo=0` — default framebuffer, same one the walker rasterizes into.

**Conclusion: the GL_DRAW_BUFFER==GL_NONE value is a red herring for this
context.** glDrawBuffers(GL_BACK) did not change the reported GL_DRAW_BUFFER
(stays 0x0) AND the walker rasterizes pixel-verified bands into that SAME
default FBO with `draw_buffer=0x0` — so "GL_NONE" does not block default-FBO
rasterization here (llvmpipe reports GL_NONE for the default framebuffer yet
still rasterizes). The emitter quad's pixels still don't land in the sampled
back/front buffer, with every observable state correct.

## Root cause narrowed

The gap is inside the emitter's OWN draw call: `primitive-setup 0x105b353d0`
issues glBindBuffer + glEnableVertexAttribArray + glVertexAttribPointer +
glDrawArrays **via @plt** (the JIT-resolved link), whereas the walker issues
the same sequence via DIRECT `mesa_fn` dlsym'd pointers. Both are the real
Mesa on the live ctx. Given err=0 / program-correct / default-FBO, the
remaining candidate mechanisms are: (a) the emit draws 0 primitives (count /
mode-table idx / spec iteration resolving to a no-op), (b) the emit's
glDrawArrays routes through an @plt entry that is a JIT stub that no-ops
instead of the real Mesa symbol, or (c) depth/stencil/cull leftover state (the
walker normalizes fixed-function state; the emitter draws at default depth 0
and could be depth-tested against the walker's leftover writes). (c) is cheap
to rule out (GL_DEPTH_TEST query at the trap); (a) needs a guest trace of the
spec/stride/mode tables; (b) needs a JIT GLES-draw trace.

## Honest scope

The engine's real emitter 0x105b35288 still RUNS its full draw path headlessly
(a first, SH66) and returns clean — this cycle refines *why the authored
quad's pixels don't land* and corrects the draw-buffer wiring to the real ES3
form. The fabricated geometry-context G remains a harness artifact (NOT a real
GUI/GuiObject), so even a landing quad is not a real login/home screen. The
standing structural wall is unchanged (the engine never self-populates a real
render-manager / session — type-4 producer vector is glue-seeded only). This
is a built-into-the-frontier diagnostic + correctness fix, not a screen.

## Repro

```
bash runs/capture_renderemitter.sh            # productized baseline (walk 6 quads + swap Ok)
env RENDEREMITTER_GLTRAP=1 ... (full recipe)  # post-emit state dump
env RENDEREMITTER_READBACK_BEFORE_SWAP=1 ...  # isolate back-buffer before swap
```

Workspace 509/0 (example-only changes).