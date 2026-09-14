# SH142 — real APK texture (studs.dds) on real mesh geometry through the engine's GLES path

## One-line result
`parse_roblox_dds_r8` decodes a real Roblox material map (DDS DDPF_LUMINANCE R8)
and the harness uploads it via glTexImage2D and samples it **over the real
MeshRenderManager preview sphere** (`smooth_sphere.mesh`) — the first real APK
texture data bound to real in-world mesh geometry through the engine's own
GLES bridge. Workspace green; elfjit example 48/0.

## What
Building on SH141 (real `.mesh` geometry through engine wrapper 0x105b35288),
SH142 binds a REAL material texture:

- `parse_roblox_dds_r8(&[u8]) -> Option<(u32,u32,Vec<u8>)>` — pure-std DDS parser
  for plain single-channel R8 (DDPF_LUMINANCE, 8-bit, rmask=0xff, no FourCC).
  Bounds-checked; rejects compressed/FourCC surfaces (those need the texture
  codec, not an R8 expand).
- New `--renderframe-mesh-tex <dds>` lever (with `--renderframe-mesh`): R8 ->
  RGBA expand (v,v,v,255), upload via the 9-arg glTexImage2D bridge, and a
  texture-sampling fragment shader whose screen-space UV reads the studs atlas.
- The mesh-mode silhouette probe is widened from "red-only" to "differs from the
  cleared background" so both the solid-red and grayscale-textured renders score.

## Verification
- **Hermetic (+2):** parses the real `studs.dds` (128x2048 R8, 262144 bytes,
  real luminance variation >40); rejects non-DDS and FourCC-compressed surfaces.
- **Real binary** (`runs/capture_mesh_tex.sh`, smooth_sphere.mesh + studs.dds):
  `parsed real mesh ... 1652 verts, 3072 faces (9216 idx)` +
  `parsed real DDS "studs.dds": 128x2048 R8 (262144 bytes)`, geometry wrapper
  Ok(0x0), **silhouette 5x5 = 25/25 drawn**, pixel histogram shows ~400k pure
  gray pixels (RGB 128-129) = the real studs R8 luminance over the sphere,
  0 crash. Captured runs/sh142-mesh-tex.png.

## Scope / honesty
Real APK pixel data (studs R8 material map) rendered on real mesh geometry
(sphere) through the engine's own geometry wrapper + GLES bridge. The UV mapping
is screen-space (single-attribute coherent renderer), so the studs atlas shows
its raw banding rather than a true sphere wrap — that's an honest limitation of
reusing the proven single-attribute path. Real camera/MVP + per-object UV remain
harness-walled (SH131d structural wall unchanged; the SH141/142 work is the
render-plane correctness bed for real in-world content).