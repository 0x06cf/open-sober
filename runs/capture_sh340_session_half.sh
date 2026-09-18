#!/bin/bash
# SH340 measurement: does the do-init pipe's SESSION half (post-do-init continuation
# 0x1023eff4c -> governor 0x102e9fa84 -> ScriptContext loader 0x101f1d8ac) fire at the
# current SH307-forward deep reach? SH308 proved the app-shell ctor/FMOD audio tail/
# StartAppWithParams half runs; the governor/Lua half is the standing gate. This measures
# which of the session-half region pcs actually execute on the deep do-init pipe.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-send-appevent"
# Regions: post-do-init 0x1023eff4c area, governor 0x102e9fa80-0x102ea3b40, ScriptContext
# loader 0x101f1d8ac, CoreScripts cell reads.
timeout 120 env $BASE JIT_REGION_WATCH=0x1023eff4c-0x1023f0040,0x102e9fa80-0x102ea3b40,0x101f1d8ac-0x101f1d940,0x10258b144-0x10258b600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S \
  > runs/sh340-sessionhalf.txt 2>&1
echo "EXIT=$?"
echo "--- session-half region hits ---"
grep "region hit" runs/sh340-sessionhalf.txt
echo "--- pipe markers ---"
grep -iE "SendAppEventOnAppReady returned|app-data-model-count|ladder done" runs/sh340-sessionhalf.txt