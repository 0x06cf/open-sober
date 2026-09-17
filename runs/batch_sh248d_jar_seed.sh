#!/bin/bash
# SH248d batch: seed the cookie-jar [0x106ed7a20] at the appstart assign site so the
# continuation (now inside nativeAppBridgeAppStart) advances past the NULL-dest string copy.
# Continuation reach ~1/3 flaky; run 6x and aggregate.
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh248d-batch.txt
rm -f "$LOG"
for i in 1 2 3 4 5 6; do
  printf "===== RUN %s =====\n" "$i" >> "$LOG"
  timeout 130 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x1022338510-0x102233a00 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    >> "$LOG" 2>&1
  echo "EXIT=$? (run $i)" >> "$LOG"
done
echo "===== AGGREGATE ====="
echo "app-name seed fires: $(grep -c 'app-name guard re-seeded' "$LOG")"
echo "sh248d jar seed fires: $(grep -c 'routeb-sh248d' "$LOG")"
echo "appstart(0x2338510) region hits: $(grep -oE 'region hit at guest pc=0x1022338[0-9a-f]{3}' "$LOG" | sort -u | tr '\n' ' ')"
echo "crash sites: $(grep -oE 'guestpc=0x102[0-9a-f]+' "$LOG" | grep -vE '0x102bd' | sort | uniq -c | sort -rn | head -6)"
echo "EXITs: $(grep -oE 'EXIT=[0-9]+' "$LOG" | tr '\n' ' ')"