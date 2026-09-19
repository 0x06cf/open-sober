#!/bin/bash
# SH381: measure the do-init ONCE-LAMBDA ctor RETURN directly at the store on the
# FULL app-start ladder. Reconciled the once-path store cell [0x106a68408] vs the
# DM-root [0x106a68818] all harness probes read (different addresses: +0x408 vs +0x818
# of page 0x106a68000). Guards fire at block-entry 0x102206d70 (x0 = `bl 0x2173b3c`
# return just before it is stored into once-slot). READ-ONLY.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh381-oncelambda.txt
rm -f "$LOG"
timeout 160 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_DOINIT_ONCELAMBDA=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-send-appevent --v2boot-send-game-loaded \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH381 once-lambda ctor RETURN readout (the decisive measurement) ==="
grep -E "routeb-sh381" "$LOG" || echo "(no SH381 line fired — block-entry 0x102206d70 never entered, once-path may not run on this env)"
echo "=== SH361 do-init dyn trace for context ==="
grep -E "routeb-doinit-dyn" "$LOG" | tail -2
echo "=== DM-root / once-slot probe ==="
grep -E "SH155 DM-root probe|once-slot" "$LOG" | tail -3
echo "=== crash / terminal ==="
echo "signals=$(grep -icE 'SIGSEGV|SIGABRT' "$LOG")"
grep -oE "guestpc=0x102[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo