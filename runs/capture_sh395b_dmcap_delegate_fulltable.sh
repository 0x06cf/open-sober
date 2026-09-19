#!/bin/bash
# SH395b: A/B causal check — does the SH174 DM-capture latch "never installs"
# (SH378/SH394) arise because the engine's OWN real allocator hook is present at
# op-new wrapper entry and JIT_DM_ALLOC_CAPTURE_DELEGATE is UNSET (the guard's
# documented safe-latch refuses)? Run the SH394 full-table composition with
# JIT_DM_ALLOC_CAPTURE_DELEGATE=1 added to the SAME env. If the latch NOW logs
# 'routed ... capture trail' (install) before any delegation crash, that PROVES:
#  (a) operator-new IS entered on this composition (region-watch already showed
#      0x102a0d9b8 hit),
#  (b) the SH394 'never installs' was an OBSERVER ARTIFACT (safe-latch refusing
#      to replace the engine's present real hook without DELEGATE), NOT a measured
#      'no DMA'. This corrects the record's single-forward-observer attribution.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh395b-dmcap-delegate-fulltable.txt
rm -f "$LOG"
timeout 110 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
  JIT_ROUTEB_DOINIT_EMPTYVEC=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  JIT_ROUTEB_APPEVENT_W19=1 JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  JIT_REGION_WATCH=0x102a0d940-0x102a0da00,0x101db1a20-0x101db1cc0,0x101d96768-0x101d96800 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh395b_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 \
  --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  --v2boot-glue-cmd-full \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DM capture latch arm (install line = the A/B discriminator) ==="
grep -E "dm_alloc_capture|capture trail|routed|routeb-dmalloc" "$LOG" | head -8
echo "=== op-new band region hits ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | awk '{print $NF}' | sort | uniq -c | sort -rn | head -20
echo "=== crash / terminal ==="
grep -aoE "SIGSEGV|SIGABRT|bad_function_call|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== glue-full + SendAppEvent ==="
grep -cE "glue-full\] SH393 done" "$LOG"; grep -E "SendAppEventOnAppReady returned" "$LOG" | head -2
rm -rf /tmp/sober_sh395b_root
exit 0