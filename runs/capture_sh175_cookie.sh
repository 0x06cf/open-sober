#!/bin/bash
# SH175 follow-on: COOKIE-INGRESS — drive the pure-native cookie worker
# 0x102203148 past the jar-init deref (now cleared by routeb_cookie_jar_guard)
# as a STANDALONE top-level jit_run on the main thread BEFORE StartApp, with
# JIT_ROUTEB_COOKIE=1 so the guard seeds the jar container + clears the two boot
# gates, w4=1 to reach the classifier/commit accumulator. No --v2boot ladder (so
# this is the only top-level jit_run in flight — single-jit_run discipline,
# SH55/64-safe). Exit 124 = timeout-after-boot (clean); the deliverable is the
# cookie-ingress line showing Ok return + jar preserved + gates cleared.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh175-cookie-ingress.txt
rm -f "$LOG"
timeout 120 env JIT_DRIVE_LIFECYCLE=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_ROUTEB_COOKIE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --cookie-ingress \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== cookie-ingress drive ==="
grep -E "cookie-ingress" "$LOG"
echo "=== StartApp ==="
grep -E "driving StartApp|StartApp returned|StartApp stopped|JNI" "$LOG" | head
echo "=== crashes / signal ==="
grep -icE "SIGSEGV|SIGABRT|stack smash" "$LOG"