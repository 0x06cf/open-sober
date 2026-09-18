#!/bin/bash
# SH297b: quantify the construction-body reach with GUEST-ADDR region-watch.
# Regions (guest = file+0x100000000):
#   construction body 0x1023f0484-0x1023f0f40
#   nativeAppBridgeAppStart 0x102362e98-0x102362f50
#   marshaler 0x1023f1210-0x1023f1300
#   EC world 0x102e24598-0x102e25200
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="1023f0484-1023f0f40,102362e98-102362f50,1023f1210-1023f1300,102e24598-102e25200"
for i in 1 2 3; do
  timeout 60 env $BASE JIT_ROUTEB_DMFN_FIELDS=1 JIT_REGION_WATCH=$RW \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE --v2boot-session-dmfn > runs/sh297-dmfn-region-$i.txt 2>&1
  echo "s2 run$i EXIT=$? $(grep -aoE 'SH29[67].*' runs/sh297-dmfn-region-$i.txt | tail -2 | tr '\n' ' ')"
done
echo "=== region hits (run1) ==="
grep -aE "region-watch" runs/sh297-dmfn-region-1.txt | sed -E 's/.*guest pc=0x(102[0-9a-f]+).*/\1/' | sort | uniq -c | sort -rn | head -30
echo "=== total region hits run1 ==="
grep -acE "region-watch" runs/sh297-dmfn-region-1.txt
echo "=== terminal ==="
grep -aE "guestpc=|dmfn returned|dmfn stopped" runs/sh297-dmfn-region-1.txt | tail -3