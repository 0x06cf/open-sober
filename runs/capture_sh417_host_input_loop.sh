#!/bin/bash
# SH417: measure the persistent-tracker host input LOOP (STATUS next-forward #3)
# on real libroblox.so. The SH416 one-shot poll rebuilds the tracker every call,
# so a press's DOWN in poll i and its MOVE in poll i+1 are mistracked as
# different pointers; SH417's loop keeps ONE tracker across iterations. This
# drives the --v2boot-input-loop rung: subscribe the registered ANativeWindow
# XID once, drain N=INPUT_LOOP_ITERS non-blocking batches through the same
# tracker, marshal each translated event into the guest nativePassInput. Inert
# on boot (no live session yet): the run must wire the real X window, run the
# bounded loop (0 real events -> 0 delivered clean across all iterations), and
# NOT crash from the input path. The known run-variable nativeInit "outside
# image" ladder lane may still fault downstream — pre-existing Route-B, not this
# change. recon-v3 deliverables re-verified green first.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh417-input-loop.txt
rm -f "$LOG"
timeout 100 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_AINPUT_BRIDGE=1 INPUT_LOOP_ITERS=4 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh417_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-input-loop \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH417 real-input loop markers ==="
grep -E "host-input loop|v2boot-input-loop" "$LOG" | head
echo "=== did the real X window wire? ==="
grep -E "anativewindow.*wired real X11 window" "$LOG" | head
echo "=== input path crash-free? (expect empty) ==="
grep -aoE "SIGSEGV|SIGABRT|panicked" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh417_root
exit 0