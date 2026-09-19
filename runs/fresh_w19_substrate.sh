#!/bin/bash
# Fresh-HEAD measurement: does the fabricated "Home" jstring route to event-code 4
# in SendAppEventOnAppReady's discriminator under the CURRENT ordered substrate
# (session.rs drive, --v2boot-session-drive) vs the old --v2boot-send-appevent rung?
# SH339 measured 0x0 (NOT 4) on the old rung; recon step-2 asserts the fabricated
# jstring DOES resolve. This answers whether the defect persists at fresh HEAD.
set -u
cd "$(dirname "$0")/.."
echo "=== A: current substrate path (--v2boot-session-drive), w19 guard armed ==="
A=/home/hermes-worker/runs/sh-w19-substrate.txt
rm -f "$A"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_ROUTEB_APPEVENT_W19=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$A" 2>&1
echo "EXIT=$?"
echo "--- discriminators (w19 guard) ---"
grep "appevent-w19" "$A" | tail -8
echo "--- SendAppEventOnAppReady result ---"
grep -E "SendAppEventOnAppReady|w19-event" "$A" | tail -4
echo "--- do-init probe ---"
grep -E "EXECUTE-DO-INIT|LIVE DM" "$A" | tail -2
exit 0