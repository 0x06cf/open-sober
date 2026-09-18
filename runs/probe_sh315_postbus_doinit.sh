#!/bin/bash
# SH315: drive MessageBus.subscribe first (SESSION-CTOR route pops the service registry
# count from 0 -> N), clear the do-init once-guard [0x106a68410].bit0, re-seed main-id,
# and re-drive StartLuaAppDM so the do-init once-lambda re-runs the DM-controller ctor
# with a NON-empty registry -> SH313 fast-path could yield a live DM-root.
# Measures registry-count + DM-root before/after.
set -u
cd "$(dirname "$0")/.."
LOG=/tmp/sh315-postbus.txt
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
timeout 150 env $BASE \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-postbus-doinit \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH315 probe lines ==="
grep -E "SH315" "$LOG"
echo "=== ladder flow ==="
grep -oE "driving [a-zA-Z]+|StartLuaAppDM[^ ]*|MessageBus.subscribe[^ ]*" "$LOG" | head
echo "=== crash ==="
grep -iE "guestpc=0x[0-9a-f]+|SIGSEGV|SIGABRT|terminate|bad_alloc" "$LOG" | tail -4
exit 0