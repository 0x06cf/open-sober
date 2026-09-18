#!/bin/bash
# SH333 — combine the two strongest SESSION-CTOR levers on ONE non-skip run:
# (a) --v2boot-session-bus (SH315: MessageBus.subscribe drives the REAL registration walk,
#     registry 0->12 headlessly, on the skip-appstart route) and
# (b) the SH320/322/323 MAIN-path dispatch stack + SH330 +0x408 app-start cross.
# Question (SH331 candidate #1, the decisive one): with the bus route having populated the
# service registry AND the do-init done-path MAIN binder-dispatch + app-start +0x408 cross
# running on the same ladder, does the registry count rise / "App" get registered / the tier-2
# controller-name cell move off "Runtime0" / DM-root or once-slot shift? Uses NO
# --v2boot-skip-appstart so V2StartAppWithParams reaches the +0x408 fork.
# (Earlier draft probe_sh332 used skip-appstart -> +0x408 unreachable; this is the corrected one.)

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
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 JIT_ROUTEB_APPSART_408SEED=1"
RW="0x1021e2a90-0x1021e2b40,0x102168798-0x102168840,0x1025f504c-0x1025f5060"
run_one() {
  local lbl=$1 EXTRA=$2
  local LOG=/tmp/sh333-${lbl}.txt; rm -f "$LOG"
  timeout 170 env $BASE $EXTRA JIT_REGION_WATCH="$RW" JIT_DUMP_PC=0x1025f501c,0x1025f5060 \
    $BIN $SO 0x2173ff4 --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-session-bus >"$LOG" 2>&1
  local EX=$?
  echo "== [$lbl] EXIT=$EX =="
  echo "  guard408=$(grep -c routeb-appstart408 "$LOG") fork501c=$(grep -c 'DUMPPC pc=0x1025f501c' "$LOG") fork5060=$(grep -c 'DUMPPC pc=0x1025f5060' "$LOG")"
  echo "  reg=$(grep -oE 'service-registry-count\[0x106fe2f08\]=[0-9]+' "$LOG" | tail -1)"
  echo "  dm=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' "$LOG" | tail -1)"
  echo "  app-registered: $(grep -c 'App@' "$LOG")  fixidx308: $(grep -oE 'SH318 per-entry.*' "$LOG" | tail -1 | head -c 120)"
  echo "  regwalk=$(grep -c 'region hit at guest pc=0x1021e2a9\|region hit at guest pc=0x1021e2b2' "$LOG") lookup=$(grep -c 'region hit at guest pc=0x1021687' "$LOG")"
  echo "  crash: $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+' "$LOG" | tail -3 | tr '\n' ' ')"
}
run_one bus408 ""
echo "---------------------------"
run_one bus408_only "JIT_ROUTEB_APPSART_408SEED=0 JIT_ROUTEB_DONEPATH_MAIN=0"
exit 0