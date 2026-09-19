#!/bin/bash
# SH402d: re-aim the frontier at the REAL StartAppWithParams boot body. RECON-V3 notes
# 0x258c6e4 (what the governor make-call reaches / what SH401 measured) is the AppBridgeV2
# app-registry HASH-INSERT helper, NOT the boot body. nativeAppBridgeV2StartAppWithParams
# boots at 0x258b144 (RECON V3: "StartAppWithParams (0x258b144)"). Regions:
#   [0x10258b144,0x10258b500)  StartAppWithParams boot prologue/body
#   [0x102207b50,0x102209000)  app-shell ctor band (post-do-init match dispatch)
#   [0x102bd1d68,0x102bd2600)  DMCONT continueAfterFlagsLoaded_
#   [0x1023eff4c,0x1023effd0)  do-init worker
# Measure: with the SH400 substrate (genuine AppBridgeV2 vt) does the driven session now
# enter the STARTAPP BOOT BODY (not just the hash-insert helper), and how deep toward the
# app-shell band / DMCONT?
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh402d-bootbody.txt
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
  JIT_REGION_WATCH=10258b144-10258b500,102207b50-102209000,102bd1d68-102bd2600,1023eff4c-1023effd0 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh402d_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== StartAppWithParams BOOT BODY (0x258b144) block-entry pcs ==="
grep -oE "pc=0x10258b[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn
echo "=== app-shell ctor band (0x102207b50) ==="
grep -oE "pc=0x10220[789][0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn | head
echo "=== DMCONT (0x102bd1d68) + do-init worker ==="
for pc in 0x102bd1d68 0x1023eff4c; do echo "  $pc hits: $(grep -acE "pc=$pc " "$LOG")"; done
echo "=== session-drive + observables ==="
grep -aE "substrate complete|nativeAppBridgeV2StartAppWithParams|MH_APP_READY" "$LOG" | tail -5
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh402d_root
exit 0