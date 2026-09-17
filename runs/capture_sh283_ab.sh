#!/bin/bash
# SH283 AB: engine5 reentry continuation park = contended pthread_mutex_lock(0x106863aa0)
# owned by a never-running spawned worker (falsifies SH282 'static-init cannot block').
# A: baseline (parks EXIT 124). B: +JIT_ROUTEB_ENG5_QMUTEX_FREE=1 (steal the contended
# fixed mutex so the continuation proceeds into the enqueue-construct world-build).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="JIT_REGION_WATCH=0x102d9713c-0x102d971c0,0x1022071ac-0x102207440,0x102b53a68-0x102b53a90,0x1022207118-0x102207200"
echo "=================== A: BASELINE ==================="
timeout 35 env $BASE $RW ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 > runs/sh283-ab-A.txt 2>&1
echo "A EXIT=$?"
echo "A mutex-2b53a68=$(grep -acE 'region hit at guest pc=0x102b53a68' runs/sh283-ab-A.txt)"
echo "A enqueue-13c=$(grep -acE 'region hit at guest pc=0x102d9713c' runs/sh283-ab-A.txt)"
echo "A construct-ac=$(grep -acE 'region hit at guest pc=0x1022071ac' runs/sh283-ab-A.txt)"
echo "A last-region=$(grep -aE 'region hit at guest pc' runs/sh283-ab-A.txt | tail -1)"
echo "=================== B: +QMUTEX_FREE ==================="
timeout 35 env $BASE $RW JIT_ROUTEB_ENG5_QMUTEX_FREE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 > runs/sh283-ab-B.txt 2>&1
echo "B EXIT=$?"
echo "B force-free=$(grep -acE 'SH283 force-freed' runs/sh283-ab-B.txt)"
echo "B mutex-2b53a68=$(grep -acE 'region hit at guest pc=0x102b53a68' runs/sh283-ab-B.txt)"
echo "B enqueue-13c=$(grep -acE 'region hit at guest pc=0x102d9713c' runs/sh283-ab-B.txt)"
echo "B construct-ac=$(grep -acE 'region hit at guest pc=0x1022071ac' runs/sh283-ab-B.txt)"
echo "B last-region=$(grep -aE 'region hit at guest pc' runs/sh283-ab-B.txt | tail -1)"