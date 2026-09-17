#!/bin/bash
# SH248b: capture x20 (the corrupt std::string `this`) on the deterministic -9
# operator_new line (x30=0x102b506bc, inside libc++ string::assign 0x2b50600).
# The ladder is run-variable (~1/3-1/2 flake BEFORE the continuation), so loop.
hit=0
for i in 1 2 3 4 5 6; do
  LOG=/home/hermes-worker/runs/open-sober/runs/sh248b-p$i.txt
  rm -f "$LOG"
  timeout 100 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
    JIT_ROUTEB_OPNEW_SIZE_GATE=1 JIT_ROUTEB_ALLOC_PROBE=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  LN=$(grep -a "fffffffffffffff7" "$LOG" | head -1)
  echo "run$i EXIT=$E"
  if [ -n "$LN" ]; then echo "   $LN"; hit=1; break; fi
done
[ "$hit" = 0 ] && echo "NO -9 line captured in this batch (ladder flaked before the continuation in all runs)"