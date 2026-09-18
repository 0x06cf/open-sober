#!/bin/bash
# SH311: does the do-init ONCE-LAMBDA (0x2206d10..0x2206d90) fire on the send-appevent
# run? do-init __call_once: ldarb once-guard [0x106a68410] (0x2206c7c) -> tbz bit0 ->
# if 0 JUMP 0x2206d10 (ONCE path: bl 0x284ce54 once, bl 0x2173b3c construct ref'd,
# then `str x0,[x23,#1032]` @0x2206d74 STORES DM-root [0x106a68818]); if 1 -> 0x2206c88
# done-path reads [0x106a68818] into x1 (leaves 0 if never populated).
# SH310's rung clears once-guard + re-seeds main-id before SendAppEvent, but DM-root
# still reads 0 after. This measures WHICH path the do-init takes (does 0x2206d10 fire?).
# One band (whole do-init) to minimize SH248b perturbation.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
RW="0x102206c40-0x102206d90"
for i in 1 2 3; do
  timeout 150 env $BASE JIT_REGION_WATCH="$RW" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
    > runs/sh311-once-$i.txt 2>&1
  echo "run$i EXIT=$? appev=$(grep -c 'SendAppEventOnAppReady returned Ok' runs/sh311-once-$i.txt)"
  echo "  once-lambda(2206d10)=$(grep -cE 'region hit at guest pc=0x102206d1[0-9a-f]' runs/sh311-once-$i.txt) done-path(2206c88)=$(grep -cE 'region hit at guest pc=0x102206c88' runs/sh311-once-$i.txt) dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh311-once-$i.txt | tail -1)"
  echo "  doinit pcs: $(grep -E 'region hit at guest pc=0x102206' runs/sh311-once-$i.txt | grep -oE 'pc=0x[0-9a-f]+' | tr -d 'pc=' | sort -u | tr '\n' ' ')"
done