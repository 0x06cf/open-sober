#!/bin/bash
# SH315b: region-watch the do-init once-lambda (0x2206d10) + DM-controller ctor chain
# (0x61e30bc, lookup 0x2168798, fast-path tail 0x61e32cc, fallback 0x61e3150) during the
# post-bus StartLuaAppDM re-drive, to see WHY DM-root stays 0 even with count=12.
set -u
cd "$(dirname "$0")/.."
LOG=/tmp/sh315b-regions.txt
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
RW="0x102206c40-0x102206e00,0x1061e30bc-0x1061e3340,0x102168798-0x102168870,0x102ba5bb8-0x102ba5c40"
timeout 150 env $BASE JIT_REGION_WATCH="$RW" \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-postbus-doinit \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits (do-init once / DM ctor / lookup / bus) ==="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u
echo "=== distinct ==="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "=== SH315 ==="
grep -E "SH315" "$LOG"
echo "=== crash ==="
grep -iE "guestpc|SIGSEGV|SIGABRT|terminate" "$LOG" | tail -3
exit 0