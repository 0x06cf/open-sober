#!/bin/bash
# SH337: drive V1 nativeAppBridgeAppStart__ (0x102338510, SH336 ABI) AFTER MessageBus.subscribe
# populates the service registry (0->12). Every prior V1 drive ran from an EMPTY registry; this
# tests whether the app-start registration walk, running against a populated count=12 registry,
# registers "App" (the DM-ctor fast-path needs ONE "App" match, SH313; tier-2 cell invariant
# "Runtime0" otherwise, SH317/318).
set -u
cd "$(dirname "$0")/.."
LOG=$1; : "${LOG:=/tmp/sh337.txt}"
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
timeout 150 env $BASE \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
  --v2boot-postbus-v1appstart >"$LOG" 2>&1
EX=$?
echo "EXIT=$EX"
echo "bus:    $(grep -aoE 'SH337 bus Ok\(0x[0-9a-f]+\)|SH337 bus stopped: [^ ]*' "$LOG" | tail -1)"
echo "V1:     $(grep -aoE 'SH337 V1 AppStart__ (Ok\(0x[0-9a-f]+\)|stopped: [^ ]*)' "$LOG" | tail -1)"
echo "after:  $(grep -aoE 'SH337 after-bus registry=[0-9]+' "$LOG" | tail -1)"
echo "post:   $(grep -aoE 'SH337 post-V1: registry=[0-9]+ DM-root\[0x106a68818\]=0x[0-9a-f]+ once-guard=0x[0-9a-f]+' "$LOG" | tail -1)"
echo "crash:  $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT' "$LOG" | tail -3 | tr '\n' ' ')"
exit 0