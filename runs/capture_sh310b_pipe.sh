#!/bin/bash
# SH310b (decisive): does SendAppEventOnAppReady BUILD + DISPATCH its app-event?
# The discriminator + version-gate are MID-BLOCK (invisible to block-entry region-watch),
# but the app-event build's `bl 2baeeec` (pipe) @0x2bb4958 IS a bl target -> block entry.
# With --v2boot-skip-appstart, 2baeeec is called ONLY by this rung in this env, so a
# region hit on 0x102baeeec PROVES the full body paced: 4x jstring->RBX-string (0x21e1fec)
# -> discriminator (w19) -> version gate [0x10683d350] -> 0x58 app-event alloc -> pipe.
# Also watch do-init 0x2206c40 (the pipe's synchronous route, seeded sync-gate).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
RW="0x102baeeec-0x102baf000,0x102206c40-0x102209000"
for i in 1 2 3; do
  timeout 150 env $BASE JIT_REGION_WATCH="$RW" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
    > runs/sh310b-pipe-$i.txt 2>&1
  echo "run$i EXIT=$? appev=$(grep -c 'SendAppEventOnAppReady returned Ok' runs/sh310b-pipe-$i.txt)"
  echo "  pipe(2baeeec) hits=$(grep -cE 'region hit at guest pc=0x102baeeec' runs/sh310b-pipe-$i.txt) doinit hits=$(grep -cE 'region hit at guest pc=0x102206c40' runs/sh310b-pipe-$i.txt)"
  echo "  pcs: $(grep -E 'region hit at guest pc' runs/sh310b-pipe-$i.txt | grep -oE 'pc=0x[0-9a-f]+' | tr -d 'pc=' | sort -u | tr '\n' ' ')"
done