#!/bin/bash
# SH278: cross the SH277-pinned initEngine_ state gate. SH277 proved a fabricated manager
# mono-tails (state [this+16]=0 -> benign tail). The engine-settings receive (0x2bd1c38,
# SH276) transitions state->3 ITSELF when [this+649]!=0 (0x2bd1cac ldrb w8,[this,#649];
# 0x2bd1cb8 cbz skips; 0x2bd1cbc mov w8,#3; 0x2bd1cc0 str w8,[this,#16]). So seeding the
# ONE byte [this+649]=1 makes the ENGINE's own receive set state=3; then driving the
# initEngine_ dispatch entry 0x2bd1cf0 takes its ==3 branch -> FIRST-ever session-state
# entry into the settings-serializer body 0x2bd1d68 (continueAfterFlagsLoaded_ continuation).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
# clean session path (skip appstart so the post-ladder engine3 rung runs)
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
for i in 1 2 3; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SL \
    > runs/sh278-engine3-$i.txt 2>&1
  echo "e3$i EXIT=$? recv=$(grep -oE '\[this\+648\]=[0-9]+ \[this\+16\]\(state\)=[0-9]+' runs/sh278-engine3-$i.txt | head -1) dispatch=$(grep -oE 'initEngine_ dispatch returned Ok\(0x[0-9a-f]+\)' runs/sh278-engine3-$i.txt | head -1) once=$(grep -oE 'once-guard\[0x6a68410\]=0x[0-9a-f]+' runs/sh278-engine3-$i.txt | tail -1) dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh278-engine3-$i.txt | tail -1)"
done