#!/bin/bash
# SH67d: reproduce the engine's REAL geometry emitter (0x105b35288) drawing a
# POPULATED N-quad 2D frame in ONE top-level jit_run (single pre-uploaded VBO,
# GL_TRIANGLES, first=0, count=6N) through the engine's OWN GL stack — closing
# the SH67c per-drive glBufferData orphan blocker by construction.
# Requires: cargo build -p arm64jit --example elfjit, real libroblox.so, Xvfb, ffmpeg.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh67d-emitter-grid.txt
PNG=/home/hermes-worker/runs/sh67d-emitter-grid.png
NQ=${RENDEREMITTER_QUADS:-6}
TEX=${RENDEREMITTER_TEX:-0}
rm -f "$LOG" "$PNG"
if [ "$TEX" = "1" ]; then
  ENVVAR="RENDEREMITTER_QUADS=$NQ RENDEREMITTER_TEX=1"
  LOG=/home/hermes-worker/runs/sh67e-emitter-grid-tex.txt
  PNG=/home/hermes-worker/runs/sh67e-emitter-grid-tex.png
else
  ENVVAR="RENDEREMITTER_QUADS=$NQ"
  LOG=/home/hermes-worker/runs/sh67d-emitter-grid.txt
  PNG=/home/hermes-worker/runs/sh67d-emitter-grid.png
fi
echo "log=$LOG png=$PNG"
rm -f "$LOG" "$PNG"
timeout 150 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=2 RENDERWALKER_WINDOW_MS=2600 RENDERWALKER_GLDEBUG=1 \
  $ENVVAR \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker --renderemitter \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1 &
PID=$!
CAP=0
for i in $(seq 1 150); do
  if grep -q "renderemitter-grid] engine emitter Ok" "$LOG"; then
    sleep 1
    DISPNUM=$(grep -oE "on :[0-9]+" "$LOG" | head -1 | tr -d 'on :')
    if [ -n "$DISPNUM" ] && [ "$CAP" = "0" ]; then
      for k in 1 2 3 4 5; do
        ffmpeg -y -loglevel error -f x11grab -video_size 1280x720 -i ":${DISPNUM}.0" \
          -frames:v 1 "$PNG" 2>/dev/null && break
        sleep 0.4
      done
      echo "captured :${DISPNUM} -> $PNG"
      CAP=1
    fi
    if grep -q "renderwalker] drained" "$LOG"; then break; fi
  fi
  if ! kill -0 "$PID" 2>/dev/null; then break; fi
  sleep 0.5
done
wait "$PID" 2>/dev/null
EXIT=$?
echo "EXIT=$EXIT"
echo "=== grid emitter OK markers ==="
grep -cF "renderemitter-grid] engine emitter Ok" "$LOG"
echo "=== per-tile readback (expect N distinct present=true) ==="
grep -cF "present=true" "$LOG"
grep -E "renderemitter-grid] tile#" "$LOG" | tail -"$NQ"
echo "=== walker (should still present) ==="
grep -cF "present walker Ok(ret=0x1)" "$LOG"
echo "=== crash/json-overflow (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|string length overflow" "$LOG"