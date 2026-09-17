#!/bin/bash
# SH259 fresh measurement: seed the settings/registry singleton once-guard [0x106a6f430]
# bit0=1 (JIT_ROUTEB_APPSART_SETTINGS_ONCE). This is the deepest app-start reach
# (0x102339d44 `bl 0x21dac2c`). With bit0 set, the factory early-returns the zeroed
# registry object 0x6a6f3f0 WITHOUT running builder 0x21dac80 — which is what hands
# control into the standing live-object map wall 0x1021dde34. Watch whether the
# orchestrator then resumes at 0x2339d48 and walks its OWN real app-start body
# (0x233a804 / 0x233af10 / 0x233bbac / 0x233bf20 / 0x233d11c / 0x233d2bc) — a fresh
# Path-B surface never reached headlessly. Keep the FULL SH258 seed set + SETTINGS_ONCE.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh259-settings-once.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
  JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x102330000-0x10233a000,0x10233a000-0x102350000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== settings-once seed fired? =="
grep -c "routeb-sh259" "$LOG" || true
echo "== all region hits =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | head -80
echo "== distinct region pcs count =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "== signals/crash =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash|guestpc=" "$LOG" || true
echo "== last guest pc lines =="
grep -oE "guestpc=0x[0-9a-f]+|terminated.*|last guest.*" "$LOG" | tail -6
echo "== ladder flow =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)|driving [a-zA-Z]+|nativeAppBridge[A-Za-z]+" "$LOG" | tail -8