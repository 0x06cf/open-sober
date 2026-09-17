#!/bin/bash
# SH236: measure StartLuaAppDM's OWN receiveCall-dispatch body on the completing
# --v2boot ladder. JIT_REGION_WATCH = the FULL StartLuaAppDM body [0x1023efe2c,0x1023f0800)
# plus the 9-arg marshaler [0x1023f1210,0x1023f1300). Result (measured, block-entry-definitive):
# StartLuaAppDM enters only 14 distinct blocks, the LAST being 0x1023f01e4, then benign-soft-returns
# Ok — it NEVER enters its own marshaler-call block at 0x1023f075c, so the bl 0x1023f1210 (EC world)
# is never reached. Marshaler region = 0 hits. Success = "driving StartLuaAppDM" +
# "StartLuaAppDM returned Ok(...)" and NO region hit at/after 0x1023f075c (max entered pc <= 0x1023f01e4).
# Pure recon; no production change; single-agent.
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/tmp/sh236-sladm.txt}
rm -f "$LOG"
timeout 110 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  JIT_REGION_WATCH=0x1023efe2c-0x1023f0800,0x1023f1210-0x1023f1300 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  --v2boot-set-filesdir > "$LOG" 2>&1
E=$?
echo "EXIT=$E"
echo "=== StartLuaAppDM body region hits (distinct pcs, sorted) ==="
grep "region-watch" "$LOG" | sed 's/.*pc=0x\([0-9a-f]*\).*/\1/' | sort -u
echo "=== hits in marshaler region [0x1023f1210,0x1023f1300) ==="
grep "region-watch" "$LOG" | grep -cE "pc=0x1023f12[0-9a-f][0-9a-f]"
echo "=== startluaappdm markers ==="
grep -iE "driving StartLuaAppDM|StartLuaAppDM returned" "$LOG"
echo "=== crash? ==="
grep -cE "SIGSEGV|SIGABRT" "$LOG" || true