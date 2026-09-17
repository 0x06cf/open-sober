#!/bin/bash
# SH248c batch: reproduce the 0x102b504e4 crash inside nativeAppBridgeAppStart.
# Continuation reach is ~1/3 flaky; run 6x and aggregate.
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh248c-batch.txt
rm -f "$LOG"
for i in 1 2 3 4 5 6; do
  printf "===== RUN %s =====\n" "$i" >> "$LOG"
  timeout 130 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x1022338ef4-0x102233a80 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    >> "$LOG" 2>&1
  echo "EXIT=$? (run $i)" >> "$LOG"
done
echo "===== AGGREGATE ====="
for i in 1 2 3 4 5 6; do
  R=$(awk "/===== RUN $i/,/EXIT=" "$LOG" | grep -m1 -E "region hit at guest pc=0x102bd2014|region hit at guest pc=0x1022338|SIGSEGV|SIGABRT|cont app-name|EXIT=" )
  echo "RUN$i: $R"
done
echo "== continuation terminal pc hit counts =="
grep -oE "guest pc=0x102bd[0-9a-f]{4}" "$LOG" | sort | uniq -c | sort -rn | head
echo "== nativeAppBridgeAppStart region hits =="
grep -oE "region hit at guest pc=0x1022338[0-9a-f]{3}" "$LOG" | sort | uniq -c | sort -rn | head
echo "== app-name seed fires =="
grep -c "app-name guard re-seeded" "$LOG"
echo "== crashes =="
grep -E "SIGSEGV|SIGABRT|bad_alloc|terminate" "$LOG" | head