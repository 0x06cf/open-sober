#!/bin/bash
# SH320: measure the do-init DONE-path dispatcher crossing its thread-match gate to the
# MAIN (DM-ctor) branch. On the plain bus run (skip-appstart + session-bus) main-id
# [0x106863a68] is restored to 0 after rung 1, so the late do-init done-path dispatcher
# 0x2206db8 takes the box-build (b.ne TAKEN -> 0x102206e34, SH319 measured).
# --v2boot-bus-mainid re-seeds main-id = this ladder thread BEFORE the bus receive, so the
# dispatcher's b.eq (0x2206df0 NOT taken) falls through to the MAIN binder-dispatch 0x206df4
# (`ldr x0,[x19,#32]` -> vt+0x30 -> br x1 @0x206e24 = DM-ctor entry). Probe the MAIN entry
# 0x102206df4 vs the box-build 0x102206e34 (SH319's) for the A/B.
set -u
cd "$(dirname "$0")/.."
LOG=$1; : "${LOG:=/tmp/sh320-mainbranch.txt}"
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
for i in 1 2 3; do
  timeout 140 env $BASE JIT_DUMP_PC=0x102206df4 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus --v2boot-bus-mainid \
    > "$LOG" 2>&1
  EXIT=$?
  MAIN=$(grep -c "DUMPPC pc=0x102206df4" "$LOG")
  SEED=$(grep -c "SH320 --v2boot-bus-mainid" "$LOG")
  SH155=$(grep -oE "SH155 DM-root.*app-data-model-count\[0x106dca000\+0xe88\]=0x[0-9a-f]+" "$LOG" | tail -1)
  echo "run$i EXIT=$EXIT main_dispatch_hit=$MAIN seed_fired=$SEED $SH155"
done
exit 0