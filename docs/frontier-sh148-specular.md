# SH148 — Blinn-Phong SPECULAR highlight on the real studs sphere

**Goal:** complete the full lighting model (ambient + diffuse + specular) on the
real smooth_sphere.mesh + studs.dds R8 through the engine's own geometry wrapper
0x105b35288 — the natural render-plane completion after SH145's diffuse term.

## Changes (crates/arm64jit/examples/elfjit.rs)

The mesh-uv fragment shader gains a specular term:
```glsl
precision highp float;                           // (mediump underflows pow^32 below ~0.7 — was washing the highlight out)
vec3 L = normalize(vec3(0.4, 0.7, 0.6));         // world light
vec3 V = vec3(0.0, 0.0, 1.0);                    // distant viewer toward +Z (camera)
vec3 H = normalize(L + V);                       // Blinn half-vector
float spec = pow(max(dot(n, H), 0.0), 32.0);     // tight glossy exponent
gl_FragColor = vec4(t.rgb * (0.45 + 0.45*d) + vec3(0.90*spec) + vec3(0.03), 1.0);
```
The exponent (32) is high enough to be a localized glint but low enough that
`highp` represents `dot^32` without underflow (verbatim `mediump` dropped it).

## Verification (real libroblox.so, runs/capture_mesh_lit.sh sh148)

- Mesh (1652v) + DDS (128x2048 R8) parsed; `real MVP + uModelRot uploaded`.
- Geometry wrapper Ok(0x0), silhouette 5x5 24/25, 0 SIGSEGV/SIGABRT.
- **Specular highlight VERIFIED by vision**: a concentrated bright glint in the
  sphere's upper-right quadrant — pure white core softening to gray, distinctly
  brighter and more localized than the surrounding diffuse gradient (a classic
  Blinn-Phong hotspot, not Lambertian). Capture runs/sh148.png.
- (The earlier attempt with `vP`/mediump showed only a smooth gradient; `highp`
  + a constant distant-viewer V produced the visible glint.)

## Honesty / scope

Host-authored shaders + real APK mesh/normals/texture through the engine's own
wrapper. This completes the standard lighting model on the real material-preview
pair — an engine-rendered glossy sphere. NOT engine self-constructed UI
(structural SH131d wall unchanged).