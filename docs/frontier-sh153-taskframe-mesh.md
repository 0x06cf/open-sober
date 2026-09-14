# SH153 — REAL 3D mesh (smooth_sphere + studs) inside type-4 task-driven frames

## Status
The self-driven task-frame plane now presents a **real Roblox 3D mesh** — the
MaterialManager preview sphere `smooth_sphere.mesh` (1652 verts / 9216 indexed
triangles) textured with `studs.dds` (128x2048 R8) and lit (diffuse + specular)
— through the engine's OWN geometry wrapper `0x105b35288`, driven by type-4
task dispatch. This closes SH151's ORIGINAL intent (real mesh geometry in task
frames) via the safe "already-current program" alternative SH151 itself
sanctioned. Flat-palette default unchanged.

## Design (elfjit.rs)
- **Cached once, outside the present loop** (`once` CachedTaskFrameMesh /
  `taskframe_mesh()`): a PURE-HOST mesh program (aPos/aUV/aNormal + uMVP +
  uModelRot + uTex, Blinn-Phong FS over the studs texel), the stride-36
  interleaved VBO (`mesh_interleave_model_uv`), the index EBO, the studs R8→RGBA
  texture, and the coherent geometry-ctx (desc/stride/prims/ibo) all built via
  `mesa_fn` (dlopen libGLESv2) in the renderinit thread context.
- **Per task frame** (`render_engine_emitter_mesh`): make-current, bind program,
  upload uMVP/uModelRot/uTex + attrib pointers, drive the engine wrapper as a
  SEPARATE top-level jit_run (indexed path: x5=n_elems nonzero), glFinish, swap
  via real ctx-vt[+24]. NO nested guest-bridge GLSL compile / GL allocation.
- **Env gate `RENDER_TASKFRAME_MESH=1`** (takes precedence over
  `RENDER_TASKFRAME_HOME`); readiness gated on `taskframe_mesh().is_none()` (the
  wrapper returns Ok(ret)=0 on a CLEAN draw, so return value ≠ readiness). Env-
  off default is byte-identical.
- `RENDER_TASKFRAME_MESH_RAW=1`: dump the frame back-buffer to
  /tmp/sh153-mesh.raw (GL_RGBA8 1280x720) for analysis/capture.

## Verified (real libroblox.so, headless llvmpipe)
- `RENDER_TASKFRAME_MESH=1`: task-driven frames log
  `task frame #N REAL MESH (engine wrapper) ret=0x0`, each driving the wrapper
  with `elems=9216` (real indexed glDrawElements through the engine's own path).
  Two distinct mesh task frames verified; 0 SIGSEGV/SIGABRT; EXIT 124 (clean).
  No palette-preset, no multi-emitter (MESH branch wins, no fallback firing).
- **Frame analysis** (1280x720 RGBA capture): 19.80% sphere-drawn pixels
  (182,484 — matches the SH144/145 sphere footprint), mean luminance 136,
  stdev 69 = genuine studs texture + diffuse shading gradient (NOT flat), with
  a concentrated bright specular highlight. Vision confirms the shaded sphere.
- Default (env unset): EXIT 124, 24 task frames, 0 crashes — unregressed.

## Why this isn't the SH151 negative
SH151 hard-failed because it compiled GLSL + allocated GL objects via NESTED
guest-bridge calls inside the frame-fn callback (heap corruption). SH153 builds
everything once via pure-host `mesa_fn` OUTSIDE the present loop and reuses it
(cached program + pre-uploaded VBO/EBO/texture), so each frame is only uniform
upload + engine wrapper jit_run — the sanctioned safe pattern proven first in
SH152 with the emitter home path.

## Honest scope
Harness selects the real assets and geometry; the engine's own wrapper +
GLES bridge render them. This is a real engine-rendered 3D object in a task-
driven frame, but remains harness-authored (SH131d structural wall: engine
self-constructed login/home unchanged).

## Repro
`runs/capture_taskv4_frame_mesh.sh` (log `runs/sh153-taskframe-mesh.txt`).
In-repo capture: `runs/sh153-mesh-frame-640.png`; full 1280 on disk
(`/tmp/sh153-mesh.raw`, `/home/hermes-worker/runs/sh153-mesh-frame-1280.png`).