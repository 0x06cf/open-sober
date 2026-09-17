#!/bin/bash
# SH275: drive nativeInitClientSettingsSigned (0x102bb070c) with the REAL version word
# 0x0306 -> readLocalFlags parse path (never driven before; SH264 only drove the plain
# variant at version=0 empty-early). Same ladder/session-set as sh269, plus the new rung.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-signed"
for i in 1 2; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S \
    > runs/sh275-signed-$i.txt 2>&1
  echo "signed$i EXIT=$? ret=$(grep -oE 'InitClientSettingsSigned returned Ok\(0x[0-9a-f]+\) w0\(parse_result\)=0x[0-9a-f]+' runs/sh275-signed-$i.txt | head -1) once=$(grep -oE 'once-guard\[0x6a68410\]=0x[0-9a-f]+' runs/sh275-signed-$i.txt | tail -1) dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh275-signed-$i.txt | tail -1)"
done