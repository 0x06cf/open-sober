#!/bin/bash
# SH350: reproducible artifact for the SH349+1 terminal cross — RET the single-caller
# name/version string-pack helper 0x101d9a708 (the NEXT terminal past SH349's SH285 cross).
# Computes where the completing --v2boot ladder now terminates-advances: with
# JIT_ROUTEB_LSM_PACK_SKIP=1, the pack helper is `ret`'d (caller takes the benign index-0
# tst/b.eq path), so the ladder no longer SIGSEGVs at 0x101d9a708 — it advances deep into the
# LSM pool-pop continuation and then terminates at the run-variable live-object family
# (bad_function_call / 0x101d9a528 / 0x102b9dee0 / 0x1021e40dc), confirming SH349's measured
# "LSM sub-call-whack-a-mole is UNBOUNDED" verdict at one deeper fencepost.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh350-pack-skip.txt
rm -f "$LOG"
timeout 200 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_LSM_INIT_SKIP=1 JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH350 pack-skip fired? ==="
grep -E "SH350" "$LOG"
echo "=== OLD SH349+1 terminal 0x101d9a708 reached? (should be 0) ==="
grep -cE "SIGSEGV.*0x101d9a708|guestpc=0x101d9a708" "$LOG"
echo "=== run-variable live-object terminal arms (SH343/346 family) ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== LSM pool-pop iterations (the SH350 advance, past 0x101d9a708) ==="
grep -cE "\[routeb-lsm\] SH341 pool-pop entry" "$LOG"
echo "=== session markers (should stay false: Route-B live-DM gate unchanged) ==="
grep -E "MH_APP_READY|DM-root" "$LOG" | tail -3