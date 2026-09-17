#!/bin/bash
# SH269 A/B: the post-ladder SESSION-CTOR rungs execute for the FIRST time headlessly
# because --v2boot-skip-appstart skips the app-start self-driver rungs
# (StartLuaAppDM / V2StartAppWithParams / V1 AppStart__) that previously terminated
# the process BEFORE the ladder loop reached the session rungs.
#  - MessageBus.subscribe returns Ok(0x3e8), EXIT 124, once-guard latches 0x1 (the
#    do-init __call_once runs via the REAL session drive) — deterministic 3/3.
#  - OnGameLoaded ridden, EXIT 124 clean.
#  - SendAppEventOnAppReady: baseline dies at governor NULL-controller
#    (guestpc=0x102ea0b9c); with JIT_ROUTEB_APPSART_GOVFLAG it advances one gate to
#    guestpc=0x102bb803c (preload-overrides live-object). Doc:
#    docs/frontier-sh269-sessionctor-executes-first-time.md
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
for i in 1 2 3; do
  timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-session-bus \
    > runs/sh269-bus-$i.txt 2>&1
  echo "bus$i EXIT=$? sub=$(grep -oE 'MessageBus.subscribe returned Ok\(0x[0-9a-f]+\)' runs/sh269-bus-$i.txt | head -1) once=$(grep -oE 'once-guard\[0x6a68410\]=0x[0-9a-f]+' runs/sh269-bus-$i.txt | tail -1) dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh269-bus-$i.txt | tail -1)"
done
timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-game-loaded \
  > runs/sh269-game.txt 2>&1
echo "game EXIT=$? reached=$(grep -c 'driving SendAppEventOnGameLoaded' runs/sh269-game.txt)"
# OnAppReady A/B (GOVFLAG off vs on)
timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
  > runs/sh269-appev-A.txt 2>&1
echo "appev-A(GOVFLAG off) EXIT=$? terminal=$(grep -oE 'guestpc=0x[0-9a-f]+' runs/sh269-appev-A.txt | sort -u | tr '\n' ' ')"
timeout 120 env $BASE JIT_ROUTEB_APPSART_GOVFLAG=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
  > runs/sh269-appev-B.txt 2>&1
echo "appev-B(GOVFLAG on) EXIT=$? govseed=$(grep -c 'seeded governor-predicate flag' runs/sh269-appev-B.txt) terminal=$(grep -oE 'guestpc=0x[0-9a-f]+' runs/sh269-appev-B.txt | sort -u | tr '\n' ' ')"