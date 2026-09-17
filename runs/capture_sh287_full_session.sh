#!/bin/bash
# SH287 A/B measurement at newest HEAD: the SH285-B settings-state self-drive
# (initEngine_ state=9 -> 10, --v2boot-session-engine9) crosses the LSM
# insert-leaf with LSM_NODES on and parks at the reader live-object wall
# 0x101db1b08. This cycle runs a NEW combination never measured together:
# the settings-state drive PLUS the full SEP-17 SESSION-CTOR rungs
# (surface-handoff XID + SendAppEventOnAppReady 'Home' in x5 + session-signed
# client-settings feed) on ONE ladder, to test whether the real app-event /
# surface session ctor advances the reader wall past 0x101db1b08.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_GOVFLAG=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 \
  --v2boot-surface-handoff --v2boot-send-appevent --v2boot-session-signed"
echo "=== A x2 (settings drive + full session-ctor rungs, LSM_NODES OFF) ==="
for i in 1 2; do
  timeout 40 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 > runs/sh287-a$i.txt 2>&1
  echo "A$i EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh287-a$i.txt) apprdy=$(grep -acE 'MH_APP_READY=1' runs/sh287-a$i.txt) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+' runs/sh287-a$i.txt | tail -1)"
done
echo "=== B x2 (LSM_NODES ON = insert-leaf crossed) ==="
for i in 1 2; do
  timeout 45 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 > runs/sh287-b$i.txt 2>&1
  echo "B$i EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh287-b$i.txt) apprdy=$(grep -acE 'MH_APP_READY=1' runs/sh287-b$i.txt) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+' runs/sh287-b$i.txt | tail -1)"
done
echo "=== post-run MH/session probes (B1) ==="
grep -aoE 'SendAppEventOnAppReady returned Ok\([^)]*\) w19-event=0x[0-9a-f]+|post-lifecycle: MH_[A-Z_]+=[01] MH_[A-Z_]+=[01] MH_[A-Z_]+=[01]|app-event post: MH_[A-Z_]+=[01] MH_[A-Z_]+=[01]|surface-handoff:.*' runs/sh287-b1.txt | tail -6