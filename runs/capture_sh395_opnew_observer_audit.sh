#!/bin/bash
# SH395: audit the single forward observer's causal validity on the furthest
# composition. SH394 measured the SH174 DM-capture latch "never even installs"
# (0 'routed capture trail') on the full-table env, but the guard routeb_dm_alloc_capture_guard
# fires ONLY at block-entry pc OP_NEW_WRAPPER (0x102a0d9b8). If the engine's operator-new
# is reached via a `b` tail-jump from a caller (or mid-block), the wrapper is NEVER a block
# entry -> the latch cannot fire -> "0 validated" is an OBSERVER ARTIFACT, not a measured
# "no DM allocation". This cycle region-watches the whole CRT operator-new band
# [0x102a0d940, 0x102a0da00) + the DMCONT operator_new entries (0x101db1a38/0x101d96768/0x101db1c60)
# on the furthest-advancing SH394 env to determine whether operator-new is entered at all,
# and at which block-entry pcs. If op-new fires at pcs != OP_NEW_WRAPPER, that PROVES the
# latch is structurally deaf on this env (and the "no DM" verdict is underdetermined by it).
# READ-ONLY: no production seed, region-watch only. Workspace must be green at start.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh395-opnew-observer-audit.txt
rm -f "$LOG"
timeout 110 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 \
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
  JIT_ROUTEB_DOINIT_EMPTYVEC=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  JIT_ROUTEB_APPEVENT_W19=1 \
  JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1 \
  JIT_DM_ALLOC_CAPTURE=1 \
  JIT_REGION_WATCH=0x102a0d940-0x102a0da00,0x101db1a20-0x101db1cc0,0x101d96768-0x101d96800 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh395_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 \
  --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  --v2boot-glue-cmd-full \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DM capture latch arm (observer install) ==="
grep -E "dm_alloc_capture|capture trail|routed|routeb-dmalloc" "$LOG" | head -6
echo "=== operator-new band region hits (is op-new entered at all?) ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | awk '{print $NF}' | sort | uniq -c | sort -rn | head -40
echo "=== did any hit fall in [0x102a0d940,0x102a0da00) the CRT wrapper band? ==="
grep -oE "region hit at guest pc=0x102a0d[0-9a-f]+" "$LOG" | sort | uniq -c
echo "=== crash / terminal ==="
grep -aoE "SIGSEGV|SIGABRT|guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== SendAppEventOnAppReady + glue-full ==="
grep -cE "glue-full\] SH393 done" "$LOG"
grep -E "SendAppEventOnAppReady returned" "$LOG" | head -2
rm -rf /tmp/sober_sh395_root
exit 0