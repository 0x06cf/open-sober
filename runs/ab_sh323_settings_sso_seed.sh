#!/bin/bash
# SH323 clean A/B: the SH322 NEXT fencepost — seed the settings global std::string
# [0x106ed7a18] so fn 0x1021f5078 (whitespace-check reached after the SH273 wall) completes
# instead of faulting fault=0x0. Both arms stack SH320 (MAIN-path reach) + SH322 (SH273 wall
# cross); FORWARD adds JIT_ROUTEB_SETTINGS_SSO_SEED=1. Markers: the SH323 seed log line and
# the first SIGSEGV's guestpc (0x1021f5078 = the SH322 terminal; advancing past = SH323 forward).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1"
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
LOG=/tmp/sh323-ab-off.txt; rm -f "$LOG"
timeout 150 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "BASELINE(SH320+322) EXIT=$? first_sigsegv=$(grep -o 'guestpc=0x1021f[0-9a-f]*' "$LOG" | head -1) seed322=$(grep -c 'routeb-sh322' "$LOG")"
LOG=/tmp/sh323-ab-on.txt; rm -f "$LOG"
timeout 150 env $BASE JIT_ROUTEB_SETTINGS_SSO_SEED=1 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "FORWARD(+SSO_SEED) EXIT=$? first_sigsegv=$(grep -o 'guestpc=0x1021f[0-9a-f]*' "$LOG" | head -1) seed323=$(grep -c 'routeb-sh323' "$LOG")"
exit 0