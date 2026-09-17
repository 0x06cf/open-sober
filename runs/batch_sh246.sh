#!/bin/bash
# SH246 batch: run each arm N times, report how far the continuation gets.
# The continuation is straight-line after the app-name guard, so a block-entry
# pc past 0x2bd2128 (the patched op_new site) is block-entry-definitive proof the
# continuation advanced past the closure-box point.
set -u
cd "$(dirname "$0")/.."
N="${2:-5}"
watch="0x102bd1d68-0x102bd2600"
for ARM in off on; do
  echo "===== ARM=$ARM x$N ====="
  for i in $(seq 1 "$N"); do
    LOG=/home/hermes-worker/runs/open-sober/runs/sh246b-${ARM}-${i}.txt
    rm -f "$LOG"
    EXTRA=""
    [ "$ARM" = on ] && EXTRA="JIT_ROUTEB_DM_CONT_OPNEW_BOX=1"
    timeout 115 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
      JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
      JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
      JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
      $EXTRA \
      JIT_REGION_WATCH="$watch" \
      ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
      --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
      > "$LOG" 2>&1
    EXIT=$?
    # block-entry pcs fired in the continuation window (sorted, last = furthest)
    pcs=$(grep -E "region hit at guest pc=0x102bd1" "$LOG" | grep -oE 'pc=0x102bd[0-9a-f]{4}' | sort -u | tr '\n' ' ')
    crash=$(grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smashing" "$LOG")
    boxpatch=$(grep -cE "SH245-closure patched op_new" "$LOG")
    # furthest = max hex pc
    furthest=$(echo "$pcs" | grep -oE '0x102bd[0-9a-f]{4}' | sort -u | tail -1)
    echo "run $i EXIT=$EXIT crash=$crash boxpatch=$boxpatch furthest=$furthest [$pcs]"
  done
done