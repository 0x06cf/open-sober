#!/bin/bash
# SH320 A/B: with --v2boot-bus-mainid the do-init done-path dispatcher fork flips.
# Baseline (seed OFF, JIT_DUMP_PC=0x102206e34 box-build) should FIRE (SH319 reach);
# seed ON should NOT fire box-build (b.ne not taken -> MAIN binder-dispatch 0x206df4).
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
# --- BASELINE (seed OFF): box-build 0x102206e34 should FIRE (SH319) ---
LOG=/tmp/sh320-baseline.txt; rm -f "$LOG"
timeout 140 env $BASE JIT_DUMP_PC=0x102206e34 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "BASELINE(seed OFF) EXIT=$? boxbuild_0x2206e34_hits=$(grep -c 'DUMPPC pc=0x102206e34' "$LOG")"
# --- SEED ON: box-build should NOT fire; probe MAIN-branch block entry 0x102206ea4 (binder fall-through target after 0x206df4) ---
LOG=/tmp/sh320-seedon.txt; rm -f "$LOG"
timeout 140 env $BASE JIT_DUMP_PC=0x102206e34 $BIN $SO 0x2173ff4 $ARGS --v2boot-bus-mainid > "$LOG" 2>&1
echo "SEED-ON EXIT=$? boxbuild_0x2206e34_hits=$(grep -c 'DUMPPC pc=0x102206e34' "$LOG") seed_fired=$(grep -c 'SH320 --v2boot-bus-mainid' "$LOG")"
echo "--- seed-on SH155 ---"
grep -E "SH155 DM-root" "$LOG" | tail -1
exit 0