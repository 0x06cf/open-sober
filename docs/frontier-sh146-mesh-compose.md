# SH146 — composed real-avatar scene: N distinct Roblox meshes in ONE frame

**Goal:** render N DISTINCT real Roblox `.mesh` objects (avatar torso + head +
MaterialManager sphere) in a single frame, each with its OWN VBO/EBO, per-object
compose-MVP (translate+scale+yaw), and SH145 diffuse lighting, all driven
sequentially through the engine's own geometry wrapper 0x105b35288 — a real
composed 3D scene (the recon-identified next headless cell, deleg_046e7f15).

## Changes (crates/arm64jit/examples/elfjit.rs)

1. `mesh_compose_mvp_shared(center, scene_ext, origin, scale, yaw, fov, aspect)`
   — a SHARED-scene-camera compose MVP: `M = T(origin)*S(scale)*Ry(yaw)*T(-center)`,
   with the camera distance derived from the MAX scene extent across all objects
   so every object is viewed from the same distance and world-space origins
   spread them correctly (a coherent composed frame, not per-object framing).
2. `--renderframe-mesh-compose <p1>,<p2>,...` — a new opt-in block that:
   - Pre-scans all objects to compute the shared scene extent.
   - For each object: parse (v2 mesh), interleave stride-36 (pos+uv+normal,
     reused from SH145), build per-object leaked VBO/EBO, re-fabricate the
     coherent renderer descriptor (per-object VBO/EBO/count), upload the
     compose-MVP + uModelRot, drive the engine's own geometry wrapper
     0x105b35288 once, and probe a per-object 3x3 silhouette.
   - One swap presents the full composed frame.
   - Reuses the mesh-uv program built by `--renderframe-mesh` + `--renderframe-mesh-tex`
     (REQUIRED flags — the compose block depends on that program + its uniform locs).

## Verification (real libroblox.so, runs/capture_mesh_compose.sh)

Objects: CompositTorsoBase.mesh (664v/416f), head.mesh (517v/846f),
smooth_sphere.mesh (1652v/3072f).

- `mesh-compose] obj 0/1/2` each: `wrapper=Ok(0)` + silhouette **9/9** — the
  engine's own geometry wrapper drew real geometry for every distinct mesh.
- `3/3 real objects composed in one frame, silhouettes=[9/9, 9/9, 9/9], swap=Ok(1)`.
- Frame analysis (correct bg exclusion): geometry across all 3 spatial thirds —
  left 5,178 px @ lum 110.9, center 467 px, right 970 px @ lum 88.8 — real lit
  content (the clear-bg comparison false-positive from b=36 is excluded).
- Vision: distinct blocky gray torso (left, grooved) + studs sphere (right, clear
  spherical lighting) — a real composed lit avatar scene.
- 0 SIGSEGV/SIGABRT. Capture runs/sh146-compose.png.
- +1 hermetic test (sh146_compose_places_objects_at_distinct_origins: two objects
  at +-3 map to separable ndc x). Workspace green.

## Honesty / scope

Host-authored per-object VBO/EBO + shaders + real APK mesh assets + lighting,
each driven through the engine's own geometry wrapper. NOT engine self-constructed
UI (structural SH131d wall unchanged). This is the composed-scene completion of
the mesh render-plane (single object -> multi-object world).