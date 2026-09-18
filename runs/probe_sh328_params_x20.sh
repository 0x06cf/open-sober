#!/bin/bash
# SH328 repro: cross the x20-params wall at fn 0x25f52b4 (V2StartAppWithParams body).
# Expect: SH328 patch fires; JIT_DUMP_PC=0x1025f5460 HITS (fn completes -> [x19]vt+16 dispatch);
# first_sigsegv ADVANCES from 0x1025f5300 (fault=0x140, SH327 endpoint) to 0x1025f501c (fault=0x0,
# ldr x8,[x0] where x0=[AppStarted+0x408]=0 - a real-AppStarted LIVE MEMBER gate, SESSION-CTOR class).
cd /home/hermes-worker/runs/open-sober
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1
JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1
JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1
JIT_ROUTEB_CONT_APPNAME_SEED=1 JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1
JIT_ROUTEB_APPSART_ADAPTER_SEED=1 JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1
JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 JIT_DUMP_PC=0x102e89150,0x1025f5460"
LOG=/tmp/sh328-probe.txt; rm -f "$LOG"
timeout 200 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "EXIT=$?"
echo "sh328_patch=$(grep -c 'SH328 params-x20' "$LOG")"
echo "0x25f5460_dispatch_hit=$(grep -c 'DUMPPC pc=0x1025f5460\|pc=0x1025f5460' "$LOG")"
echo "first_sigsegv=$(grep -o 'guestpc=0x[0-9a-f]*' "$LOG" | head -1)"
echo "--- gate x19/x20 at fault ---"
grep -A8 'SIGSEGV' "$LOG" | grep -E 'guestpc=|x19=|x20=|x0=' | head -6