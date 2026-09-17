#!/bin/bash
# SH296 (default-inert, opt-in --v2boot-session-dmfn): drive the DM-CONSTRUCTION
# handler 0x1023f03b4 DIRECTLY (SH255 proved it indirect-dispatch-only; never executed
# headlessly). Entry dispatch = blr this.vt[32] (ret0 -> construction body); fields
# [this+120/128/136/144] gate depth. This MEASURES the fn headlessly FIRST time and the
# one-next-unsynthesized-object gate on the DM line (SH248 loop).
# Stage 1 (no JIT_ROUTEB_DMFN_FIELDS): empty receiver -> expect benign soft-return.
# Stage 2 (JIT_ROUTEB_DMFN_FIELDS=1): fields seeded to the routeb singleton -> deep
# construction body, measure the exact terminal gate.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
# --- Stage 1: empty receiver ---
for i in 1 2 3; do
  timeout 45 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-dmfn > runs/sh296-dmfn-s1-$i.txt 2>&1
  echo "s1 run$i EXIT=$? $(grep -aoE 'SH296.*' runs/sh296-dmfn-s1-$i.txt | tail -2 | tr '\n' ' ')"
done
echo "=== SH296 stage1 markers (run1) ==="
grep -aE "SH296" runs/sh296-dmfn-s1-1.txt | tail -6
echo "=== stage1 terminal ==="
grep -aE "SIGSEGV|guestpc|fault=|stopped:|returned Ok" runs/sh296-dmfn-s1-1.txt | tail -6
# --- Stage 2: fields seeded ---
for i in 1 2 3; do
  timeout 45 env $BASE JIT_ROUTEB_DMFN_FIELDS=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-dmfn > runs/sh296-dmfn-s2-$i.txt 2>&1
  echo "s2 run$i EXIT=$? $(grep -aoE 'SH296.*' runs/sh296-dmfn-s2-$i.txt | tail -3 | tr '\n' ' ')"
done
echo "=== SH296 stage2 markers (run1) ==="
grep -aE "SH296" runs/sh296-dmfn-s2-1.txt | tail -8
echo "=== stage2 terminal ==="
grep -aE "SIGSEGV|guestpc|fault=|stopped:|returned Ok" runs/sh296-dmfn-s2-1.txt | tail -6