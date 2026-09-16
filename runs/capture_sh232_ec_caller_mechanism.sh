#!/bin/bash
# SH232: MECHANISM-level reachability of the EC DM-creation world's CALLER bodies.
# SH231 watched only the EC TARGET region [0x102e1c650,0x102e25200) = 0 hits. But two EC
# callers live INSIDE ladder rungs (StartLuaAppDM +0x1468 -> bl 0x2e24598; V2InitWithParams
# +0x6a114 -> bl 0x2e24468). This v2 measures HOW DEEP those enclosing functions get before
# soft-returning, turning SH231a's static "callers never reached" inference into a RUNTIME
# mechanism: do the caller bodies even translate themselves as blocks (entry pcs fire)?
# Ranges (guest = file+0x100000000):
#   EC target region:       0x102e1c650 0x102e25200
#   StartLuaAppDM EC block: 0x1023f1270 0x1023f1300   (marshalling 0x1270..0x1294 -> bl 0x2e24598, post-ret 0x1298)
#   V2Init body deep EC blk:0x1023cfd40 0x1023cfd70   (before bl 0x2e24468)
#   V2Init body entry area: 0x1023cfb40 0x1023cfc7c   (mid-fn near the EC callers)
#   govtail control:        0x102e9fa80 0x102ea3b40   (ladder-completion control)
set -u
cd "$(dirname "$0")/.."
N=${1:-3}
EC=0; GOV=0; SLDM=0; V2DEEP=0; V2MID=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh232-$i.txt"
  rm -f "$LOG"
  timeout 175 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    JIT_REGION_WATCH=0x102e1c650-0x102e25200,0x1023f1270-0x1023f1300,0x1023cfd40-0x1023cfd70,0x1023cfb40-0x1023cfc7c,0x102e9fa80-0x102ea3b40 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  E=$?
  ech=$(grep -oE "region hit at guest pc=0x102e(1c6|1e|20|21|22|23|24)" "$LOG" | sort -u | wc -l)
  sldm=$(grep -oE "region hit at guest pc=0x1023f12" "$LOG" | sort -u | wc -l)
  v2d=$(grep -oE "region hit at guest pc=0x1023cfd4" "$LOG" | sort -u | wc -l)
  v2m=$(grep -oE "region hit at guest pc=0x1023cfb" "$LOG" | sort -u | wc -l)
  gov=$(grep -oE "region hit at guest pc=0x102e9|region hit at guest pc=0x102ea" "$LOG" | sort -u | wc -l)
  EC=$((EC+ech)); SLDM=$((SLDM+sldm)); V2DEEP=$((V2DEEP+v2d)); V2MID=$((V2MID+v2m)); GOV=$((GOV+gov))
  echo "run $i: EXIT=$E ec_target_pcs=$ech sldm_ecblock_pcs=$sldm v2deep_pcs=$v2d v2mid_pcs=$v2m govtail_pcs=$gov"
done
echo "=== SH232 caller-mechanism: total ec_target=$EC sldm_ecblock=$SLDM v2deep=$V2DEEP v2mid=$V2MID govtail=$GOV (across $N runs) ==="
[ "$EC" -ge 1 ] && echo ">>> EC WORLD TARGET REACHED (forward motion)" || echo ">>> EC target still 0."
[ "$SLDM" -ge 1 ] && echo ">>> StartLuaAppDM reaches its OWN EC-arg block (mechanism: within-fn advance)" || echo ">>> StartLuaAppDM soft-returns BEFORE its EC-arg block -> its bl 0x2e24598 is un-translatable-on-ladder."
[ "$V2MID" -ge 1 ] && echo ">>> V2InitWithParams mid-body reached" || echo ">>> V2InitWithParams never reaches deep EC-caller body either."