#!/bin/bash
# SH344c correction A/B: the app-shell ctor band [0x102207b50..0x102209000] is NOT a
# stable negative on the SH343-deepened full ladder. frontier-sh344b reported "0 hits"
# from a single run that diverged to the activity-lifecycle route (guestpc 0x10284cf5c).
# A 3-run A/B shows it FIRES ~58 pcs deep on 2/3 runs (2/3 -> SH285 reader/pop 0x101db1b08
# or bad_function_call terminal; 1/3 -> activity-lifecycle route 0x10284cf5c). The band is
# the recon-sh165fwd __cxa_guard FastLog warmer (0x102207b50 adrp+AcqRel+tbz), NOT a DM/UI
# builder — so firing it advances logging-warm, not DM construction. Run-variable, 3 runs.
set -u
cd "$(dirname "$0")/.."
B="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_REGION_WATCH=0x102207b50-0x102209000,0x1023eff64-0x1023f0100"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus"
for i in 1 2 3; do
  timeout 150 env $B ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S \
    > runs/sh344c-req$i.txt 2>&1
  ex=$?
  deep=$(grep -oE "region hit at guest pc=0x102208[0-9a-f]{3}" runs/sh344c-req$i.txt | sort -u | wc -l)
  b50=$(grep -oE "region hit at guest pc=0x102207[0-9a-f]{3}" runs/sh344c-req$i.txt | sort -u | wc -l)
  term=$(grep -oE "guestpc=0x[0-9a-f]+|terminating due to [a-z_]+" runs/sh344c-req$i.txt | sort -u | tr '\n' ' ' | head -c 100)
  echo "run$i EXIT=$ex appshell_pcs=$((deep+b50)) deep_208x=$deep term=$term"
done