#!/bin/bash
# SH277: measure the SEP-17 initEngine_/getFlagsFromEngine_ state-dispatch gate at the
# SH275+SH276 feed state (client-settings + engine-settings-signed both latched).
#  (A) CLEAN SESSION PATH (--v2boot-skip-appstart): the do-init app-shell ctor
#      0x102207b50 runs 78 distinct blocks (deeper than SH239's 61), nativeGameGlobalInit
#      returns Ok clean, EXIT 124 — but the initEngine_ dispatch [0x102bd1a30,0x102bd1d08)
#      gets 0 region hits (only the SH276 engine-settings receive 0x102bd1c38 itself fires
#      inside that window). A fabricated manager (state word [this+16]=0) can never enter a
#      settings body — initEngine_ dispatch state==3->0x2bd1d68 / ==5->0x2bd24b4 / ==9->
#      0x2bd2668, else benign tail (mov x0,x19,#0x14; b pthread_mutex_unlock).
#  (B) FULL LADDER (no skip-appstart): with the settings feeds, app-start is
#      DETERMINISTIC 3/3 at the SH268 LSM free-list write-off wall 0x101d9a528 (terminal
#      UNCHANGED — client/engine-settings feeds are consumers, they do NOT shift app-start).
# Doc: docs/frontier-sh277-init-engine-gate-feed-measured.md
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-session-signed --v2boot-session-engine --v2boot-skip-appstart"
# (A) clean session path: app-shell ctor deep reach + initEngine_ dispatch coverage
RW="0x102bd1a30-0x102bd1d08,0x102bd1d68-0x102bd2128,0x102207b50-0x102209000"
timeout 120 env $BASE JIT_REGION_WATCH="$RW" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SL > runs/sh277-clean.txt 2>&1
echo "clean EXIT=$? appshell_pcs=$(grep -aoE 'region hit at guest pc=0x102207[0-9a-f]+' runs/sh277-clean.txt | wc -l) initEngine_dispatch_pcs=$(grep -aoE 'region hit at guest pc=0x102bd1[0-9a-f]+' runs/sh277-clean.txt | grep -ac '0x102bd1a30\|0x102bd1a3c\|0x102bd1d08')"
# (B) full ladder: app-start terminal with settings feeds (deterministic LSM free-list wall)
S="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-session-signed --v2boot-session-engine"
RBASE="$BASE JIT_ROUTEB_APPSART_LSM_NODES=1"
RW2="0x101d9a400-0x101d9a580"
for i in 1 2 3; do
  timeout 120 env $RBASE JIT_REGION_WATCH="$RW2" ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $S > runs/sh277-full-$i.txt 2>&1
  echo "full$i EXIT=$? term=$(grep -aoE 'guestpc=0x101d9a528' runs/sh277-full-$i.txt | head -1)"
done