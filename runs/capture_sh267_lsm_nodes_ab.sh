#!/bin/bash
# SH267 A/B: cross the LocalStorageManager INSERT-leaf wall (guestpc=0x101db1d04)
# that SH260 parked and that blocks the SEP-17 session-drive rungs (SH264-266 all
# latent because the ladder dies here first). New default-inert
# JIT_ROUTEB_APPSART_LSM_NODES=1 gives each of the 0x2000 sub-slots a real leaked
# zeroed node cell so the insert's atomic-OR (0x2b9ea40) lands in valid memory
# instead of addr 0. OFF = baseline (crash 0x101db1d04); ON = seed fires, run
# advances into the LSM free-list/pop path to a NEW terminal 0x101d9a528.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
for tag in OFF ON; do
  EXTRA=""
  [ "$tag" = "ON" ] && EXTRA="JIT_ROUTEB_APPSART_LSM_NODES=1"
  LOG=runs/sh267-ab-$tag.txt
  rm -f "$LOG"
  timeout 120 env $BASE $EXTRA \
    JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x102330000-0x102350000 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
    > "$LOG" 2>&1
  echo "[$tag] EXIT=$? nodecells=$(grep -c 'SH267 per-node' "$LOG") terminal=$(grep -oE 'guestpc=0x[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
done