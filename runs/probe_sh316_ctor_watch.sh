#!/bin/bash
# SH316: region-watch the DM-controller ctor's FULL body on the PLAIN bus run (no
# --v2boot-postbus-doinit), where once-guard LATCHES (0x1). The SH315 postbus re-drive
# clears the guard / re-seeds main-id (a different, perturbed state). Here we watch
# whether the ctor's NORMAL-REGISTRATION tail (0x61e31d8 register / 0x61e3aa4 builder)
# executes and whether it can populate DM-root. Also watch the lookup.
set -u
cd "$(dirname "$0")/.."
LOG=/tmp/sh316-ctor-watch.txt
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
# ctor 0x61e30bc..0x61e32f0 full; lookup 0x2168798..0x2168870; builder 0x61e3aa4..0x61e3b80
RW="0x1061e30bc-0x1061e32f0,0x102168798-0x102168870,0x1061e3aa4-0x1061e3c00,0x102206c40-0x102206e00"
timeout 140 env $BASE JIT_REGION_WATCH="$RW" \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits (sorted) ==="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u
echo "=== distinct count ==="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "=== SH315/155 ==="
grep -E "SH315 service-registry-count|SH155 DM-root" "$LOG"
echo "=== crash ==="
grep -iE "guestpc=|SIGSEGV|SIGABRT|terminate" "$LOG" | tail -4
exit 0