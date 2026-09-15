#!/bin/bash
# SH164-diagnostic: does the current ladder reach the DM-creator path at all?
# Watch the recon-pinned DM regions to see which are entered on a real run:
#   0x102bd1a38  getFlagsFromEngine_ (async CDN flag-fetch gate)
#   0x102bd1cf0  NativeDataModelManager::initEngine_ (first-login DM creator)
#   0x102dbcc1c  setDataModelToCurrent body (registry dispatch)
#   0x102d87b18  DataModelPatcher::apply gate ([sp+160]=live DM)
# Uses the known-good SH130 combined ladder recipe + region/dump watches.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh164-dm-path.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 \
  RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x102bd1a38-0x102bd1d00 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles \
  --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== DM-region watches (any entry = engine reaches DM path) ==="
grep -E "region-watch\].*(102bd1|g)" "$LOG" | sort -u | head -20
echo "=== ladder ==="
grep -E "ladder done|joined cleanly" "$LOG" | head -3
echo "=== frames ==="
grep -cE "taskv4-frame\] present #" "$LOG"
echo "=== crashes ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG"