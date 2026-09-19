#!/bin/bash
# SH360: measure the do-init empty-vector gate (JIT_ROUTEB_DOINIT_EMPTYVEC) on the completing
# ladder. Armed gate should fire [routeb-doinit] SH360 at the app-shell band walker entry
# (guest 0x102208e58) and the run must stay confirmed-green (SH155 DM-root probe + 0 signals).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh360-doinit-emptyvec.txt
MAX=${COMPLETING_RETRY_MAX:-6}
ROOT=/tmp/sober_r1root360
run_once() {
  rm -f "$LOG"
  rm -rf "$ROOT"
  timeout 120 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
    JIT_ROUTEB_PRELOAD_VALUECELL=1 \
    JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
    SOBER_ANDROID_ROOT="$ROOT" \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-set-filesdir --v2boot-r1-stage \
    > "$LOG" 2>&1
  local exit=$?
  local probe=$(grep -acE "SH155 DM-root probe" "$LOG")
  local crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  echo "attempt: exit=$exit probe=$probe crash=$crash"
  [ "$probe" -ge 1 ] && [ "$crash" -eq 0 ]
}
ok=0
for i in $(seq 1 "$MAX"); do
  echo "=== capture attempt $i/$MAX ==="
  if run_once; then ok="$i"; break; fi
done
echo "=== confirmed-green full completion on attempt: ${ok:-NONE} ==="
echo "=== SH360 do-init empty-vector gate firings ==="
grep -c "empty-vector gate" "$LOG" || true
grep "empty-vector gate" "$LOG" | head -4
echo "=== SH155 DM-root probe (terminal marker) ==="
grep -E "SH155 DM-root probe" "$LOG" | tail -1
echo "=== crash check (confirmed-green = 0 SIGSEGV/SIGABRT) ==="
grep -cE "SIGSEGV|SIGABRT" "$LOG"