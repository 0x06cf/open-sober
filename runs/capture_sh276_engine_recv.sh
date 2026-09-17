#!/bin/bash
# SH276: drive nativeActivity_onEngineSettingsReceived (0x102bd1c38) — the SEP-17
# directive's OTHER named engine-settings primitive — on a fabricated zeroed manager
# `this`, so its [this+648] engine-settings-received flag latches. Never driven before
# (SH264-275 drove lifecycle natives + client-settings, not this transition). Same
# ladder/session-set as sh269, plus the new --v2boot-session-engine rung.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine"
for i in 1 2; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S \
    > runs/sh276-engine-$i.txt 2>&1
  echo "engine$i EXIT=$? recv=$(grep -oE 'engine-settings-received\)=[0-9a-f]+' runs/sh276-engine-$i.txt | head -1) once=$(grep -oE 'once-guard\[0x6a68410\]=0x[0-9a-f]+' runs/sh276-engine-$i.txt | tail -1) dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh276-engine-$i.txt | tail -1)"
done