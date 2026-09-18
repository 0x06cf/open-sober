#!/bin/bash
# Probe: reproduce the SH323 forward and dump full registers at the terminal wall
# 0x102256510 to pin the caller contract (x0 this, x5/x17 return, sp pair) for the
# next SESSION-CTOR decision. Same base as ab_sh323_settings_sso_seed.sh forward arm.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1 \
  JIT_ROUTEB_SETTINGS_SSO_SEED=1 JIT_DUMP_REGION=0x10225500-0x10225680"
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
LOG=/tmp/sh324-probe.txt; rm -f "$LOG"
timeout 150 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "EXIT=$?"
echo "first_sigsegv=$(grep -o 'guestpc=0x[0-9a-f]*' "$LOG" | head -1)"
echo "sigsegv_lines:"; grep -iE "segv|signal|fault" "$LOG" | head -5
echo "dump_region_fires=$(grep -c 'JIT_DUMP_REGION\|canary\|dump-region\|region' "$LOG")"
echo "--- register dumps near terminal ---"
grep -iE "x0=|x[0-9]+=|regs|dump" "$LOG" | tail -25
echo "--- tail log ---"; tail -30 "$LOG"