#!/bin/bash
# SH394: the genuinely-never-run intersection — SH393's FULL safe app-command table
# drive (engine's OWN process_cmd consuming the whole APP_CMD queue in lifecycle order,
# 15/20 safe, cmd 11 INIT_WINDOW fires) COMBINED with the SH174 DM-allocation capture
# latch (CAPTURE-ONLY, NO delegate) on the furthest-advancing SH378 env. Question: does
# feeding the engine its full real command queue + the world-build env finally route a
# dispatch to make_shared<DataModel>? The single forward observer (SH174 latch) flips
# with [validated] if a genuine in-image-vtable DataModel allocates.
#
# SH393's own capture set the full-table drive but NOT JIT_DM_ALLOC_CAPTURE; SH378 ran
# the capture latch but NOT the full-table drive. Neither composed both. This is that
# never-run composition.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh394-dmcap-fulltable.txt
confirm=0
for i in 1 2 3; do
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
    JIT_ROUTEB_APPEVENT_W19=1 \
    JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1 \
    JIT_DM_ALLOC_CAPTURE=1 \
    SOBER_ANDROID_ROOT=/tmp/sober_sh394_root$i \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 \
    --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
    --v2boot-glue-cmd-full \
    > "$LOG" 2>&1
  EXIT=$?
  done_lines=$(grep -acE "glue-full\] SH393 done" "$LOG")
  crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  echo "attempt $i: exit=$EXIT done=$done_lines crash=$crash"
  if [ "$done_lines" -ge 1 ]; then confirm=1; break; fi
  rm -rf /tmp/sober_sh394_root$i
done
echo "EXIT final=$EXIT"
echo "=== full-table drive arrived (SH393 marker) ==="
grep -E "glue-full\] SH393 done" "$LOG" | tail -1
echo "=== DM capture latch arm (must show 'routed ... capture trail') ==="
grep -E "routeb.dm_alloc_capture|DM_ALLOC|capture trail|latch" "$LOG" | head -6
echo "=== gold: validated make_shared<DataModel> (0 = not reached) ==="
grep -cE "\[validated\]" "$LOG"
echo "=== any capture lines (allocation traffic through trail) ==="
grep -E "FIRST call#|bytes=" "$LOG" | head -8
echo "=== SendAppEventOnAppReady return ==="
grep -E "SendAppEventOnAppReady returned" "$LOG" | head -3
echo "=== terminal guestpcs / faults ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|bad_function_call|outside image" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root probe|MH_APP_READY|app-data-model-count|ladder done|SH394" "$LOG" | tail -8
rm -rf /tmp/sober_sh394_root* 2>/dev/null
echo "RESULT=fulltable_confirm:$confirm validated:$(grep -acE '\[validated\]' "$LOG")"
exit 0