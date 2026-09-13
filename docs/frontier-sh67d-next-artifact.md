# SH67d — Next-artifact derivation: the engine's own emitter drawing a POPULATED multi-element frame (single-call GL_TRIANGLES), + honest indexed-path pin

Read-only research. All addresses guest (= file vaddr + 0x100000000). Binary: `~/.cache/open-sober/robbox/libroblox.so` (109MB aarch64, verified = apk `lib/arm64-v8a/libroblox.so`). Docs read: SH67b/c, SH66, SH65, SH64, STATUS.md; harness: `render_engine_emitter_quad` (elfjit.rs:1021), `render_engine_present_walker` (:910), `walker_item_draw_thunk` (:651), `walker_patch_full_body` (:853). NO code edited, nothing built/run.

## 1. EXACT emitter dispatch (disasm 0x105b35288, zero ambiguity)

Prologue: `x19=G(x0)`, `w22=w1`(mode_idx), `w23=w2`(first), `w20=w4`(count), `w21=w5`(idxflag), `w1=w3`(geom_key); unconditional `bl primitive_setup 0x105b353d0` first (so EVERY resize re-binds the VBO + re-wires attrib pointers — emitter never self-uploads). Then `ldrh w8,[G+0x8e]`(elem_type), `ldr x9,[G+0x78]`(elem_buf), `w3=w8*w23`.

```
w21(idxflag)==0                       ── non-indexed branch (0x105b35310)
   x9(elem_buf)==0  → glDrawArrays@plt 0x1062d7840 : mode=table[0x225780+w22*4], first=w2, count=w4   ← SH67b proven path
   x9(elem_buf)!=0  → glDrawElements@plt 0x1062d7830 : mode=table[idx], count=w4, type = (elem_type==2?0x1403 USHORT : 0x1405 UINT), indices=EBO+ elem_type*first, w4=w21
w21(idxflag)!=0                       ── indexed branch (0x105b352cc)
   x9==0 → helper 0x105b3a238 (first=w2,mode,count=w4)
   x9!=0 → helper 0x105b3a22c (count=w4,type,indices=EBO+elem_type*first)
```

Mode table @0x10225780 dwords = `[0x4, 0x1, 0x0, 0x5]` → mode_idx `0`=GL_TRIANGLES, `3`=GL_TRIANGLE_STRIP. Vertex-format table @0x10cecf8c: `vform[1]={2,FLOAT}`, `vform[3]={4,FLOAT}` (matches harness spec). **The non-indexed draw is a plain, stateless `glDrawArrays(table[mode], first=w2, count=w4)` into whatever GL_ARRAY_BUFFER primitive_setup just bound (the G+0x48/0x58 BD slots' shared VBO).** There is NO engine-side loop/cache over `first` or `count` — the engine draws exactly the requested vertex range in one call. `first` IS a vertex offset into the bound buffer; using it for per-tile offsets is the textbook, correct use.

## 2. Blocker pinned: harness-side buffer orphaning, NOT an engine limitation

- Working bisection case = "reuse one already-uploaded VBO, no glBufferData, emit N times" — the shared VBO is uploaded once BEFORE any primitive_setup wires attrib pointers, then emitter re-drives re-bind + re-draw the same buffer. Clean.
- All failing cases (per-tile re-upload, fresh buffer object, shared-buffer-with-first variation where the harness re-touched the buffer between drives) re-issue host `glBufferData`/`glBindBuffer` on a buffer the engine's primitive_setup has already wired into the **VAO-less global GLES2 context** attrib-pointer state. Mesa llvmpipe orphans the old storage on glBufferData → the wired attrib pointers dangle → next draw silently drops or llvmpipe aborts (exit 134 = SIGABRT, observed on the first=0 variant). The engine code path is correct; the harness drive caused the orphan.
- **Why per-tile-first on a truly-static shared VBO can still fail:** the engine's GL_TRIANGLE_STRIP (mode 3) with a large `count` fuses all tiles into ONE continuous strip (connect-geometry triangles), and if the harness "shared" VBO re-uploaded content per tile it re-enters the orphan class. Both are avoidable.
- **Indexed path is dead on llvmpipe regardless** (SH65's proven finding: glDrawElements+EBO silently rasterizes nothing, valid/link/validate all OK, no error). Do NOT pursue G+0x78!=0 or idxflag=1.

## 3. Concrete artifact — ONE emitter call renders the whole populated frame

Requirement: multi-element 2D frame from the engine's own emitter, structurally impossible to hit the blocker. Answer: **a single emitter `jit_run` with a pre-uploaded VBO holding ALL N quads as GL_TRIANGLES (6 verts/quad), `mode_idx=0`, `first=0`, `count=6N`.** No per-tile re-drive, no inter-draw buffer change → the orphan/drop class is closed BY CONSTRUCTION, and GL_TRIANGLES keeps quads isolated (no strip fusion).

- DUP the geometry builder in `render_engine_emitter_quad`: upload one VBO with `N*6` verts `[pos2+color4]` stride 24 (verts = triA(0,1,2)+triB(0,2,3) per quad, tile `t` at NDC offset color-indexed per tile so readback can assert per-tile). Keep `G` identical (G+0x78=0, G+0x8e=0, BD slots → this one VBO, stride tab both 24).
- Drive: `st.x[0]=G; st.x[1]=0` (mode_idx→GL_TRIANGLES); `st.x[2]=0` (first); `st.x[3]=0` (geom_key); `st.x[4]=6*N` (count); `st.x[5]=0` (non-indexed); `jit_run(iimg, ibase, 0x105b35288, &st)` — one call. Keep the SH67b pre-draw fixes: `glDrawBuffers(1,{GL_BACK})`, `glReadBuffer(GL_BACK)`, viewport 1280x720, default FBO + depth/cull/blend/scissor off, `glFinish` before readback.
- Add env `RENDEREMITTER_QUADS=N` (default e.g. 6 for a 3x2 grid). Verify marker lines:
  ```
  [elfjit:renderemitter] built engine geometry ctx G=... M=... VBO=... quads=6 total_verts=36 GL_TRIANGLES stride=24
  [elfjit:renderemitter] engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=36)
  [elfjit:renderemitter] BACK-BUFFER-DIRECT center tile#2=(640,240) rgba(<tile2 color>) | t0=... t5=...   ← per-tile distinct readback
  [elfjit:renderemitter] engine emitter Ok(ret=0x0) swap=Ok(1) center=(640,360) rgba(...)
  ```
- **Install/verify:** add `let nq = env RENDEREMITTER_QUADS` before the driver in `render_engine_emitter_quad`; set count=6*nq and gen the nq*6 verts (reuse the existing glGenBuffers/glBufferData once). Run `./runs/capture_renderemitter.sh` with `RENDEREMITTER_QUADS=6`. Assert: `grep "draw_mode=0x4"` present, ≥N distinct per-tile readback colors, `swap=Ok(1)`, exit 124, `grep -icE "SIGSEGV|SIGABRT"` = 0. This does NOT prove the mesh/stride ABI end-to-end beyond the single quad (secondary-attrib stride is unchanged from SH67b) — it proves the engine's own emitter draws multiple isolated elements into one presented frame headlessly.

**Honest scope:** proves the ENGINE's own draw path (primitive_setup→glDrawArrays via real Mesa) rasterizes a genuinely populated, multi-element frame in the live engine context, presented via the real swap. It does NOT prove a real GUI gesture (fabricated G, not a GuiObject; no real session).

## 4. If the single-call path still dead → best alternative (per-node walker integration: DEAD BY CONSTRUCTION)

Routing the emitter `0x105b35288` as a scene node's render-obj `vt[+24]` in the present walker `0x105b2eec0` (x19=R) is **not possible without reopening the SH64 desync SIGSEGV**: the emitter is guest code, so calling it from inside the walker's executing block as the per-node draw requires a nested `jit_run`/`run_guest_callback` → `clear_block_cache()` evicts the executing present-loop block → crash. The SH64/SH65 invariant is absolute: the per-node draw MUST be the pure-host registered thunk (`walker_item_draw_thunk` via `host_call_at`), which cannot jit_run the guest emitter. So the emitter cannot be a per-node draw; it must remain a **separate top-level `jit_run` on the renderinit thread after each walker** (exactly the current SH67b architecture).

**Therefore the unblocked ladder toward "real client renders its own screens":** keep the emitter as a standalone top-level run, and feed it scene-shaped content — fabricate N layered rect quads that mirror a real login/home layer (background + button + text rects) via the single GL_TRIANGLES call above, and (optionally) read the per-node scene list (R+0x180/0x188) to size/place them per populated node. The real wall (Lua app-shell / nativeGameGlobalInit parks / no in-image GuiObject→scene append) is unchanged.

## Recommended next frontier (ranked by value / effort)
1. **Single-call GL_TRIANGLES multi-quad emitter frame (this artifact, §3).** Low effort (reuse existing builder), closes the multi-element gap, no new desync risk. Value: highest reachable.
2. **Scene-shaped emitter content.** Feed 6 tiles as a login/home-style layout (layer quads), textured later (needs GL_TEXTURE upload — same one-time-pre-upload discipline). Medium effort.
3. **Per-node geometry sizing from the real scene list** (R+0x180 head/tail) → emitter placement mirrors populated nodes. Low-medium, but builds on #1/#2.
4. Per-node emitter `vt[+24]` integration — **do not attempt** (desync-class SIGSEGV, §4). Defer a JIT GLES-draw trace of the orphan mechanism only if #1 surprisingly fails; the single-call design should make it moot.