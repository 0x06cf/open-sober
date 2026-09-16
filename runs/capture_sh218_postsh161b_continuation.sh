#!/bin/bash
# SH218: post-SH217 fresh measurement — SH217 corrected the SH161b mode-2 seed window
# to the REAL tail-block entry [0x102e9fcc4,0x102e9fdc8], so fn 0x1023c12c0 now takes its
# benign b.eq no-op and the fault-prone transition body (reads [0x6a70700], dispatches
# 23c14dc/1504/1574 + conditional FMOD-AAudio 0x626b6d0) is BYPASSED on the completing ladder.
# ALL prior governor-tail negatives (SH197/204/209: "ends at the 0x102ea30dc canary-ret")
# were measured with the seed window BROKEN (0 fires) => transition body LIVE.
#
# THIS IS A NEW STATE NOBODY HAS RE-MEASURED. Question: with the transition body bypassed,
# does the completing ladder advance the governor tail DEEPER — into the world-build body
# 0x102ea3b14 (SH199/204 latent gate) or any DM/construction body past the prior canary-ret
# terminal? This is the operator's "hunt the DMCONT continuation" at the newest state.
#
# Region-watch:
#  GOV-TAIL  [0x102e9fa80,0x102ea4000)  - full governor tail region
#  WORLD-BUILD [0x102ea3b14,0x102ea3c50) - SH199 deep app-start world-build body (0x102ea3b14)
#  DM-CREATOR [0x102bd1a30,0x102bd1d08)  - NativeDataModelManager getFlagsFromEngine_/initEngine_
#  DM-CREATOR-B [0x102bd21d4,0x102bd2600) - initializeLuaApp_/startLuaApp_
# Why it matters: if the tail now passes 0x102ea30dc and reaches 0x102ea3b14 (world-build) or a
# DM-creator pc, that is Route-B FORWARD MOTION at the newest state. If it still ends at the
# canary-ret, the live-DM structural gate is reconfirmed under the corrected-SH161b state.
set -u
cd "$(dirname "$0")/.."
N=${1:-4}
EXEC=0; TAIL=0; WBODY=0; DM=0; CRASH=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh218-$i.txt"
  rm -f "$LOG"
  timeout 120 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    JIT_REGION_WATCH=0x102e9fa80-0x102ea4000,0x102ea3b14-0x102ea3c50,0x102bd1a30-0x102bd1d08,0x102bd21d4-0x102bd2600 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  ex=$(grep -ac "SH161b" "$LOG")
  # governor-tail hits (whole tail region)
  tail_pcs=$(grep -oE "region hit at guest pc=0x102e9f[a-f0-9]+|region hit at guest pc=0x102ea3[0-5][a-f0-9]" "$LOG" | sort -u | wc -l)
  # deepest tail pc actually hit
  deep=$(grep -oE "region hit at guest pc=0x(102e9f|102ea)[0-9a-f]+" "$LOG" | grep -oE "0x102ea[0-9a-f]+" | sort -u | tail -3 | tr '\n' ' ')
  # world-build body hits (0x102ea3b14+)
  wb=$(grep -E "region hit at guest pc=0x102ea3[b]" "$LOG" | sort -u | wc -l)
  # DM-creator hits
  dm=$(grep -oE "region hit at guest pc=0x102bd[12][0-9a-f]+" "$LOG" | sort -u | wc -l)
  crash=$(grep -acE "SIGSEGV|SIGABRT" "$LOG")
  # deepest pc overall in the tail region
  TAIL=$((TAIL+tail_pcs)); WBODY=$((WBODY+wb)); DM=$((DM+dm)); CRASH=$((CRASH+crash))
  echo "run $i: EXIT=$E sh161b_seedlines=$ex tail_pcs=$tail_pcs deepest=$deep worldbuild_pcs=$wb dmcreator_pcs=$dm crashes=$crash"
  # always print the tail deepest pcs
  grep -oE "region hit at guest pc=0x102ea[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo ""
done
echo "=== SH218 summary ($N runs): tail_pcs=$TAIL worldbuild_pcs=$WBODY dmcreator_pcs=$DM crashes=$CRASH ==="
if [ "$WBODY" -ge 1 ] || [ "$DM" -ge 1 ]; then
  echo ">>> ROUTE-B FORWARD MOTION BELOW THE PRIOR CANARY-RET TERMINAL (new state, post-SH217)"
else
  echo ">>> No forward motion: tail still ends at/above 0x102ea30dc; live-DM gate reconfirmed under corrected-SH161b state"
fi