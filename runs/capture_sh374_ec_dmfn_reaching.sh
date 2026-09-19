#!/bin/bash
# SH374: combine the SH373 reaching-env (which deterministically crosses the SH285
# persistence wall 5/5 via JIT_ROUTEB_LSM_APPEND_SKIP ON TOP of the SH371
# DM_CONT_M48_SEED + CONT_APPNAME_SEED) with the DMFN/EC-world marshaller drive.
# SH298-302 measured the EC-world reader-gate block 0x2e24694 as NEVER ENTERED
# (0/3) because control diverged to the SH285 persistence wall BEFORE reaching the
# EC marshaller interior. SH373's append-skip changes that precondition: with SH285
# deterministically crossed, the DMFN drive may now REACH the EC marshaller's
# reader-gate / advance past its fault=0x10 terminal into the app-request build.
# This is the first combination of the two; single-agent, read-only except the
# existing default-inert DMFN/EC guards.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh374-ec-dmfn-reaching.txt
MAX=${SH374_RETRY_MAX:-4}
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 --v2boot-session-dmfn"
RW="0x23f0484-0x23f0900,0x2362e98-0x2362f50,0x23f1210-0x23f1300,0x2e24598-0x2e25200"
run_once() {
  rm -f "$LOG"
  timeout 75 env $BASE JIT_ROUTEB_DMFN_FIELDS=1 JIT_ROUTEB_DMFN_REGISTER=1 \
    JIT_ROUTEB_EC_ARG1=1 JIT_ROUTEB_EC_ARG0VT=1 \
    JIT_ROUTEB_EC_REALSESSION=1 JIT_ROUTEB_EC_READERGATE_FRAME=1 JIT_REGION_WATCH=$RW \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE > "$LOG" 2>&1
  local ec=$(grep -acE "routeb-ec|EC world|0x102e245|EC_ARG1|EC_ARG0VT|EC_REALSESSION" "$LOG")
  local reader=$(grep -acE "0x102e24694|0x102e246b0|reader-gate" "$LOG")
  echo "attempt: exit=$? ec-marker=$ec reader=$reader"
  [ "$ec" -ge 1 ]
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; break; fi
done
echo "=== EC-world markers ==="
grep -aE "routeb-ec|EC world|0x102e245|EC_ARG1|EC_ARG0VT|EC_REALSESSION|reader-gate|0x102e246" "$LOG" | tail -12
echo "=== terminal pcs ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT" "$LOG" | tail -10
echo "=== dmfn/continue markers ==="
grep -aE "SH29[6-8]|DRIVING DM|dmfn returned|dmfn stopped|continueAfterFlagsLoaded|append-skip|SH285|0x101db1b08" "$LOG" | tail -10
echo "won-attempt=${won:-none}"