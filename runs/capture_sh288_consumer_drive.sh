#!/bin/bash
# SH288 A/B: drive the engine's OWN NEVER-RUN WORKER consume loop (guest
# 0x10220778c) — the CONSUMER half of the producer-only SESSION-CTOR path SH283/284
# measured (state=5/9 reentry enqueues + cond_signals queue 0x106863a70, work flag
# [0x106863b08]; the spawned worker at 0x2d97d70 never runs headlessly to consume it).
# New opt-in --v2boot-session-consumer (elfjit.rs, default-inert, inside the engine3
# block, after the engine9 drive): drives 0x10220778c as its own jit_run. It locks
# session mutex 0x106863aa0 (JIT_ROUTEB_ENG5_QMUTEX_FREE steal fires at its 0x2b53a68
# call site), then pops a queue item and processes it via 0x102207950 (builds
# [0x106a63b00] + GlobalInit sub 0x221942c).
#
# A (consumer OFF) = SH285-B baseline: engine9 + LSM_NODES, parks at the LSM
#   reader/pop live-object wall 0x101db1b08.
# B (consumer ON) = 0x102207950 (item-processor) region-hit => the engine's own
#   consumer world-build loop executes headlessly for the first time.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="JIT_REGION_WATCH=0x102207950-0x102207984,0x102207844-0x1022078b8"
echo "=== A (consumer OFF = SH285-B baseline) ==="
timeout 35 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 > runs/sh288-a1.txt 2>&1
echo "A1 EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh288-a1.txt) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+|guestpc=0x1022[0-9a-f]+' runs/sh288-a1.txt | tail -1)"
echo "=== B x2 (consumer ON, item-proc region watched) ==="
for i in 1 2; do
  timeout 45 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 $RW ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 --v2boot-session-consumer > runs/sh288-b$i.txt 2>&1
  echo "B$i EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh288-b$i.txt) cons=$(grep -acE 'SH288 consumer-drive|SH288 consumer (returned|stopped|post)' runs/sh288-b$i.txt) itemproc=$(grep -acE 'region hit at guest pc=0x1022079' runs/sh288-b$i.txt) work=$(grep -aoE 'work-flag \[0x106863b08\]=0x[0-9a-f]+' runs/sh288-b$i.txt | tail -1) built=$(grep -aoE '\[0x106a63b00\]\(once-built\)=0x[0-9a-f]+' runs/sh288-b$i.txt | tail -1) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+|guestpc=0x1022[0-9a-f]+' runs/sh288-b$i.txt | tail -1)"
done
echo "=== SH288 consumer markers (B1) ==="
grep -aE "SH288|SH284 state=9" runs/sh288-b1.txt | tail -8