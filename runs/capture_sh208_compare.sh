#!/bin/bash
# SH208: is the SH104/105 stack-smash correlated with the SH202 V2 on-demand
# patcher (which mutates .text mid-run concurrently with the JIT block cache)?
# Hypothesis: the on-demand rewind/patch path is the canary-writer (guest frame
# unwinding after a mid-block .text edit). Test: 8 runs WITH on-demand vs
# 8 runs WITHOUT (SH200-only, the SH207-style baseline), count stack-smash /
# SIGSEGV / clean.
set -u
cd "$(dirname "$0")/.."
mode=${1:-on}
N=${2:-8}
BASE="env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 JIT_ROUTEB_DM_SERVICE_NODE=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1"
[ "$mode" = "on" ] && BASE="$BASE JIT_ROUTEB_V2_ONDEMAND=1"
CLEAN=0; SMASH=0; SEGV=0
for i in $(seq 1 "$N"); do
  LOG=/tmp/sh208-m$mode-$i.txt
  timeout 55 $BASE ./target/debug/examples/elfjit \
    ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  if grep -q "stack smashing detected" "$LOG"; then SMASH=$((SMASH+1)); echo "run $i: EXIT=$E STACK-SMASH ($mode)"; 
  elif grep -qiE "SIGSEGV|SIGABRT" "$LOG"; then SEGV=$((SEGV+1)); echo "run $i: EXIT=$E SIGSEGV/ABRT ($mode): $(grep -miE 'SIGSEGV|SIGABRT' "$LOG"|head -1)"; 
  else CLEAN=$((CLEAN+1)); echo "run $i: EXIT=$E clean ($mode)"; fi
done
echo "=== $mode: $CLEAN clean / $SMASH stack-smash / $SEGV sigsegv-abrt / $N ==="