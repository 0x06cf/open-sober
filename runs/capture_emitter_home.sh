#!/bin/bash
# SH150: the 'home' half of 'login/home' — the REAL FPSBackground.png launcher
# backdrop (opaque 1024x1024 auth artwork) through the engine's own emitter path,
# with the RO-BLOX wordmark overlay. Mirrors the proven capture_emitter_multi.sh
# invocation exactly (RENDEREMITTER_LAYOUT=home + RENDEREMITTER_MULTI + walker).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh150-home.txt
PNG=/home/hermes-worker/runs/sh150-home.png
rm -f "$LOG" "$PNG"
timeout 180 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=2 RENDERWALKER_WINDOW_MS=2800 RENDERWALKER_GLDEBUG=1 \
  RENDEREMITTER_LAYOUT=home RENDEREMITTER_MULTI=1 RENDEREMITTER_HOME=1 \
  ./target/debug/examples/elfjit /home/hermes-worker/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker --renderemitter \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1 &
PID=$!
CAP=0
for i in $(seq 1 200); do
  if grep -q "renderemitter-multi] engine emitter Ok" "$LOG"; then
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
echo "=== sprite loads (expect FPSBackground 1024x1024 + logo_white_1x) ==="
grep -E "renderemitter-multi] loaded real sprite" "$LOG" | head -10
grep -cF "renderemitter-multi] WARN: failed" "$LOG" | sed 's/^/WARN-failures: /'
echo "=== FPSBackground decode ==="
grep -E "FPSBackground" "$LOG" | head -4
echo "=== probes ==="
grep -E "renderemitter-multi] probe" "$LOG" | head -12
echo "present=true count:"; grep -cF "present=true" "$LOG"
echo "=== engine emitter OK + swap + walker ==="
grep -E "renderemitter-multi] engine emitter Ok" "$LOG" | head -4
grep -cF "present walker Ok(ret=0x1)" "$LOG" | sed 's/^/walker-OK: /'
echo "=== stability ==="
grep -icE "SIGSEGV|SIGABRT|panic|string length overflow" "$LOG" | sed 's/^/crash-marker-count: /'