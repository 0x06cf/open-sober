#!/bin/bash
# SH319: reproducible capture of the do-init DONE-path dispatcher now executing (SESSION-CTOR).
# On the plain bus run (skip-appstart + session-bus) do-init's once LATCHES (once-slot=0x400000b),
# so the done-path dispatcher 0x2206db8 runs its non-main box-build branch. Pin the reach with
# JIT_DUMP_PC at the box-build entry 0x102206e34 (UNPERTURBED single-pc probe, EXIT 124 no crash).
set -u
cd "$(dirname "$0")/.."
LOG=$1; : "${LOG:=/tmp/sh319-donepath.txt}"
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
for i in 1 2 3; do
  timeout 140 env $BASE JIT_DUMP_PC=0x102206e34 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus \
    > "$LOG" 2>&1
  EXIT=$?
  HIT=$(grep -c "DUMPPC pc=0x102206e34" "$LOG")
  SH155=$(grep -oE "SH155 DM-root.*app-data-model-count\[0x106dca000\+0xe88\]=0x[0-9a-f]+" "$LOG" | tail -1)
  echo "run$i EXIT=$EXIT donepath_boxbuild_hit=$HIT $SH155"
done
exit 0