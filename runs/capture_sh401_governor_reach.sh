#!/bin/bash
# SH401: MEASURED the do-init -> governor reach through the genuine AppBridgeV2
# singleton (the frontier's named next gate). SH400's ordered session substrate
# self-constructs AppBridgeV2 [0x106a705e8] to its genuine relocated vt 0x1063a3410
# from atom [5/16] StartLuaAppDM onward; the downstream --v2boot StartLuaAppDM rung
# then executes the do-init `blr [vt+0x18]` @0x23effbc into the REAL governor
# 0x102e9fa84 for the first time (region-watch: governor body 0x102e9fa84 -> gov
# dispatch [blr x9 @0x2e9fb54] -> make-call `bl 0x258c6e4` StartAppWithParams ->
# StartAppWithParams body 0x258c7b4 all FIRE). DMCONT 0x102bd1d68 is the NEXT
# (not-yet-reached) gate. 2/2 reproducible, EXIT 124 stable, 0 crash.
#
# Measure: whether the ordered session drive executes the real governor body + its
# StartAppWithParams make-call (region hits on 0x102e9fa84/0x10258c6e4), whether the
# do-init worker + blr dispatch fire (0x1023eff4c/0x1023effac), and whether DMCONT
# (0x102bd1d68) is still the standing next gate.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh401-governor-reach.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_REGION_WATCH=102e9fa84-102e9fea4,10258c6e4-10258d000,102bd1d68-102bd2600,1023eff4c-1023effd0 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh401_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== THE FORWARD: do-init -> governor -> StartAppWithParams reach chain ==="
for pc in 0x102e9fa84 0x102e9fb54 0x102e9fb58 0x10258c6e4 0x10258c7b4; do
  n=$(grep -acE "pc=$pc " "$LOG")
  echo "  $pc governor/StartAppWithParams hits: $n"
done
echo "=== do-init worker + blr dispatch fired ==="
for pc in 0x1023eff4c 0x1023effa0 0x1023effac; do
  n=$(grep -acE "pc=$pc " "$LOG")
  echo "  $pc do-init hits: $n"
done
echo "=== NEXT gate (DMCONT 0x102bd1d68) reached? ==="
n=$(grep -acE "pc=0x102bd1d68 " "$LOG")
echo "  DMCONT 0x102bd1d68 hits: $n"
echo "=== session-drive per-atom + AppBridgeV2 ==="
grep -aE "substrate complete|nativeAppBridgeStartLuaAppDM: returned|MH_APP_READY" "$LOG" | tail -4
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG")
echo "crash-signals=$crash"
grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh401_root
exit 0