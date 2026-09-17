#!/bin/bash
# SH265 (single-agent): extend the SEP-17 session-drive to drive the dataModel-bindings
# live-binder receive nativeAppBridgeV2SendAppEventOnGameLoaded (guest 0x102bb429c) —
# SH264's honest "next candidate, still un-driven". It is the sibling of OnAppReady
# (already driven): marshals 3 jstrings into a 0x50 AppEvent, dispatches via 0x2baeeec
# into the SAME app-bridge pipe/do-init the ladder drives — but from the REAL
# dataModel-bindings receive path the SEP-17 directive names. Full seed set same as
# capture_sh264; adds --v2boot-send-game-loaded. Materializes OnGameLoaded's event
# vtable (0x10635dfe8, SH126b) + seeds the pipe sync-gate so the body completes.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh265-gameloaded-drive.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
  JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x102330000-0x10233a000,0x10233a000-0x102350000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== lifecycle natives driven + their returns =="
grep -E "v2boot-session" "$LOG"
echo "== OnGameLoaded specific =="
grep -iE "game-loaded|OnGameLoaded|SH126b" "$LOG"
echo "== settings-once seed fired? =="
grep -c "routeb-sh259" "$LOG" || true
echo "== distinct region pcs count =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "== signals/crash =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash|guestpc=" "$LOG" || true
echo "== last guest pc / terminal =="
grep -oE "guestpc=0x[0-9a-f]+|terminated.*|EXIT [0-9]+" "$LOG" | tail -5
echo "== project milestone probes =="
grep -oE "MH_FLAGS_LOADED=[0-9]+ MH_ENGINE_INITIALIZED=[0-9]+ MH_APP_READY=[0-9]+|AppBridgeV2[^=]*=0x[0-9a-f]+|once-guard\[0x6a68410\]=\[0-9a-f\]+\" "$LOG" | tail -5