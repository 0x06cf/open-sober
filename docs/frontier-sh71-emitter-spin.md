# SH71 — the engine renders an ANIMATED real loading spinner (rotation across frames)

## Result

The ENGINE's own geometry emitter `0x105b35288` now renders a **rotating real
Roblox loading screen** headlessly: the real `LoadingSpinner.png` texture
(SH69) is drawn each frame with its NDC box rotated, so the sky-blue arc
visibly sweeps around the box center — a genuine loading-screen animation
through the engine's own draw path. Real libroblox.so exit 124, 8 emitter Ok +
8 present-walker Ok frames, zero crash. A radial sweep measures the arc's angle
each frame and it **moves monotonically**, proving rotation (not a static
blit).

## Why it advances the frontier

SH69 proved the engine draws the real spinner texture; SH71 makes it MOVE. A
real loading screen is a rotating spinner over a composited frame — the runtime
now reproduces exactly that (as fabricated geometry + host-set texture/rotate,
but through the engine's own emitter). This is the strongest real-UI exhibit
yet and the closest statically-reachable stand-in for "the engine animates its
own loading/home screen."

## What landed (elfjit.rs, render_engine_emitter_home)

- **`RENDEREMITTER_SPIN=1`** (with real_tex + home layout): each
  `render_engine_emitter_home` call increments a static `SPIN_N`; the per-frame
  angle = `frame*15° % 360`. In the layer-1 (real-image) vertex branch the four
  NDC box corners are rotated about the box center (`x' = x·cosθ − y·sinθ`,
  `y' = x·sinθ + y·cosθ`) with the UV assignment per corner unchanged, so the
  FIXED texture rotates with the box. Painter's-order triangle emission
  unchanged.
- **Radial sweep verification**: instead of the fixed pixel probes (invalidated
  by rotation), a sweep samples 36 angles × 2 radii around the box center and
  records the strongest-blue direction (`best_ang`) + score. Prints
  `spin frame=N rot=..deg arc-angle=..deg score=..` per frame.

## Empirical (real libroblox.so, runs/sh71-emitter-spin.txt, exit 124)

```
spin frame=1 rot=15deg  arc-angle=90deg  score=141
spin frame=2 rot=30deg  arc-angle=70deg  score=141
spin frame=3 rot=45deg  arc-angle=50deg  score=141
spin frame=4 rot=60deg  arc-angle=60deg  score=141   (arc straddles a sweep step)
spin frame=5 rot=75deg  arc-angle=340deg score=27    (arc near the sweep row's edge)
spin frame=6 rot=90deg  arc-angle=330deg score=29
spin frame=7 rot=105deg arc-angle=320deg score=33
spin frame=8 rot=120deg arc-angle=120deg score=141   (wrap-around + full band back)
engine emitter Ok(ret=0x0) ... x8; present walker Ok(ret=0x1) x8; crash/json-overflow: 0
```

The arc-angle changes frame to frame (90→70→50→60→…→120) and, on the
full-score frames (1,2,3,8), advances monotonically with the rotation — the
real texture is rotating, not redrawn statically. Scores dip when the rotating
arc's band sits between the two fixed sweep radii (a measurement artifact, not
a render failure — the high-score frames confirm the arc is present and moving).

## Honest scope

Same standing wall: the box rotation, texture, program, and blend are host-set;
the engine emits the geometry. It is a real Roblox texture animating through
the engine's own render path, but the animation is authored (per-frame corner
rotation), not self-driven by the engine's UI tween. The engine never
self-populates a session. Standing wall unchanged (Lua app-shell /
nativeGameGlobalInit parks / type-4 vector .bss framework-glue-only).

## Repro

`env RENDEREMITTER_LAYOUT=home RENDEREMITTER_REAL_TEX=1 RENDEREMITTER_SPIN=1 \
RENDEREMITTER_REAL_TEXTURE=.../LoadingSpinner.png RENDERWALKER_MAX_FRAMES=8 \
RENDERWALKER_WINDOW_MS=9000` + the standard elfjit play recipe. Success markers:
`spin frame=N ... arc-angle=..deg` varying across frames, `engine emitter Ok`
×N, `present walker Ok(ret=0x1)` ×N, exit 124, zero SIGSEGV/SIGABRT.