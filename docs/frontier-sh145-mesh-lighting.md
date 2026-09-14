# SH145 — diffuse LIGHTING on the real studs-textured sphere

**Goal:** the only remaining empty mesh render-plane cell. The mesh parser
already produced per-vertex normals (RbxMeshV2.normals @ 2697, parsed @ lef32
voff+12/16/20) but the renderer DROPPED them. SH145 uploads a 3rd attribute
(aNormal, loc2), computes a world-space normal in the VS via
`mat3(uModelRot) * aNormal`, and shades with diffuse `dot(N,L)` in the FS:
`color = tex * (0.30 + 0.70 * max(dot(N,L), 0.0))`.

## Changes (crates/arm64jit/examples/elfjit.rs)

1. `mesh_interleave_model_uv` now emits stride-36 `[px,py,pz,1, u,v, nx,ny,nz]`
   (9 floats/vert) with real per-vertex normals (or a [0,0,1] default when the
   mesh has none, keeping a uniform stride). New `mesh_normals` helper.
2. Coherent renderer: THIRD primitive — attrib2 = aNormal at byte offset 24,
   fmt2 = {size3, GL_FLOAT} (verified against the real format table @0x100cecf8c:
   format[2]={3, 0x1406, 0}); stride 16->36. Bind aNormal->slot2.
3. VS: `vN = mat3(uModelRot) * aNormal`; new uniform `uModelRot` (model
   rotation, identity for the static view; yaw-matching under orbit so normals
   stay world-aligned and lighting tracks the rotating object).
4. FS: diffuse `dot(N,L)` with fixed world-space light dir = normalize(0.4,0.7,0.6)
   and ambient 0.30 + diffuse 0.70.
5. Orbit loop (SH144) also uploads the yawed uModelRot each frame.

## Verification (real libroblox.so, runs/capture_mesh_lit.sh)

- Mesh + DDS parsed (1652v / 128x2048 R8).
- `real MVP + uModelRot uploaded (uMVP loc=0x0 uModelRot loc=0x1; 1652 verts
  interleaved stride-36, normals lit)` — BOTH uniforms resolved.
- Geometry wrapper Ok(0x0), silhouette 5x5 **24/25 drawn**, 4 swaps Ok, **0 crash**.
- **Real lighting gradient**: sphere luminance mean=72.4, stdev=34.3, range up to
  255 — a genuine directional falloff (bright upper-left lum~116 -> dark
  bottom/edges lum~38) instead of SH143's FLAT gray. Vision confirms: bright top,
  darker bottom + edge falloff, studs pattern warping over the 3D surface.
  Capture runs/sh145-lit.png.
- +1 hermetic test (sh145_interleave_includes_real_per_vertex_normals asserts
  stride-36 with the real normal bytes). Workspace green (537/0 + sh14 example 14/0).

## Honesty / scope

Host-authored VBO/shaders + real APK mesh + texture + normals + lighting through
the engine's own geometry wrapper 0x105b35288. NOT engine self-constructed UI
(structural SH131d wall unchanged). This completes the mesh render-plane cell
matrix (geometry / UV / camera-MVP / orbit / lighting); the next ascent is
multi-mesh compose (several real .mesh objects with per-object transforms +
lighting in one frame).