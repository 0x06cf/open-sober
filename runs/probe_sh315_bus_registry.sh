#!/bin/bash
# SH315 probe: does the real MessageBus.subscribe -> nativeAppBridgeAppStart route
# (with --v2boot-skip-appstart so the loop completes) reach the service REGISTRATION
# walk and change the service-registry count [0x106fe2f08]? The DM-controller ctor
# fast-path (SH313) yields a live DM-root iff count >= 1 with an 'App' entry.
set -u
cd "$(dirname "$0")/.."
LOG=/tmp/sh315-bus.txt
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
RW="0x1021e2a90-0x1021e2b40,0x1021dde00-0x1021de100,0x102338c00-0x102339000,0x102339000-0x10233a000,0x1021ddbc8-0x1021ddf00"
timeout 150 env $BASE JIT_REGION_WATCH="$RW" \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits (registration-walk / app-start entry / map-build) ==="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | head -40
echo "=== distinct region pcs ==="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "=== subscription flow ==="
grep -oE "MessageBus.subscribe (returned|stopped)[^ ]*[^ ]*|driving nativeAppBridge[A-Za-z]+|app-start" "$LOG" | head
echo "=== crash ==="
grep -iE "guestpc=0x[0-9a-f]+|SIGSEGV|SIGABRT|terminate|bad_alloc" "$LOG" | tail -4
echo "=== registry/DM probe ==="
grep -oE "DM-root\[0x106a68818\]=0x[0-9a-f]+|once-guard\[0x6a68410\]=0x[0-9a-f]+|app-data-model-count\[[^]]+\]=0x[0-9a-f]+" "$LOG" | tail -4
exit 0