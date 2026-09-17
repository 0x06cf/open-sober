#!/bin/bash
# SH285 A/B: the engine's OWN initEngine_ settings-state self-drive (state=9 -> 10,
# --v2boot-session-engine9) crosses the LocalStorageManager INSERT-LEAF wall when the
# LSM_NODES per-node-cell seed is enabled, advancing the SESSION-CTOR terminal one
# fencepost deeper than SH260/284 ever reached.
#
# A (baseline, LSM_NODES off): 3/3 SIGSEGV at guestpc=0x101db1d04 (LSM insert-leaf,
#   the SH260/SH284-parked terminal) — the settings-state self-drive parks AT the
#   insert leaf.
# B (+LSM_NODES): 3/3 SIGSEGV at guestpc=0x101db1b08 (the LSM reader/pop path, one
#   fencepost DEEPER) — the insert-leaf's atomic-OR (0x2b9ea40 ldset into the seeded
#   node cell) now lands in valid memory and the drive advances past it.
#
# SH267 measured the LSM_NODES cross ONLY on the full app-start ladder (StartLuaAppDM
# -> nativeAppBridgeAppStart). This closes the gap: the cross also fires on the
# engine's OWN session-state path (initEngine_ state=5/9 -> GlobalInit-reentry
# continuation -> app-start) that SH280-284 drove. Cause-level SESSION-CTOR advance;
# does NOT manufacture a DataModel (Route-B live-DM gate UNCHANGED).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
echo "=== A x3 (settings-state drive, LSM_NODES OFF = SH284 baseline) ==="
for i in 1 2 3; do
  timeout 30 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 > runs/sh285-a$i.txt 2>&1
  echo "A$i EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh285-a$i.txt) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+' runs/sh285-a$i.txt | tail -1)"
done
echo "=== B x3 (settings-state drive, LSM_NODES ON = insert-leaf crossed) ==="
for i in 1 2 3; do
  timeout 35 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 > runs/sh285-b$i.txt 2>&1
  echo "B$i EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh285-b$i.txt) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+' runs/sh285-b$i.txt | tail -1)"
done