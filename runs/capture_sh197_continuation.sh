#!/bin/bash
# SH197 repro: Route-B do-init continuation chain, all three regions watched in ONE run.
# Proves (a) JIT_REGION_WATCH multi-range parsing works, and (b) the do-init body ->
# app-shell ctor 0x102207b50 -> governor 0x102e9fa84 -> governor tail executes real
# engine code headlessly at HEAD. Expect region hits in all three ranges, EXIT 124,
# 0 SIGSEGV / 0 SIGABRT.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh197-continuation.txt
rm -f "$LOG"
timeout 90 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x1023eff4c-0x1023f0000,0x102207b50-0x102207c40,0x102e9fa84-0x102ea3b40 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== do-init body / app-shell ctor / governor continuation hits ==="
grep -E "region hit at guest pc=0x1023eff4c|region hit at guest pc=0x102207b50|region hit at guest pc=0x102e9fa84|SH158" "$LOG"
echo "=== StartLuaAppDM return (real heap addr, not 0x3e8) ==="
grep -E "StartLuaAppDM returned" "$LOG"
echo "=== crash summary (must be 0) ==="
grep -cE "SIGSEGV|SIGABRT|stack smash" "$LOG"