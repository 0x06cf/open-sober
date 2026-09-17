#!/bin/bash
# SH262 A/B: app-start self-drive drain dispatch — does seeding the drain's global
# [0x106a70c90] with the all-leaf adapter (JIT_ROUTEB_APPSART_DRAIN_SEED) advance the
# drain past the std::bad_function_call terminal? Baseline (BYE) = full SH259 seed set
# WITHOUT the new seed; A-arm (ON) = same + JIT_ROUTEB_APPSART_DRAIN_SEED=1.
# Drain reach is run-variable (~2/5 fires); run N each arm and compare terminal class.
set -u
cd "$(dirname "$0")/.."
ARM="${1:-ab}"   # 'base' | 'on' | 'ab'
N="${2:-5}"
BASE_ENV="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
  JIT_REGION_WATCH=0x10233bc80-0x10233bd00"
run_arm () {
  local label="$1" extra="$2" i
  for i in $(seq 1 "$N"); do
    local LOG=/home/hermes-worker/runs/open-sober/runs/sh262-$label-r$i.txt
    timeout 120 env $BASE_ENV $extra \
      ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
      --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
      > "$LOG" 2>&1
    local EXIT=$?
    local DH=$(grep -c 'region hit' "$LOG")
    local TERM=$(grep -oE 'guestpc=0x[0-9a-f]+|bad_function_call|stack smash|terminate' "$LOG" | tail -1)
    local SEED=$(grep -q "routeb-sh262" "$LOG" && echo "seed-fires" || echo "-")
    echo "$label-r$i EXIT=$EXIT drain_hits=$DH term=$TERM sh262=$SEED"
  done
}
echo "=== ARM: $ARM (N=$N per arm) ==="
if [ "$ARM" = "base" ] || [ "$ARM" = "ab" ]; then
  echo "--- baseline (BYE, no DRAIN_SEED) ---"; run_arm base ""
fi
if [ "$ARM" = "on" ] || [ "$ARM" = "ab" ]; then
  echo "--- A-arm (ON, JIT_ROUTEB_APPSART_DRAIN_SEED=1) ---"; run_arm on "JIT_ROUTEB_APPSART_DRAIN_SEED=1"
fi