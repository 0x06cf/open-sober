#!/bin/bash
# SH315c: after the SESSION-CTOR bus route, dump the registered services in the
# registry array [0x106fe6180] (count [0x106fe2f08]) to see WHICH services got
# registered headlessly (is 'App'/'Execute' -> the DM-controller among them?).
set -u
cd "$(dirname "$0")/.."
LOG=/tmp/sh315c-regdump.txt
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
timeout 150 env $BASE \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH155 final probe ==="
grep -E "service-registry-count|DM-root probe" "$LOG" | tail -3
echo "=== crash ==="
grep -iE "guestpc|SIGSEGV|SIGABRT|terminate" "$LOG" | tail -3
exit 0