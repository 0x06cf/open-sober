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
engine swap `Ok(1)`), but the authored quad's pixels are NOT confirmed in the
buffer our readback/X-grab samples — the engine renders the emit into **its own
render-target FBO binding**, not the default framebuffer we read. So SH66 proves
the engine's real geometry-emitter path is now *reachable and runnable*
headlessly (a first — previously only a disasm), but the emitted content's
pixel landing is not yet captured. Standing structural wall unchanged (the
emitter is still driven with a fabricated `G`, not a real UI/GuiObject screen).
**Next:** query the engine's current draw-framebuffer binding during the emit and
either read back from that FBO, or force-bind the default framebuffer for the
emit so the authored quad lands in the sampled/visible surface — converting
"emitter runs" into "emitter visibly draws".

## Commands

`./runs/capture_renderemitter.sh` (`--renderemitter` alongside `--renderwalker`).
Verify markers: `engine emitter Ok` ×2, `swap=Ok(1)`, exit 124, 0 crash.