#!/bin/bash
# SH330 exact repro: sh328 command line + JIT_ROUTEB_APPSART_408SEED=1.
# Expect guard fires, 0x1025f501c crossed (no SIGSEGV there).
cd "$(dirname "$0")/.."
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
JIT_ROUTEB_CONT_APPNAME_SEED=1 JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
JIT_ROUTEB_APPSART_408SEED=1 JIT_DUMP_PC=0x102e89150,0x1025f5460,0x1025f501c,0x1025f5060"
LOG=/tmp/sh330-repro.txt; rm -f "$LOG"
timeout 200 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "EXIT=$?"
echo "guard_fires=$(grep -c 'routeb-appstart408' "$LOG")"
echo "crossed_25f50=$(grep -c 'DUMPPC pc=0x1025f50' "$LOG")"
echo "first_sigsegv=$(grep -o 'guestpc=0x[0-9a-f]*' "$LOG" | head -1)"
echo "sigsegv_at_501c=$(grep -c 'guestpc=0x1025f501c' "$LOG")"
echo "--- guard lines ---"; grep 'routeb-appstart408' "$LOG" | head
echo "--- dump at 0x1025f5060 (downstream) ---"; grep 'DUMPPC pc=0x1025f5060' "$LOG" | head -1