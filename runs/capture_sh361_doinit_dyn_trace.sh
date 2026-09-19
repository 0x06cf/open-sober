#!/bin/bash
# SH361: measure the new READ-ONLY dynamic DM-ctor trace (JIT_ROUTEB_DOINIT_DYN_TRACE) on
# the SH344-style full app-start ladder (the one that reaches the NativeDataModelManager /
# DM-creator band 0x102bd1a30..0x102bd1d08). The trace fires at the do-init worker entry
# 0x2206db8 and reports whether the MAIN branch's [container+32] (cbz @0x2206df8) is NULL
# (-> bails to MessageBus_getLastRaw 0x2206ea4, DM-ctor `br x1` @0x2206e24 NOT reached) or
# non-NULL (-> reaches the DM-ctor dispatch). Read-only: no guest mutation.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh361-doinit-dyn.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH361 dynamic DM-ctor trace (the decisive readout) ==="
grep -E "routeb-doinit-dyn" "$LOG" || echo "(no SH361 trace line fired — worker entry 0x2206db8 never entered on this run)"
echo "=== DM-root probe / once-slot / app-data-model ==="
grep -E "SH155 DM-root probe|SendAppEventOnAppReady returned|app-data-model-count" "$LOG" | tail -4
echo "=== crash / terminal ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo