#!/bin/bash
# SH207 (Route-B): measure the THREE class-name registry maps headlessly — dest resolver
# (0x106dca0e70), bulk-registrar SOURCE (0x106dca0e90), per-class register (0x106dca0f60).
# The bulk registrar 0x2208ae8 (in nativeGameGlobalInit) copies entries FROM source INTO
# dest; when BOTH are empty (headless) it cleanly no-ops and the getService walker returns
# not-found. Prior cycles (SH191/194) read only dest+register, never source — this is the
# first direct source read. Marker (routeb-dmsvc SH192+ line) shows all three EMPTY.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh207-resolver-source.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 JIT_ROUTEB_DM_SERVICE_NODE=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT (expect 124; a rare 134/139 after 'planted DM' is the pre-existing SH55/64 clone-worker flake)"
echo "=== SH207 three-map measurement ==="
grep -E "SH192\+|SH191:|SIGSEGV|SIGABRT" "$LOG" | head -20