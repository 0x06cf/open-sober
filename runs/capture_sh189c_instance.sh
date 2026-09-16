#!/bin/bash
# SH189c EXPERIMENTAL: the real PlayerGui instance ctor chain headlessly. This is the Route-B
# frontier probe. With the class-name registry populated (PlayerGui+ScreenGui descriptors) +
# DM planted into the creator's current-DM global 0x107333948, the pair-consumer 0x10255d0e4
# (driven with x0=dm so the instance ctor gets a non-null owner) drives
#   core creator 0x102373458 -> operator-new -> blr 0x255d1b4 (ctor functor) -> PlayerGui ctor
# 0x255d1dc -> instance ctor 0x2374310.
# VERIFIED (2/2 isolated): the chain EXECUTES TO COMPLETION headlessly (DROVE ok, EXIT 124,
# 0 crash). The pre-fix x23-owner null-deref (EXIT 134) is gone since x0=dm feeds the owner.
# HONEST residual: out={0,0} — the instance is allocated (ret x0 host ptr) but not yet surfaced
# through the out-buffer, so self-construction is not yet OBSERVED (next step: walk the
# op-new'd object for the PlayerGui vptr 0x106648950).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh189c-instance.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT (expect 124 = the instance ctor chain DROVE ok to completion; the x23-owner
fault was fixed by passing x0=dm. 134 would mean a fresh next-gate fault. A rare 139 after
'planted DM' is the pre-existing SH55/64 clone-worker flake, NOT this guard.)"
echo "=== SH189c instance drive (the frontier probe) ==="
grep -E "routeb-dmins" "$LOG"