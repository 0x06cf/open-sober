#!/bin/bash
# SH343: cross the SH341 poison fencepost on the persistence lane. With
# JIT_ROUTEB_LSM_KEYFIX=1, routeb_lsm_keyfix_guard at the LSM pool-pop write-site
# (0x101d9a528) substitutes the ONE poisoned .text KEY (0x101d968e4, caller LR=
# 0x10626b6dc = the 0x626b6d0 pool-pop wrapper) with a valid host-heap cell so the
# `str x8,[x1]` write lands in real memory and the pop completes instead of ABRT.
# MEASURED (real libroblox.so): pre-fix the ladder ABRTs at 0x101d9a528 (the SH341
# terminal); post-fix it advances one fencepost to guestpc=0x101db1b08 (SH285's LSM
# reader/pop terminal, fault=0xffffffffffffffff RBX-poisoned live-object pointer) —
# genuine forward progress on the persistence-lane (STATUS candidate #2).
# Full send-appevent + session ladder, same env as capture_sh341_lsm_keytrace.sh + KEYFIX.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh343-lsm-keyfix.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== keyfix crossing events (must be >=1 for a crossing) ==="
grep -c "routeb-lsm-keyfix" "$LOG"
echo "=== the ONE poisoned key trace ==="
grep -E "routeb-lsm.*POISONED" "$LOG" | tail -2
echo "=== terminal guestpcs (pre-fix: 0x101d9a528; post-fix advance: 0x101db1b08) ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== SIGSEGV/SIGABRT detail ==="
grep -E "\[SIGSEGV\]|\[SIGABRT\]" "$LOG" | tail -2