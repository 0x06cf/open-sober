#!/bin/bash
# SH283f: does the SH283-B enqueue-construct COMPLETE (fire its ret tails 0x2d971b4
# and 0x2207418) so the fixed queue 0x106863a70 is genuinely constructed, or does
# the state=5 body still park inside the construct world-build?
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
# enqueue tail 0x2d971ac (stp+ret region), construct terminal 0x2207418 (ret), the
# continuation's cond_signal 0x2b52e80, and the enqueue's construct-caller 0x2d9715c.
RW="JIT_REGION_WATCH=0x102d971ac-0x102d971b8,0x102207414-0x102207444,0x102b52e80-0x102b52e9c,0x1022071ac-0x102207230"
timeout 40 env $BASE $RW JIT_ROUTEB_ENG5_QMUTEX_FREE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 > runs/sh283f-complete.txt 2>&1
echo "EXIT=$?"
echo "state5ok=$(grep -acE 'SH280 state=5 body direct returned Ok' runs/sh283f-complete.txt)"
echo "stateval=$(grep -aoE '\[this\+16\]\(state\)=[0-9]+' runs/sh283f-complete.txt)"
echo "enqueue-ret-2d971b4=$(grep -acE 'region hit at guest pc=0x102d971b4' runs/sh283f-complete.txt)"
echo "construct-ret-2207418=$(grep -acE 'region hit at guest pc=0x102207418' runs/sh283f-complete.txt)"
echo "construct-loop-2d9715c=$(grep -acE 'region hit at guest pc=0x102d9715c' runs/sh283f-complete.txt)"
echo "cond_signal-52e80=$(grep -acE 'region hit at guest pc=0x102b52e80' runs/sh283f-complete.txt)"
echo "last-region=$(grep -aE 'region hit at guest pc' runs/sh283f-complete.txt | tail -1)"