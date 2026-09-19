#!/bin/bash
# SH379: FULL --v2boot ladder (app-start DRIVEN, the SH344 env that reaches the governor 22 pcs
# + DM-creator band) + the SH376/377 governor-crossing gates (GOVFLAG + PRELOAD_VALUECELL + PACK_SKIP)
# that were never run on the full ladder (only on the --v2boot-skip-appstart send-appevent env).
# Never-run intersection: does the full ladder, with the governor/preload/pack walls all crossed,
# advance past the long-standing persistence-lane terminal toward the governor -> app-shell -> Lua
# path? Region-watch the governor, DM-creator, setDataModelToCurrent, app-shell-ctor, ScriptContext.
# Uses ONLY existing default-inert guards; no production path edited.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh379-full-ladder-govgates.txt
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
  JIT_REGION_WATCH=0x102bd1a30-0x102bd1d40,0x102dbcc10-0x102dbcd40,0x102e9fa80-0x102ea3b40,0x102207b50-0x102209000,0x101f1d8ac-0x101f1d940 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits (band pcs) ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | awk '{print $NF}' | sort | uniq -c | sort -rn | head -30
echo "=== per-band counts (DM-creator 2bd1 / dmcc 2dbcc / governor ea / appshell 207b / scriptcontext 1f1d8ac) ==="
for b in "2bd1" "2dbcc" "ea3" "207b" "1f1d8ac"; do printf "%s: " "$b"; grep -cE "region hit at guest pc=0x$b" "$LOG"; done
echo "=== terminal guestpcs / faults ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|bad_function_call|outside image" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "SendAppEventOnAppReady returned|DM-root probe|MH_APP_READY|app-data-model-count|ladder done|SendAppEventOnGameLoaded returned" "$LOG" | tail -8