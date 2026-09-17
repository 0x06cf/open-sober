#!/bin/bash
# SH283b: stable park-point of engine5 rung. Minimal watch: continuation 0x102207118 entry,
# its internal steps, the enqueue 0x2d9713c (target of its bl@0x2207154). No perturbing pc.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
for run in 1 2; do
RW="JIT_REGION_WATCH=0x102207118-0x102207200,0x1022d9713c-0x1022d971c0,0x1022071ac-0x102207440"
timeout 45 env $BASE $RW ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
  > runs/sh283b-park-$run.txt 2>&1
echo "run$run EXIT=$?"
echo "  cont-7118=$(grep -acE 'region hit at guest pc=0x102207118' runs/sh283b-park-$run.txt)"
echo "  enqueue-13c=$(grep -acE 'region hit at guest pc=0x1022d9713c' runs/sh283b-park-$run.txt)"
echo "  construct-ac=$(grep -acE 'region hit at guest pc=0x1022071ac' runs/sh283b-park-$run.txt)"
echo "  callee-in-engine5=$(grep -aE 'region hit at guest pc=0x10275a2'  runs/sh283b-park-$run.txt | wc -l)"
echo "  last=$(grep -aE 'region hit at guest pc' runs/sh283b-park-$run.txt | tail -1)"
done