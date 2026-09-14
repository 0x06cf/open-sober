#!/bin/bash
# Loop the SH128 redrive capture until we get a clean ladder (the combined run is
# run-variable SH55/64; ~50% crash in the ladder) AND check the redrive presented
# frames. Keeps the LAST full log + a per-attempt summary line.
for i in 5 6; do
  echo "================ attempt $i ================"
  LOG=/home/hermes-worker/runs/open-sober/runs/sh128-redrive-frame.txt
  timeout 220 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 \
    RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 TASKV4_REDRIVE_MS=7000 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot \
    --v2boot-surface-handoff --v2boot-send-appevent \
    --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles \
    --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 \
    --deque-redrive \
    --persist-roundtrip --kicker 0x106863af8 > "$LOG" 2>&1
  E=$?
  echo "EXIT=$E ladder=$(grep -c 'ladder done' $LOG) join=$(grep -c 'joined cleanly' $LOG) redrive=$(grep -c 'redrive\] running drain' $LOG) redrive_skip=$(grep -cE 'redrive\] SKIP' $LOG) frames=$(grep -c 'taskv4-frame\] present #' $LOG) kills=$(grep -icE 'SIGSEGV|SIGABRT' $LOG)"
  if grep -q "redrive\] running drain" "$LOG"; then
    echo "--- redrive detail ---"; grep -E "redrive\]" "$LOG" | head -12
  fi
  if [ "$E" = "0" ]; then break; fi
done