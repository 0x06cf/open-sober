#!/bin/bash
# SH309 A/B: extend the SH308 measured reach of the SendAppEventOnAppReady pipe —
# the pipe (bl 0x2baeeec) drives do-init 0x2206c40 -> app-shell ctor 0x102207b50 ->
# FMOD audio tail 0x5fb30b4 -> and re-converges on nativeAppBridgeV2StartAppWithParams
# 0x10258b144. This probe watches the FULL StartAppWithParams body + the downstream
# session-ctor bands (post-do-init 0x1023eff4c, governor 0x102e9fa84, EC world,
# StartLuaAppDM, ScriptContext loader 0x101f1d8ac) to find the NEXT live-object gate
# past StartAppWithParams' own `bl 2baeeec` (all were 0 hits before SH308).
# BASIS = SH307-forward env (JIT_ROUTEB_PRELOAD_VALUECELL=1), SendAppEventOnAppReady
# returns Ok, do-init/app-shell-ctor/audio/StartApp reach is the new forward reach.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart"
RW="0x10258b144-0x102590000,0x1023eff4c-0x1023f0800,0x102e9fa84-0x102ea0c40,0x102e24598-0x102e25200,0x101f1d8ac-0x101f1dc00,0x102206c40-0x102209000,0x1022baeeec-0x1022baf300"
timeout 150 env $BASE JIT_REGION_WATCH="$RW" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S --v2boot-send-appevent \
  > runs/sh309-rw.txt 2>&1
echo "EXIT=$?"
echo "appev=$(grep -c 'SendAppEventOnAppReady returned Ok' runs/sh309-rw.txt)"
echo "=== region hits by band ==="
band() { # lo hi name
  local lo=$1 hi=$2 name=$3
  local hits=$(grep -E "region hit at guest pc" runs/sh309-rw.txt | grep -oE "pc=0x[0-9a-f]+" | tr -d 'pc=' | sort -u \
    | while read p; do if [ $((16#$p)) -ge $((16#$lo)) ] && [ $((16#$p)) -lt $((16#$hi)) ]; then echo $p; fi; done)
  echo "$name: hits=$(echo "$hits" | grep -c .) last=$(echo "$hits" | tail -1)"
}
band 0x10258b144 0x102590000 "StartAppV2"
band 0x1023eff4c 0x1023f0800 "post-do-init"
band 0x102e9fa84 0x102ea0c40 "governor"
band 0x102e24598 0x102e25200 "EC-world"
band 0x101f1d8ac 0x101f1dc00 "ScriptContext"
band 0x102206c40 0x102209000 "do-init/ctor"
band 0x1022baeeec 0x1022baf300 "pipe"
echo "=== all distinct region pcs (sorted) ==="
grep -E "region hit at guest pc" runs/sh309-rw.txt | grep -oE "pc=0x[0-9a-f]+" | tr -d 'pc=' | sort -u | tr '\n' ' '; echo
echo "markers: dmroot=$(grep -oE 'DM-root\[0x106a68818\]=0x[0-9a-f]+' runs/sh309-rw.txt | tail -1)"