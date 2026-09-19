#!/bin/bash
# SH364: reproducible artifact for the REAL-STRING messageBus receive probe +
# the cb-body-entry measurement. SH347 measured the receive cb's DM-holder read
# (0x102bd7474) NEVER fires after an EMPTY-payload publish — but the empty payload
# was the only variant tried, and only the holder READ was measured. SH364:
#   (a) adds a cb-body-entry guard (file 0x2bd744c, sub sp,#128 prologue, BEFORE the
#       DM-holder read) that distinguishes "cb never entered" (publish-side topic-match
#       failure) from "cb entered, DM-holder null" (live-DM-side gate); READ-ONLY.
#   (b) drives publishRaw with a REAL structured payload (SH347 used b"").
# Same runbook as capture_sh347_busrecv.sh + --v2boot-skip-appstart so the ladder
# completes to the post-ladder session rungs. JIT_ROUTEB_BUSRECV=1 arms both guards.
set -u
cd "$(dirname "$0")/.."
LOG=$1; : "${LOG:=/tmp/sh364-recv-real.txt}"
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_BUSRECV=1 \
  JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1"
timeout 150 env $BASE \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
  --v2boot-session-bus --v2boot-session-pub-real >"$LOG" 2>&1
EX=$?
echo "EXIT=$EX"
echo "pub-real: $(grep -aoE '\[elfjit:v2boot-pub-real\] [^\\n]*' "$LOG" | tail -4 | tr '\n' ' ')"
echo "cbentry: $(grep -aE 'SH364 .*cb BODY ENTERED|cbentry' "$LOG" | tail -2 | tr '\n' ' ')"
echo "busrecv: $(grep -aE 'routeb-busrecv|SH347 cb' "$LOG" | tail -3 | tr '\n' ' ')"
echo "bus:    $(grep -aoE 'MessageBus.subscribe returned Ok\(0x[0-9a-f]+\)|MessageBus.subscribe stopped: [^ ]*' "$LOG" | tail -1)"
echo "postDM: $(grep -aoE 'post-publish DM-root\\[0x106a68818\\]=0x[0-9a-f]+' "$LOG" | tail -1)"
echo "crash:  $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT' "$LOG" | tail -3 | tr '\n' ' ')"
exit 0