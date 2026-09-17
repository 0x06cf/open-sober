#!/bin/bash
# SH248e batch: seed the app-start once-cell global [0x106b0bdf0] -> -1 so the DMCONT
# continuation (now inside nativeAppBridgeAppStart) skips the pthread_mutex_lock once-branch
# instead of SIGSEGV'ing at 0x102339208 (NULL once-cell pointer deref).
# Continuation reach ~1/3 flaky; run 6x and aggregate.
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh248e-batch.txt
rm -f "$LOG"
for i in 1 2 3 4 5 6; do
  printf "===== RUN %s =====\n" "$i" >> "$LOG"
  timeout 130 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x102339000-0x102339500,0x102339500-0x10233b000 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    >> "$LOG" 2>&1
  echo "EXIT=$? (run $i)" >> "$LOG"
done
echo "===== AGGREGATE ====="
echo "sh248e once seed fires: $(grep -c 'routeb-sh248e' "$LOG")"
echo "once-fn(0x2339208) region hits: $(grep -oE 'region hit at guest pc=0x1023391f8|region hit at guest pc=0x102339208|region hit at guest pc=0x102339018|region hit at guest pc=0x102339020' "$LOG" | sort | uniq -c)"
echo "crash sites: $(grep -oE 'guestpc=0x102[0-9a-f]+' "$LOG" | grep -vE '0x102bd' | sort | uniq -c | sort -rn | head -6)"
echo "EXITs: $(grep -oE 'EXIT=[0-9]+' "$LOG" | tr '\n' ' ')"