#!/bin/bash
# SH342 measurement: does the SH198-surface world-build gate (JIT_ROUTEB_SETWORLDBUILD=1,
# seeds [0x106a70568]=1 so the V2Init rung falls through to bl 0x102ea3b14) now drive control
# INTO the deeper do-init->world-build->nativeAppBridgeAppStart__ (0x102338510) continuation
# on the canonical SH340 SH307-forward deep reach? The world-build fn 0x102ea3b14 calls
# operator-new(0x18) + ctor 0x2eacce4 + nativeAppBridgeAppStart__ 0x2365960; SH198/SH202 measured
# it as never-reliably-reached. Region-watch the world-build body + nativeAppBridgeAppStart__ to
# measure whether the SETWORLDBUILD gate unlocks it (a deeper DMCONT continuation, not a re-tread).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-send-appevent"
# bands: world-build fn 0x102ea3b14 body, nativeAppBridgeAppStart__ 0x102338510, post-do-init, governor
timeout 120 env $BASE JIT_ROUTEB_SETWORLDBUILD=1 JIT_ROUTEB_V2_ONDEMAND=1 \
  JIT_REGION_WATCH=0x102ea3b14-0x102ea3bf0,0x102338510-0x102338700,0x1023eff4c-0x1023f0040,0x102e9fa80-0x102ea3b40 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S \
  > runs/sh342-worldbuild.txt 2>&1
echo "EXIT=$?"
echo "--- world-build region hits (fn 0x102ea3b14) ---"
grep "region hit" runs/sh342-worldbuild.txt | grep -cE "0x102ea3b[0-9a-f]"
echo "--- nativeAppBridgeAppStart__ hits (0x102338510) ---"
grep "region hit" runs/sh342-worldbuild.txt | grep -cE "0x1023385[0-9a-f]"
echo "--- all region hit pcs (dedup) ---"
grep -oE "guest pc=0x[0-9a-f]+" runs/sh342-worldbuild.txt | sort -u
echo "--- worldbuild gate fired? ---"
grep -c "routeb-worldbuild" runs/sh342-worldbuild.txt
echo "--- pipe/ctor markers ---"
grep -iE "SendAppEventOnAppReady returned|app-data-model-count|ladder done|DM-root" runs/sh342-worldbuild.txt | tail -8