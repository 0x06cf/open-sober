#!/bin/bash
# SH130: COMBINED-FRAME UNBLOCK — the worker-admission gate makes the serialized
# combined run (--v2boot ladder + --renderinit + taskv4-self-driven frames)
# present REAL task-driven frames in the SAME run that constructs the session.
# Recon deleg_5a9376df: the combined flake (false "stack smashing" during
# rung-0 nativeInit, SH55/64) is the engine's self-spawned clone workers
# (pthread_create start_routine 0x10284d168, tids 1/2) running guest code
# CONCURRENTLY with the ladder's rung jit_runs. JIT_SERIALIZE_RENDER only gates
# the harness renderinit thread, NOT the engine's clones. SH130 adds a host-side
# admission gate (jit.rs WORKER_ADMISSION_GATE): when JIT_SERIALIZE_RENDER=1 +
# --v2boot, each worker's top-level jit_run parks until LADDER_DONE (no guest
# bytes touched; deadlock-safe per SH93's NOP'ed CEvent barrier + no-op join).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh130-combined-frames.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 \
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
echo "=== worker gate (SH130) ==="
grep -E "worker-gate\]" "$LOG"
echo "=== ladder + serialization ==="
grep -E "ladder done|joined cleanly" "$LOG" | head -3
echo "=== REAL TASK-DRIVEN FRAMES PRESENTED IN THE COMBINED RUN (the deliverable) ==="
grep -cE "taskv4-frame\] present #" "$LOG"
echo "=== crashes / signal ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"