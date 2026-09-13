#!/bin/bash
# SH82 repro: --v2boot ladder now UNPARKS rung 1 — the GlobalInit thread-dispatch
# main-thread-id cell [0x106863a68] is seeded with the ladder thread's own
# pthread_self, so nativeGameGlobalInit takes the match path and runs its real
# do-init instead of parking in the 0x2207648 completion spin forever. Expect the
# ladder to ADVANCE past rung 1 and fault FURTHER at guestpc 0x1021daf78 (the next
# gate) rather than parking (pre-SH82: exit 124, never prints "after
# nativeGameGlobalInit").
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh82-v2boot-advance.txt
rm -f "$LOG"
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH82 seed (must appear once, before rung 1) ==="
grep -E "seeded main-thread-id" "$LOG"
echo "=== rung 1 — ADVANCED past the park? (fault: guestpc 0x1021daf78) ==="
grep -E "nativeGameGlobalInit|after nativeGameGlobalInit" "$LOG"
grep -oE "guestpc=0x1021daf78" "$LOG" | head -1
echo "=== (baseline pre-SH82 parked here: exit 124, no fault) ==="
echo "=== product baseline (no --v2boot) sanity ==="
grep -oE "persist\] live datastore" "$LOG" | head -1