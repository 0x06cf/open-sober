#!/bin/bash
# SH212: characterize the current residual ladder flakes at HEAD — FMOD/AAudio NULL-this
# + raced flag-manager system_error. Canonical completing ladder, GSDSP + outside-trace on.
# Prints per-run EXIT / clean | crash(guestpc,GSDSP) and a final clean-count. Doc:
# docs/frontier-sh212-residual-flakes-characterized.md
set -u
cd "$(dirname "$0")/.."
N=${1:-8}
CLEAN=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh212-$i.txt"; rm -f "$LOG"
  timeout 105 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
    JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
    JIT_GUEST_STACK_DUMP=1 JIT_OUTSIDE_TRACE=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    --v2boot-set-filesdir > "$LOG" 2>&1
  E=$?
  if grep -aqE "SIGSEGV|SIGABRT|system_error" "$LOG"; then
    sig=$(grep -aoE "SIGSEGV|SIGABRT|system_error" "$LOG" | head -1)
    pc=$(grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | head -1)
    gsdsp=$(grep -aoE "GUEST\(0x[0-9a-f]+\)" "$LOG" | head -2 | tr '\n' ' ')
    echo "run $i: EXIT=$E CRASH [$sig] $pc gsdsp=$gsdsp"
  else
    CLEAN=$((CLEAN+1)); echo "run $i: EXIT=$E CLEAN"
  fi
done
echo "=== SH212: clean=$CLEAN/$N ==="
echo "Expected residuals (non-seedable): FMOD AAudio NULL-this @guest 0x106240d8c; raced std::system_error in nativeInitializeNativeFlags."