#!/bin/bash
# SH344: Route-B re-attack measurement — does the SH343-deepened FULL --v2boot ladder
# (app-start driven, LSM keyfix landed) reach the NativeDataModelManager DM-creator /
# initEngine_ band? SH340 measured the skip-appstart send-appevent path: governor silent.
# This measures whether the app-start-driving full ladder (which SH231 showed reaches the
# governor tail) now ALSO enters the DM-creator band (getFlagsFromEngine_/initEngine_,
# 0x102bd1a30..0x102bd1d08) and setDataModelToCurrent-registry dispatch (0x102dbcc10).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh344-routeb-reach.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_REGION_WATCH=0x102bd1a30-0x102bd1d40,0x102dbcc10-0x102dbcd40,0x102e9fa80-0x102ea3b40,0x101f1d8ac-0x101f1d940 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits by band ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | awk '{print $NF}' | cut -c1-12 | sort | uniq -c | sort -rn
echo "=== DM-creator / setDataModelToCurrent-registry / governor / ScriptContext hits ==="
grep -cE "guest pc=0x102bd1|guest pc=0x102dbcc" "$LOG"
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root probe|MH_APP_READY|app-data-model-count|ladder done|SendAppEventOnAppReady returned" "$LOG" | tail -8