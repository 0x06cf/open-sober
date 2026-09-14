#!/bin/bash
# SH152: type-4 task-driven frames carry REAL home artwork (FPSBackground +
# RO-BLOX wordmark) through the engine's OWN geometry emitter 0x105b35288,
# presented via the real ctx. This is the SH151-sanctioned "already-current
# program" design: the emitter uses a CACHED textured program + pre-uploaded
# texture as a SEPARATE top-level jit_run (no nested guest-bridge GLSL compile,
# the class that SIGABRT'd in SH151). The flat-palette task-frame plane is
# preserved: omitting RENDER_TASKFRAME_HOME returns the proven 24-frame path.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh152-taskframe-home.txt
rm -f "$LOG"
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  RENDER_TASKFRAME_HOME=1 RENDEREMITTER_HOME=1 \
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
echo "=== task-driven REAL-CONTENT frame present (SH152 marker) ==="
grep -E "taskv4-frame\] task frame #.*(REAL HOME ARTWORK|PALETTE HOME)" "$LOG"
echo "=== the multi-emitter emitted the real artwork + swapped ==="
grep -E "renderemitter-multi\] engine emitter Ok" "$LOG"
echo "=== real FPSBackground probe readback (a REAL artwork texel) ==="
grep -E "probe 'FPSBackground.png'" "$LOG" | head -3
echo "=== crash summary (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"