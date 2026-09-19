#!/bin/bash
# SH402b: NEVER-RUN composition — the SH400 SESSION-CTOR advance (ordered session
# substrate -> genuine AppBridgeV2 vt 0x1063a3410 -> real governor -> StartAppWithParams)
# combined with the OLDER rungs that SH371 measured FIRING DMCONT continueAfterFlagsLoaded_
# DEEP headlessly (--v2boot-session + --v2boot-surface-handoff + --v2boot-send-appevent
# + --v2boot-send-game-loaded + --v2boot-session-bus). SH371 fired DMCONT 25+ blocks on
# those rungs; SH400/401 reached the governor through the genuine singleton. Never co-run.
# Question: does the genuine-single drive, once the session/brisk rungs also run, advance
# StartAppWithParams's body past its 2-block soft-return into DMCONT?
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh402b-combined.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_REGION_WATCH=10258c6e4-10258d100,102bd1d68-102bd2600,102e9fa84-102e9fea4,102bd8ce8-102bd8e30 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh402b_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== StartAppWithParams body block-entry pcs ==="
grep -oE "pc=0x10258[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn
echo "=== DMCONT (0x102bd1d68) + dispatcher (0x102bd8ce8) + sub (0x102bd8dac) ==="
for pc in 0x102bd1d68 0x102bd8ce8 0x102bd8dac 0x102bd2600; do
  echo "  $pc hits: $(grep -acE "pc=$pc " "$LOG")"
done
echo "=== governor / do-init ==="
for pc in 0x102e9fa84 0x10258c7b4 0x1023eff4c; do echo "  $pc hits: $(grep -acE "pc=$pc " "$LOG")"; done
echo "=== session-drive + lifecycle observables ==="
grep -aE "substrate complete|post-lifecycle|post: MH_|AppBridgeV2\[" "$LOG" | tail -6
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
grep -aoE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh402b_root
exit 0