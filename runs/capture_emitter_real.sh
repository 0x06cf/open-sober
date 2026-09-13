#!/bin/bash
# SH69: reproduce the engine's REAL geometry emitter (0x105b35288) drawing a
# layered "login/home"-style frame whose PANEL slot is a REAL Roblox APK UI
# texture — the loading-spinner (ui/LoadingScreen/LoadingSpinner.png, 100x100
# RGBA, real alpha) — decoded offline via flate2 and sampled through the
# engine's own textured path (SH68's atlas), aspect-correct. Probes prove:
#   real-arc-blue       = an OPAQUE sky-blue arc body pixel (real texture pixel)
#   transparent-center  = real alpha 0 -> backdrop shows through (NOT the old 0.55 panel)
#   transparent-right   = arc is left-only -> correct orientation/aspect
#   backdrop            = unchanged corner
# Requires: cargo build -p arm64jit --example elfjit, real libroblox.so, Xvfb, ffmpeg.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh69-emitter-real.txt
PNG=/home/hermes-worker/runs/sh69-emitter-real.png
rm -f "$LOG" "$PNG"
timeout 150 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=2 RENDERWALKER_WINDOW_MS=2600 RENDERWALKER_GLDEBUG=1 \
  RENDEREMITTER_LAYOUT=home RENDEREMITTER_REAL_TEX=1 \
  RENDEREMITTER_REAL_TEXTURE=/home/hermes-worker/.cache/open-sober/android-env/assets/content/textures/ui/LoadingScreen/LoadingSpinner.png \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker --renderemitter \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1 &
PID=$!
CAP=0
for i in $(seq 1 150); do
  if grep -q "renderemitter-home] engine emitter Ok" "$LOG"; then
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
echo "=== real-texture decode (expect 100x100 RGBA8) ==="
grep -E "renderemitter-real] decoded" "$LOG" | head -2
echo "=== scene-list read + layout (expect match=true, real_tex=true) ==="
grep -E "renderemitter-home] scene list|renderemitter-home] LAYOUT=" "$LOG" | head -3
echo "=== real probes (expect 4 present=true) ==="
grep -E "renderemitter-home] real probes:" "$LOG" | head -2
grep -cF "present=true" "$LOG"
grep -E "renderemitter-home] (backdrop|real-arc-blue|transparent-center|transparent-right)" "$LOG" | tail -8
echo "=== engine emitter OK + walker ==="
grep -cF "renderemitter-home] engine emitter Ok" "$LOG"
grep -cF "present walker Ok(ret=0x1)" "$LOG"
echo "=== crash/json-overflow (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|string length overflow" "$LOG"