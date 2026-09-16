#!/usr/bin/env bash
# SH201: precise v2 singleton-dispatch family scanner — a characterization
# lever. Verifies the hermetic scanner finds the 384-site objB-getter family on
# the real image, and documents the empirical negative: blindly patching the
# family (JIT_ROUTEB_V2FAMILY) crash-loops the run (over-patch, SH200's warned
# class). The runtime family patch is REVERTED; only the scanner + tests ship.
set -u
ROOT=/home/hermes-worker/runs/open-sober
cd "$ROOT"
echo "=== hermetic scanner tests ==="
cargo test -p arm64jit --example elfjit sh201 2>&1 | grep -E "test result|sh201"
echo "=== baseline ladder (SH200 only) must be stable: EXIT 124, 0 crash ==="
LOG="${1:-/tmp/sh201-baseline.txt}"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_SETWORLDBUILD=1 \
  "$ROOT/target/debug/examples/elfjit" \
  /home/hermes-worker/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
echo "exit=$?"
echo "SH200 patched: $(grep -c 'SH200 patched' "$LOG")  rungs Ok: $(grep -c 'returned Ok' "$LOG")  SIGSEGV/ABRT: $(grep -cE 'SIGSEGV|SIGABRT' "$LOG")"