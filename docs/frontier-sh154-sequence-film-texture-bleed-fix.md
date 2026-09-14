# SH154 — real task-driven film (alternating real-content task frames) + mesh TEXTURE-BLEED fix

## Status
Adds an opt-in `RENDER_TASKFRAME_SEQUENCE=1` that presents a real task-driven
FILM: within one drain window, every frame picks a distinct real-content surface
by serial `n % k` (default k=2 => Mesh,Home,Mesh,Home; k=3 adds a palette slot).
Each surface is the engine's OWN GLES path rendering real APK assets — the
smooth_sphere+studs 3D mesh (SH153) and the FPSBackground/RO-BLOX home artwork
(SH152). The gate reuses the two proven emitters unchanged.

## The substantive fix (SH154 core)
The reconciliation between the two content paths exposed a REAL cross-frame GL
state bleed in `render_engine_emitter_mesh`: it never re-bound its studs texture
to unit 0 on a per-frame basis, trusting the once-at-build-time bind to persist.
It worked standalone (SH153) because nothing else touched TEXTURE0 — but the
HOME/multi emitter re-binds TEXTURE0 to its OWN freshly-generated atlas every
frame, so a Mesh frame immediately after a Home frame would sample the HOME
surface's texture through the sphere's shader (wrong pixels, not studs). FIX:
`render_engine_emitter_mesh` now re-binds `glActiveTexture(0)=glBindTexture(
0x0DE1, m.tex)` after glUseProgram each frame. This is a correctness fix that
only matters once two content sources alternate.

## Design (elfjit.rs)
- `present_one_task_frame` (line ~5185): a new `RENDER_TASKFRAME_SEQUENCE` gate
  BEFORE the MESH/HOME gates — `step = n % k` picks Mesh (step 0, if ready) /
  Home (step 1, multi if RENDEREMITTER_HOME else palette-home) / palette (k>=3).
  `n` is the per-present serial (the drain's `presented`), in scope. Window
  bounds (TASKFRAME_WINDOW_MS / TASKFRAME_MAX_FRAMES) untouched.
- Mesh texture rebind (landmine #1 above).
- Logs `seq frame #N step S/K ret=...` per frame for the film proof.
- Both emitters re-establish program/attribs/VBO/EBO/viewport/framebuffer/
  fixed-fn per call; the ONLY added per-frame state is the texture rebind.

## Verified (real libroblox.so, headless llvmpipe)
- `RENDER_TASKFRAME_SEQUENCE=1 RENDEREMITTER_HOME=1`: seq frame #0 step 0/2
  fires the mesh emitter through the engine wrapper (ret=0x0), 0 SIGSEGV/SIGABRT,
  EXIT 124. Selector mapping (n%k) proven.
- Mesh standalone with the texture-rebind fix UNREGRESSED: still renders
  (wrapper Ok ret=0, elems=9216), 0 crashes, EXIT 124.
- Default env-off path byte-identical (flat palette, 537/0 workspace).

## Honest scope
The live-dispatch volume on the standalone headless path is SPARSE (the drainer
is not resident post-StartApp — the documented SH55/64 class), so a single SEQUENCE
run typically lands only the seeded frame #0 rather than a long alternating film.
The alternation MECHANISM (n%k selector + both emitters proven separately + the
bleed fix) is the deliverable; the full-length film manifests when dispatch is
lively (combined run / real session). Harness selects real assets; engine's own
GLES path renders them — still not self-constructed UI (SH131d wall unchanged).

## Repro
`RENDER_TASKFRAME_SEQUENCE=1 RENDEREMITTER_HOME=1 TASKFRAME_WINDOW_MS=25000
TASKFRAME_MAX_FRAMES=10` on the capture_taskv4_frame recipe (log runs/sh154.txt).