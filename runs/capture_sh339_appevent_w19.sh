#!/bin/bash
# SH339 repro (4/4 measured): the fabricated "Home" jstring does NOT route to event-code 4
# in SendAppEventOnAppReady's discriminator — captured MID-EXECUTION at the real block
# boundary 0x102bb46b8 where the parsed 4th jstring's libc++ SSO header is read.
#   baseline arm (SH307 forward, --v2boot-send-appevent) + JIT_ROUTEB_APPEVENT_W19=1
#   EXPECT: "[elfjit:appevent-w19] ... [sp].b0=0xc ... event-code ... 0" (Home is size 4,
#   would need b0=0x8 to route to code 4); handle bytes = 486f6d65 ("Home").
# And the do-init pipe still fires (app-data-model-count 0x1) with event-code 0.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_APPEVENT_W19=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-send-appevent"
timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S \
  > runs/sh339-w19-repro.txt 2>&1
echo "EXIT=$?"
echo "--- appevent-w19 captures ---"
grep "appevent-w19" runs/sh339-w19-repro.txt
echo "--- pipe still fires (app-data-model) ---"
grep -iE "app-data-model-count|SendAppEventOnAppReady returned" runs/sh339-w19-repro.txt