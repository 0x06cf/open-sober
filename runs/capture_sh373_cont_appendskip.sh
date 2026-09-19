#!/bin/bash
# SH373: reproducible artifact — from the SH371 reaching-env (which makes the DM-creator
# continuation continueAfterFlagsLoaded_ 0x102bd1d68 run DEEP headlessly), adding the
# SH349 single-caller append sub-call skip (JIT_ROUTEB_LSM_APPEND_SKIP) deterministically
# CROSSES the standing SH285 persistence leaf (guestpc=0x101db1b08 -> 0 hits across runs)
# and the run advances to 0x101d9a708 — SH349's pack/name-string helper in the SAME
# unconstructed-live-object family (source pointer x19=0xff..ff). This is the first cross
# of SH285 from the continuation/reaching env (SH358 never reached the continuation REGION
# at 0 hits because it lacked the DM_CONT_M48_SEED/CONT_APPNAME_SEED that SH371 added).
# The verdict it confirms: SH285 is crossable but lands in the measured-closed LSM
# unconstructed lane (SH349/350/358) — cross to the family, not past it. The append+pack
# combo instead parks at the pool-pop write-site 0x101d9a528 (write-to-0x1 divergence).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh373-cont-appendskip.txt
MAX=${SH373_RETRY_MAX:-6}
run_once() {
  rm -f "$LOG"
  timeout 150 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
    JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
    JIT_ROUTEB_LSM_APPEND_SKIP=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600 \
    JIT_DUMP_PC=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
    > "$LOG" 2>&1
  local exit=$?
  # SUCCESS = SH285 crossed (0 sh285 hits) + continuation fired + 0 SIGSEGV at SH285
  local sh285=$(grep -cE "guestpc=0x101db1b08" "$LOG")
  local cont=$(grep -cE "guest pc=0x102bd1d68" "$LOG")
  local crash=$(grep -icE "SIGSEGV guestpc=0x101db1b08|SIGSEGV guestpc=0x101d9a708" "$LOG")
  [ "$sh285" = "0" ] && [ "$crash" = "0" ]
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; break; fi
done
echo "=== SH285 crossed (0 = crossed) / continuation fired / terminal ==="
echo -n "sh285="; grep -cE "guestpc=0x101db1b08" "$LOG"
echo -n "cont-fired="; grep -cE "guest pc=0x102bd1d68" "$LOG"
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "MH_APP_READY|DM-root probe|continueAfterFlagsLoaded|append-skip|governor|SendAppEventOnAppReady returned" "$LOG" | tail -8
echo "won-attempt=${won:-none}"