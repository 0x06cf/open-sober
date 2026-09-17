#!/bin/bash
# SH282: re-measure the engine5 rung's real terminal (SH281's "box +0xaa0 junk park" premise
# refuted). Region-watch the GlobalInit-reentry callee 0x275a23c, its intermediate bl 0x2206f04,
# and the continuation 0x2207118 + the enqueue construct 0x22071ac. Expect: the engine5 rung
# reaches the callee + intermediate, then EXIT 124 (clean timeout park) — not a SIGSEGV. The lock
# at 0x2207118 targets FIXED global 0x106863aa0 (static-init .bss, cannot block); the real park is
# the enqueue-construct on fixed object 0x106863a70 = SH174/204 live-object class.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SLBASE="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3"
RW="JIT_REGION_WATCH=0x102207118-0x102207200,0x10275a23c-0x10275a298,0x102206f04-0x102206fac,0x10221674c8-0x1022167c0"
timeout 40 env $BASE $RW ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SLBASE --v2boot-session-engine5 \
  > runs/sh281-bisect.txt 2>&1
echo "EXIT=$?"
echo "reentry-callee=$(grep -acE 'region hit at guest pc=0x10275a23c' runs/sh281-bisect.txt)"
echo "intermediate-f04=$(grep -acE 'region hit at guest pc=0x102206f04' runs/sh281-bisect.txt)"
echo "continuation=$(grep -acE 'region hit at guest pc=0x102207118' runs/sh281-bisect.txt)"
grep -aoE 'SIG[A-Z]+|guestpc=0x[0-9a-fx]+' runs/sh281-bisect.txt | sort -u | head