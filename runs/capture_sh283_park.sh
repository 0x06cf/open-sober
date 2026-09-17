#!/bin/bash
# SH283: precise park-point of the engine5 state=5 -> reentry -> enqueue-construct chain.
# Region-watch the FULL chain incl. the callee tail, the continuation, the enqueue
# 0x2d9713c, and the construct 0x22071ac + intermediate 0x2206f04. Register-dump at park.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="JIT_REGION_WATCH=0x102207118-0x102207200,0x10275a23c-0x10275a2a0,0x102206f04-0x102206fac,0x1022071ac-0x102207440,0x1022d9713c-0x1022d971c0,0x10284d100-0x10284d200,0x102bd24b4-0x102bd2600"
timeout 45 env $BASE $RW ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
  > runs/sh283-park.txt 2>&1
echo "EXIT=$?"
echo "callee-a23c=$(grep -acE 'region hit at guest pc=0x10275a23c' runs/sh283-park.txt)"
echo "callee-a25c=$(grep -acE 'region hit at guest pc=0x10275a25c' runs/sh283-park.txt)"
echo "callee-a278=$(grep -acE 'region hit at guest pc=0x10275a278' runs/sh283-park.txt)"
echo "cont-7118=$(grep -acE 'region hit at guest pc=0x102207118' runs/sh283-park.txt)"
echo "inter-f04=$(grep -acE 'region hit at guest pc=0x102206f04' runs/sh283-park.txt)"
echo "construct-ac=$(grep -acE 'region hit at guest pc=0x1022071ac' runs/sh283-park.txt)"
echo "enqueue-13c=$(grep -acE 'region hit at guest pc=0x1022d9713c' runs/sh283-park.txt)"
echo "last-region: $(grep -aE 'region hit at guest pc' runs/sh283-park.txt | tail -1)"
grep -aoE 'SIG[A-Z]+|guestpc=0x[0-9a-fx]+|pc [0-9a-fx]+ outside' runs/sh283-park.txt | sort -u | head