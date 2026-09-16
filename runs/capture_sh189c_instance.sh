#!/bin/bash
# SH189c EXPERIMENTAL: the real PlayerGui instance ctor chain headlessly. This is the Route-B
# frontier probe. With the class-name registry populated (PlayerGui+ScreenGui descriptors) +
# DM planted into the creator's current-DM global 0x107333948, the pair-consumer 0x10255d0e4
# drives the core creator 0x102373458 -> operator-new -> blr 0x255d1b4 (ctor functor) ->
# real ctor 0x255d1dc. VERIFIED it CONSTRUCTS a real object (vptr 0x106796dc0 at 0x2374358)
# then faults on the deep unseeded owner member (ldr x8,[x23] at 0x2374378, x23=0) ->
# expects EXIT 134 = the next unsynthesized-object gate (x23 owner), NOT a clean run.
# This script documents that gate. Do NOT merge its env into the ^clean SH189/SH189b capture.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh189c-instance.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT (expect 134 = x23 owner next-gate fault, the instance drive's honest boundary)"
echo "=== SH189c instance drive (the frontier probe) ==="
grep -E "routeb-dmins" "$LOG"
echo "=== the next-gate fault (x23 owner deref at 0x102374378) ==="
grep -E "guestpc=0x1023743|fault=0x0" "$LOG" | head -2