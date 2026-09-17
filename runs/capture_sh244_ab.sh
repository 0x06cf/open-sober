#!/bin/bash
# SH244 A/B: mgr vt[+0x30] verb. Baseline (write_leaf=0): the engine-init getter
# 0x102174c04 b.ne-skips its own StartLuaAppDM call (0x10242a5e4) and diverts to
# the FMOD/AAudio tail 0x624e6c0 (never returns to dispatcher 0x102bd8d18).
# Forward (JIT_ROUTEB_DM_MGR_MINUS2=1, write_leaf_minus2 returns -2): the getter's
# `cmn w0,#0x2` takes the branch -> bl nativeAppBridgeStartLuaAppDM from INSIDE
# engine-init. Watch both + dispatcher resume pcs.
set -u
cd "$(dirname "$0")/.."
LEVER="$1"   # off|on
LOG=/home/hermes-worker/runs/open-sober/runs/sh244-${LEVER}.txt
rm -f "$LOG"
if [ "$LEVER" = "on" ]; then EXTRA="JIT_ROUTEB_DM_MGR_MINUS2=1"; else EXTRA=""; fi
# Region-watch: getter 0x102174c04, its SLADM bl 0x10242a5e4, FMOD tail 0x624e6c0,
# dispatcher resume 0x2bd8d18..0x2bd8d68, sub + continueAfterFlagsLoaded_.
timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 $EXTRA \
  JIT_REGION_WATCH=0x102174c00-0x102174d40,0x10242a5e4-0x10242b1c0,0x10624e6c0-0x10624e740,0x102bd8d18-0x102bd8d68,0x102bd8dac-0x102bd9058,0x102bd1d68-0x102bd2600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "[$LEVER] EXIT=$EXIT"
echo "== getter() fired + which regions =="
grep -E "region hit at guest pc=0x(102174c|10242a|10624e6|102bd8d18|102bd8dac|102bd1d68)" "$LOG" | grep -oE 'pc=0x[0-9a-f]+' | sort -u
echo "== SLADM self-call (0x10242a5e4) entered? =="
grep -cE "region hit at guest pc=0x10242a5e4" "$LOG"
echo "== StartLuaAppDM return + crash =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)" "$LOG" | tail -1
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG" || true