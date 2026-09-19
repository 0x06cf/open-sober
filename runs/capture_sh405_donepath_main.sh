#!/bin/bash
# SH405: can the SH320 main-id seed flip the do-init off the FALL-THROUGH box-build arm onto the
# MAIN dispatch br x1 @0x2206e24 -> 0x10258b5d8 body? SH404 measured the fall-through (b.ne @0x2206df0
# taken, pthread_self != [0x106863a68]) even though the SH361 trace read [container+32] non-NULL with
# vt[+48]=0x10258b5d8. Its env did NOT arm JIT_ROUTEB_DONEPATH_MAIN (the SH320 guard that re-seeds
# main-id to the EXECUTING thread at 0x2206db8 so b.eq is taken). Region-watch the MAIN dispatch
# br x1 @0x2206e24 + the 0x10258b5d8 body + DMCONT, with DONEPATH_MAIN armed, on the same SH404 env.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh405-donepath-main.txt
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
  JIT_ROUTEB_DONEPATH_MAIN=1 \
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
  JIT_REGION_WATCH=2206e24-2206e30,2206e28-2206f00,10258b5d8-10258d000,102bd1d68-102bd2600,1021f3748-1021f4700 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh405_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits: MAIN br x1 @0x2206e24 / fall-through @0x2206e28 / 0x258b5d8 body / DMCONT ==="
grep -a "region-watch\] region hit" "$LOG" | grep -aoE "guest pc=0x[0-9a-f]+" | tr -d ' ' | sort | uniq -c
echo "=== block-entry pcs in main-dispatch/fallthrough window (2206e24..2206f00) ==="
grep -aoE "pc=0x102206e[0-9a-f]+ |pc=0x102206f[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c
echo "=== SH320 seed fired? ==="
grep -a "routeb-sh320" "$LOG" | head
echo "=== SH361 trace ==="
grep -a "doinit-dyn" "$LOG" | head -2
echo "=== 0x10258b5d8 body entered? ==="
grep -acE "pc=0x10258b5d8 " "$LOG"
echo "=== guestpc terminals / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
echo "  terminals: $(grep -aoE 'guestpc=0x[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
rm -rf /tmp/sober_sh405_root
exit 0