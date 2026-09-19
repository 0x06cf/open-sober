#!/bin/bash
# SH385: compose ALL the LSM persistence-lane crossings (KEYFIX + APPEND_SKIP + PACK_SKIP)
# with the full reaching env that reaches the DM-creator continuation, and measure whether
# the ladder can finally advance PAST the persistence family to the do-init MAIN dispatch
# body 0x10258b5d8 (SH362's measured never-executing gate). SH379 ran append+pack but
# LACKED keyfix and drained to pool-pop 0x101d9a528; keyfix is the SH341/343 crossing for
# exactly that pop. The cycle (keyfix->SH285, append->pack, pack->pool-pop) has never been
# composed-end-to-end while watching 0x258b5d8. Uses ONLY existing default-inert guards.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh385-composed-lsm.txt
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
  JIT_ROUTEB_APPEVENT_W19=1 \
  JIT_REGION_WATCH=0x10258b5d8-0x10258b900,0x102bd1d68-0x102bd2600,0x102dbcc10-0x102dbcd40,0x102e9fa80-0x102ea3b40,0x102207b50-0x102209000,0x101f1d8ac-0x101f1d940 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== do-init dispatch body 0x258b5d8(258b) region hits (SH362 gate, should be 0) ==="
for b in "258b" "2bd1" "2dbcc" "ea3" "207b" "1f1d8ac"; do printf "%s: " "$b"; grep -cE "region hit at guest pc=0x$b" "$LOG"; done
echo "=== keyfix / skip / cross markers ==="
grep -cE "routeb-lsm-keyfix" "$LOG"
for m in append-skip pack-skip; do printf "%s:" "$m"; grep -oE "$m[a-z ]*" "$LOG" | head -1; done
echo "=== terminal guestpcs / signals ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|bad_function_call|outside image|SendAppEventOnAppReady returned" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "SendAppEventOnAppReady returned|DM-root probe=|MH_APP_READY|app-data-model-count|ladder done|routeb-lsm-keyfix" "$LOG" | tail -10