#!/bin/bash
# SH416: measure the real X event-source input poll on real libroblox.so. The
# input axis delivery path (deliver_motion -> nativePassInput) was complete but
# never fed REAL desktop events; this drives the --v2boot-input-poll rung, which
# subscribes the registered ANativeWindow XID to pointer/button/motion and drains
# one non-blocking batch into the guest. Inert on boot (no live session yet): the
# run must wire the real X window, poll it (0 real events -> 0 delivered clean),
# and NOT crash from the input path. The known run-variable nativeInit "outside
# image" ladder lane may still fault downstream — that is pre-existing Route-B, not
# this change. recon-v3 deliverables re-verified green first.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh416-input-poll.txt
rm -f "$LOG"
timeout 100 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_AINPUT_BRIDGE=1 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh416_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-input-poll \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH416 real-input poll markers ==="
grep -E "host-input poll|v2boot-input-poll" "$LOG" | head
echo "=== did the real X window wire? ==="
grep -E "anativewindow.*wired real X11 window" "$LOG" | head
echo "=== input path crash-free? (expect empty) ==="
grep -aoE "SIGSEGV|SIGABRT|panicked" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh416_root
exit 0