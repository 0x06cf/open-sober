#!/bin/bash
# SH227 repro: (a) hermetic — closure-build binder-dispatch DECODE CORRECTED:
# the load 0xf9401260 at 0x102206df4 is `ldr x0,[x19,#32]` (imm12=4 * size 8), NOT
# `[x19,#4]` as SH225/226 mislabeled (AArch64 unsigned-imm LDR scales imm12 by the access
# size; SH156's original #32 was correct); (b) measured bypass: on the completing ladder a
# 3-run region-watch proves the dispatch block 0x102206df4..0x102206e24 (incl. br x1) NEVER
# executes — closure-build enters, the `b.ne` thread-match gate (0x206df0, cmp pthread_self
# vs stored main-id) routes to the LocalStorageManager non-match path instead.
# Expect: sh227 ok; region hits at 0x102206db8/0x102206dec/0x102206e34/0x102206e3c but NONE
# at 0x102206df4..0x102206e24; EXIT 124, 0 SIGSEGV/SIGABRT.
set -u
cd "$(dirname "$0")/.."
echo "=== (a) hermetic corrected-decode pin === "
cargo test -p arm64jit --example elfjit sh227 2>/dev/null | tail -3
for i in 1 2 3; do
  LOG=/tmp/sh227-run$i.txt
  rm -f "$LOG"
  timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_REGION_WATCH=0x102206db8-0x102206e40,0x1023efe2c-0x1023f0020 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot \
    --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  echo "run$i EXIT=$?"
done
echo "=== closure-build entry + pre-work (fires) === "
grep -hE "region hit at guest pc=0x102206db8|region hit at guest pc=0x102206dec" /tmp/sh227-run*.txt | sort | uniq -c
echo "=== NON-MATCH path (LocalStorageManager) — where the ladder actually goes === "
grep -hE "region hit at guest pc=0x102206e3" /tmp/sh227-run*.txt | sort | uniq -c
echo "=== binder-dispatch block (MUST be ZERO across all runs = never executed) === "
grep -hE "region hit at guest pc=0x102206df|region hit at guest pc=0x102206e0|region hit at guest pc=0x102206e2" /tmp/sh227-run*.txt | sort | uniq -c
echo "(0 hits = the br x1 DM dispatch bypassed by the b.ne gate)"
echo "=== crash summary across runs (want 0) === "
grep -hcE "SIGSEGV|SIGABRT" /tmp/sh227-run*.txt | tr '\n' ' '; echo