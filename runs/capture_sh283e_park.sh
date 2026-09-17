#!/bin/bash
# SH283e: capture the LAST executed guest pc of the engine5 park via JIT_TRACE (tail only).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
timeout 30 env $BASE JIT_TRACE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
  > runs/sh283e-trace.txt 2>&1
echo "EXIT=$?"
echo "=== last 15 trace lines ==="
grep -avE '^\[(resolver|region-watch)\]' runs/sh283e-trace.txt | grep -aE 'SH280|SH278|engine5|state=5|0x102bd24b4|0x10227|0x102d9|trace' | tail -15
echo "=== file tail ==="
tail -8 runs/sh283e-trace.txt