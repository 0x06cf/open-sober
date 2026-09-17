#!/bin/bash
# SH280: 3-run batch driving the initEngine_ settings-state machine's state=5 body
# (0x2bd24b4) DIRECTLY as its own jit_run (opt-in --v2boot-session-engine5) before
# the state=3 serializer self-drive. Full SH279 seed env. The state=5 body executes
# headlessly for the FIRST time (sets state->6) and its config dispatch
# bl 0x2bce0d4 (w2=1) -> b 275a0c4 (GlobalInit-reentry) faults at guestpc=0x10275a148
# (x21=[config+56]=0 -> fault=0x0) — the settings-config content live-object wall,
# one machine-step PAST SH279's LSM wall, on the engine's OWN session-state path.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
for i in 1 2 3; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
    > runs/sh280-e5-$i.txt 2>&1
  echo "e5$i EXIT=$? state5=$(grep -cE 'SH280 driving' runs/sh280-e5-$i.txt) crash=$(grep -oE 'guestpc=0x[0-9a-fx]+' runs/sh280-e5-$i.txt | head -1) terminal=$(grep -oE 'SIGSEGV|SIGABRT' runs/sh280-e5-$i.txt | tail -1)"
done
echo "=== A/B baseline (no engine5 rung: engine-ONLY, should hit standing LSM wall 0x101db1d04) ==="
timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE > runs/sh280-baseline.txt 2>&1
echo "baseline EXIT=$? crash=$(grep -oE 'guestpc=0x[0-9a-fx]+' runs/sh280-baseline.txt | head -1)"