#!/bin/bash
# SH315d: retry the SESSION-CTOR bus route until a clean run reaches the SM155 probe
# and dumps WHICH services got registered (is 'App'/'Execute' among them?).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
for i in 1 2 3 4 5; do
  LOG=/tmp/sh315d-$i.txt
  rm -f "$LOG"
  timeout 150 env $BASE \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus \
    > "$LOG" 2>&1
  EXIT=$?
  CT=$(grep -oE "SH315 service-registry-count\[0x106fe2f08\]=[0-9]+" "$LOG" | tail -1)
  DM=$(grep -oE "DM-root\[0x106a68818\]=0x[0-9a-f]+" "$LOG" | tail -1)
  echo "run$i EXIT=$EXIT $CT $DM"
  # print the entries line if present
  grep -oE "SH315 service-registry-count.*entries=.*" "$LOG" | tail -1
done
exit 0