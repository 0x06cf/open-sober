#!/bin/bash
# SH4xx: reproduce the canary stack-smash wall with the SH106-NEXT store-watch
# armed — the store-watch NAMES the exact guest str/stp writers that store a
# foreign HOST pointer (0x700000000000..0x800000000000) into a guest frame's
# canary window [x29-0x60, x29]. This is the standing nativeGameGlobalInit /
# app-shell ctor full-boot wall; the runbook carries the 3-gate crossing env plus
# JIT_CANARY_STORE_WATCH=1 (default-inert instrument, adds no guest bytes).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh4xx-canary-store-watch.txt
rm -f "$LOG"
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_CANARY_STORE_WATCH=1 JIT_TRACE=0 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --persist-roundtrip \
  --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== named canary-window writers (is_canary_slot=true) ==="
grep "is_canary_slot=true" "$LOG" | sed -E 's/.*pc=0x([0-9a-f]+) dst=.*/\1/' | sort | uniq -c | sort -rn | head
echo "=== total store-watch firings ==="
grep -c "canary-store-watch" "$LOG"
echo "=== the LAST store before the abort (candidate clobber) ==="
grep -E "is_canary_slot=true" "$LOG" | tail -1
echo "=== stack-smash present? ==="
grep -c "stack smashing" "$LOG" || echo 0