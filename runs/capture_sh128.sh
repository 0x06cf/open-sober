#!/bin/bash
# SH128: COMBINED-FRAME re-drive — the full --v2boot serialized ladder completes
# ("ladder done", StartApp RETURNS), THEN --deque-redrive makes the MAIN thread
# re-run the engine's drain pop-loop 0x102856e40 (root={headcell,tag}, x1 =
# scheduler via TLS getter, finite timeout). --deque-node-live injects type-4
# nodes into the live re-driver; the type-4 thunk bumps PENDING_PRESENTS; the
# renderinit presenter (currency thread) holds open and presents real frames.
# This unifies session-construction + task-driven rendering into ONE run —
# the frontier-sh127 structural gap (no thread resident in the drain post-return).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh128-redrive-frame.txt
rm -f "$LOG"
timeout 300 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 \
  RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 TASKV4_REDRIVE_MS=7000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles \
  --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 \
  --deque-redrive \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== ladder + render serialization ==="
grep -E "ladder done|LADDER_DONE received|joined cleanly" "$LOG" | head -3
echo "=== RE-DRIVE path (SH128) ==="
grep -E "redrive\]" "$LOG"
echo "=== REAL TASK-DRIVEN FRAMES PRESENTED (the deliverable) ==="
grep -E "taskv4-frame\] present #" "$LOG" | tail -5
echo "=== present count ==="
grep -cE "taskv4-frame\] present #" "$LOG"
echo "=== node pops ==="
grep -cE "NODE .* POPPED" "$LOG"
echo "=== crashes ==="
grep -icE "SIGSEGV|SIGABRT|stack st" "$LOG"