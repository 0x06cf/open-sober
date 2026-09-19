#!/bin/bash
# SH462: run the SH415 do-init-completion capture with JIT_DMROOT_STORE_WATCH=1
# — the dynamic STORE-level trace that names any guest 64-bit store landing in
# the DM-holder window (once-slot [0x106a68408] .. DM-root [0x106a68818]), i.e.
# whether a reached writer ever populates the Route-B DM holder (vs the wall
# being structural = no store ever targets it headlessly). Same env as SH415.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh462-dmroot-storewatch.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_DMROOT_STORE_WATCH=1 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh462_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DM-root / once-slot STORE-WATCH fires (the writer question) ==="
grep -E "dmroot-store-watch" "$LOG"
echo "=== count of distinct dmroot-store-watch fires ==="
grep -cE "dmroot-store-watch" "$LOG" || true
echo "=== EXECUTE-DO-INIT live-DM probe ==="
grep -E "EXECUTE-DO-INIT live-DM probe" "$LOG"
echo "=== substrate ==="
grep -aoE "substrate complete: [0-9]+/16 atoms returned non-zero Ok" "$LOG"
echo "=== terminal / crash ==="
grep -aoE "SIGSEGV|SIGABRT|EXIT [0-9]+|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh462_root
exit 0