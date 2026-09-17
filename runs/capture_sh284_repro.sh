#!/bin/bash
# SH284 reproducibility: drive the LAST never-driven initEngine_ state body (state=9,
# 0x2bd2668). Reuses the SH279 manager seeds (app-name 'Home' + [this+0x40] config +
# [config+56] buffer) + the SH283 JIT_ROUTEB_ENG5_QMUTEX_FREE steal. The state=9 body
# sets state->10 then config-dispatch (w2=1) -> `b 275a0c4` (the GlobalInit-reentry
# continuation the SH283 gate carries through). Verify state->10 AND the body returns Ok
# (reentry single-step advance past 5->7). Run 3x to confirm determinism.
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
timeout 40 env $BASE JIT_ROUTEB_ENG5_QMUTEX_FREE=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine9 > runs/sh284-repro-$i.txt 2>&1
echo "run$i EXIT=$? state9ok=$(grep -acE 'SH284 state=9 body direct returned Ok' runs/sh284-repro-$i.txt) stateval=$(grep -aoE 'post state=9 direct: \\[this\\+16\\]\\(state\\)=[0-9]+' runs/sh284-repro-$i.txt)"
done