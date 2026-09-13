#!/bin/bash
# SH66: reproduce the engine's REAL geometry emitter (0x105b35288) drawing an
# authored quad through the engine's OWN GL stack (primitive-setup 0x105b353d0 ->
# glVertexAttribPointer + glDrawArrays via @plt -> real Mesa), desync-safe as its
# own top-level jit_run after the walker, presented via the real engine swap.
#
# Requires: cargo build -p arm64jit --example elfjit, real libroblox.so at
# ~/.cache/open-sober/robbox/libroblox.so, Xvfb, ffmpeg (x11grab).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh66-renderemitter.txt
PNG=/home/hermes-worker/runs/sh66-renderemitter.png
rm -f "$LOG" "$PNG"
timeout 140 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=2 RENDERWALKER_WINDOW_MS=2600 RENDERWALKER_GLDEBUG=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker --renderemitter \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1 &
PID=$!
CAP=0
for i in $(seq 1 140); do
  if grep -q "engine emitter Ok" "$LOG"; then
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
echo "=== emitter OK markers (expect N=walker_frames) ==="
grep -cF "engine emitter Ok" "$LOG"
echo "=== emitter pixel line ==="
grep -E "engine emitter Ok" "$LOG" | tail -2
echo "=== walker quads + swap ==="
grep -cF "real colored quad" "$LOG"
grep -cF "present walker Ok(ret=0x1)" "$LOG"
echo "=== crash/json-overflow (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|string length overflow" "$LOG"