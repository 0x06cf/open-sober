#!/bin/bash
# Probe SH325-A: does SH160 init3-gate NOP actually clear the 0x102256510 terminal?
# Runs StartLuaAppDM (no --v2boot-skip-appstart) so the JIT_ROUTEB_DM_SEED block incl
# routeb_patch_startapp_init3_gates fires. A/B: JIT_ROUTEB_INIT3=1 vs default.
set -u
cd "$(dirname "$0")/.."
RUN () {
  local init3="$1"
  local BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    V2BOOT_WARMUP_MS=1200 ${init3} \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1 \
    JIT_ROUTEB_SETTINGS_SSO_SEED=1"
  local BIN=./target/debug/examples/elfjit
  local SO=~/.cache/open-sober/robbox/libroblox.so
  local ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session"
  local LOG=/tmp/sh325A-$2.txt; rm -f "$LOG"
  timeout 150 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
  echo "RUN $2 EXIT=$? scalar=$(grep -oE 'guestpc=0x[0-9a-f]+' "$LOG" | head -1) sig=$(grep -icE 'SIGSEGV|SIGABRT' "$LOG") SH160=$(grep -ic 'SH160' "$LOG") SH325=$(grep -ic 'SH325' "$LOG")"
  echo "  SH160 lines:"; grep -iE "SH160|init3 gate" "$LOG" | head -3
}
RUN "" "default"
RUN "JIT_ROUTEB_INIT3=1" "init3"