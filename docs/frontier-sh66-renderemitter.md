# SH66 — the engine's REAL geometry emitter (0x105b35288) is now RUNNABLE headlessly as a desync-safe top-level jit_run

## Summary

SH63-65 drove the engine's real scene renderer + per-node present walker, but
each per-node draw was a **host-side** mesh (distinct colored quad via the
cached real-Mesa program in the walker thunk). SH66 targets the ENGINE's OWN
geometry-draw machinery: guest geometry emitter `0x105b35288` (which internally
calls primitive-setup `0x105b353d0`, binds VBOs, sets vertex-attrib pointers
from the engine's format table `0xcecf8c`, and dispatches `glDrawArrays`/`glDrawElements`
via @plt → real Mesa). Delivered: the emitter **runs** headlessly — its full
draw path executes with zero crash / zero desync, exit 124, via an authored
guest geometry-context `G` fabricated from a fresh disasm-derived layout.

```
[elfjit:renderemitter] built engine geometry ctx G=0x... M=0x... VBO=3 spec[2] stride=24
[elfjit:renderemitter] engine emitter Ok(ret=0x0) swap=Ok(1) ...
[elfjit:renderwalker] drained: 2 real engine-per-node present-walker frames presented ...
EXIT=124
```

## What landed

- **`render_engine_emitter_quad()`** (new, `--renderemitter`): builds a guest
  geometry-context `G` from the confirmed disasm layout (recon-derived + fresh
  objdump):
  - `G+0x38` = mesh `M`, `G+0x48`/`G+0x58` = BD slot array (attr 0/1 → same BD),
    `G+0x78` = element-buffer (0 → non-indexed), `G+0x8e` = u16 elem-type.
  - `M+0x48`/`M+0x50` = spec array begin/end (24B entries), `M+0x60` = stride table.
  - spec entries (confirmed disasm): `+0` attr idx, `+4` offset addend, `+8`
    format idx, `+12` attrib-loc-enum (0→0,1→1,2→+2,3→+4), `+16` size addend.
  - vertex-format table `0xcecf8c` (12B/entry {size,type,norm}) — confirmed
    `[1]={2,FLOAT}`, `[3]={4,FLOAT}`. draw-mode table `0x225780` — `[3]=0x5` (GL_TRIANGLE_STRIP).
  - Uploads an authored 4-vertex colored quad into a real VBO (host
    glGenBuffers/glBufferData), binds the cached shader program, drives the
    emitter as its OWN top-level `jit_run(0x105b35288, G, w1=3, w2=4, w3=0, w4=0, w5=0)`.
- **Desync-safe by construction:** the emitter is a separate top-level `jit_run`
  (run after each walker frame on the currency-owning renderinit thread), never
  nested inside the present-walker block — so no executing translation is
  invalidated (closes the SH64 nested-jit_run class).
  - After the emit, binds engine ctx + swaps via ctx-vt[+24] → `Ok(1)`.
- Build fixes traced to the exact engine layout: BD slot indexing is
  `[G+0x48 + attr*0x10]` (attr 1 needs its OWN slot at G+0x58, not shared),
  stride-table entries are integers not element offsets, buffers are 8-aligned
  (Box leak of u64, guest==host), the emitter gets its own guest stack.

## Empirical (real libroblox.so, exit 124, zero SIGSEGV/SIGABRT)

- `engine emitter Ok(ret=0x0) swap=Ok(1)` ×2 (one per walker frame) — the
  engine's own primitive-setup → attribute-bind → draw dispatch executes cleanly
  and the real engine swap succeeds.
- Walker still draws its 6 real colored quads (pixel-verified in SH65).
- Productized baseline re-verified: exit 124, swap Ok(0x1), persist 45B byte-exact,
  0 crash. Workspace 509/0.

## Honest scope — the pixel-proof gap

The emitter **executes** (returns Ok, primitive-setup runs, attrs ordered,
engine swap `Ok(1)`), but the authored quad's pixels are NOT confirmed in the buffer our readback/X-grab samples.

**SH66b root-cause probe (draw-buffer binding):** an FBO-binding probe added
before the emit reads `glGetIntegerv(GL_DRAW_FRAMEBUFFER_BINDING)` and
`GL_DRAW_BUFFER` on the current context:
```
[elfjit:renderemitter] draw_fbo=0 draw_buffer=0x0 (before emit)
```
`GL_FRAMEBUFFER_BINDING=0` (default FBO) is fine, but **`GL_DRAW_BUFFER=0x0`
(= GL_NONE)** — the context's default framebuffer has NO draw buffer wired to
the visible surface at the instant the emitter runs. So the emitter's
glDrawArrays rasterizes into a buffer with no draw target → pixels are dropped
(not a crash, not a GL error — same silent-no-op class as SH65's
glDrawElements=UNSIGNED_BYTE finding). The fix for the next cycle: set
`glDrawBuffer(GL_BACK, 0x0405)` (or `GL_COLOR_ATTACHMENT0` for an FBO) on the
current context before driving the emitter, so its draw lands on the presented
surface. This is a concrete, addressable lever — the engine's real
geometry-emitter path is proven *runnable* end-to-end, and the remaining gap is
a single state wire (draw buffer), not an ABI/structure problem.

Standing structural wall unchanged (the
emitter is still driven with a fabricated `G`, not a real UI/GuiObject screen).
**Next:** set the draw buffer to GL_BACK before the emit and read back — closing
"emitter runs" → "emitter visibly draws".

## Commands

`./runs/capture_renderemitter.sh` (`--renderemitter` alongside `--renderwalker`).
Verify markers: `engine emitter Ok` ×2, `swap=Ok(1)`, exit 124, 0 crash.