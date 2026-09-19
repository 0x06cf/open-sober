#!/bin/bash
# SH402c: confirm the SH371 DMCONT-firing env STILL reaches continueAfterFlagsLoaded_
# (0x102bd1d68) DEEP at this exact HEAD (SH400/401 advanced). SH371 measured DMCONT
# 25+ blocks deep on --v2boot-session + surface-handoff + send-appevent +
# send-game-loaded + session-bus (+ DMCONT/M48/APPNAME/APPSART seeds), terminating at
# the SH285 wall. The SH401 frontier says DMCONT is the NEXT standing gate — so first
# re-verify it is still reachable at this HEAD, and measure how deep it now goes.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh402c-dmcont-confirm.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_REGION_WATCH=102bd1d68-102bd2600,102bd8ce8-102bd8e30 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh402c_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DMCONT continueAfterFlagsLoaded_ (0x102bd1d68) fired? ==="
n=$(grep -acE "pc=0x102bd1d68 " "$LOG"); echo "  DMCONT hits: $n"
echo "=== dispatcher (0x102bd8ce8) + sub (0x102bd8dac) ==="
for pc in 0x102bd8ce8 0x102bd8dac 0x102bd1f64 0x102bd2600; do echo "  $pc hits: $(grep -acE "pc=$pc " "$LOG")"; done
echo "=== DMCONT deep block-entry pcs (sorted, top 25) ==="
grep -oE "pc=0x102bd[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn | head -25
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session observables ==="
grep -aE "post-lifecycle|SendAppEvent|MH_APP_READY|contin" "$LOG" | tail -4
rm -rf /tmp/sober_sh402c_root
exit 0