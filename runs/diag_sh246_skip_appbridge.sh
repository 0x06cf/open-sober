#!/bin/bash
# SH246 DIAGNOSTIC: prove the bad_alloc after SH245's DMCONT continuation is in
# nativeAppBridgeAppStart (0x2338ef4, called at 0x2bd2058 in the continuation's
# straight-line block, its own op_new variant 0x1d96768 NULLs at 0x28/0x20).
# off = only the continuation op_new box patch (my SH246 lever) — expect bad_alloc.
# skip = ALSO ret the bl 0x2338ef4 -> if bad_alloc disappears / continuation
#        advances past 0x2bd2058, the crasher is definitively nativeAppBridgeAppStart.
set -u
cd "$(dirname "$0")/.."
ARM="$1"   # off|skip
LOG=/home/hermes-worker/runs/open-sober/runs/sh246d-${ARM}.txt
rm -f "$LOG"
EXTRA=""
case "$ARM" in
  skip) EXTRA="JIT_ROUTEB_DM_CONT_SKIP_APPBRIDGE=1" ;;
esac
timeout 115 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
  JIT_ROUTEB_DM_CONT_OPNEW_BOX=1 $EXTRA \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x1022338ef4-0x102233900 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "[$ARM] EXIT=$EXIT"
echo "== patch lines =="
grep -aE "SH245-closure|SH246-skip" "$LOG" | tail -3
echo "== bad_alloc / crash =="
grep -acE "bad_alloc|terminate|SIGSEGV|SIGABRT" "$LOG"
echo "== continuation progress (furthest block-entry pc) =="
grep -aE "region hit at guest pc=0x102bd1" "$LOG" | grep -aoE 'pc=0x102bd[0-9a-f]{4}' | sort -u | tr '\n' ' '; echo
echo "== nativeAppBridgeAppStart region (0x2338ef4) hits =="
grep -aE "region hit at guest pc=0x1022338" "$LOG" | grep -aoE 'pc=0x[0-9a-f]+' | sort -u | tr '\n' ' '; echo