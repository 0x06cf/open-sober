#!/bin/bash
# SH376b: the genuinely-new combined env — SH373 crossing env (DM_CONT_M48 + CONT_APPNAME +
# LSM_APPEND_SKIP, which crosses SH285 5/5) + GOVFLAG + PRELOAD_VALUECELL. The SH352 completing
# ladder has GOVFLAG+PRELOAD_VALUECELL but LACKS the SH371 reaching seeds; the SH373 crossing env
# lacks GOVFLAG+PRELOAD_VALUECELL. This intersection was never run. If the crossing env + both
# session-forward levers hold, the crossed-SH285 terminal should advance through the governor and
# preload walls to a FRESH downstream site.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh376b-combined.txt
MAX=${SH376B_RETRY_MAX:-4}
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
  JIT_ROUTEB_DOINIT_EMPTYVEC=1 JIT_ROUTEB_APPEND_SKIP=1"
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus"
run_once() {
  rm -f "$LOG"
  timeout 90 env $BASE JIT_ROUTEB_APPEVENT_W19=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SL > "$LOG" 2>&1
  local gov=$(grep -acE "routeb-sh269|governor-predicate" "$LOG")
  local sh307=$(grep -acE "sh307|PRELOAD_VALUECELL|preload-getter" "$LOG")
  local sh285=$(grep -acE "guestpc=0x101db1b08" "$LOG")
  local sh308=$(grep -acE "do-init|DM-root probe|MH_APP_READY|startAppWithParams|0x258c6e4" "$LOG")
  echo "attempt: exit=$? gov=$gov sh307=$sh307 sh285=$sh285"
  [ "$gov" -ge 1 ]
  echo "$sh308" | tail -3
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; fi
done
echo "=== terminal ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|outside image|SendAppEventOnAppReady returned|DM-root probe|MH_APP_READY" "$LOG" | tail -16
echo "won-attempt=${won:-none}"