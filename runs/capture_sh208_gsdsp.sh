#!/bin/bash
# SH208: JIT_GUEST_STACK_DUMP characterization of the ladder's 3rd residual
# flake site (0x1021e34b0, FMOD/JNICallProtocol). Goal: prove definitively
# whether it is (a) a FIXED .bss global -> SH116-class, seedable code-patch to a
# low page, or (b) a run-variable HOST-heap object (live-walker class) -> the
# documented non-seedable SH55/64 do-not-chase. GSDSP tags the fault caller.
set -u
cd "$(dirname "$0")/.."
N=${1:-6}
FAULTED=0
for i in $(seq 1 "$N"); do
  LOG=/tmp/sh208-gsdsp-$i.txt
  timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 JIT_ROUTEB_DM_SERVICE_NODE=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 JIT_GUEST_STACK_DUMP=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  F=$(grep -iE "SIGSEGV|SIGABRT" "$LOG" | grep -i "fault\|guestpc" | head -1)
  G=$(grep -iE "GSDSP" "$LOG" | head -2)
  if [ -z "$F" ]; then echo "run $i: EXIT=$E CLEAN"; else
    FAULTED=$((FAULTED+1)); echo "run $i: EXIT=$E FAULT: $F"
    [ -n "$G" ] && echo "    $G"
  fi
done
echo "=== $((N-FAULTED))/$N clean, $FAULTED faulted ==="