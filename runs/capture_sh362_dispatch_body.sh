#!/bin/bash
# SH362: measure the NEW READ-ONLY body trace (JIT_ROUTEB_DISPATCH_BODY_TRACE) at the do-init
# MAIN-branch dispatch target fn 0x258b5d8 (guest 0x10258b5d8 = StartAppWithParams+0x494), which
# SH361's dynamic trace PROVED the br x1 @0x2206e24 lands on. SH361 measured the dispatch LANDS
# here; this trace answers whether that body actually EXECUTES headlessly and WHICH of its two
# init paths it takes (path A @0x258b60c when flag byte [0x106a64da0]!=0: nativePreloadFlagOverrides
# -> blr vt[+144] -> 0x2366694 -> nativePreloadFlagOverrides -> blr vt[+296]; path B @0x258b640 when
# flag==0: 0x2367270 -> blr vt[+144] -> 0x2366694 -> 0x2367270 -> blr vt[+296]).
# Read-only: no guest mutation.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh362-dispatch-body.txt
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
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 JIT_ROUTEB_DISPATCH_BODY_TRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH362 body trace (the decisive readout) ==="
grep -E "routeb-dispatch-body" "$LOG" || echo "(no SH362 trace line fired — body fn 0x258b5d8 never entered on this run)"
echo "=== SH361 dynamic DM-ctor trace (context) ==="
grep -E "routeb-doinit-dyn" "$LOG" || echo "(no SH361 trace line fired)"
echo "=== DM-root probe / once-slot / app-data-model ==="
grep -E "SH155 DM-root probe|SendAppEventOnAppReady returned|app-data-model-count" "$LOG" | tail -4
echo "=== crash / terminal ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo