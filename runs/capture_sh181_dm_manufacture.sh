#!/bin/bash
# SH181 repro: JIT-side RBX::DataModel manufacture lever (default-inert, env-gated).
# Verifies the genuine-vptr manufactured DM (vt=0x1067162f0) is planted into the current-DM
# holder *0x106391908 (setDataModelToCurrent getter target) at StartLuaAppDM entry, and that no
# downstream headless consumer dispatches the DM vtable (region-watch on the app-shell ctor
# 0x1057d6ef4 = 0 entered) -> latent-but-correct, clean ladder, EXIT 124 (timeout after done).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh181-dm-manufacture.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
  JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x1057d6ef4-0x1057d7100 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the manufactured-DM plant (the deliverable) ==="
grep -E "routeb-dmmanufacture" "$LOG"
echo "=== did the DM app-shell ctor region ever execute? (want 0) ==="
grep -c "entered region" "$LOG"
echo "=== holder pre/post (should show was=0x106358d40 -> genuine-vptr DM) ==="
grep -oE "current-DM holder 0x106391908 \(was [0-9a-fx]+\)" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|joined cleanly|SendAppEventOnAppReady returned|StartLuaAppDM returned" "$LOG" | head -5
echo "=== real crashes (want 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"