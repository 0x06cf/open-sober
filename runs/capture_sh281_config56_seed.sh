#!/bin/bash
# SH281: A/B — seed [config+56] -> leaked 0x100 zeroed buf in the --v2boot-session-engine5 rung
# (SH280's state=5 direct drive). Hypothesis: the GlobalInit-reentry fault at guestpc=0x10275a148
# (`ldr x0,[x21,x8]`, x21=[config+56]=0) is a plain NULL deref, and the callee 0x275a23c DROPS the
# read value (overwrites x0 with operator_new(0x40)) — so [config+56] only needs a valid buffer,
# crossing one machine-gate past SH280. A/B vs the SH280 terminal.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
# The --v2boot-session-engine5 rung now includes the SH281 [config+56] seed (same build).
for i in 1 2 3; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
    > runs/sh281-e5-$i.txt 2>&1
  echo "e5$i EXIT=$? state5=$(grep -cE 'SH280 driving' runs/sh281-e5-$i.txt) c56seed=$(grep -cE 'SH281 seeded' runs/sh281-e5-$i.txt) crash=$(grep -oE 'guestpc=0x[0-9a-fx]+' runs/sh281-e5-$i.txt | head -1) sig=$(grep -oE 'SIG[A-Z]+' runs/sh281-e5-$i.txt | tail -1)"
done