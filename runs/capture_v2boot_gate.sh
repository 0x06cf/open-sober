#!/bin/bash
# SH81 repro: Route-B v2boot ladder with the dispatch-gate force + singleton-record
# seed — the ladder now runs CRASH-FREE (0 SIGSEGV/SIGABRT), nativeInitializeNativeFlags
# (rung 0) returns, and the productized render/persist baseline stays green in the same
# run. Expect EXIT=124 (stable idle at the timeout), not 134/abort.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh81-v2boot-gate.txt
rm -f "$LOG"
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --renderframe-drawprobe \
  --renderframe-triangle --renderframe-quad --renderframe-quad-loop 3 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== routeB patches ===="
grep -E "routeB" "$LOG"
echo "=== v2boot rungs (order + per-rung vector) ==="
grep -E "v2boot\] (after|driving|returned|stopped|ladder done)" "$LOG"
echo "=== crash summary (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"
echo "=== persist baseline ==="
grep -E "persist\] live datastore" "$LOG"
echo "=== render baseline ==="
grep -E "renderframe-(triangle|quad)\]( readback| geometry)|quad-loop\] iter" "$LOG" | head -8