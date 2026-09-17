#!/bin/bash
# SH283d: non-dedup JIT_DUMP_PC probe (fires every entry) on the continuation chain to
# determine, per-pass, exactly how deep the engine5 path goes. Dump on:
#  - 0x102207118 continuation entry
#  - 0x102d9713c enqueue entry (correct addr)
#  - 0x1022071ac construct entry
#  - 0x10275a294 callee tail (b 2207118)
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
DUMP="JIT_DUMP_PC=0x102207118"
grep -q 'next' /dev/null
timeout 45 env $BASE $DUMP ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
  > runs/sh283d-park.txt 2>&1
echo "EXIT=$?"
echo "cont-dumps=$(grep -acE '^DUMPPC pc=0x102207118' runs/sh283d-park.txt)"
grep -aE '^DUMPPC' runs/sh283d-park.txt | tail -3 | cut -c1-160
echo "SH280-lines:"
grep -aE 'SH280|state=5|returned Ok|stopped' runs/sh283d-park.txt | tail -6