#!/bin/bash
# SH375: re-test the SH362 closure under the SH373 reaching-env. SH362 measured the
# do-init MAIN-branch dispatch body fn 0x258b5d8 (StartAppWithParams+0x494) is NEVER
# entered because EVERY run faults at the SH285 persistence wall (0x101db1b08) BEFORE
# control reaches it — that was a REACHABILITY closure premised on the SH285 wall being
# terminal. SH373 then CROSSED SH285 deterministically 5/5 (JIT_ROUTEB_LSM_APPEND_SKIP on
# top of the SH371 DM_CONT_M48_SEED + CONT_APPNAME_SEED reaching env). So SH362's premise
# ("run dies at SH285, body unreachable") is now obsolete: with SH285 crossed, the
# 0x258b5d8 dispatch body may FIRE. This is the first combination of the two — a pure
# map-completion measurement (READ-ONLY JIT_ROUTEB_DISPATCH_BODY_TRACE + the crossing env).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh375-dispatch-body-reaching.txt
MAX=${SH375_RETRY_MAX:-4}
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1"
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus"
run_once() {
  rm -f "$LOG"
  timeout 80 env $BASE JIT_ROUTEB_DISPATCH_BODY_TRACE=1 JIT_ROUTEB_APPEVENT_W19=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SL > "$LOG" 2>&1
  local body=$(grep -acE "routeb-dispatch-body" "$LOG")
  local sh285=$(grep -acE "guestpc=0x101db1b08" "$LOG")
  echo "attempt: exit=$? dispatch-body-hits=$body sh285=$sh285"
  [ "$body" -ge 1 ] || [ "$sh285" = "0" ]
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; fi
done
echo "=== SH362 dispatch-body trace (0/ALL = body never entered) ==="
grep -aE "routeb-dispatch-body" "$LOG" | tail -12
echo "=== SH285 hits (0 = crossed) ==="
grep -acE "guestpc=0x101db1b08" "$LOG"
echo "=== terminal ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|outside image" "$LOG" | tail -10
echo "=== session/continuation/append markers ==="
grep -aE "continueAfterFlagsLoaded|append-skip|DM-root probe|MH_APP_READY|governor|SendAppEventOnAppReady returned|0x10258b5d8" "$LOG" | tail -10
echo "won-attempt=${won:-none}"