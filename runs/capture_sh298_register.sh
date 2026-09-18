#!/bin/bash
# SH298 (default-inert, --v2boot-session-dmfn + JIT_ROUTEB_DMFN_FIELDS=1 +
# JIT_ROUTEB_DMFN_REGISTER=1): the construction body 0x23f0484 executes, builds the
# app-server box (x26), and calls registration fn 0x21e45c8 @0x23f05b8 with
# x0=[this+136] (DOUBLE deref of arg1[8]). SH297 (arg1[8]=singleton) resolved x0 to the
# singleton's VTABLE (leaf table) whose [+8] is a host-leaf addr -> 0x21e45c8 faults at
# `ldr x8,[x22,#24]` (fault=0x7f..e8), ONE BL before nativeAppBridgeAppStart (bl@0x23f05f8).
# SH298: arg1[8] = a CELL whose value = singleton OBJECT, so x0=[cell]=singleton (real 0x80
# obj, [+8]=0) and 0x21e45c8 completes -> body falls through to bl nativeAppBridgeAppStart.
# Compare: SH297 (fields only) vs SH298 (fields+register). 3 runs each. Expect SH298 to
# CROSS 0x21e45c8 and hit the app-start region (0x2362e98) or the standing live-object wall.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="0x23f0484-0x23f0900,0x2362e98-0x2362f50,0x23f1210-0x23f1300,0x2e24598-0x2e25200"
# --- stage A: SH297 baseline (fields only, no register cell) ---
for i in 1 2 3; do
  timeout 60 env $BASE JIT_ROUTEB_DMFN_FIELDS=1 JIT_REGION_WATCH=$RW \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE --v2boot-session-dmfn > runs/sh298-a-sh297-$i.txt 2>&1
  echo "A(SH297) run$i EXIT=$? $(grep -aoE 'SH29[78].*|stopped:.*|returned Ok.*' runs/sh298-a-sh297-$i.txt | tail -2 | tr '\n' ' ')"
done
echo "=== A(SH297) terminal (run1) ==="
grep -aE "SIGSEGV|guestpc|fault=|stopped:|returned Ok" runs/sh298-a-sh297-1.txt | tail -5
# --- stage B: SH298 stage3 (fields + register cell) ---
for i in 1 2 3; do
  timeout 60 env $BASE JIT_ROUTEB_DMFN_FIELDS=1 JIT_ROUTEB_DMFN_REGISTER=1 JIT_REGION_WATCH=$RW \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE --v2boot-session-dmfn > runs/sh298-b-register-$i.txt 2>&1
  echo "B(SH298) run$i EXIT=$? $(grep -aoE 'SH29[78].*|stopped:.*|returned Ok.*' runs/sh298-b-register-$i.txt | tail -2 | tr '\n' ' ')"
done
echo "=== B(SH298) stage3 markers (run1) ==="
grep -aE "SH298|SH296|region-watch|dmfn returned|dmfn stopped" runs/sh298-b-register-1.txt | head -20
echo "=== B(SH298) terminal (run1) ==="
grep -aE "SIGSEGV|guestpc|fault=|stopped:|returned Ok|region-watch:.*hit" runs/sh298-b-register-1.txt | tail -10