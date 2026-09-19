#!/bin/bash
# SH376: test the genuinely-unrun combination — the SH373 crossing env (LSM_APPEND_SKIP,
# crosses SH285 5/5) PLUS the SH269 governor-predicate seed GOVFLAG. SH375's ledger
# attributed the terminal to "SetInitParams SIGABRT", but actual runs show SetInitParams/
# V2Init both soft-return benignly and the genuine crash is the governor NULL-DM deref at
# 0x102ea0b9c (reads [x21+1032]=[controller+0x408]=0) — the SH269 wall, already provisioned
# but never armed in that env. Running now with GOVFLAG to see if the governor terminal advances.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh376-govflag-crossing.txt
MAX=${SH376_RETRY_MAX:-4}
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1"
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus"
run_once() {
  rm -f "$LOG"
  timeout 80 env $BASE JIT_ROUTEB_DISPATCH_BODY_TRACE=1 JIT_ROUTEB_APPEVENT_W19=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SL > "$LOG" 2>&1
  local gov=$(grep -acE "routeb-sh269|governor-predicate" "$LOG")
  local sh285=$(grep -acE "guestpc=0x101db1b08" "$LOG")
  local body=$(grep -acE "routeb-dispatch-body" "$LOG")
  echo "attempt: exit=$? govflag=$gov sh285=$sh285 dispatch-body=$body"
  [ "$gov" -ge 1 ]
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; fi
done
echo "=== terminal ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|outside image|startAppWithParams|0x258c6e4|do-init|DM-root probe|MH_APP_READY" "$LOG" | tail -16
echo "=== govflag + governor markers ==="
grep -aE "routeb-sh269|governor|0x102ea0b9c|0x2ea3a84|SendAppEventOnAppReady" "$LOG" | tail -12
echo "won-attempt=${won:-none}"