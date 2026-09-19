#!/bin/bash
# SH372: classify the SH285 terminal reached from the DM-CONTINUATION path
# (continueAfterFlagsLoaded_ 0x102bd1d68, SH371 now runs deep into it) with a full
# register + guest-stack dump. SH285 classified the terminal ONLY from the
# settings-state path ([obj+0x50]=0xff..ff backward byte-copy). The continuation
# reaches the SAME pc 0x101db1b08 — does it fault on the SAME object, or a
# DIFFERENT unconstructed field? That is a genuinely-unexplored corner.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh372-continuation-terminal.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_DUMP_PC=1 JIT_GUEST_STACK_DUMP=1 \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== continuation block-entry pcs (sorted) ==="
grep -oE "guest pc=0x102bd[0-9a-f]+" "$LOG" | awk '{print $NF}' | sort | uniq -c | sort -rn
echo "=== terminal register dump + guest stack (the classification) ==="
grep -A6 -E "\[SIGSEGV\]|fault=0xffffffffffffffff guestpc=0x101db1b08|GSDSP" "$LOG" | head -40
echo "=== did the continuation fire? ==="
grep -cE "guest pc=0x102bd1d68" "$LOG"
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo