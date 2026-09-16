#!/bin/bash
# SH229 repro: FULL DataModel ctor drive — leave the DM ctor's subobject call
# (`bl 0x23f6b0c` @ 0x1023f60b8) INTACT (instead of SH187's NOP) so the FULL ctor
# (incl. its internal 361-entry class index built by 0x2374c90 at obj+0x2a0) runs.
# This is a NEW measurement — SH187 only ever measured the partial NOP'd build.
# The ladder must stay clean; EXIT 124 (outer timeout after completion) with 0 SIGSEGV/SIGABRT.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh229-full-real-ctor.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_REALCTOR_FULL=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the SH229 FULL real-ctor drive fire (the deliverable) ==="
grep -E "routeb-realctor" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|SendAppEventOnAppReady returned|StartLuaAppDM returned" "$LOG" | head -5
echo "=== real crashes (want 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"