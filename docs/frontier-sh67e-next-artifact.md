# SH67e — Next artifact: TEXTURED populated frame via the engine's real emitter (0x105b35288)

Read-only disasm (deleg_7604fc6a), real libroblox.so; guest = file vaddr + 0x100000000. Extends SH67d's single-call GL_TRIANGLES grid.

## (A) Does the emitter's destructive path ever bind/upload textures? — NO

Disasm of emitter 0x105b35288 + primitive_setup 0x105b353d0 — the complete GL PLT slot set:

- **emitter** (file 0x5b35288): `bl primitive_setup` (unconditional, first), then non-indexed → `glDrawArrays@plt 0x62d7840`, indexed → `glDrawElements@plt 0x62d7830`; helpers 0x105b3a22c/0x238 (indexed only); `glGetError@plt 0x62d7580`; internal log 0x626d1d0. **No glUseProgram, no glUniform***.
- **primitive_setup** (file 0x5b353d0): attribute-wiring loop over `[M+0x48..M+0x50)`: `glBindBuffer@plt 0x62d77b0` (0x8892), `glEnableVertexAttribArray@plt 0x62d7850`, `glVertexAttribPointer@plt 0x62d7860`, `glGetError`; optional `glBindBuffer(0x8893,EBO)`; `glDisableVertexAttribArray@plt 0x62d7870`; `__stack_chk_fail`. Returns attr mask `w0 = OR(1 << loc)`.

The texture PLT slots (`glActiveTexture@plt 0x62d75e0`, `glBindTexture@plt 0x62d75f0`, `glGenTextures@plt 0x62d7980`, `glTexImage2D@plt 0x62d79a0`, `glCompressedTexImage2D@plt 0x62d7990`, `glTexSubImage2D@plt 0x62d79f0`, `glDeleteTextures@plt 0x62d7790`) exist as imports but are NEVER reached from this path — they belong to the texture-manager/shader subsystems.

**Conclusion: the emitter is a pure vertex-attribute geometry dispatcher (VBO bind → wire attrib pointers from spec/vform tables → glDrawArrays/glDrawElements). It has NO texture concept — it draws with whatever program + texture the caller left bound.** A textured Engine draw must come through (B).

## (B) Concrete artifact — texture through the SAME emitter (single jit_run, one VBO)

primitive_setup loops over ALL spec entries (termination `(spec_end−begin)/24`), so a 3rd texcoord attribute is wired exactly like pos/color.

**G/M/spec/stride/BD additions** (textured branch, stride 24→**32** = pos2@0 + color4@8 + uv2@24; gate `RENDEREMITTER_TEX=1`):
- `spec[2]` @ `spec+48`: attr=2, offset-addend=24, format-idx=**1** (reuse `vform[1]={2,FLOAT}`, size 2, type 0x1406), attrib-loc-enum=**2** (disasm 0x5b35424..64: enum 0→loc0, 1→loc1, 2→size_addend+2, 3→size_addend+4, else −1 ⇒ loc **2**), size_addend=0.
- `M+0x50` (spec end) = `spec + 72` (was +48). `stride_tab[2]` = u64 **32** at `+16` (`ldr x23,[M+0x60 + attr*8]`).
- BD slot `G+0x68` = `bd0` (BD slots `G+0x48 + attr*0x10`, disasm 0x5b3547c), `bd0+0x48`=VBO. `G+0x78=0`, `G+0x8e=0` unchanged. Emitter drive same as SH67d: `x1=0`(GL_TRIANGLES), `x2=0`, `x3=0`, `x4=6N`, `x5=0`; primitive_setup returns mask `0b111`.

**Shader commits** — a separate cached textured program (do NOT mutate the shared walker_mesh_program the walker thunk also uses):
- `glBindAttribLocation` before link: `aPos=0`, `aColor=1`, `aTex=2`. VS: `attribute vec2 aTex; varying vec2 vUV; vUV=aTex;`. FS: `precision mediump float; uniform sampler2D uTex; gl_FragColor = vec4(texture2D(uTex,vUV).rgb,1.0)` (or `* vColor` for color+texture blend so each tile stays distinguishable).
- `loc_uTex = glGetUniformLocation(prog,"uTex")` (≈0); `glUniform1i(loc_uTex,0)`. Since the emitter never calls glUseProgram/glUniform*, bind program + set the sampler once before the emit.

**Host-side texture setup** (via mesa_fn; emitter won't): `glGenTextures`, `glActiveTexture(0x84C0)`+`glBindTexture(0x0DE1,tex)`, `glTexParameteri` MIN/MAG=NEAREST (no mipmaps). Uncompressed min: `glTexImage2D(0x0DE1,0,0x1908,w,h,0,0x1908,0x1401,pixels)`. Compressed (matches real Roblox asset path via the mixed bridge): `glCompressedTexImage2D(0x0DE1,0,0x9278 ETC2-RGBA8/SH31 or 0x93B0 ASTC-4x4/SH32,w,h,0,imageSize,data)` — llvmpipe lacks native ETC2/ASTC so the bridge decodes via texture_codec.

**Orphan risk — texture upload is SAFE.** The SH67c blocker was glBufferData orphaning GL_ARRAY_BUFFER storage referenced by engine-wired attrib pointers. glTexImage2D/glCompressedTexImage2D mutate independent GL_TEXTURE_2D state, never touch GL_ARRAY_BUFFER/attribs → no orphan class. Pre-upload once anyway.

**Post-emit cleanup:** `glDisableVertexAttribArray(2)` after the emit (walker immediately reuses the VAO-less global context; a stale aTex pointer would corrupt the walker's next frame).

**Verify markers** (env `RENDEREMITTER_TEX=1`):
```
[elfjit:renderemitter-grid] tex uploaded (glTexImage2D/glCompressedTexImage2D 0x9278) uTex loc=0 prog=0x..
[elfjit:renderemitter-grid] built engine geometry ctx G=.. M=.. VBO=.. quads=N total_verts=6N GL_TRIANGLES stride=32 spec[3] aPos=0 aColor=1 aTex=2
[elfjit:renderemitter-grid] engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=6N) quads=N swap=Ok(1)
[elfjit:renderemitter-grid] tile#t center(..) rgba(..) diff=[..] present=true   (N distinct textured panels)
present walker Ok(ret=0x1) ... exit 124, SIGSEGV/SIGABRT grep=0
```

## Honest scope

Proves the ENGINE's own geometry path (primitive_setup → glDrawArrays) rasterizes multiple fabricated, textured quads whose UVs it interpolates from the spec-wired aTex stream, against a pre-uploaded host texture, presented via the real swap. The texture bind, sampler uniform, and program are host-set preconditions (the emitter has no texture concept); geometry authored, not a real scene. Does NOT prove a real UI/GuiObject, textured asset pipeline, or engine-internal glBindTexture — those remain behind the standing structural wall (no self-populated render session).

## Ranked next frontier
1. Land SH67e textured emitter grid (this artifact).
2. Layered scene-shaped content sized/placed from the real scene list (R+0x180/0x188) into a login/home-like layering (RENDEREMITTER_LAYOUT=home).
3. Blend path (glEnable(GL_BLEND)+glBlendFunc before emit) so overlapping layered textures composite like real UI. Pairs with #2.
4. Per-node emitter vt[+24] integration — do not attempt (SH64 desync SIGSEGV).