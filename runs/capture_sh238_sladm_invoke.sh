#!/bin/bash
# SH238: CROSS StartLuaAppDM's receiveCall dispatch-select to the EC INVOKE slot
# (JIT_ROUTEB_SLADM_INVOKE=1) and MEASURE whether StartLuaAppDM advances past its
# benign soft-return into the marshaler-call block 0x1023f075c (bl -> EC world
# 0x102e24598 via the 9-arg marshaler 0x1023f1210) or faults at a live-object deref.
# Region-watch: StartLuaAppDM body [0x1023efe2c,0x1023f0800), marshaler [0x1023f1210,
# 0x1023f1300), EC world [0x102e1c650,0x102e25200). BASELINE (lever off) = flow stops
# at 0x1023f01e4, marshaler 0 hits, EC world 0 hits (SH236). Success (forward) = the
# early dispatch crossed -> [routeb-sladm] CROSSED line present AND a marshaler [pc
# 0x1023f12xx] or EC-world region hit. Fault = GSDSP pin on the specific live-object deref.
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/tmp/sh238-sladm.txt}
LEVER=${JIT_ROUTEB_SLADM_INVOKE:-1}
rm -f "$LOG"
timeout 110 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  JIT_ROUTEB_SLADM_INVOKE="$LEVER" \
  JIT_GUEST_STACK_DUMP=1 \
  JIT_REGION_WATCH=0x1023efe2c-0x1023f0800,0x1023f1210-0x1023f1300,0x102e1c650-0x102e25200 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  --v2boot-set-filesdir > "$LOG" 2>&1
E=$?
echo "EXIT=$E"
echo "=== SLADM cross marker ==="
grep -E "routeb-sladm" "$LOG" || echo "(no [routeb-sladm] cross fired)"
echo "=== max StartLuaAppDM body pc entered (distinct, sorted asc) ==="
grep "region-watch" "$LOG" | sed 's/.*pc=0x\([0-9a-f]*\).*/\1/' | sort -u
echo "=== marshaler region [0x1023f1210,0x1023f1300) hits ==="
grep "region-watch" "$LOG" | grep -cE "pc=0x1023f12[0-9a-f][0-9a-f]"
echo "=== EC world [0x102e1c650,0x102e25200) hits ==="
grep "region-watch" "$LOG" | grep -cE "pc=0x102e1[c-f][0-9a-f]"
echo "=== startluaappdm markers ==="
grep -iE "driving StartLuaAppDM|StartLuaAppDM returned" "$LOG"
echo "=== fault (GSDSP) ==="
grep -iE "SIGSEGV|SIGABRT|GSDSP|guestpc" "$LOG" | head -20