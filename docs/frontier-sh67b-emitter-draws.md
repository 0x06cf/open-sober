# SH67b — RESOLVED: the engine's real geometry emitter now VISIBLY draws its quad

## Result

The SH66/SH67 open gap is closed. The engine's own geometry emitter
`0x105b35288`, driven headlessly as a desync-safe top-level jit_run, now
**rasterizes its authored quad into the presented frame** through the engine's
own GL stack (primitive-setup 0x105b353d0 -> glVertexAttribPointer + glDrawArrays
via @plt -> real Mesa). Root cause was a **harness argument-inversion bug**, not
the draw buffer.

## Root cause (disasm-proven by research subagent deleg_b1698660, read-only)

The emitter function's ACTUAL argument mapping (file 0x5b35288 decode) is:

```
0x5b3529c  w22 = w1          ; mode_idx
0x5b352a8  w20 = w4          ; -> glDrawArrays COUNT
0x5b352ac  w23 = w2          ; -> glDrawArrays FIRST
0x5b352a4  w21 = w5
...
non-indexed path 0x5b35384:
  mov w1, w23                ; glDrawArrays first = arg2
  mov w2, w20                ; glDrawArrays count  = arg4
  bl glDrawArrays@plt        ; GL_TRIANGLE_STRIP from mode table [0x225780 + w22*4]
```

i.e. **emit(G=x0, modeidx=x1, first=x2, geomkey=x3, count=x4, idxflag=x5)**.

The SH66 harness drove `x1=3, x2=4("count"), x4=0("first")` — the two fields
were **inverted** relative to the emitter's real layout, so the engine issued

```
glDrawArrays(GL_TRIANGLE_STRIP, first=4, count=0)
```

`count==0` is a legal, error-free no-op — which is why the emit returned clean
(exit 124, GL error 0x0, current program unchanged, walker program linked,
attrib locs 0/1) yet drew **nothing**. This exactly reproduced every SH66/SH67
observed symptom and fully explains the "draws but no pixels" puzzle.

**Fix** (crates/arm64jit/examples/elfjit.rs `render_engine_emitter_quad`,
correct-by-default): `st.x[2]=0` (first), `st.x[4]=4` (count). The `@plt`
GOT entry names the real `glDrawArrays@Base` Mesa symbol (JUMP_SLOT@0x67d2318,
relocation `R_AARCH64_JUMP_SLOT glDrawArrays@Base`) so the draw routes to real
Mesa; the mode table `0x225780` bytes `[4,1,0,5,...]` confirm `table[3]=0x5`
GL_TRIANGLE_STRIP.

## Empirical (real libroblox.so, exit 124, zero crash) — runs/sh67b-swap.txt + runs/sh66-renderemitter.txt

`engine emitter Ok(ret=0x0) swap=Ok(1)` — the quad is pixel-verified in BOTH the
back-buffer-direct read (`RENDEREMITTER_READBACK_BEFORE_SWAP=1`) AND the
presented (post-swap) frame:

```
BACK-BUFFER center=(640,360) rgba(64,223,172)  | y100=(125,121,219) y200=(153,114,191)
  y300=(181,106,163) y360=(64,223,172) y420=(81,219,155) y500=(103,213,133)
  y600=(131,205,105) y650=(145,201,91)
```

A smooth violet->magenta->teal->green->yellow-orange gradient across the whole
vertical extent — exactly the authored colored quad's interpolated colors
(verts: BL violet / BR teal / TR orange / TL green). The dark backdrop
(13,13,20) and the walker's discrete bands are gone from the sampled line; the
walker's 6 real colored quads and 2 `present walker Ok(ret=0x1)` remain in the
same run. Captured frame runs/sh67b-emitter-quad.png.

## Honest scope

The ENGINE's real geometry emitter now draws engine-detailed content (an
authored colored quad / default-Rect primitive) headlessly, through the
engine's own GLES path, pixel-verified and presented via the real engine swap —
the exact primitive a login/home UI layer's default-Rect emitter produces. It is
NOT yet a populated login/home screen: the geometry-context G is a fabricated
coherent object, and the standing structural wall is unchanged — the engine
never self-populates a real render-manager/session (render scene-list head/tail
R+0x180/0x188 has no in-image writer; type-4 producer vector [0x106829ea8] is
.bss, framework-glue-installed only). This is the closest statically-reachable
screen precursor, and it converts "emitter runs" into "engine draws its own real
primitives" — measured.

## Repro

```
bash runs/capture_renderemitter.sh          # product path: emitter quad + walker, swap Ok(1)
env RENDEREMITTER_READBACK_BEFORE_SWAP=1 ... # isolate the back buffer before swap
env RENDEREMITTER_GLTRAP=1 ...               # post-emit state dump (diagnostic)
```

Workspace 509/0 (example-only changes).