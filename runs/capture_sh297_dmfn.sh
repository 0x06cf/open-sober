#!/bin/bash
# SH297 (default-inert, --v2boot-session-dmfn + JIT_ROUTEB_DMFN_FIELDS=1):
# CORRECTED seed for the DM-construction fn 0x1023f03b4. SH296 seeded this+136/144
# but the body CLOBBERS those from arg1[8]/arg1[16] (arg1 was a zeroed buffer ->
# cbz x21 safety-epilogue at 0x23f04d0 fired before any this-field read). This
# stage seeds the REAL gates: coherent arg1 ([arg1+8]=[arg1+16]=singleton that
# become this+136/144) + this+120 + this+128 (refcount factory 0x2b4ea48).
# Expect: the construction body (0x23f0484) ENTERS pass t 0x23f04d0 and runs toward
# nativeAppBridgeAppStart 0x2362e98 / marshaler 0x23f1210 / EC world 0x2e24598.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
# construction body + app-start + marshaler + EC world regions
RW="0x23f0484-0x23f0900,0x2362e98-0x2362f50,0x23f1210-0x23f1300,0x2e24598-0x2e25200"
for i in 1 2 3; do
  timeout 60 env $BASE JIT_ROUTEB_DMFN_FIELDS=1 JIT_REGION_WATCH=$RW \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE --v2boot-session-dmfn > runs/sh297-dmfn-s2-$i.txt 2>&1
  echo "s2 run$i EXIT=$? $(grep -aoE 'SH29[67].*' runs/sh297-dmfn-s2-$i.txt | tail -3 | tr '\n' ' ')"
done
echo "=== SH297 stage2 markers (run1) ==="
grep -aE "SH29[67]|region-watch|dmfn returned|dmfn stopped" runs/sh297-dmfn-s2-1.txt | head -25
echo "=== construction-body terminal (run1) ==="
grep -aE "SIGSEGV|guestpc|fault=|stopped:|returned Ok" runs/sh297-dmfn-s2-1.txt | tail -8