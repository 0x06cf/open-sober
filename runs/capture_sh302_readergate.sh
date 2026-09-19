#!/bin/bash
set -u
cd /home/hermes-worker/runs/open-sober
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_DMFN_FIELDS=1 JIT_ROUTEB_DMFN_REGISTER=1 JIT_ROUTEB_EC_ARG1=1 JIT_ROUTEB_EC_ARG0VT=1 \
  JIT_ROUTEB_EC_REALSESSION=1 JIT_ROUTEB_EC_READERGATE=1 JIT_ROUTEB_EC_READERGATE_FRAME=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="0x1023c5538-0x1023c55f0,0x1023f1654-0x1023f16c0,0x102b504e4-0x102b50500,0x102e24678-0x102e247dc"
for i in 1 2 3; do
  timeout 80 env $BASE JIT_REGION_WATCH=$RW \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    $SLBASE --v2boot-session-dmfn > runs/sh302b-v2interior-r$i.txt 2>&1
  E=$?
  hits=$(grep -aoE "region hit at guest pc=0x[0-9a-f]+" runs/sh302b-v2interior-r$i.txt | sort -u | tr '\n' ' ')
  echo "run$i EXIT=$E hits=[$hits] $(grep -aoE 'SH296 dmfn returned Ok\([^)]*\)' runs/sh302b-v2interior-r$i.txt | tail -1)"
done
echo "=== r1 EC+V2Init region hits + sh300/302 + terminal ==="
grep -aE "routeb-sh30[02]|routeb-sh355|region hit at guest pc=0x102e2|region hit at guest pc=0x1023c5|region hit at guest pc=0x1023f16" runs/sh302b-v2interior-r1.txt | head; echo "term:"; grep -aoE "SIGSEGV|SIGABRT|guestpc=0x[0-9a-f]+|returned Ok" runs/sh302b-v2interior-r1.txt | tail -4