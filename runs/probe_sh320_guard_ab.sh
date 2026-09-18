#!/bin/bash
# SH320 A/B via the JIT block-entry guard (JIT_ROUTEB_DONEPATH_MAIN=1). The do-init
# DONE-path dispatcher 0x2206db8 forks at b.ne @0x2206df0 on main-id==pthread_self:
# - seed OFF: box-build branch 0x102206e34 fires (SH319 reach) — the dispatcher forks to
#   the non-main box-build because the done-path runs on a spawned clone worker whose
#   pthread_self != the ladder-thread main-id the harness seeds.
# - seed ON (JIT_ROUTEB_DONEPATH_MAIN=1): the crate block-entry guard seeds main-id to the
#   EXECUTING jit thread at 0x2206db8 (a real block entry), so b.eq is taken -> the run
#   proceeds into the MAIN binder-dispatch 0x206df4 (DM-ctor entry condition). We probe the
#   NEXT reachable block entry after the binder-dispatch to observe the fork flip: box-build
#   0x102206e28 IS a block entry (b.ne target); if the guard works it should NOT be entered
#   as a NEW block while the dispatcher is running the MAIN path.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
for i in 1 2; do
  LOG=/tmp/sh320-guard-run$i.txt; rm -f "$LOG"
  timeout 140 env $BASE JIT_ROUTEB_DONEPATH_MAIN=1 JIT_DUMP_PC=0x102206e28 \
    $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
  EXIT=$?
  SEED=$(grep -c "routeb-sh320" "$LOG")
  BOX_BLOCK=$(grep -c "DUMPPC pc=0x102206e28" "$LOG")
  SH155=$(grep -oE "once-slot\[0x106a68408\]=0x[0-9a-f]+ DM-root\[0x106a68818\]=0x[0-9a-f]+" "$LOG" | tail -1)
  echo "run$i EXIT=$EXIT sh320_seed_fired=$SEED boxbuild_block_entry_hits=$BOX_BLOCK $SH155"
done
exit 0