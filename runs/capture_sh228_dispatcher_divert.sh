#!/bin/bash
# SH228 repro: (a) hermetic — engine-init dispatcher sub/continueAfterFlagsLoaded_ anchors
# pinned against the real libroblox.so; (b) measured — on ONE completing --v2boot ladder run
# (DMCONT=1) with 4 region windows, fnB (0x102bd1b98) AND the dispatcher (0x102bd8ce8) BOTH
# fire as block entries, but sub_2bd8dac (reached by the UNCONDITIONAL `bl 0x2bd8d60` inside the
# dispatcher) and continueAfterFlagsLoaded_ (0x102bd1d68, the vt[+0x1f0] target) NEVER fire.
# This closes SH166's open question (a) at block-entry confidence and corrects SH226's
# "benign-completes via 2nd-frame sub" mechanism (sub never enters).
# Expect: sh228 ok; region hits at 0x102bd1b98 + 0x102bd8ce8 ONLY; 0 at 0x102bd8dac +
# 0x102bd1d68; EXIT 124, 0 SIGSEGV/SIGABRT.
set -u
cd "$(dirname "$0")/.."
echo "=== (a) hermetic anchor pin ==="
cargo test -p arm64jit --example elfjit sh228 2>/dev/null | tail -3
LOG=/home/hermes-worker/runs/sh228-dispatcher-divert.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
  JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x102bd1b98-0x102bd1d10,0x102bd8ce8-0x102bd8d68,0x102bd8dac-0x102bd8e30,0x102bd1d68-0x102bd1e10 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
echo "EXIT=$?"
echo "=== fnB + dispatcher entries (both fire) ==="
grep -hE "region hit at guest pc=0x102bd1b98|region hit at guest pc=0x102bd8ce8" "$LOG"
echo "=== sub_2bd8dac + continueAfterFlagsLoaded_ (MUST be ZERO -> never entered) ==="
grep -hE "region hit at guest pc=0x102bd8dac|region hit at guest pc=0x102bd1d68" "$LOG" || echo "(0 hits — block-entry-definitive: dispatcher diverts before the bl sub)"
echo "=== crash summary (want 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smash" "$LOG"