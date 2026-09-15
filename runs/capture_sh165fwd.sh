#!/bin/bash
# SH165-fwd (manager re-seed) repro v2: verify the scoped re-seed of the NativeDataModelManager
# singleton holder at guest 0x102727550 fires on fnB entry (JIT_ROUTEB_DMFORCE=1) and the whole
# engine-init pipeline (fnB 0x102bd1b98 -> 0x102bd8ce8 manager vt dispatch) benign-completes:
# 0 SIGSEGV/ABORT, ladder done.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh165fwd-manager-reseed.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x102bd1a30-0x102bd1d08 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the SH165-fwd manager re-seed (the deliverable) ==="
grep -E "SH165 manager singleton holder" "$LOG"
echo "=== the SH165 dm-force shell (vt[+0x30]=engine-init) ==="
grep -E "fabricated NativeDataModelManager instance" "$LOG"
echo "=== region-watch: fnB engine-init region entered? ==="
grep -E "entered region 0x102bd1a30" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|joined cleanly|SendAppEventOnAppReady returned" "$LOG" | head -5
echo "=== crashes / signal ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"
echo "=== tail ==="
tail -20 "$LOG"