#!/bin/bash
# SH347: drive the messageBus experience-launch RECEIVE half headlessly.
# SH185 closed publish->cb (cb reads [DataModelBindings+16]) by STATIC judgment only;
# SH269/315/337 MEASURED the subscribe half runs Ok(0x3e8). This drives subscribe + THEN
# publishRaw (0x102334684) so the engine's real cb (file 0x2bd7444/0x2bd76e8) fires, and
# the new routeb_busrecv_holder_guard (JIT_ROUTEB_BUSRECV=1) snapshots [DataModelBindings+16]
# at pc 0x102bd7474. Converts SH185's static-only closure into a measured result.
set -u
cd "$(dirname "$0")/.."
LOG=$1; : "${LOG:=/tmp/sh347-recv.txt}"
rm -f "$LOG"
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_BUSRECV=1"
timeout 150 env $BASE \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
  --v2boot-session-bus --v2boot-session-pub >"$LOG" 2>&1
EX=$?
echo "EXIT=$EX"
echo "pub:    $(grep -aoE '\[elfjit:v2boot-pub\] (MessageBus.publishRaw|publishRaw|post-publish)[^\n]*' "$LOG" | tail -3 | tr '\n' ' ')"
echo "busrecv: $(grep -aE 'routeb-busrecv|SH347 cb' "$LOG" | tail -3 | tr '\n' ' ')"
echo "bus:    $(grep -aoE 'MessageBus.subscribe returned Ok\(0x[0-9a-f]+\)|MessageBus.subscribe stopped: [^ ]*' "$LOG" | tail -1)"
echo "postDM: $(grep -aoE 'post-publish DM-root\\[0x106a68818\\]=0x[0-9a-f]+' "$LOG" | tail -1)"
echo "crash:  $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+|SIGSEGV|SIGABRT' "$LOG" | tail -3 | tr '\n' ' ')"
exit 0