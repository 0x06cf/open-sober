#!/bin/bash
# SH245 A/B: patch the engine-init getter's FMOD/AAudio tail `b 0x624e6c0` (0x2174c80)
# -> `ret` so the getter returns to the dispatcher (0x2bd8d18) instead of diving into a
# tail that never returns. Watch whether the dispatcher then resumes and reaches
# sub_2bd8dac -> the vt[+0x1f0] dispatch = the REAL continueAfterFlagsLoaded_ (DMCONT).
# Arms: off (no patch) | on (GETTER_TAIL_RET=1) | on.m2 (GETTER_TAIL_RET=1 + MGR_MINUS2=1).
set -u
cd "$(dirname "$0")/.."
ARM="$1"   # off|on|on.m2
LOG=/home/hermes-worker/runs/open-sober/runs/sh245-${ARM}.txt
rm -f "$LOG"
EXTRA=""
case "$ARM" in
  on) EXTRA="JIT_ROUTEB_GETTER_TAIL_RET=1" ;;
  on.m2) EXTRA="JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_MGR_MINUS2=1" ;;
  on.m48) EXTRA="JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1" ;;
  on.m2m48) EXTRA="JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_MGR_MINUS2=1 JIT_ROUTEB_DM_CONT_M48_SEED=1" ;;
esac
timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 $EXTRA \
  JIT_REGION_WATCH=0x102174c00-0x102174c84,0x102bd8d18-0x102bd8dac,0x102bd8dac-0x102bd9060,0x102bd1d68-0x102bd2600,0x10242a5e4-0x10242b1c0 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "[$ARM] EXIT=$EXIT"
echo "== which watched regions fired (block-entry pcs) =="
grep -E "region hit at guest pc=0x(102174c|102bd8d18|102bd8dac|102bd1d68|10242a5e4)" "$LOG" | grep -oE 'pc=0x[0-9a-f]+' | sort -u
echo "== SH245 patch line (if any) =="
grep -E "SH245 patched|WARN SH245" "$LOG" | tail -2
echo "== continueAfterFlagsLoaded_ (0x102bd1d68) hits (0 = still latent) =="
grep -cE "region hit at guest pc=0x102bd1d68" "$LOG" || true
echo "== StartLuaAppDM return + crash =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)" "$LOG" | tail -1
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG" || true