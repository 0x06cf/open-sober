#!/bin/bash
# SH248f batch: fabricate the app-lifecycle adapter object at [0x106b0bde0] so the DMCONT
# continuation's nativeAppBridgeAppStart closure dispatch (vt[0] blr @0x233904c) passes the
# NULL-adapter wall (SIGSEGV 0x102339020) and advances ~0x500 deeper (next wall 0x1021dde34).
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh248f-batch.txt
rm -f "$LOG"
for i in 1 2 3 4 5 6; do
  printf "===== RUN %s =====\n" "$i" >> "$LOG"
  timeout 130 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x102339000-0x102339500,0x102339500-0x10233b000 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    >> "$LOG" 2>&1
  echo "EXIT=$? (run $i)" >> "$LOG"
done
echo "===== AGGREGATE ====="
echo "sh248f adapter seed fires: $(grep -c 'routeb-sh248f.*seeded app-lifecycle' "$LOG")"
echo "fabricated object log: $(grep -c 'fabricated app-lifecycle adapter object' "$LOG")"
echo "appstart advance region hits: $(grep -oE 'region hit at guest pc=0x102339[0-9a-f]{3}' "$LOG" | sort -u | tr '\n' ' ')"
echo "crash sites: $(grep -oE 'guestpc=0x102[0-9a-f]+' "$LOG" | grep -vE '0x102bd|0x1023390' | sort | uniq -c | sort -rn | head -6)"
echo "EXITs: $(grep -oE 'EXIT=[0-9]+' "$LOG" | tr '\n' ' ')"