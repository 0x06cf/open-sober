#!/bin/bash
# SH403: does the StartApp boot body reach the app-bridge pipe bl 0x2baeeec + do-init
# 0x2206c40? RECON-V3: StartAppWithParams (0x258b144) AND StartLuaAppDM converge on the
# app-bridge pipe bl 0x2baeeec -> do-init 0x2206c40 -> match dispatch -> app-shell ctor
# 0x102207b50 -> post-do-init 0x1023eff4c. Region-watch the pipe + do-init + post-do-init
# + the StartApp boot body past 0x258b268 (the last sh402-logged block-entry pc) to see
# how deep the ordered substrate drive advances toward the do-init that builds the DM.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh403-pipe-doinit.txt
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
  JIT_REGION_WATCH=10258b144-10258b400,102baeeec-102baf000,102206c40-102207000,1023eff4c-1023effd0,102bd1d68-102bd2600 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh403_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== StartApp boot body pcs (0x258b144..0x258b400) ==="
grep -oE "pc=0x10258b[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn
echo "=== app-bridge pipe (0x2baeeec) entered? ==="
for pc in 0x102baeeec 0x102baef40 0x102baef70; do echo "  $pc: $(grep -acE "pc=$pc " "$LOG")"; done
echo "=== do-init (0x2206c40) + post-do-init worker (0x1023eff4c) ==="
for pc in 0x102206c40 0x102206db8 0x1023eff4c; do echo "  $pc: $(grep -acE "pc=$pc " "$LOG")"; done
echo "=== DMCONT (0x102bd1d68) ==="
echo "  DMCONT: $(grep -acE "pc=0x102bd1d68 " "$LOG")"
echo "=== session-drive + observables ==="
grep -aE "substrate complete|nativeAppBridgeV2StartAppWithParams|MH_APP_READY" "$LOG" | tail -4
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh403_root
exit 0