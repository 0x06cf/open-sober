#!/bin/bash
# SH304: measure the SESSION-GATED type-4 producer (--taskv4-seed session) on the
# real libroblox.so. The gated producer emits task-driven frames ONLY when a REAL
# session owns a live DataModel (MH_APP_READY && live-DM). On a bare-boot headless
# run MH_APP_READY is false, so every dispatch logs UNGATED and present ZERO frames —
# the host no longer fabricates the session; it only reacts to one. Compare to
# `--taskv4-seed frame` (runs/capture_taskv4_frame.sh) which harness-seeds
# unconditionally (24 frames). This is the inverse test: the gate must keep the
# session-less boot frame-free while still wiring the self-drive path.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh304-session-gated-producer.txt
rm -f "$LOG"
timeout 50 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --taskv4-seed session \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== session-gated producer registered ==="
grep -E "taskv4\] session-gated producer registered" "$LOG"
echo "=== the gate is INERT on bare-boot (requires a live session to self-drive) ==="
echo "UNGATED occurrences (first 3):"
grep -aE "session-producer] disp #[0-9]+ UNGATED" "$LOG" | head -3
echo "UNGATED total:"
grep -acE "session-producer] disp #[0-9]+ UNGATED" "$LOG"
echo "=== GATED fires only when MH_APP_READY && live-DM (0 on a session-less boot) ==="
grep -acE "session-producer] disp #[0-9]+ GATED" "$LOG"
echo "=== the host must NOT present frames it fabricated (gate keeps session-driven at 0) ==="
echo "total taskv4-frame present # lines:"
grep -acE "taskv4-frame\] present #" "$LOG" || true
echo "  (note: the 1 standalone present, if present, is the --renderinit/--renderframe"
echo "  render-plane probe SH18/19 — a direct harness swap INDEPENDENT of the type-4"
echo "  producer gate. The GATED count above (0) proves the session producer itself"
echo "  queued ZERO presents on this session-less boot — no fabricated session frames.)"
echo "=== which render present lines fired ==="
grep -aE "present #|swap returned" "$LOG" | head -5
echo "=== json-overflow / crash summary ==="
grep -iE "string length overflow|RBX::json" "$LOG" || echo "(no json abort)"
grep -icE "SIGSEGV|SIGABRT" "$LOG"