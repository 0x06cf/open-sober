#!/bin/bash
# SH332c — the decisive "App"-registration measurement, FIXED run set: stack the full
# SH320/322/323 MAIN-path forward stack (DONEPATH_MAIN + LIFECYCLE_EARLYRET + SETTINGS_SSO_SEED)
# so the do-init MAIN binder-dispatch does NOT die at the SH273 lifecycle wall (0x1021f3748),
# then region-watch the service-REGISTRATION walk (fn 0x21e2a90) + name->service lookup
# (0x2168798) + the +0x408 app-start fork. Question (SH331 candidate #1): with the MAIN path
# driven live, does the registration walk register "App" / move the controller-name cells,
# and does SH330's +0x408 cross then let app-start drive the registry? No --v2boot-skip-appstart.

set -u
cd "$(dirname "$0")/.."
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 \
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
  JIT_ROUTEB_APPSART_408SEED=1"
RW="0x1021e2a90-0x1021e2b40,0x102168798-0x102168840,0x1025f504c-0x1025f5060"
LOG=/tmp/sh332c.txt; rm -f "$LOG"
timeout 150 env $BASE JIT_REGION_WATCH="$RW" JIT_DUMP_PC=0x1021f3748,0x1025f501c,0x1025f5060 \
  $BIN $SO 0x2173ff4 --jni --startapp 0x258b144 --v2boot --v2boot-session >"$LOG" 2>&1
EX=$?
echo "EXIT=$EX"
echo "guard408=$(grep -c routeb-appstart408 "$LOG") lifecycle_seed=$(grep -c routeb-sh322 "$LOG") sso_seed=$(grep -c routeb-sh323 "$LOG")"
echo "regwalk_hits=$(grep -c 'region hit at guest pc=0x1021e2a9\|region hit at guest pc=0x1021e2b2' "$LOG")"
echo "lookup_hits=$(grep -c 'region hit at guest pc=0x1021687' "$LOG")"
echo "fork_501c=$(grep -c 'DUMPPC pc=0x1025f501c' "$LOG") fork_5060=$(grep -c 'DUMPPC pc=0x1025f5060' "$LOG") life_3748=$(grep -c 'DUMPPC pc=0x1021f3748' "$LOG")"
echo "crash: $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+' "$LOG" | tail -3 | tr '\n' ' ')"
echo "--- region hits ---"; grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | sed 's/^/  /'
echo "--- seed/fork lines ---"; grep -E "routeb-sh320|routeb-appstart408|SH315 service-registry|SH318 per-entry|DM-root\[0x106a68818\]" "$LOG" | tail -6
exit 0