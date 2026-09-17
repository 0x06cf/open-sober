#!/bin/bash
# SH240b: narrow region-watch on the dispatcher flow to find WHERE it leaves.
# fnB(0x102bd1b98) -> bl 0x2bd8ce8 (dispatcher). Dispatcher: blr vt+0xf8 (0x2bd8d2c),
# blr vt+0x108 (0x2bd8d50), then bl sub_2bd8dac (0x2bd8d60) -> dispatches vt+0x1f0
# = continueAfterFlagsLoaded_ (0x102bd1d68). Measure each resumption block separately.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh240b-dmcont-flow.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
  JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x102bd8d30-0x102bd8d34,0x102bd8d54-0x102bd8d60,0x102bd8dac-0x102bd8db0,0x102bd1d68-0x102bd1d74 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== resumption/sub/cont hits ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | sort | uniq -c
echo "=== anything with 2bd8/2bd1 ==="
grep -iE "2bd8|2bd1" "$LOG" | grep -v hashfix | head
echo "=== tail ==="
tail -6 "$LOG"