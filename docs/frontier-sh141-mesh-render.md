# SH141 — the engine's own wrap renders REAL Roblox mesh geometry

## One-line result
`parse_roblox_mesh_v2` decodes the APK's real binary `version 2.00` mesh assets
and the coherent renderer + engine geometry wrapper `0x105b35288` render a
**real in-world/avatar mesh** — the first non-host-authored 3D geometry through
the GLES bridge. Workspace green; elfjit example 46/0.

## What
The existing `--renderframe-triangle` path fabricated a *coherent* renderer
(real 1-primitive list, vertex-descriptor + stride tables, IBO) plus real GL
resources and drove the engine's own geometry wrapper. SH141 parameterizes it
with **real Roblox `.mesh` v2 assets** from the APK (`content/avatar/...`, `content/models/...`)
instead of the hand-written 3-index triangle:

- `parse_roblox_mesh_v2(&[u8]) -> Option<RbxMeshV2>` — pure-std parser for the
  Roblox binary mesh format (`version 2.00\n` + FileMeshHeaderV2: u16 sh=12,
  u8 vertStride(36 no-RGBA / 40 with-RGBA), u8 faceStride=12, u32 nVerts,
  u32 nFaces; then per-vert pos[3]f32/norm[3]f32/uv[2]f32/tangent[4]i8/color[4],
  then u32 triangle index triples). Bounds-checked; validated byte-exact against
  every real asset (`13+12+nv*sv+nf*12 == filesize`).
- `mesh_positions_to_ndc(&mesh, fit)` — center + uniform-scale model-space
  positions (cm) into NDC so the pass-through VS shows the whole shape.
- New `--renderframe-mesh <path>` lever feeds the resulting vec4 VBO + u32 EBO
  (real index count) into the existing coherent renderer, and the wrapper drive
  uses `n_elems` instead of the hard-coded 3.
- New mesh-mode silhouette probe: a 5x5 readback grid counts how many points are
  on the mesh interior, proving a real, bounded, non-degenerate footprint.

## Verification
- **Hermetic (+4 mesh_tests):** parses the real `CompositQuad.mesh` (4v/2f) and
  `CompositTorsoBase.mesh` (664v/416f, 1248 idx; positions span >100 cm, all
  finite, real UVs), and the NDC transform centers/scales a box to the fit
  fraction with w=1.
- **Real binary** (`runs/capture_mesh.sh`, standalone renderframe path):
  - `CompositTorsoBase.mesh`: `parsed real mesh ... 664 verts, 416 faces (1248
    idx)`, coherent renderer `elems 1248 mesh=CompositTorsoBase.mesh`, geometry
    wrapper `0000`, **309,707 red px (33.61% frame silhouette)**, 5x5 grid 7/25
    drawn, 0 crash.
  - `smooth_sphere.mesh` (MaterialManager preview sphere, 1652v/3072f, 9216 idx):
    wrapper `Ok(0x0)`, **521,972 red px (56.64%)**, 5x5 grid 13/25 drawn — a real
    circular silhouette (shown elliptical only because NDC x stretches to the
    wide viewport). Captures runs/sh141-mesh.png / sh141-sphere.png.

## Scope / honesty
This is host-*authored* geometry upload but real APK **mesh data** (real avatar
torso + real MaterialManager sphere), driven through the engine's own
primitive-setup + geometry wrapper + real indexed glDrawElements. It is NOT the
engine self-constructing in-world objects (the SH131d structural wall is
unchanged). It moves the render plane from "host triangle/quad/test textures" to
"real shipped Roblox 3D assets through the engine's GLES path" — the base the
next SH (bind a real material texture: `smooth_sphere.mesh` + `studs.dds`
R8→RGBA) builds on.