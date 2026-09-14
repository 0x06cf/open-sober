#!/bin/bash
# SH161b repro v2: include JIT_ROUTEB_DM_SEED=1 (used by SH161's governor-tail
# verification) + JIT_ROUTEB_TAILTRACE to observe the epilogue block entry, so the
# SH161b impl[+0x2b8]=2 seed actually reaches the governor tail 0x102e9fe04.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh161b-tail-eq-seed.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_TAILTRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH161b tail-eq seed (the deliverable) ==="
grep -E "SH161b impl\[+0x2b8\] seeded|governor-tail DISPATCH seeded" "$LOG"
echo "=== tail trace (block entries in the governor tail/epilogue) ==="
grep -E "routeb-tailtrace" "$LOG" | tail -15
echo "=== ladder completion ==="
grep -E "ladder done|joined cleanly|SendAppEventOnAppReady returned" "$LOG" | head -5
echo "=== crashes / signal ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"