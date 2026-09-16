#!/bin/bash
# SH205: post-family fault is SEEDABLE SH116-class singleton (flag-manager global
# 0x10672739b0), NOT the migration gate. With SH116b the V2 ladder is 10/10 clean
# (SendAppEvent Ok). Run N=10 by default; prints clean/fault tally + SH116b fired.
set -u
cd "$(dirname "$0")/.."
N=${1:-10}
clean=0; fault=0; fired=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh205-$i.txt"
  rm -f "$LOG"
  timeout 110 env \
    JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_SETWORLDBUILD=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  f=$(grep -ac 'SH116b patched' "$LOG")
  fired=$((fired+f))
  if grep -aqcE "SIGSEGV|SIGABRT" "$LOG"; then
    fault=$((fault+1)); echo "run $i: FAULT (SH116b=$f)"
  else
    clean=$((clean+1)); echo "run $i: clean (SH116b=$f ladder=$(grep -ac 'SendAppEventOnAppReady returned Ok' "$LOG"))"
  fi
done
echo "=== summary: $clean clean / $fault fault / $N total; SH116b fired $fired runs ==="
echo "expect clean=$N fault=0 fired=$N (SH205: 10/10 clean; was 8/12 before)"