#!/bin/bash
# SH307 A/B: force the nativePreloadFlagOverrides value branch (NOP the tbz) + seed its
# value cell so the getter returns non-NULL, crossing SendAppEventOnAppReady's standing
# preload-overrides terminal (guestpc=0x102bb803c). SH270 wired the value cell but the
# guard helper routes the getter to the CONSTRUCT branch (returns 0), so the wire was
# never read ("inert"). Forcing the VALUE branch is the untried combination.
#   BASELINE (SH307 off): SIGSEGV at guestpc=0x102bb803c (EXIT 139) — standing wall.
#   FORWARD (JIT_ROUTEB_PRELOAD_VALUECELL=1): patch fires (tbz->nop @0x102dae5fc +
#   value-cell [0x106a64d78]=fabricated all-leaf obj), SendAppEventOnAppReady returns
#   Ok(...), 0 SIGSEGV, ladder completes clean (EXIT 124).
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
timeout 120 env $BASE ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
  > runs/sh307-appev-A.txt 2>&1
echo "A(GOVFLAG only) EXIT=$? wall=$(grep -c 'guestpc=0x102bb803c' runs/sh307-appev-A.txt) sigsegv=$(grep -c SIGSEGV runs/sh307-appev-A.txt)"
timeout 120 env $BASE JIT_ROUTEB_PRELOAD_VALUECELL=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
  > runs/sh307-appev-B.txt 2>&1
echo "B(+VALUECELL) EXIT=$? sh307=$(grep -c 'SH307 preload value-branch' runs/sh307-appev-B.txt) sigsegv=$(grep -c SIGSEGV runs/sh307-appev-B.txt) appev=$(grep -c 'SendAppEventOnAppReady returned Ok' runs/sh307-appev-B.txt)"