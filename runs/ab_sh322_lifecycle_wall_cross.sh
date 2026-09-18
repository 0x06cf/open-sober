#!/bin/bash
# SH322 clean A/B on the SH273 lifecycle wall crossing (guard-gated).
# Baseline needs JIT_ROUTEB_DONEPATH_MAIN=1 (to reach the wall via the SH320 MAIN path);
# then with JIT_ROUTEB_LIFECYCLE_EARLYRET=1 the guard seeds the caller pair [x1] so fn
# 0x21f3748's tbnz -> canary-check+ret no-op, and nativePostClientSettingsLoadedInitialization3
# completes instead of SIGSEGV'ing at fault=0x50. Markers: the guard's "routeb-sh322" seed log
# and the absence/presence of the SIGSEGV at guestpc=0x1021f3748.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
# Barrier arm: without LIFECYCLE_EARLYRET the MAIN-path dispatch should fault at the lifecycle wall.
LOG=/tmp/sh322-ab-off.txt; rm -f "$LOG"
timeout 150 env $BASE JIT_ROUTEB_DONEPATH_MAIN=1 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "BASELINE(+DONEPATH_MAIN) EXIT=$? wall_sigsegv=$(grep -c 'SIGSEGV.*guestpc=0x1021f3748' "$LOG")"
# Forward arm: add the SH322 early-ret guard.
LOG=/tmp/sh322-ab-on.txt; rm -f "$LOG"
timeout 150 env $BASE JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "FORWARD(+EARLYRET) EXIT=$? seed_routeb_sh322=$(grep -c 'routeb-sh322' "$LOG") wall_sigsegv=$(grep -c 'SIGSEGV.*guestpc=0x1021f3748' "$LOG") first_sigsegv=$(grep -o 'guestpc=0x1021f[0-9a-f]*' "$LOG" | head -1)"
exit 0