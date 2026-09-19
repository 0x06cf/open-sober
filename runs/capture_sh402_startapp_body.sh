#!/bin/bash
# SH402: map the StartAppWithParams body interior on the SH401 governor-reach env.
# SH401 measured the governor 0x102e9fa84 -> `bl 0x258c6e4` StartAppWithParams enters
# (0x258c6e4 + early body 0x258c7b4 both 1 hit) but DMCONT 0x102bd1d68 = 0 hits.
# This probes the FULL interior so we can see exactly how deep StartAppWithParams
# gets before it stops/park/faults, and whether any interior call reaches the
# DMCONT engine-init continuation. Region-watch the whole [0x258c6e4,0x258d100)
# body plus DMCONT [0x102bd1d68,0x102bd2600) + the do-init worker/gor dispatch.
# Goal: name the next structural wall past StartAppWithParams's entry.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh402-startapp-body.txt
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
  JIT_REGION_WATCH=10258c6e4-10258d100,102bd1d68-102bd2600,102e9fa84-102e9fea4,1023eff4c-1023effd0 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh402_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== StartAppWithParams body block-entry pcs (whole interior, sorted) ==="
grep -oE "pc=0x10258[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn
echo "=== DMCONT (0x102bd1d68) reached? ==="
n=$(grep -acE "pc=0x102bd1d68 " "$LOG"); echo "  DMCONT hits: $n"
echo "=== governor + do-init chain ==="
for pc in 0x102e9fa84 0x102e9fb58 0x10258c6e4 0x10258c7b4 0x1023eff4c; do
  echo "  $pc hits: $(grep -acE "pc=$pc " "$LOG")"
done
echo "=== session-drive per-atom + observables ==="
grep -aE "substrate complete|nativeAppBridgeV2SendAppEventOnAppReady\(Home|nativeAppBridgeStartLuaAppDM|MH_APP_READY" "$LOG" | tail -5
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh402_root
exit 0