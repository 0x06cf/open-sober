#!/bin/bash
# SH266 (single-agent, cone suppressed): drive the SEP-17 messageBus receive
# Java_com_roblox_universalapp_messagebus_MessageBus_subscribe (guest 0x102ba5bb8) —
# the ONLY Route-B candidate SH185 closed by STATIC judgment alone ("subscribe
# registered only inside the migration-gated initializeLuaApp_", never driven). New
# opt-in --v2boot-session-bus rung drives it as a real guest entry on the single
# ladder thread (SH55/64), reusing boot_sp/tpidr + fabricated env/thiz + 4 jstrings
# (b1 = "experience-launch"). Its body bl's nativeAppBridgeAppStart 0x2343c10 (REAL
# app-start marshaller) directly. Full seed set same as capture_sh265.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh266-bus-subscribe-drive.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
  JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x102330000-0x10233a000,0x10233a000-0x102350000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== lifecycle natives driven + their returns =="
grep -E "v2boot-session" "$LOG"
echo "== MessageBus.subscribe specific =="
grep -iE "MessageBus.subscribe|v2boot-session-bus" "$LOG"
echo "== distinct region pcs count =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "== signals/crash =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash" "$LOG" || true
echo "== last guest pc / terminal =="
grep -oE "guestpc=0x[0-9a-f]+|terminated.*|EXIT [0-9]+" "$LOG" | tail -5
echo "== project milestone probes =="
grep -oE "MH_FLAGS_LOADED=[0-9]+ MH_ENGINE_INITIALIZED=[0-9]+ MH_APP_READY=[0-9]+" "$LOG" | tail -5