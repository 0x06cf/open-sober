# SH143 — Real per-vertex UV + real camera/MVP on the engine's own GLES path

**Goal:** the ranked next render-plane ascent over SH142. SH142 sampled a real
APK texture (studs.dds R8) over real mesh geometry (smooth_sphere.mesh) but with
SCREEN-SPACE planar UV (`gl_FragCoord`), no per-vertex UV, and positions
NDC-baked (pass-through VS). SH143 replaces all three:
- REAL per-vertex UVs parsed from the .mesh are uploaded and interpolated.
- A real perspective camera/model-view-projection (MVP) is computed from the
  mesh bbox + a fixed axis-aligned camera and uploaded as a shader uniform.
- Model-space positions are transformed `gl_Position = uMVP * aPos` at draw time
  (no NDC baking).

## Changes (crates/arm64jit/examples/elfjit.rs)

1. **New pure helpers** (near the mesh parser):
   - `mesh_bbox(m) -> ([f32;3], f32)` — bbox center + max extent.
   - `mesh_positions_model(m) -> Vec<f32>` — raw model-space vec4 (not NDC-baked).
   - `mesh_uvs(m) -> Vec<f32>` — per-vertex [u,v], parsed but previously dropped.
   - `mat4_perspective / mat4_translate / mat4_mul` — column-major OpenGL helpers.
   - `mesh_interleave_model_uv(m, fov, aspect, &mut mvp)` — interleaves
     [px,py,pz,1, u,v] per vertex (stride 24) AND writes the column-major MVP
     (P*V*M: translate-to-center * translate(-d along z) * perspective), camera
     at (0,0,+d) framed on the bbox.

2. **Shader switch**: when `--renderframe-mesh` AND `--renderframe-mesh-tex` are
   both present (`mesh_uv_mode`), the VS becomes
   `gl_Position = uMVP * aPos; vUV = aUV;` and the FS samples `texture2D(uTex,vUV)`
   (real per-vertex UV) instead of the screen-space sampler. `aUV` bound to slot 1.

3. **Stride-24 VBO 2-primitive renderer**: the coherent renderer uses
   stride_tbl[0]=24 and TWO primitives: attrib0 = aPos (offset 0, fmt3 size4),
   attrib1 = aUV (offset 16, fmt1 size2 float) — the proven quad pattern.

4. **MVP uniform upload**: `glUniformMatrix4fv` resolved via the int resolver's
   Mesa binding (pure-int+pointer ABI: loc,count,transpose,ptr) with transpose=0
   (column-major), 16 floats from the bbox-derived MVP.

## Verification (real libroblox.so, runs/capture_mesh_uv.sh)

- `smooth_sphere.mesh` parsed: 1652v / 3072f / 9216 idx.
- `studs.dds` parsed: 128x2048 R8 (262144 bytes).
- `real MVP uploaded to uMVP loc=0x0 (1652 verts interleaved stride-24)`.
- Geometry wrapper `0x5b35288` returned Ok(0x0) — real indexed glDrawElements.
- **Silhouette 5x5: 24/25 drawn** — the sphere is now FRAMED by the perspective
  camera (not edge-to-edge as SH142's planar full-frame), a real projection.
- **Centroid readback RGBA(129,129,129,255)** — real studs luminance via real
  per-vertex UV (not the screen-space atlas strip).
- Non-background pixels 100% grayscale (real R8 studs data, no color bleed);
  corners read clear-color (0,0,76).
- 0 SIGSEGV/SIGABRT; 4 swaps Ok(0x1). Capture: runs/sh143-mesh-uv.png.
- +4 hermetic tests (sh143_mat4_translate_places_rows, _perspective_standard,
  _mvp_maps_origin_is_finite, _mesh_bbox_center_and_extent). Workspace green.

## Honesty / scope

Host-authored VBO/shaders through the ENGINE'S OWN geometry wrapper + primitive
setup, with real APK mesh + texture data and a real camera transform. This is
the render-plane ascent (per-object UV + camera/MVP), NOT engine self-constructed
session UI (that remains behind the confirmed STRUCTURAL Lua app-shell wall,
SH131d). It is the first real projection-camera draw of a real Roblox asset.