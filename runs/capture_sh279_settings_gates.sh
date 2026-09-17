#!/bin/bash
# SH279: 3-run batch of the SH278 engine3 rung + SH279 settings-config + appname
# seeds (full SH269 seed env). The initEngine_ state=3 serializer now runs its WHOLE
# body and self-drives into app-start depth: terminal moves from SH278's
# guestpc=host-thunk fault=0x300 (settings-config NULL lock) to the standing LSM
# insert-leaf wall guestpc=0x101db1d04 (SH260). LSM_NODES variant crosses one deeper
# (0x101db1b08, SH267 free-list).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
for i in 1 2 3; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine3 \
    > runs/sh279-batch-$i.txt 2>&1
  echo "e3$i EXIT=$? s3=$(grep -oE 'state=[0-9]+' runs/sh279-batch-$i.txt | head -1) term=$(grep -oE 'SIGSEGV|SIGABRT|returned Ok\(0x[0-9a-f]+\)' runs/sh279-batch-$i.txt | tail -1) crashpc=$(grep -oE 'guestpc=0x[0-9a-f]+' runs/sh279-batch-$i.txt | head -1) scfg=$(grep -cE 'SH279 seeded \[this\+0x40\]' runs/sh279-batch-$i.txt) appname=$(grep -cE 'SH279 pre-seeded' runs/sh279-batch-$i.txt) reshred=$(grep -cE 'app-name guard re-seeded' runs/sh279-batch-$i.txt)"
done
echo "=== LSM_NODES variant (cross LSM wall from engine3) ==="
timeout 120 env $BASE JIT_ROUTEB_APPSART_LSM_NODES=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine3 \
  > runs/sh279-lsm-nodes.txt 2>&1
echo "nodes EXIT=$? term=$(grep -oE 'SIGSEGV|SIGABRT|returned Ok\(0x[0-9a-f]+\)' runs/sh279-lsm-nodes.txt | tail -1) crashpc=$(grep -oE 'guestpc=0x[0-9a-f]+' runs/sh279-lsm-nodes.txt | head -1)"
echo "=== A/B baseline (no engine3 rung: should complete clean EXIT 124) ==="
timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE > runs/sh278-ab-baseline.txt 2>&1
echo "baseline EXIT=$? term=$(grep -oE 'SIGSEGV|SIGABRT|returned Ok\(0x[0-9a-f]+\)' runs/sh278-ab-baseline.txt | tail -1)"