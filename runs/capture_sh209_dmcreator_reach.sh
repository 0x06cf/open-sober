#!/bin/bash
# SH209: does the now-completing V2 ladder (SH202+ V2_ONDEMAND makes the full
# ladder complete clean 8/12) REACH a NativeDataModelManager DM-construction site?
# SH164 measured the DM-creator region (getFlagsFromEngine_/initEngine_
# 0x102bd1a30..0x102bd1d08 + startLuaApp_ 0x102bd2504) at ZERO region hits pre-SH202,
# when the ladder always stopped at the V2Init outside-image flake first. The state
# has since changed (full ladder completes). Re-measure reachability of the DM
# creator on the completing path.
# POSITIVE CONTROL: governor-tail region 0x102e9fa80..0x102ea3b40 fires every clean
# run (SH197/204). If the control fires but the DM-creator regions stay 0, that is a
# FRESH measured reachability negative at the post-SH202 state (Route-B live-DM
# world-build gate reconfirmed on the completing path). If a DM-creator region fires,
# that is ROUTE-B FORWARD MOTION — a real construction body newly reached.
set -u
cd "$(dirname "$0")/.."
N=${1:-4}
DM=0; GOV=0; UNREACH=0; CRASH=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh209-$i.txt"
  rm -f "$LOG"
  timeout 110 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    JIT_REGION_WATCH=0x102bd1a30-0x102bd1d08,0x102bd21d4-0x102bd2600,0x102e9fa80-0x102ea3b40 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  # DM-creator hits = pcs in [0x102bd1a30,0x102bd1d08) or [0x102bd21d4,0x102bd2600)
  dmh=$(grep -E "\[region-watch\] region hit at guest pc=0x(102bd[12])" "$LOG" | grep -oiE "pc=0x102bd1[a-f0-9]|pc=0x102bd2[1-6][a-f0-9]" | sort -u | wc -l)
  # governor hits (control) = any 0x102e9f... pc
  gov=$(grep -oE "pc=0x102e9f[a-f0-9]+|pc=0x102ea[a-f0-9]+" "$LOG" | sort -u | wc -l)
  crash=$(grep -acE "SIGSEGV|SIGABRT" "$LOG")
  DM=$((DM+dmh)); GOV=$((GOV+gov)); CRASH=$((CRASH+crash))
  echo "run $i: EXIT=$E dmcreator_pcs=$dmh govtail_pcs=$gov crashes=$crash"
  if [ "$dmh" -ge 1 ]; then echo "  !! DM-CREATOR REGION HIT -> ROUTE-B CONTACT" >> /tmp/sh209-dmhits.txt; fi
  grep -E "\[region-watch\] region hit at guest pc=0x102bd[12]" "$LOG" | head -5
done
echo "=== SH209 summary: total dmcreator_pcs=$DM (across $N runs) govtail_pcs=$GOV crashes=$CRASH ==="
if [ "$DM" -ge 1 ]; then echo ">>> ROUTE-B FORWARD MOTION: a NativeDataModelManager DM-construction body is REACHED on the completing ladder"; else echo ">>> FRESH NEGATIVE: DM-creator region stays 0 on the completing ladder (post-SH202 state); Route-B live-DM world-build gate reconfirmed at newest state."; fi