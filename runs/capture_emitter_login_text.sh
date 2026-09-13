#!/bin/bash
# SH77/78: reproduce the engine's REAL geometry emitter (0x105b35288) drawing the
# Roblox LOGIN form with real TEXT LABELS — "Log In" (white glyphs over the
# green button), "Email address" / "Password" (dark-slate placeholders over the
# two input fields) — rasterized from the APK's own SourceSansPro-Bold.ttf by a
# pure-std TrueType outline rasterizer (SH77) at 2x resolution into a shared
# vertical atlas with transparent GUARD rows between sprites (SH78), in ONE
# top-level jit_run with GL_BLEND.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh78-emitter-login-text-2x.txt
PNG=/home/hermes-worker/runs/sh78-emitter-login-text-2x.png
rm -f "$LOG" "$PNG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=2 RENDERWALKER_WINDOW_MS=2800 RENDERWALKER_GLDEBUG=1 \
  RENDEREMITTER_LAYOUT=home RENDEREMITTER_MULTI=1 RENDEREMITTER_LOGIN=1 \
  RENDEREMITTER_LIVE=1 RENDEREMITTER_GLTRAP=1 \
  ./target/debug/examples/elfjit /home/hermes-worker/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker --renderemitter \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1 &
PID=$!
CAP=0
for i in $(seq 1 240); do
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
echo "=== sprite loads (real auth art + synthesized solid rows) ==="
grep -E "loaded real (auth|sprite)" "$LOG" | head -10
echo "=== text label rasterization ==="
grep -E "rasterized text label" "$LOG" | head -6
echo "=== engine emitter + swap (expect sprites=9 swap Ok(1)) ==="
grep -E "renderemitter-multi] engine emitter Ok" "$LOG" | head -4
echo "=== PROBES: present=true (expect wordmark + Log In + Email + Password byte-exact) ==="
grep -E "renderemitter-multi] probe" "$LOG" | head -40
echo "present=true count:"; grep -cF "present=true" "$LOG"
echo "=== walker + stability ==="
grep -cF "present walker Ok(ret=0x1)" "$LOG" | sed 's/^/walker-OK: /'
grep -icE "SIGSEGV|SIGABRT|panic|string length overflow" "$LOG" | sed 's/^/crash-marker-count: /'