#!/bin/bash
# SH248 batch: 3 probe runs to get a completing run (the run-variable 0x6240de8
# FMOD flake kills ~1/3; the continuation's 0x28 bad_alloc needs a completing run).
for i in 1 2 3; do
  LOG=/home/hermes-worker/runs/open-sober/runs/sh248-p$i.txt
  rm -f "$LOG"
  timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
    JIT_ROUTEB_OPNEW_SIZE_GATE=1 JIT_ROUTEB_ALLOC_PROBE=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  EXIT=$?
  echo "###### run $i EXIT=$EXIT ######"
  echo "-- TAIL summary --"
  grep -a "TAIL" "$LOG" | sed -E 's/pc=0x[0-9a-f]+ //' | sort | uniq -c | sort -rn | head -25
  echo "-- WRAP summary --"
  grep -a "WRAP" "$LOG" | grep -av "TAIL" | sed -E 's/pc=0x[0-9a-f]+ //' | sort | uniq -c | sort -rn | head -15
  echo "-- continuation region hits --"
  grep -aoE "region hit at guest pc=0x102bd[0-9a-f]{4}" "$LOG" | sort -u
  echo "-- crash --"
  grep -aiE "SIGSEGV|SIGABRT|terminat|bad_alloc" "$LOG" | tail -2
  echo
done