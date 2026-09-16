#!/usr/bin/env bash
# SH200: V2StartAppWithParams now genuinely returns Ok(0x0) instead of the
# run-variable "outside image" soft-return (a scoped-seedable objB-vtable
# singleton-dispatch family, SH198's "non-seedable" verdict falsified). Patches
# the 4 located dispatch sites under JIT_SH115_SINGLETON_PATCH and verifies V2Start
# completion. default-inert (JIT_SH115_SINGLETON_PATCH=1), single ladder, EXIT 124.
set -u
LOG="${1:-/tmp/sh200-v2dispatch.txt}"
ROOT=/home/hermes-worker/runs/open-sober
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_OUTSIDE_TRACE=1 \
  "$ROOT/target/debug/examples/elfjit" \
  /home/hermes-worker/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
echo "exit=$?"
echo "--- SH200 markers ---"
grep -cE "SH200 patched V2 dispatch" "$LOG"
grep -E "V2StartAppWithParams returned Ok" "$LOG"