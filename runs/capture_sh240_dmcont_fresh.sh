#!/bin/bash
# SH240: re-measure the DMCONT continuation at HEAD. The engine-init dispatcher
# 0x102bd8ce8's `bl sub_2bd8dac` (-> vt+0x1f0 = REAL continueAfterFlagsLoaded_ 0x102bd1d68)
# is UNCONDITIONAL at 0x2bd8d60 (disasm), yet SH228 (older HEAD) measured 0 hits.
# Watch the dispatcher body + the real continuation + the post-app-start structural
# gate 0x2bd2080 region to see if DMCONT now fires it fresh.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh240-dmcont.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
  JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x102bd8ce8-0x102bd9060,0x102bd1d68-0x102bd24b4,0x102bd2080-0x102bd2200 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DMCONT manager routing ==="
grep -E "SH165 manager singleton holder" "$LOG"
echo "=== region-watch hits (dispatcher/continue/area) ==="
grep -E "entered region" "$LOG" | sort | uniq -c
echo "=== continues / nativeAppBridgeAppStart / faults ==="
grep -iE "continueAfterFlags|appStart|2bd20|SIGSEGV|SIGABRT|fault" "$LOG" | head -20
echo "=== tail ==="
tail -15 "$LOG"