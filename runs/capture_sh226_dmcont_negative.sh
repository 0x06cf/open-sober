#!/bin/bash
# SH226 repro: (a) do-init binder-dispatch chain reconciliation (hermetic sh226 pins the
# full StartLuaAppDM union-build -> dispatcher -> do-init -> closure-build binder dispatch
# against the real libroblox.so), and (b) the DMCONT continuation MEASURED NEGATIVE:
# even with the fabricated manager's vt[+0x1f0] routed to the REAL continueAfterFlagsLoaded_
# (0x102bd1d68), that function is NEVER entered on the completing ladder (the +0xf8/+0x108
# leaf network-fetch returns no flags and the pipeline benign-completes without the
# continuation) -- converting the operator's named "continueAfterFlagsLoaded_ -> app-shell"
# re-attack lever into a measured not-reached-headlessly result, not a guess.
# Expect: (a) sh226 ok; (b) manager-cont guard fires, 0 region hits in 0x102bd1d68-0x102bd2600,
# EXIT 124, 0 SIGSEGV/SIGABRT -- reproducing the negative.
set -u
cd "$(dirname "$0")/.."
echo "=== (a) hermetic reconciliation pin ==="
cargo test -p arm64jit --example elfjit sh226 2>/dev/null | tail -4
LOG=/home/hermes-worker/runs/open-sober/runs/sh226-dmcont-negative.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
  JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
echo "EXIT=$?"
echo "=== manager-cont guard (fires) ==="
grep -E "continuation-routed manager|REAL continueAfterFlagsLoaded_" "$LOG"
echo "=== region-watch hits in continueAfterFlagsLoaded_ (MUST be NONE -> measured negative) ==="
grep -E "region hit at guest pc=0x102bd1d" "$LOG"
echo "(0 hits above = continueAfterFlagsLoaded_ never entered)"
echo "=== crash summary (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smash" "$LOG"