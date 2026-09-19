#!/bin/bash
# SH371: narrow block-entry trace of the engine-init dispatcher body to pin WHICH
# leaf vt-dispatch (vt[+0xf8] @0x2bd8d2c or vt[+0x108] @0x2bd8d50) diverts control
# away from the fall-through `bl sub 0x2bd8dac` (which SH228 measured NEVER fires).
# Regions:
#   0x102bd8ce8..0x102bd8e30  dispatcher body (entry -> blr vt+0x1f0)
#   0x102bd8dac..0x102bd8e30  sub_2bd8dac body
#   0x102bd1b98..0x102bd1d08  fnB engine-init band
#   0x102bd1d68..0x102bd2600  continueAfterFlagsLoaded_ (DMCONT target)
# Goal: record every block-entry pc in the dispatcher to see exactly how far the
# pipe advances before divergence (getter? vt+0xf8 leaf? vt+0x108 leaf?) and
# whether continueAfterFlagsLoaded_ ever gets dispatched.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh371-dispatcher-body.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_REGION_WATCH=0x102bd8ce8-0x102bd8e30,0x102bd8dac-0x102bd8e30,0x102bd1b98-0x102bd1d08,0x102bd1d68-0x102bd2600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== engine-init dispatcher body + sub + fnB + continueAfterFlagsLoaded_ block-entry pcs (sorted) ==="
grep -oE "guest pc=0x102bd(8|1)[0-9a-f]+" "$LOG" | awk '{print $NF}' | sort | uniq -c | sort -rn
echo "=== did sub_2bd8dac (0x102bd8dac) fire? (0 = NO -> diverted before bl sub) ==="
grep -cE "guest pc=0x102bd8dac" "$LOG"
echo "=== did continueAfterFlagsLoaded_ (0x102bd1d68) fire? ==="
grep -cE "guest pc=0x102bd1d68" "$LOG"
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root probe|MH_APP_READY|app-data-model-count|ladder done|SendAppEventOnAppReady returned|continueAfterFlagsLoaded_" "$LOG" | tail -8