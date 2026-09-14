#!/bin/bash
# SH127: SERIALIZED combined run — the full --v2boot rung ladder completes
# ("ladder done") FIRST, THEN the render pipeline recovers the real ctx
# (JIT_SERIALIZE_RENDER=1 makes renderinit wait for LADDER_DONE so render
# jit_runs never overlap the ladder's, avoiding the SH55/64 block-cache
# desync), AND the --deque-node-live producer is gated on LADLED_DONE +
# RENDERCTX so its injected type-4 nodes dispatch AFTER the presenter is
# ready -> real task-driven frames present (swap Ok(0x1)) in the SAME
# reproducible ladder+render run (pre-SH127: 0 frames, pending=0 — the
# 400-tick capture budget burned pre-recovery and the injector gave up).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh127-serial-ladder-frame.txt
rm -f "$LOG"
timeout 300 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 \
  RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles \
  --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the SH127 gate (producer waited for LADLER_DONE+RENDERCTX) ==="
grep -E "serialized gate passed|gave up after" "$LOG"
echo "=== ladder + render serialization ==="
grep -E "ladder done|LADDER_DONE received|published RENDERCTX|joined cleanly" "$LOG"
echo "=== REAL TASK-DRIVEN FRAMES PRESENTED (the deliverable) ==="
grep -E "taskv4-frame\] present #" "$LOG" | tail -5
echo "=== present count ==="
grep -cE "taskv4-frame\] present #" "$LOG"
echo "=== node pops (real type-4 drain activity) ==="
grep -cE "NODE .* POPPED" "$LOG"
echo "=== crash / json summary ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"
grep -iE "string length overflow" "$LOG" || echo "(no json abort)"