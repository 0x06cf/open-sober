#!/bin/bash
# SH358: Route-B session-ctor forward — does the DMCONT continuation (vt[+0x1f0]=REAL
# continueAfterFlagsLoaded_ 0x102bd1d68) now REACH the post-app-start tail `bl 0x2bd2058`
# when the SH349 append-skip + SH350 pack-skip (which clear the exact SH285 LSM terminal)
# are armed TOGETHER with it? SH344c measured the continuation caps at 0x101db1b08 (the
# SH285 LSM reader/pop wall) one hop before app-start, but that predates SH349/350.
# Region-watch the continuation serialize body + the post-app-start tail.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh358-dmcont-lsmskip.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2160,0x2bd2050-0x2bd2160 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== continuation serialize-body + post-app-start-tail region hits ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | awk '{print $NF}' | sort -u | while read pc; do
  if [ "$pc" \> "0x102bd1d68" ] && [ "$pc" \< "0x102bd2160" ]; then echo "  CONTINUATION/TAIL: $pc"; fi
done
echo "=== did the app-start bl (0x2bd2058 band) get entered? ==="
grep -acE "guest pc=0x102bd205|guest pc=0x102bd20|guest pc=0x102bd21" "$LOG"
echo "=== LSM skips fired? ==="
grep -aE "LSM_APPEND_SKIP|LSM_PACK_SKIP|append-skip|pack-skip|matched tst|benign index-0" "$LOG" | head -5
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== SH285 terminal still present? ==="
grep -acE "guestpc=0x101db1b08|0x101d9a708" "$LOG"
echo "=== session markers ==="
grep -E "DM-root probe|MH_APP_READY|app-data-model-count|SendAppEventOnAppReady returned|ladder done" "$LOG" | tail -8