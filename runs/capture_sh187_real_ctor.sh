#!/bin/bash
# SH187 repro: drive the REAL DataModel ctor (wrapper 0x1023f5ff8 -> bl 0x1023f6038) via
# run_guest_callback so it constructs a genuine RBX::DataModel through its real code.
# Expected (recon deleg_1c244411): the wrapper's descriptor ABI (desc[+0]=P0 small struct,
# [+8]=&A u64, [+16]=&B u32, [+24]=&C u64, [+32]=&D u64) + obj >= 0x998. If the drive survives,
# obj's first three words are the GENUINE vptr set {0x67162e8,0x67163a0,0x67163f8}.
# The ctor is COLD (0 callers) so no ladder path enters it — only this explicit host drive does.
# Ladder must stay clean; EXIT 124 (timeout after completion) with 0 SIGSEGV/SIGABRT.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh187-real-ctor.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the SH187 real-ctor drive fire (the deliverable) ==="
grep -E "routeb-realctor" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|SendAppEventOnAppReady returned|StartLuaAppDM returned" "$LOG" | head -5
echo "=== real crashes (want 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"