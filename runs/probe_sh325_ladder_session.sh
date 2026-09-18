#!/bin/bash
# Probe SH325: full v2boot ladder + SEP-17 session drive, measure post-run state
# (MH_* milestones, AppBridgeV2 slot, DM-root, terminal pc) to find next unblocked gate.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  V2BOOT_WARMUP_MS=1200 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1 \
  JIT_ROUTEB_SETTINGS_SSO_SEED=1"
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-session-set --v2boot-skip-appstart"
LOG=/tmp/sh325-ladder.txt; rm -f "$LOG"
timeout 200 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "EXIT=$?"
echo "=== first_sigsegv/terminal ==="
grep -oE 'guestpc=0x[0-9a-f]+' "$LOG" | head -1
echo "=== milestone lines ==="
grep -iE "post-lifecycle|post-lifecycle:|MH_FLAGS|MH_APP_READY|MH_ENGINE|milestone|onFlagsLoaded|onEngineInitialized|onAppReady" "$LOG" | head -20
echo "=== AppBridgeV2 slot / gov / DM-root ==="
grep -iE "AppBridgeV2\[|SH158|singleton\[0x106a705e8\]|DM-root|routeb_manufactured|GENUINE DM|0x106a68818" "$LOG" | head -15
echo "=== v2boot rungs ok ==="
grep -iE "returned Ok|driving native|driving.*@ guest" "$LOG" | head -30
echo "=== exit / final ==="
grep -iE "stopped:|stopped |EXIT|signal|fault|SIGSEGV|Aborted|terminated" "$LOG" | head
echo "=== tail ==="
tail -15 "$LOG"