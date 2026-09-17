#!/bin/bash
# SH246: the SH245-active continuation (continueAfterFlagsLoaded_ 0x102bd1d68)
# reaches `bl operator_new(0x28)` (0x102bd2128) which returns NULL headlessly
# (allocator-activation gate [0x10727570c].bit0 clear, size>0xa) -> bad_alloc.
# Patch the 3-slot call site to materialize a leaked 0x40 box into x0 (drops bl).
# Arms: off (no box patch, expect bad_alloc) | on (JIT_ROUTEB_DM_CONT_OPNEW_BOX=1).
# Watch the continuation region + op_new site + where it dies next.
set -u
cd "$(dirname "$0")/.."
ARM="$1"   # off|on
LOG=/home/hermes-worker/runs/open-sober/runs/sh246-${ARM}.txt
rm -f "$LOG"
EXTRA=""
case "$ARM" in
  on) EXTRA="JIT_ROUTEB_DM_CONT_OPNEW_BOX=1" ;;
esac
timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
  $EXTRA \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x102bd2120-0x102bd2140 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "[$ARM] EXIT=$EXIT"
echo "== SH246 patch line (if any) =="
grep -E "SH245-closure" "$LOG" | tail -3
echo "== continuation (0x102bd1d68) + op_new-site (0x102bd2120) region hits =="
grep -E "region hit at guest pc=0x102bd(1d68|2120)" "$LOG" | grep -oE 'pc=0x[0-9a-f]+' | sort -u
echo "== crash / terminate =="
grep -icE "SIGSEGV|SIGABRT|terminate|bad_alloc|stack smashing" "$LOG" || true
echo "== continuation last-block / tail =="
grep -E "continueAfterFlagsLoaded|region hit at guest pc=0x102bd" "$LOG" | grep -oE 'pc=0x102bd[0-9a-f]{4}' | sort -u | tail -8