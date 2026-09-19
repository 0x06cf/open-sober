#!/bin/bash
# SH388: READ-ONLY reachability probe of the SEP-15 setDataModelToCurrent re-attack cone door
# (getter 0x102dbcc10 / body 0x102dbcc1c) that SH387 byte-anchored but NEVER measured for
# live headless execution. Arms the probe (JIT_ROUTEB_DMSVC_GETTER=1) on the full --v2boot
# reaching env + the manufactured-DM plant (JIT_ROUTEB_DM_MANUFACTURE) so the current-DM
# holder [0x106391908] may hold a planted genuine-vptr DM — then answers: does the engine
# ever EXECUTE the getter/body headlessly, and does a planted holder survive to be consumed?
# Read-only observation + the SH385-verified guard set; ZERO production path default changed.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh388-dmsvc-getter.txt
rm -f "$LOG"
timeout 130 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
  JIT_ROUTEB_DOINIT_EMPTYVEC=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  JIT_ROUTEB_LSM_KEYFIX=1 JIT_ROUTEB_LSM_KEYTRACE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 \
  JIT_ROUTEB_DMSVC_GETTER=1 \
  JIT_ROUTEB_APPEVENT_W19=1 \
  JIT_REGION_WATCH=0x102dbcc10-0x102dbcd40,0x10258b5d8-0x10258b900,0x102bd1d68-0x102bd2600,0x101d9a508-0x101d9a740 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH388 setDataModelToCurrent cone-door reachability (the key question) ==="
grep -aE "routeb-dmsvc-getter|SH388" "$LOG" | head -10
printf "GETTER fires: "; grep -ac "SH388 GETTER ENTERED" "$LOG"
printf "BODY fires:  "; grep -ac "SH388 BODY ENTERED" "$LOG"
echo "=== manufactured-DM plant + holder survival ==="
grep -aE "routeb-dmmanufacture|SH187b: planted|MANUFACTURED" "$LOG" | head -5
echo "=== region hits (2dbcc door / 258b dispatch / 2bd1 DMCONT / 1d9a LSM) ==="
for b in "2dbcc" "258b" "2bd1" "1d9a"; do printf "%s: " "$b"; grep -acE "region hit at guest pc=0x$b" "$LOG"; done
echo "=== terminal guestpcs / signals / session markers ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|bad_function_call|outside image|SendAppEventOnAppReady returned|DM-root probe=|MH_APP_READY|app-data-model-count" "$LOG" | sort -u | tr '\n' ' '; echo