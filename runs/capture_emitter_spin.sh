#!/bin/bash
# SH71: reproduce the engine rendering an ANIMATED real loading spinner — the
# real LoadingSpinner.png NDC box is rotated per frame (RENDEREMITTER_SPIN=1) so
# the real texture visibly spins through the engine's own emitter path. A radial
# sweep measures the arc's angle each frame; it must ADVANCE across frames.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh71-emitter-spin.txt
rm -f "$LOG"
timeout 140 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=8 RENDERWALKER_WINDOW_MS=9000 RENDERWALKER_GLDEBUG=1 \
  RENDEREMITTER_LAYOUT=home RENDEREMITTER_REAL_TEX=1 RENDEREMITTER_SPIN=1 \
  RENDEREMITTER_REAL_TEXTURE=/home/hermes-worker/.cache/open-sober/android-env/assets/content/textures/ui/LoadingScreen/LoadingSpinner.png \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker --renderemitter \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== spin sweep (arc-angle must vary across frames) ==="
grep -E "spin frame=" "$LOG" | head -10
echo "=== frames (expect N emitter + N walker) ==="
grep -cF "engine emitter Ok" "$LOG"
grep -cF "present walker Ok(ret=0x1)" "$LOG"
echo "=== crash/json-overflow (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|string length overflow" "$LOG"