#!/bin/bash
# SH177: COOKIE READ-BACK — after --cookie-ingress self-constructs the jar,
# write a classified value into it (jit::cookie_jar_write_value) and drive
# nativeGetCookiesInNetscapeFormat 0x1021ff6b0 on its jar-driven Route B
# (routeb_patch_cookie_readback NOPs the two branch gates) so the ENGINE re-emits
# the value as an RFC6265 '#HttpOnly_' line into the x8 out-string. Standalone
# (no --v2boot) single top-level jit_run discipline. Exit 124 = timeout-after-boot.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh177-cookie-readback.txt
rm -f "$LOG"
timeout 150 env JIT_DRIVE_LIFECYCLE=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_ROUTEB_COOKIE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --cookie-ingress --cookie-readback \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== cookie-ingress drive ==="
grep -E "cookie-ingress" "$LOG" | head -12
echo "=== cookie-readback drive ==="
grep -E "cookie-readback|cookie-rb" "$LOG"
echo "=== OUT / jar / RFC6265 emission ==="
grep -E "OUT |jar |class|token|RFC" "$LOG" | head
echo "=== crashes / signal ==="
grep -icE "SIGSEGV|SIGABRT|stack smash" "$LOG"