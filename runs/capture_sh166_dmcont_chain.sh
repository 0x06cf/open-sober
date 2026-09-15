#!/bin/bash
# SH166 trace: under DMCONT (JIT_ROUTEB_DMCONT=1), watch the full fnB -> 0x102bd8ce8 ->
# continueAfterFlagsLoaded_ (0x102bd1d68) chain to find the EMPIRICAL execution floor.
# Question: does vt[+0x1f0] (continueAfterFlagsLoaded_) actually get dispatched when the
# fabricated manager M's +0x1f0 is routed to real 0x102bd1d68?
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh166-dmcont-chain.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_DMCONT=1 \
  JIT_REGION_WATCH=0x102bd1a30-0x102bd9058 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH165 manager re-seed fired? ==="
grep -E "SH165 manager singleton holder" "$LOG"
echo "=== region-watch hits along the fnB->0x102bd8ce8->continueAfterFlagsLoaded_ chain ==="
grep -E "region-watch" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|joined cleanly|SendAppEventOnAppReady returned" "$LOG" | head -5
echo "=== crashes / signal ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"
echo "=== tail ==="
tail -25 "$LOG"