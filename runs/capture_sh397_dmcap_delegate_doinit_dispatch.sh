#!/bin/bash
# SH397: never-run intersection — the SH174 DM-allocation capture latch with the
# WORKING SH395 DELEGATE observer (JIT_DM_ALLOC_CAPTURE_DELEGATE=1, the only mode
# that installs over the engine's real allocator hook, measured SH395) ON TOP of
# the deepest FULL-ladder env that reaches the do-init MAIN dispatch (the sh361
# dyn-trace env: no --v2boot-skip-appstart, so StartLuaAppDM's do-init br x1
# @0x2206e24 -> vt[+48]=0x10258b5d8 fires every run before the SH285 wall).
# SH395b ran DELEGATE only on the ----v2boot-skip-appstart full-table env, which
# NEVER reaches that dispatch. This closes that gap: measure, with a WORKING
# observer, how many validated make_shared<DataModel> the furthest-advancing
# dispatch-reaching ladder produces. If 0 despite reaching the DM-ctor dispatch,
# the 'no live DM' verdict is finalized AT the dispatch with trustworthy
# instrumentation.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh397-dmcap-delegate-doinit-dispatch.txt
rm -f "$LOG"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh397_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the do-init MAIN dispatch DID/DID-NOT reach the DM-ctor (sh361 trace) ==="
grep -E "routeb-doinit-dyn" "$LOG" || echo "(no SH361 trace line — worker entry 0x2206db8 never entered this run)"
echo "=== DM-capture DELEGATE latch arming (install line = trustworthy observer active) ==="
grep -aE "dm_alloc_capture|capture trail|routed|routeb-dmalloc|FIRST call" "$LOG" | head -8
echo "=== validated make_shared<DataModel> count (the verdict) ==="
grep -acE "\[validated\]" "$LOG"
grep -aE "\[validated\]" "$LOG" | head -5
echo "=== terminal / crash ==="
grep -aoE "SIGSEGV|SIGABRT|bad_function_call|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root probe|SendAppEventOnAppReady returned|MH_APP_READY" "$LOG" | tail -4
rm -rf /tmp/sober_sh397_root
exit 0