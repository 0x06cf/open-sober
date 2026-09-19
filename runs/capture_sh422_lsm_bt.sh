#!/bin/bash
# SH422: name the caller CHAIN into the standing SH285/SH341 persistence lane on
# real libroblox.so. The one-shot bp-chain guard fires at the LSM READER entry
# 0x101d99e30 with JIT_ROUTEB_LSM_BT=1 and logs the saved-lr caller chain
# ([routeb-lsm-bt] .. caller chain lrs: [0]x <- [1]x <- ..), naming WHICH do-init
# callee reaches the persistence lane. Uses the SH408 far-reach env (DONEPATH_MAIN
# + SETFIX + GOVFLAG + filesdir + DYNAMIC) because that composition was MEASURED
# to ENTER the reader band 0x1d99e30 and cross the 0x101db1b08 reader wall — the
# --v2boot-less run aborts at the canary wall before the reader entry. Default-
# inert: without JIT_ROUTEB_LSM_BT=1 the guard reads the env and returns.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh422-lsm-bt.txt
rm -f "$LOG"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_LSM_BT=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 JIT_ROUTEB_DONEPATH_MAIN=1 \
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh422_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive --v2boot-set-filesdir \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH422 caller-chain marker (the deliverable) ==="
grep -aE "routeb-lsm-bt\] SH422" "$LOG" | head
echo "=== reader-entry count (0 = reader not reached in this composition) ==="
grep -acE "routeb-lsm-bt\] SH422" "$LOG"
echo "=== crash summary ==="
grep -aoE "SIGSEGV|SIGABRT|stack smashing|panicked" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh422_root
exit 0