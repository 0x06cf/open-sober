#!/bin/bash
# SH321 clean A/B on the SH320 MAIN-path downstream reach (guard-gated).
# Reach marker: JIT_DUMP_PC=0x1021f3748 (SH273 lifecycle-notify body). Baseline (guard OFF)
# must be 0 hits; crate-guard ON (JIT_ROUTEB_DONEPATH_MAIN=1) must be >0 -> the do-init MAIN
# binder-dispatch climbs into real engine settings-init (caller 0x2270024->0x2270050 bl 0x21f3748)
# then faults at the SH273 lifecycle-registry live-object wall.
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
LOG=/tmp/sh321-ab-off.txt; rm -f "$LOG"
timeout 130 env $BASE JIT_DUMP_PC=0x1021f3748 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "BASELINE EXIT=$? reach_0x1021f3748=$(grep -c 'pc=0x1021f3748' "$LOG")"
LOG=/tmp/sh321-ab-on.txt; rm -f "$LOG"
timeout 130 env $BASE JIT_ROUTEB_DONEPATH_MAIN=1 JIT_DUMP_PC=0x1021f3748 $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "GUARD-ON EXIT=$? reach_0x1021f3748=$(grep -c 'pc=0x1021f3748' "$LOG") guard_seed=$(grep -c 'routeb-sh320' "$LOG")"
exit 0