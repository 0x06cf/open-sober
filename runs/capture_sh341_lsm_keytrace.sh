#!/bin/bash
# SH341: repro for the LSM free-list pop KEY attribution. With JIT_ROUTEB_LSM_KEYTRACE=1,
# routeb_lsm_keytrace_guard logs every pool-pop fn entry (0x101d9a5a0) + pop write-site
# (0x101d9a528) with x0=KEY + x30=LR(caller), classifying the key. MEASURED: exactly ONE
# poisoned .text key (0x101d968e4, caller LR=0x10626b6dc = the 0x626b6d0 pool-pop wrapper,
# the "FMOD AAudio 626b6d0" site) — the crash (fault=0x101d968e4 at guestpc=0x101d9a528);
# all other 390 pops carry valid host-heap keys and complete. This is SH267's ON arm
# (reaches the free-list pop terminal) + the keytrace env.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh341-lsm-keytrace.txt
rm -f "$LOG"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== total pool-pop keytrace events ==="
grep -c "routeb-lsm" "$LOG"
echo "=== ALL poisoned (.text/EXEC) keys + their caller (the attribution) ==="
grep -E "routeb-lsm.*POISONED" "$LOG" | head
echo "=== terminal (crash site) ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== SIGSEGV fault detail ==="
grep -E "\[SIGSEGV\]" "$LOG" | tail -1