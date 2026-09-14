#!/bin/bash
# SH126 repro: SendAppEventOnAppReady (guest 0x102bb463c) builds a real 0x58
# app-event object whose vtable 0x635e068 is ALL-ZERO (.data.rel.ro, 0 relocs)
# -> terminal blr at 0x102bb4984 is `blr 0` = benign soft-return -> the rung body
# never completes and MH_FLAGS_LOADED/APP_READY stay false. SH126 materializes the
# vtable's +0x20/+0x28 slots to the benign leaf + seeds the pipe sync-gate
# [0x10683d010]=-1 so the pipe takes the synchronous do-init path (bl 0x2206c40),
# and clears the once-guard + re-seeds main-id so the do-init reconstructs.
# Ladder-only (no render threads) = the stable 3/3 path per SH124 characterization.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh126-run.txt
rm -f "$LOG"
timeout 300 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH126 vtable materialize + pipe sync-gate seed ==="
grep -E "SH126" "$LOG"
echo "=== sendapp rung outcome (Ok vs soft-return 'outside image') ==="
grep -E "SendAppEventOnAppReady" "$LOG"
echo "=== session-advance probe (MH_*) ==="
grep -E "session-advance probe|app-event post|MH_FLAGS" "$LOG"
echo "=== ladder done + join ==="
grep -E "ladder done|joined|returned Ok" "$LOG" | tail -12