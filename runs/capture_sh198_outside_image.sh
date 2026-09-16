#!/bin/bash
# SH198 repro: deterministic diagnostic for the run-variable "outside image" stop
# (SH176/SH103/SH109 singleton-vtable host-pointer class). JIT_OUTSIDE_TRACE
# prints the last ~16 guest block pcs at the out-of-image stop, showing the exact
# transition out without the megabyte JIT_TRACE dump. Expect 3-5 out-of-image
# reports (one per V2 rung), repeating <10 lines each, EXIT 124, 0 SIGSEGV/SIGABRT.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh198-outside-image-ringbuf.txt
rm -f "$LOG"
timeout 100 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  JIT_OUTSIDE_TRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== out-of-image source transitions (deterministic) ==="
grep -A8 "recent block pcs" "$LOG" | head -30
echo "=== how many rungs hit the outside-image stop ==="
grep -cE "stopped: run_loop: pc .* outside image" "$LOG"
echo "=== crash summary (must be 0) ==="
grep -cE "SIGSEGV|SIGABRT|stack smash" "$LOG"