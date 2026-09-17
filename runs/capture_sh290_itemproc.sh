#!/bin/bash
# SH290 (default-inert, opt-in --v2boot-session-itemproc): drive the engine's
# OWN worker item-PROCESSOR 0x102207950 in ISOLATION (not the whole consume
# loop) so its ONCE-BUILD completes + is read back — the SH288 consumer drive
# crossed this gate (region-hits 0x102207950) but SIGSEGV'd at the downstream
# SH273 lifecycle wall before [0x106a63b00] was ever read. A fabricated ZEROED
# queue-item makes both indir blr dispatches cbz-skip ([item+32]=0 -> vt[+48]
# skipped; [item+48]=0 -> 0x22193a0 helper skipped), so only the once-guard
# [0x106a63b08] __call_once (0x284ce54) -> once-body (builds [0x106a63b00] via
# 0x2173b3c string-map insert, guard-release 0x284cf5c) + the clock helper
# 0x221942c run, then it returns cleanly.
# MEASURED (real libroblox.so, 2/3 clean readback; run-variable 3rd never
# reached the rung = known ladder flake): item-proc returned Ok,
# once-guard 0 -> 0x101 (done-set), once-built [0x106a63b00] = 0x800000c — the
# engine's own session-object cell now stores a real token headlessly.
# Honest: does NOT manufacture a DM (MH_* false, DM-root 0); Route-B live-DM
# gate UNCHANGED; SH174 latch stays the single forward hook. Terminal after the
# ladder = SH285-B LSM reader/pop live-object wall 0x101db1b08 (baseline parity).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
for i in 1 2 3; do
  timeout 45 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-itemproc > runs/sh290-itemproc-$i.txt 2>&1
  echo "run$i EXIT=$? oncebuilt=$(grep -aoE 'once-built\)=0x[0-9a-f]+' runs/sh290-itemproc-$i.txt | tail -1) guard=$(grep -aoE 'once-guard=0x[0-9a-f]+' runs/sh290-itemproc-$i.txt | tail -1) ret=$(grep -aoE 'SH290 item-proc returned Ok\(0x[0-9a-f]+\)' runs/sh290-itemproc-$i.txt | tail -1) term=$(grep -aoE 'guestpc=0x101db1[0-9a-f]+' runs/sh290-itemproc-$i.txt | tail -1)"
done
echo "=== SH290 markers (run1) ==="
grep -aE "SH290|once-built" runs/sh290-itemproc-1.txt | tail -6