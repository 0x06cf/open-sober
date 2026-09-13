# SH68 — next artifact: engine-emitter LAYERED "login/home" frame with GL_BLEND compositing, sized from the real scene list

Read-only disasm (deleg_e63d6f7e). Extends `render_engine_emitter_grid` (elfjit.rs:1329) + `emitter_tex_program()` (:1271). Guest = file vaddr + 0x100000000.

## (A) Read the scene-list node count to drive the layout — YES, exact additions

The harness already holds everything: `render_scene_base(node_count)` (:280) leaks R and stashes statics; every `--renderemitter` iteration runs with a live scene list.

- **Existing statics:** `RENDERSCENE_BASE: AtomicU64` (:91) — the leaked R base (guest==host, identity-mapped); `SCENE_NODES: AtomicU64` (:96) — laid node count, Relaxed.
- Add inside the layout branch:
```rust
let r = RENDERSCENE_BASE.load(core::sync::atomic::Ordering::Relaxed);
let n_scene = SCENE_NODES.load(core::sync::atomic::Ordering::Relaxed);
let head = unsafe { *(r as *const u64).add(0x180/8) };
let tail = unsafe { *(r as *const u64).add(0x188/8) };
let derived = tail.wrapping_sub(head) / 0x28;
```
`R+0x180`=head, `R+0x188`=tail (render_scene_base :324-325), 0x28-stride one-past-end. Guard: if `n_scene==0` fall back to env `RENDEREMITTER_LAYER_NODES` default 4. `n_scene` drives the layout: 1 node ⇒ backdrop only; 4+ ⇒ full home stack; N>4 ⇒ one text-strip per surplus node, sized per-node.

## (B) Exact emitter draw — 5 layered quads, one GL_TRIANGLES emit, blend on

Same `tex=true` path as SH67e (stride 32 = pos2@0 + color4@8 + uv2@24; spec[3] at spec+48, spec end M+0x50=spec+72, stride_tab[+16]=32, BD G+0x68=bd0, G+0x78=0, G+0x8e=0). **One VBO, 5 quads × 6 verts = 30 verts, one `jit_run 0x105b35288` with x1=0(GL_TRIANGLES), x2=0, x3=0, x4=30, x5=0.** Triangle order = painter's order.

**Blend insertion** — in the normalize step (elfjit.rs:1506), replace `ds(0x0BE2)` for the home branch:
```rust
let enable: Option<extern "C" fn(u32)> = mesa_fn(h, b"glEnable\0");
let bfs: Option<extern "C" fn(u32,u32,u32,u32)> = mesa_fn(h, b"glBlendFuncSeparate\0");
if let Some(e)=enable { e(0x0BE2); }                       // GL_BLEND
if let Some(bf)=bfs    { bf(0x0302,0x0303,0x0302,0x0303); } // S_ALPHA, ONE_MINUS_SRC_ALPHA
```
The emitter sets no GL state — blend is host-set before the single emit, applies to the engine's glDrawArrays. Per-layer alpha lives in the TEXTURE (SH67e FS = texture2D only), so upload a **5-texel RGBA strip** (one solid per layer, straight alpha).

| layer | NDC x0,x1 | NDC y0,y1 | palette (straight RGBA) | uS |
|---|---|---|---|---|
| L0 backdrop | -1.0, 1.0 | -1.0, 1.0 | (0.10,0.10,0.12,1.00) | 0.1 |
| L1 panel | -0.55, 0.55 | -0.36, 0.42 | (0.24,0.26,0.32,0.55) | 0.3 |
| L2 button bar | -0.40, 0.40 | -0.30,-0.14 | (0.16,0.62,0.44,0.95) | 0.5 |
| L3a title strip | -0.40, 0.40 | 0.28, 0.34 | (0.90,0.88,0.80,0.80) | 0.7 |
| L3b field strip | -0.40, 0.10 | 0.10, 0.17 | (0.35,0.38,0.45,0.85) | 0.9 |

VBO push order = array order (backdrop→…→field), later layers composite over earlier. Panel alpha 0.55 makes every overlap a verifiable blend. **Expected readbacks** (blend src*src.a + dst*(1-src.a)):
- backdrop-only corner (fx,fy)=(51,691): rgb(26,26,31).
- **panel∘backdrop at (640,360): r=0.24·.55+0.10·.45=0.177, g=0.26·.55+0.10·.45=0.188, b=0.32·.55+0.12·.45=0.230 ⇒ rgb≈(45,48,59)** = the blend-equation proof (neither backdrop nor pure panel).
- button∘(above) (640,281): rgb≈(41,152,109).
- title (640,472): rgb≈(193,189,175).

## (C) Verify-marker sequence + install

Env `RENDEREMITTER_LAYOUT=home` (keep RENDEREMITTER_QUADS/TEX for the grid). Add a `layout: &str` arg to `render_engine_emitter_grid`; main dispatch sets it from env, forces tex=true when home. Run `--renderscene --renderwalker --renderemitter` on the currency thread (renderscene block runs first, laying R). Markers:
```
[elfjit:renderemitter-home] scene list R=0x.. n_scene=3 head=0x.. tail=0x.. derived=3 match=true
[elfjit:renderemitter-home] LAYOUT=home layers=5 blend=enabled(SRC_ALPHA,ONE_MINUS_SRC_ALPHA) tex 5x1 RGBA uploaded uTex loc=0 prog=0x..
[elfjit:renderemitter-home] engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=30) layers=5 swap=Ok(1)
[elfjit:renderemitter-home] backdrop   (51,691) rgba(26,26,31) diff<=1 present=true
[elfjit:renderemitter-home] panel-blend(640,360) rgba(45,48,59) expect~(45,48,59) blend=ok  <- the compositing proof
[elfjit:renderemitter-home] button     (640,281) rgba(41,152,109) present=true
[elfjit:renderemitter-home] title-bar  (640,472) rgba(193,189,175) present=true
present walker Ok(ret=0x1) ... exit 124, SIGSEGV/SIGABRT/json-overflow grep=0
```

## Honest scope

Proves the ENGINE's own primitive_setup→emitter rasterizes a single top-level multi-rect frame with non-uniform per-layer geometry, per-layer texture sampling, and GL_BLEND alpha compositing whose overlap readbacks match the blend equation — a statically-fabricated login/home-precursor, sized/placed from the real scene list (R+0x180/0x188 → SCENE_NODES). NOT a real GuiObject, NOT a self-populated render session: texture, program, blend are host-set (emitter has no such concepts); geometry authored; the scene list only drives quad count (engine never self-populates UI items). Standing structural wall unchanged.

## Ranked recommendation
1. Land SH68 layered blended layout — lowest effort, highest reachable proof: engine path does genuine UI-style alpha compositing.
2. Texture upgrade ETC2/ASTC — polish.
3. Do NOT attempt per-node emitter vt[+24] (SH64 desync). Real wall (self-populated session) out of reach statically.