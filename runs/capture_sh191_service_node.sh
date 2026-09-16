#!/bin/bash
# SH191 (Route-B service-node attach): host-link the engine SELF-CONSTRUCTED PlayerGui
# instance as a real PlayerGui SERVICE NODE on [dm+0x68] ([node+0x18]==0x87e), then drive
# the engine's OWN getService walker 0x105e09bc8 to RESOLVE "PlayerGui" and materialize
# the instance — moving PlayerGui from standalone (SH190e) to service-resolvable under
# the genuine DataModel.
# SUCCESS MARKERS (from routeb-dmsvc lines):
#   SH191: linked PlayerGui service node ... at [dm+0x68]
#   SH191: *** CONFIRMED — engine getService('PlayerGui') RESOLVED the self-constructed
#          PlayerGui service node ... from the genuine DM headlessly ***
#   no SIGSEGV / EXIT 124
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh191-service-node.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 JIT_ROUTEB_DM_INSTANCE_NOP=1 \
  JIT_ROUTEB_DM_SERVICE_NODE=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH191 service-node drive (the frontier probe) ==="
grep -E "routeb-dmsvc SH191|routeb-dmins|SIGSEGV|SIGABRT|stack" "$LOG"