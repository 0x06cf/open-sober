# SH144 — camera-orbit: render the real studs-textured sphere from a rotating camera

**Goal:** extend SH143's real MVP with a **camera orbit** — render the real
smooth_sphere.mesh (1652v) + studs.dds R8 texture N times, each at a different
model yaw about +Y, through the engine's OWN geometry wrapper
0x105b35288 -> primitive-setup -> indexed glDrawElements. A real, rotating,
textured 3D scene — NOT harness-fabricated per-frame content (same VBO/EBO, only
the MVP uniform changes per frame).

## Changes (crates/arm64jit/examples/elfjit.rs)

1. `mat4_rotate_y(ang)` — column-major right-handed rotation about +Y.
2. `mesh_orbit_mvp(center, ext, fov, aspect, yaw)` — recompute the full
   P * V * Ry(yaw) * T(-center) MVP (same bbox-framed camera as SH143, model
   yawed). The bbox center+extent are captured once at parse (mesh_center /
   mesh_ext), and mesh_interleave_model_uv now takes a `yaw_rad` arg (0 =
   SH143 head-on).
3. New flag `--renderframe-mesh-uv-orbit <N>` (only active in mesh_uv_mode): for
   each of N frames, clear to a distinct dark backdrop, re-upload the yawed MVP
   via glUniformMatrix4fv (transpose=0, the int-resolver Mesa binding), re-drive
   the engine's own geometry wrapper over the SAME stride-24 VBO/EBO, and swap
   on the real ctx — the sphere visibly rotates.
4. MVP uMVP program/location saved (mesh_uv_program / mesh_uv_mvp_loc) so the
   orbit loop re-uses the already-linked program.

## Verification (real libroblox.so, runs/capture_mesh_uv_orbit.sh, N=4)

- Mesh + DDS parsed (1652v / 128x2048 R8).
- `real MVP uploaded to uMVP loc=0x0`.
- **All 4 orbit frames rendered**: yaw=0.00/1.57/3.14/4.71 rad, each
  `wrapper=Ok(0) swap=Ok(1)` — engine's real indexed draw + swap on every frame.
- Silhouette 5x5 **24/25 drawn** maintained through the rotation.
- **Two captured frames differ by 85.3% of pixels**; both show the real sphere
  (182,484 geometry px, 100% grayscale studs luminance) with the distinct orbit
  clear colors (20,20,76 vs 10,10,76). Vision confirms the studs pattern is
  oriented differently between frames (head-on vs rotated grooves) — real rotation.
- 0 SIGSEGV/SIGABRT. Capture: runs/sh144-orbit-f0.png / f1.png.
- +2 hermetic tests (sh144_mat4_rotate_y_unit_axes, sh144_orbit_mvp_rotates_and_frames).
- SH141 solid-red mesh path verified UNREGRESSED (309,707 red px / 33.61%).

## Honesty / scope

Host-authored VBO/geometry + real APK mesh + texture + real MVP, driven through
the engine's own GLES wrapper with camera motion. NOT engine self-constructed UI
(structural SH131d wall). The orbit frames share one VBO/EBO (only the uniform
changes) — genuine rotating real-textured geometry, not per-frame authored
content.