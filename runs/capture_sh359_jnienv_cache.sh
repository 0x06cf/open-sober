#!/bin/bash
# SH359: measure the JNI_OnLoad cached-JavaVM seed. JNI_OnLoad+0xc10 (0x2174c04)
# reads cached JavaVM* from [0x107275550] and calls vm->GetEnv (slot 6, v0x10006).
# When that cell is 0, GetEnv is skipped -> env stays NULL -> SH358 faults at
# 0x1021e1c00. Enable JIT_ROUTEB_JNIENV_CACHE so [0x107275550]=fabricated vm and
# watch whether JNI_OnLoad's GetEnv path now runs (VM_GetEnv trace) instead of
# NULL-env faulting. Pure boot (--jni, no ladder) to isolate the mechanism.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh359-jnienv-cache.txt
rm -f "$LOG"
timeout 60 env \
  JIT_ROUTEB_JNIENV_CACHE=1 JIT_TRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== seed fired? ==="
grep -aE "\[elfjit:jnienv\]" "$LOG" | head -3
echo "=== VM_GetEnv traced (env was acquired, not NULL-skipped)? ==="
grep -acE "\[jni\] VM_GetEnv" "$LOG"
echo "=== NULL-JNIEnv fault present? ==="
grep -aiE "guestpc=0x1021e1c00|0x1021e1c00|NULL JNIEnv" "$LOG" | head -5 || echo "(none)"
echo "=== crash signals ==="
grep -acE "SIGSEGV|SIGABRT" "$LOG"
echo "=== last guest pcs ==="
grep -aoE "guestpc=0x[0-9a-f]+|stopped at 0x[0-9a-f]+" "$LOG" | tail -5