#!/bin/bash
# SH167 end-to-end probe: enable JIT_DM_ALLOC_CAPTURE on the DMCONT ladder. The guard seeds the
# CRT operator-new ACTIVE hook so every operator-new blr's the capture trail (real calloc + log).
# Question: does the boot stay clean with ALL operator-new routed through the trail, and does the
# trail log/capture DM-plausible allocations?
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh167-dm-alloc-capture.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_DM_ALLOC_CAPTURE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH167 seed fired? ==="
grep -E "routeb-dmalloc" "$LOG" | head -5
echo "=== capture count / first captures ==="
grep -cE "\[routeb-dmalloc\] capture" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|SendAppEventOnAppReady returned" "$LOG" | head -3
echo "=== crashes / signals ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"
echo "=== tail ==="
tail -12 "$LOG"