#!/bin/bash
# SH208: characterize the SH55/64 residual run-variable flakes on the full V2
# ladder. Goal: make the Route-B ladder deterministically clean (real JIT
# hardening, not a seed). The 2 known residual sites (0x104c393f0 system-dialog,
# 0x10284ce54 do-init __call_once blr x2) keep a "corrected" run ~8/10 clean.
set -u
cd "$(dirname "$0")/.."
N=${1:-8}
RM=/tmp/sh208-flake-faults.txt; rm -f "$RM"
CLEAN=0; FAULT=0
for i in $(seq 1 "$N"); do
  LOG=/tmp/sh208-run-$i.txt
  timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 JIT_ROUTEB_DM_SERVICE_NODE=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  # extract fault details
  FAULTLINE=$(grep -iE "SIGSEGV|SIGABRT|fault pc|guestpc|CAUGHT" "$LOG" | head -4)
  if [ -z "$FAULTLINE" ] || [ "$E" = "0" ] || [ "$E" = "124" ]; then
    CLEAN=$((CLEAN+1))
    echo "run $i: EXIT=$E CLEAN"
  else
    FAULT=$((FAULT+1))
    echo "run $i: EXIT=$E FAULT"
    echo "$FAULTLINE" | sed 's/^/    /'
    echo "* run $i ($E): $FAULTLINE" >> "$RM"
  fi
done
echo "=== $CLEAN clean / $FAULT fault / $N total ==="
echo "=== distinct fault signatures ==="
[ -f "$RM" ] && sort "$RM" | uniq -c || echo "(none)"