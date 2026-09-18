#!/bin/bash
# SH320 clean A/B on the SH319-exact box-build marker 0x102206e34 (a DUMPPC-samplable point).
# baseline (JIT_ROUTEB_DONEPATH_MAIN OFF): SH319 proved 0x102206e34 FIRES (box-build, b.ne taken).
# crate-guard ON: dispatcher b.eq taken -> MAIN binder-dispatch -> box-build must NOT fire.
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
LOG=/tmp/sh320-ab2-baseline.txt; rm -f "$LOG"
timeout 140 env $BASE JIT_DUMP_PC=0x102206e34 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "BASELINE EXIT=$? boxbuild_0x2206e34=$(grep -c 'DUMPPC pc=0x102206e34' "$LOG") guard_seed=$(grep -c 'routeb-sh320' "$LOG")"
LOG=/tmp/sh320-ab2-guard.txt; rm -f "$LOG"
timeout 140 env $BASE JIT_ROUTEB_DONEPATH_MAIN=1 JIT_DUMP_PC=0x102206e34 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "CRATE-GUARD EXIT=$? boxbuild_0x2206e34=$(grep -c 'DUMPPC pc=0x102206e34' "$LOG") guard_seed=$(grep -c 'routeb-sh320' "$LOG")"
exit 0