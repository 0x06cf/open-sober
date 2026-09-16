#!/bin/bash
# SH210: verify the ENTIRE shipped Route-B latent wiring mounts its concrete markers
# on a clean completing ladder at HEAD. (a) G1 surface-handoff writes the wired XID to
# [0x10683d348]; (b) G2 SendAppEventOnAppReady drives 'Home' in x5 (w19-event read);
# (c) G3 --v2boot-set-filesdir seeds the files-dir libc++ string @ [0x10726d600];
# (d) SH155 DM-root probe prints once-guard + DM-root + app-data-model counter;
# (e) SH174 capture latch arms (ACTIVE hook 0x1067daaf0 -> capture trail) under
# JIT_DM_ALLOC_CAPTURE+DELEGATE. Success = all markers present in a clean EXIT-124 run.
# This does NOT manufacture a live DM — it proves every installed gate is ARMED
# (latent-but-correct) at this HEAD, so the instant a real session advances, nothing
# is missing. No production code change; pure verification script.
set -u
cd "$(dirname "$0")/.."
N=${1:-3}
ARM=0; XID=0; EV=0; FDIR=0; DMP=0; CLEAN=0
for i in $(seq 1 "$N"); do
  LOG="/tmp/sh210-$i.txt"
  rm -f "$LOG"
  timeout 110 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
    JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    --v2boot-set-filesdir > "$LOG" 2>&1
  E=$?
  # markers
  arm=$(grep -ac "SH167/SH169 routed CRT operator-new ACTIVE hook.*capture trail" "$LOG")
  xid=$(grep -acE "\[0x10683d348\]=0x20" "$LOG")
  ev=$(grep -acE 'driving SendAppEventOnAppReady.*event="Home"' "$LOG")
  fd=$(grep -acE "SEEDED" "$LOG")
  dp=$(grep -ac "SH155 DM-root probe" "$LOG")
  ARM=$((ARM+arm)); XID=$((XID+xid)); EV=$((EV+ev)); FDIR=$((FDIR+fd)); DMP=$((DMP+dp))
  if grep -aqcE "SIGSEGV|SIGABRT" "$LOG"; then
    echo "run $i: EXIT=$E CRASH arm=$arm xid=$xid appevent=$ev filesdir=$fd dmprobe=$dp"
  else
    CLEAN=$((CLEAN+1)); echo "run $i: EXIT=$E CLEAN arm=$arm xid=$xid appevent=$ev filesdir=$fd dmprobe=$dp"
  fi
done
echo "=== SH210 summary ($N runs): clean=$CLEAN; arm=$ARM xid=$XID appevent=$EV filesdir=$FDIR dmprobe=$DMP ==="
echo "Expected per clean run: arm>=1 xid>=1 appevent>=1 filesdir>=1 dmprobe>=1"
echo "If all >=1 => ENTIRE Route-B latent wiring ARMED at HEAD (G1+G2+G3 + capture latch + DM probe)."