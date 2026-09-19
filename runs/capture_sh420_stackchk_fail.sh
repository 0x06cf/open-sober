#!/bin/bash
# SH420: reproduce the GENUINE `*** stack smashing detected ***` canary wall
# with the __stack_chk_fail host shim armed (JIT_STACKCHK_DUMP=1). The shim
# fires ONCE at the check failure (zero scheduling perturbation, unlike the
# store-watch) and names the failing guest frame via the dispatcher's
# CURRENT_GUEST_PC (the return addr of the `bl __stack_chk_fail` = inside the
# failing function's epilogue) + the canary slot content. This is the SH4xx-next
# / STATUS-#2 instrument: it correlates the genuine wall against ITS OWN canary
# slot (the store-watch filtered to host-ptr values only and perturbed the run).
# Expect: exactly one `[stack_chk_fail] __guest_pc=0x102206d90 ...` line followed
# by `*** stack smashing detected ***` and EXIT 134 (the standing do-init
# once-path / nativeGameGlobalInit body wall).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh420-stackchk-fail.txt
rm -f "$LOG"
timeout 60 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_STACKCHK_DUMP=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --persist-roundtrip \
  --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the one-shot stack_chk_fail dump (names the wall) ==="
grep -E "stack_chk_fail|stack smashing" "$LOG"
echo "=== reached nativeGameGlobalInit (rung 2)? ==="
grep -c "driving nativeGameGlobalInit" "$LOG" || echo 0
echo "=== crash summary ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"