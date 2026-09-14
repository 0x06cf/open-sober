#!/bin/bash
# SH153: type-4 task-driven frames carry a REAL 3D mesh (smooth_sphere.mesh +
# studs.dds, 9216 indexed triangles) through the engine's OWN geometry wrapper
# 0x105b35288 — the SH151-sanctioned cached-program design (program/VBO/EBO/
# texture/geometry-ctx built ONCE outside the present loop via pure-host
# mesa_fn; per-frame only uniform upload + engine-wrapper jit_run, no nested
# guest-bridge GLSL). Default env-off path stays the proven flat-palette plane.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh153-taskframe-mesh.txt
rm -f "$LOG" /tmp/sh153-mesh.raw
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  RENDER_TASKFRAME_MESH=1 \
  TASKFRAME_WINDOW_MS=20000 TASKFRAME_MAX_FRAMES=8 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --taskv4-seed frame \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== cached mesh built once (warm-up) ==="
grep -E "taskframe-mesh\] built cached" "$LOG"
echo "=== task-driven REAL MESH frames (SH153 marker) ==="
grep -E "task frame #.*REAL MESH" "$LOG"
echo "=== engine wrapper draw (indexed glDrawElements) ==="
grep -E "taskframe-mesh\] engine wrapper Ok" "$LOG"
echo "=== crash summary (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"