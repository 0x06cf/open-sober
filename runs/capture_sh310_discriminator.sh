#!/bin/bash
# SH310: VERIFY the fabricated 'Home' jstring resolves the SendAppEventOnAppReady
# discriminator. The rung passes 'Home' in x5 (x19=x5, -> dest[sp] via helper
# 0x21e1fec). Disasm: len==4 -> compare 4 bytes to 0x656d6f48 ("Home" LE) ->
# b.eq 0x2bb47c4 (movz w19,#4) [else "Chat"->0x2bb47cc w19=1, else 0x2bb47bc w19=0];
# the event discriminator then feeds w19 -> [x29,#-64] -> app-event build bl 21fd00c
# (only if version gate [0x683d350] low>=6 && byte1>=5, else skip to 0x2bb4824).
# The POST-RUN x19 read is SH308-INVALID (callee-saved), so the RELIABLE observation
# is a block-entry region-watch on 0x2bb47c4 (the 'Home' branch). If it fires, the
# fabricated jstring + x5 placement genuinely resolves 'Home' (w19=4 path), closing
# candidate (b) as a measurement artifact; if not, helper 0x21e1fec isn't resolving.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
# One focused band: SendAppEventOnAppReady discriminator+merge+version-gate+app-event build
RW="0x102bb47bc-0x102bb4830"
for i in 1 2 3; do
  timeout 150 env $BASE JIT_REGION_WATCH="$RW" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
    > runs/sh310-disc-$i.txt 2>&1
  echo "run$i EXIT=$? appev=$(grep -c 'SendAppEventOnAppReady returned Ok' runs/sh310-disc-$i.txt)"
  echo "  home-branch(0x2bb47c4): $(grep -E 'region hit at guest pc=0x102bb47c[48]' runs/sh310-disc-$i.txt | wc -l)"
  echo "  discrim+gate+event pcs: $(grep -E 'region hit at guest pc' runs/sh310-disc-$i.txt | grep -oE 'pc=0x[0-9a-f]+' | tr -d 'pc=' | sort -u | tr '\n' ' ')"
done
echo "markers: dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh310-disc-1.txt | tail -1)"