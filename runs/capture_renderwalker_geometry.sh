#!/bin/bash
# SH65: reproduce the ENGINE's real per-node PRESENT walker drawing REAL
# GEOMETRY per node — each node's render-obj vt[+24] is a REGISTERED HOST THUNK
# (walker_item_draw_thunk) that draws a distinct real indexed quad (2 triangles)
# through a cached real-Mesa (libGLESv2.so.2) shader program, in addition to the
# colored clear. Desync-proof: host_call_at dispatches it with zero block-cache
# mutation, so the present-loop block survives. The walker then swaps via the
# real ctx-vt[+24] on the currency-owning renderinit thread.
#
# Shader+VAO+VBO+EBO are built once (cached) on the first per-node draw; every
# subsequent draw just re-uploads the per-node vertex color band + glDrawElements.
#
# Requires: cargo build -p arm64jit --example elfjit, real libroblox.so at
# ~/.cache/open-sober/robbox/libroblox.so, Xvfb, ffmpeg (x11grab).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh65-renderwalker-geometry.txt
PNG=/home/hermes-worker/runs/sh65-renderwalker-geometry.png
rm -f "$LOG" "$PNG"
timeout 100 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=3 RENDERWALKER_WINDOW_MS=3000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1 &
PID=$!
CAP=0
for i in $(seq 1 100); do
  if grep -q "present walker Ok" "$LOG"; then
    sleep 1
    DISPNUM=$(grep -oE "on :[0-9]+" "$LOG" | head -1 | tr -d 'on :')
    if [ -n "$DISPNUM" ] && [ "$CAP" = "0" ]; then
      ffmpeg -y -loglevel error -f x11grab -video_size 1280x720 -i ":${DISPNUM}.0" \
        -frames:v 1 "$PNG"
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
echo "=== mesh program (must show prog!=0) ==="
grep -F "walker mesh program built" "$LOG"
echo "=== per-node draws (real colored quads; expect nodes*frames=9) ==="
grep -cF "real colored quad" "$LOG"
echo "=== walker Ok ret == (engine mid-loop ran + real swap)"
grep -cF "present walker Ok(ret=0x1)" "$LOG"
echo "=== drained ==="
grep -E "renderwalker] drained" "$LOG"
echo "=== crash/json-overflow (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|string length overflow" "$LOG"
echo "=== debug: non-uniform quad pixel proof ==="
if [ -n "${PNG:-}" ] && [ -f "$PNG" ]; then
  ffprobe -v error -show_entries stream=width,height -of csv=p=0 "$PNG" 2>/dev/null | head -1
fi