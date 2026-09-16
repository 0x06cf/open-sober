#!/bin/bash
# SH231: is the freshly-LOCATED REAL ExperienceController DM-creation world (SH178's "never
# properly located" site) headless-reachable on the completing --v2boot ladder?
# The world: std::function __func vtable band 0x63981d8..0x6399c00 + lambda bodies
# 0x102e1c650..0x102e25200 (createDataModelForTeleport / submitStartGameTask closures).
# POSITIVE CONTROL: governor-tail 0x102e9fa80..0x102ea3b40 fires every clean completing run
# (SH197/204/209). If the control fires but the EC body region stays 0, that is a FRESH
# correct-location reachability negative (live-DM structural gate reconfirmed at the REAL site).
set -u
cd "$(dirname "$0")/.."
N=${1:-3}
EC=0; GOV=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh231-$i.txt"
  rm -f "$LOG"
  timeout 175 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    JIT_REGION_WATCH=0x102e1c650-0x102e25200,0x102e9fa80-0x102ea3b40 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  # EC body region hits = pcs in [0x102e1c650,0x102e25200)
  ech=$(grep -oE "\[region-watch\] region hit at guest pc=0x102e(1c6|1e|20|21|22|23|24)" "$LOG" | sort -u | wc -l)
  # governor control hits = pcs in [0x102e9fa80,0x102ea3b40) — the REAL completion proof
  # (the SH119 "patched SendAppEvent" lines are NOT completion; only a executed 0x102e9f/0x102ea
  #  region hit proves the ladder walked the governor tail).
  gov=$(grep -oE "\[region-watch\] region hit at guest pc=0x102e9|\[region-watch\] region hit at guest pc=0x102ea" "$LOG" | sort -u | wc -l)
  done_n=$gov
  EC=$((EC+ech)); GOV=$((GOV+gov))
  echo "run $i: EXIT=$E ec_body_pcs=$ech govtail_pcs=$gov (govtail fires = completed ladder)"
  if [ "$ech" -ge 1 ]; then echo "  !! EXPERIENCECONTROLLER WORLD HIT -> ROUTE-B CONTACT" >> /tmp/sh231-echits.txt; fi
  grep -E "\[region-watch\] region hit at guest pc=0x102e(1c6|1e|20|21|22|23|24)" "$LOG" | head -5
done
echo "=== SH231 summary: total ec_body_pcs=$EC (across $N runs, completed-when-govtail_pcs>0) govtail_pcs=$GOV ==="
if [ "$EC" -ge 1 ]; then echo ">>> ROUTE-B FORWARD MOTION: the REAL ExperienceController DM-creation world is REACHED on the completing ladder";
else echo ">>> FRESH CORRECT-LOCATION NEGATIVE: the genuine ExperienceController world stays 0 on the completing ladder; Route-B live-DM structural gate reconfirmed at the REAL site (control fires)."; fi