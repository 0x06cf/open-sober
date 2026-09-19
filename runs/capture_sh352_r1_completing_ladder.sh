#!/bin/bash
# SH352: reproducible artifact for the R1 flags-loaded gate address fix + the completing
# skip-appstart ladder. With the corrected gate (0x1072739d4), the R1 content-path primary
# gate actually ARMS (5/5 gates write, none dropped) on a completing --v2boot ladder.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh352-r1-fixed.txt
ROOT=/tmp/sober_r1root3
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
  SOBER_ANDROID_ROOT="$ROOT" \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-set-filesdir --v2boot-r1-stage \
  > "$LOG" 2>&1
echo "EXIT=$?" >> "$LOG"
echo "=== SH352 flags-loaded gate corrected (should be 0x1072739d4 0x0->0x1, NOT unmapped) ==="
grep -E "\[r1\] gate @0x1072739d4" "$LOG"
echo "=== all 5 loader gates armed (none should say unmapped) ==="
grep -E '\[r1\] loader gates' "$LOG"
echo "=== R1 content staged ==="
grep -E "\[r1\] STAGED" "$LOG"
echo "=== SendAppEventOnAppReady / OnGameLoaded return (completing ladder) ==="
grep -E "SendAppEventOnAppReady (returned|stopped)|SendAppEventOnGameLoaded (returned|stopped)" "$LOG"
echo "=== post-ladder session probe ==="
grep -E "SH155 DM-root probe|service-registry-count" "$LOG" | tail -2
echo "=== crash check (should be no SIGSEGV/SIGABRT on this ladder) ==="
grep -cE "SIGSEGV|SIGABRT" "$LOG"