#!/bin/bash
# SH244: does the dispatcher's 'getter' 0x102174c04 tail-branch into FMOD/AAudio
# 0x624e6c0 (instead of returning), leaving the dispatcher interior/sub never
# resumed? Region-watch the getter, its tail target, and the dispatcher's
# post-getter pcs + sub_2bd8dac + continueAfterFlagsLoaded_.
# Success/answer markers printed by grep below.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh244-dispatcher-tail.txt
rm -f "$LOG"
# JIT_REGION_WATCH: dispatch=getter 0x102174c04, tail=0x624e6c0, dispatcher resume
# pcs (0x2bd8d18/0x2bd8d30/0x2bd8d54/0x2bd8d60), sub_2bd8dac, continueAfterFlagsLoaded_
timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_REGION_WATCH=0x102174c00-0x102174d40,0x10624e6c0-0x10624e740,0x102bd8d18-0x102bd8d68,0x102bd8dac-0x102bd9058,0x102bd1d68-0x102bd2600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== getter tail-target 0x624e6c0 fired? (if yes, getter tail-branches, not ret) ==="
grep -E "JIT_REGION.*10624e6c0|region.*10624e6|hits" "$LOG" | head -20
echo "=== dispatcher post-getter pcs / sub / continueAfterFlagsLoaded_ hits ==="
grep -E "102bd8d18|102bd8dac|102bd1d68|102bd8d30|102bd8d54" "$LOG" | head -10
echo "=== StartLuaAppDM return + crash ==="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)" "$LOG" | tail -1
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG" || true