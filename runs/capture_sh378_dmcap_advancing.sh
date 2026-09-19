#!/bin/bash
# SH378: does the SH377 ADVANCING env (crossing+GOVFLAG+PRELOAD+PACK_SKIP) + the SH174
# DM-allocation capture latch (CAPTURE-ONLY, NO delegate) ever fire make_shared<DataModel>?
# SH344 ran the capture with JIT_DM_ALLOC_CAPTURE_DELEGATE=1, whose host-side delegation
# disrupted the deep full ladder (std::bad_function_call EXIT 139) BEFORE a clean readback.
# The capture-only safe latch (no DELEGATE) on the furthest-forward env has never been
# measured. This reuses ONLY existing default-inert guards; no production path edited.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh378-dmcap-advancing.txt
rm -f "$LOG"
timeout 100 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
  JIT_ROUTEB_DOINIT_EMPTYVEC=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  JIT_ROUTEB_APPEVENT_W19=1 \
  JIT_DM_ALLOC_CAPTURE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 \
  --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DM capture latch arm (must show 'routed ... capture trail') ==="
grep -E "routeb.dm_alloc_capture|DM_ALLOC|capture trail|latch" "$LOG" | head -6
echo "=== gold: validated make_shared<DataModel> (0 = not reached) ==="
grep -cE "\[validated\]" "$LOG"
echo "=== any capture lines (allocation traffic through trail) ==="
grep -E "FIRST|bytes=" "$LOG" | head -8
echo "=== SendAppEventOnAppReady return ==="
grep -E "SendAppEventOnAppReady returned" "$LOG" | head -3
echo "=== terminal guestpcs / faults ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|bad_function_call|outside image" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root probe|MH_APP_READY|app-data-model-count|ladder done|pack.*0x101d9a708" "$LOG" | tail -8