#!/bin/bash
# SH124/SH125 repro: (SH124) the --v2boot ladder thread is JOINED after StartApp's
# jit_run returns AND the guest's libc exit-family is intercepted (spawned-thread
# exit unwinds its jit_run instead of killing the whole process), so the deep
# DM/app-shell do-init construction is neither torn down by main() nor aborted by
# the guest's own exit(232) — the ladder now completes to "ladder done" + the
# session-advance probe (pre-SH124: exit 232, log cut after json-fix clamps fired).
# (SH125) seeds the do-init flags-loaded latch [0x106a683e8].bit0=1 so the do-init
# consumes the live DM slot (recon deleg_5e2c8480).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh124-run.txt
rm -f "$LOG"
timeout 300 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles \
  --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== json-fix clamps (do-init reached the writer) ==="
grep -c "json-fix" "$LOG"
echo "=== session lever (setfix substitutions) ==="
grep -c "routeb-setfix" "$LOG"
echo "=== SH125 flags-loaded latch seed ==="
grep -c "SH125 seeded flags-loaded latch" "$LOG"
echo "=== exit-family intercept ==="
grep -c "SH124 intercepting guest" "$LOG"
echo "=== ladder completed to end? ==="
grep -E "ladder done|session-advance probe|v2boot-join|returned Ok" "$LOG" | tail -10
echo "=== do-init deep region reached (0x102208xx) ==="
grep -oE "guestpc=0x102208[0-9a-f]+" "$LOG" | head -3