#!/bin/bash
# SH300: A/B the EC-world realsession flag seed. ON arm fires
# `[routeb-sh300] seeded EC-world realsession flag [0x106d31e28]=1`; terminal
# parity (SH285-B LSM wall 0x101db1b08) proves dormant-by-measurement prep.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
COMMON="JIT_ROUTEB_DMFN_FIELDS=1 JIT_ROUTEB_DMFN_REGISTER=1 JIT_ROUTEB_EC_ARG1=1 JIT_ROUTEB_EC_ARG0VT=1"
SUM="/home/hermes-worker/runs/sh300-ab.txt"
rm -f "$SUM"
for arm in OFF ON; do
  EXTRA=""
  [ "$arm" = ON ] && EXTRA="JIT_ROUTEB_EC_REALSESSION=1"
  t=$(mktemp)
  timeout 55 env $BASE $COMMON $EXTRA \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE --v2boot-session-dmfn > "$t" 2>&1
  E=$?
  fired=$(grep -ac "routeb-sh300" "$t")
  term=$(grep -aoE "SIGSEGV|SIGABRT|guestpc=0x[0-9a-f]+" "$t" | tail -2 | tr '\n' ' ')
  dmfn=$(grep -aoE "SH296 dmfn returned Ok\([^)]*\)" "$t" | tail -1)
  echo "$arm EXIT=$E fired=$fired dmfn=$dmfn term=$term" | tee -a "$SUM"
  mv "$t" "/home/hermes-worker/runs/sh300-$arm-$$.txt"
done
echo "=== saved ==="
cat "$SUM"