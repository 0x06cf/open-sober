#!/bin/bash
# SH263: probe the SINGLE forward hook (SH174 routeb_dm_alloc_capture latch) LIVE
# during the DEEPEST headless app-start reach at THIS HEAD. Run the SH259 full seed
# set (which walks do-init->governor->app-start self-drive the farthest) with
# JIT_DM_ALLOC_CAPTURE=1 + DELEGATE=1 (the SH167/169 delegating trial, armed on the
# engine's real operator-new wrapper 0x102a0d9b8) + JIT_ROUTEB_DM_REALCTOR=1 (drives
# the GENUINE-vptr DM through the real ctor wrapper 0x1023f5ff8, whose allocations
# route through operator-new and would hit the trail). Records whether ANY
# DM-signature (in-image-vtable-validated) allocation passes through the trail during
# the deepest app-start self-drive. The latch is budget-bounded + delegating + default-
# inert; its minimal eprintln log is the SH248b-class non-perturbing probe (no per-entry
# syscall). Silent latch = measured re-confirmation that the forward hook stays latent at
# the deepest headless reach (no real make_shared<DataModel>).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh263-dmalloc-latch.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
  JIT_ROUTEB_DM_REALCTOR=1 JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x102330000-0x10233a000,0x10233a000-0x102350000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== capture trail invoked / validated (the forward-hook result) =="
grep -cE "routeb-dmalloc" "$LOG" || true
echo "--- last capture lines (FIRST/validated are the DM-signature hits) ---"
grep -E "routeb-dmalloc" "$LOG" | tail -10
echo "== realctor genuine-match fired? =="
grep -cE "routeb-realctor.*GENUINE|routeb-realctor.*MATCH" "$LOG" || true
grep -E "routeb-realctor" "$LOG" | tail -5
echo "== all region hits (deepest app-start reach) =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "== signals/crash =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash|guestpc=" "$LOG" || true
echo "== last guest pc lines =="
grep -oE "guestpc=0x[0-9a-f]+|terminated.*|last guest.*" "$LOG" | tail -6
echo "== ladder flow =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)|driving [a-zA-Z]+|nativeAppBridge[A-Za-z]+" "$LOG" | tail -8