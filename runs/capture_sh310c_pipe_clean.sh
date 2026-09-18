#!/bin/bash
# SH310c (clean confirmation): pipe-only band to avoid the do-init-abort perturbation
# (SH248b class). If SendAppEventOnAppReady returns Ok AND pipe 0x102baeeec fires, the
# fabricated 'Home' jstring round-trips: helper 0x21e1fec -> SSO "Home" -> discriminator
# w19 -> version gate -> 0x58 app-event -> pipe -> do-init. Candidate (b) CLOSED clean.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
RW="0x102baeeec-0x102baf000"
for i in 1 2 3; do
  timeout 150 env $BASE JIT_REGION_WATCH="$RW" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
    > runs/sh310c-pipe-$i.txt 2>&1
  echo "run$i EXIT=$? appev=$(grep -c 'SendAppEventOnAppReady returned Ok' runs/sh310c-pipe-$i.txt) pipe_hits=$(grep -cE 'region hit at guest pc=0x102baeeec' runs/sh310c-pipe-$i.txt) pipe_beyond=$(grep -cE 'region hit at guest pc=0x102baef' runs/sh310c-pipe-$i.txt)"
done