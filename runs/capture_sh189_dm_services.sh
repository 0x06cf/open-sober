#!/bin/bash
# SH189 repro: seed an empty DM service container + drive the REAL PlayerGui class-register
# getter 0x10201fce0 so the engine's global class-name registry gains a PlayerGui descriptor
# (classid 0x87e), headlessly, on the SH187-constructed genuine DM. The once-body 0x10201fda4
# builds/registers the PlayerGui ClassInfo at 0x106c980b8; success = class-desc counter
# [0x106dca0e28] > 0 AND cached desc [0x106c97f28] != 0.
# Reuses the SH187 real-ctor ladder so the genuine DM is constructed + planted to the holder.
# Expected: EXIT 124 (timeout after completion) with 0 SIGSEGV/SIGABRT;
#   [routeb-dmsvc] SH189: class-desc counter [0x106dca0e28] = 1.. (want > 0)
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh189-dm-services.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_DM_SERVICES=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH189 DM-service seed + PlayerGui class-register drive (the deliverable) ==="
grep -E "routeb-dmsvc" "$LOG"
echo "=== SH187 real-ctor (DM construction, upstream) ==="
grep -E "routeb-realctor" "$LOG" | head -5
echo "=== ladder completion ==="
grep -E "ladder done|SendAppEventOnAppReady returned|StartLuaAppDM returned" "$LOG" | head -5
echo "=== real crashes (want 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"