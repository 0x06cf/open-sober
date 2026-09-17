#!/bin/bash
# SH295 (default-inert, opt-in --v2boot-session-itemproc): drive the LAST
# never-driven item-proc edge -- [item+48]. SH290 drove the once-build,
# SH291 the idempotent re-entry, SH292 the real [item+32]->vt[+48] per-item
# dispatch. SH295 sets item[+48] to a benign continuation obj (so the cbz at
# 0x22079d8 is NOT taken -> `bl 0x22193a0` EXECUTES), keeps [item+32] on the
# benign leaf, and MEASURES where it lands. Disasm predicts: 0x22193a0 ->
# `mov w2,wzr` -> trampoline 0x22076e8 -> dispatcher 0x2850ef0 -> cbz w8 ->
# 0x2850fa8 -> `mov x0,x1; bl 0x28511c4` = the vt[+112] LIVE-OBJECT wall
# (SH273/SH174), reached BEFORE the benign-cell subpaths (0x28506a4/0x28508a8
# reading [0x1068262e8], SH294). So seeding [0x1068262e8] cannot unlock it.
# This MEASURES that terminal (judgment -> measured).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_ITEM48_CELL_SEED=1 JIT_REGION_WATCH=0x102850a0-0x102851c4,0x1022079d8-0x1022079e4,0x1022193a0-0x102219440"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
for i in 1 2 3; do
  timeout 45 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-itemproc > runs/sh295-item48-$i.txt 2>&1
  echo "run$i EXIT=$? $(grep -aoE 'SH29[0-9] [^ ]+ post:.*|SH295 \[item\+48\] edge [a-z ]*|SH295 \[item\+48\] post:.*' runs/sh295-item48-$i.txt | tail -3 | tr '\n' ' ')"
done
echo "=== SH295 markers (run1) ==="
grep -aE "SH295" runs/sh295-item48-1.txt | tail -8
echo "=== region hits ==="
grep -aE "region-watch|SIGSEGV|guestpc|fault=" runs/sh295-item48-1.txt | tail -8