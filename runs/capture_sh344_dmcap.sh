#!/bin/bash
# SH344: re-arm the SH174 DM-allocation capture latch on the SH343-deepened full
# ladder. SH343's routeb_lsm_keyfix_guard crossed the LSM poison fencepost; the
# ladder now advances one fencepost deeper into the session-ctor region. This
# measures whether that deeper reach lets the engine's OWN make_shared<DataModel>
# allocation (operator-new capture trail at pc 0x102a0d9b8) fire — the single
# SH174 forward hook. If JIT_DM_ALLOC_CAPTURE stays silent, the live-DM wall is
# confirmed still structural (measurement, not a crossing).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh344-dmcap.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DM capture latch arm line (must be present for latch armed) ==="
grep -E "SH167|routeb-dmalloc|DM_ALLOC" "$LOG" | head -5
echo "=== gold: a VALIDATED make_shared<DataModel> allocation (0 = not reached) ==="
grep -cE "\[validated\]" "$LOG"
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root|MH_APP_READY|app-data-model-count|ladder done" "$LOG" | tail -8