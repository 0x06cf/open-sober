#!/bin/bash
# SH264 (SEP-17 directive, single-agent): the operator's primary lever redirects from
# single-object seeds at the app-start map wall to building a REAL Android Activity/
# AppBridge session drive — drive the lifecycle natives the engine asserts on
# (SH184) as real guest entries: nativeOnResumed, initAppShellReporter,
# JNIAppLifecycleNativeAdapter_setActive, nativeAppBridgeSetInitParams. All four are
# JNI-RECEIVE entries (verified ZERO in-image bl callers) that nothing drives today.
# The new --v2boot-session stage runs them ON THE SAME single ladder thread (SH55/64
# single-jit_run discipline) BEFORE the GlobalInit/app-start orchestration, reusing
# boot_sp/tpidr + the fabricated Activity/thiz + AutoValue init-params jobject; the
# existing app-start seed set (SH248c-f + SH259) then walks app-start to its terminal
# exactly as in capture_sh259. Watch whether (a) each lifecycle native returns Ok cleanly
# (was unified benign/absent), (b) any MH_* milestone or AppBridgeV2 singleton state
# shifts, and (c) the app-start terminal pc changes vs the standing localstorage/238
# wall — the measurable "does feeding the REAL Activity lifecycle advance the upstream
# session ctor" A/B.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh264-session-drive.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
  JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x102330000-0x10233a000,0x10233a000-0x102350000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== lifecycle natives driven + their returns =="
grep -E "v2boot-session" "$LOG"
echo "== settings-once seed fired? =="
grep -c "routeb-sh259" "$LOG" || true
echo "== all region hits =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | head -60
echo "== distinct region pcs count =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "== signals/crash =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash|guestpc=" "$LOG" || true
echo "== last guest pc / terminal =="
grep -oE "guestpc=0x[0-9a-f]+|terminated.*|EXIT [0-9]+" "$LOG" | tail -5
echo "== project milestone probes =="
grep -oE "MH_FLAGS_LOADED=[0-9]+ MH_ENGINE_INITIALIZED=[0-9]+ MH_APP_READY=[0-9]+|AppBridgeV2[^=]*=0x[0-9a-f]+|DM-root[^ ]*|once-guard\[0x6a68410\]=\[0-9a-f\]+" "$LOG" | tail -5