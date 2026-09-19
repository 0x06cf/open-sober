#!/bin/bash
# SH377: does the corrected combined env (crossing + GOVFLAG + PRELOAD_VALUECELL) change the
# SH350 pack-helper closure? SH350/358 measured the pack-skip on OLDER envs (append-skip only /
# DMCONT+append). The genuinely-new intersection (crossing-env + GOVFLAG + PRELOAD_VALUECELL +
# PACK_SKIP) was never run. SH350 proved 0x101d9a708 has exactly ONE caller (bounded single-call
# skip), so RET'ing it is legitimate (not the unbounded hundreds-of-callers `bl 1d9d8b0` family).
# This runs the pack-skip to see where the advanced terminal lands in the new corrected env.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh377-packskip-combined.txt
MAX=${SH377_RETRY_MAX:-4}
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
  JIT_ROUTEB_DOINIT_EMPTYVEC=1 JIT_ROUTEB_LSM_PACK_SKIP=1"
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus"
run_once() {
  rm -f "$LOG"
  timeout 90 env $BASE JIT_ROUTEB_APPEVENT_W19=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SL > "$LOG" 2>&1
  local ap=$(grep -acE "SendAppEventOnAppReady returned" "$LOG")
  local pack=$(grep -acE "pack.*0x101d9a708|pack-skip|routeb.*pack" "$LOG")
  local dm=$(grep -acE "DM-root probe|MH_APP_READY" "$LOG")
  echo "attempt: exit=$? appevent_return=$ap pack_skip=$pack dm_markers=$dm"
  [ "$ap" -ge 1 ]
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; fi
done
echo "=== terminal ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT|outside image|SendAppEventOnAppReady returned|DM-root probe|MH_APP_READY|0x101d9a708|0x101d9a528|0x101d9a5a0|pool-pop" "$LOG" | tail -20
echo "won-attempt=${won:-none}"