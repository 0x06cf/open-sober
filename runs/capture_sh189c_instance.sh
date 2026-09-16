#!/bin/bash
# SH189c/SH190 EXPERIMENTAL: the real PlayerGui instance ctor chain headlessly. This is the Route-B
# frontier probe. With the class-name registry populated (PlayerGui+ScreenGui descriptors) +
# DM planted into the creator's current-DM global 0x107333948, the pair-consumer 0x10255d0e4
# (driven with x0=dm so the instance ctor gets a non-null owner) drives
#   core creator 0x102373458 -> operator-new -> blr 0x255d1b4 (ctor functor) -> PlayerGui ctor
# 0x255d1dc -> instance ctor 0x2374310.
# VERIFIED (3/3 isolated): the chain EXECUTES TO COMPLETION headlessly (DROVE ok, EXIT 124,
# 0 crash), AND the constructed instance object is now OBSERVED vía the ctor-entry capture
# (routeb_dm_instance_ctor_capture at pc 0x10255d1dc): obj vptr = 0x106796dc0 = the genuine
# instance-ctor 0x2374310 vtable (in-image), layout readable. This is SH190's observed-instance
# milestone — the ret/out initial walk pointed at a string (red herring); the authoritative read
# is the ctor-entry object capture. The derived PlayerGui-class vptr 0x106648950 is NOT yet
# applied headlessly (the sub-init mounts the instance base; the derived write 0x255d21c doesn't
# land) = the next structural step (instance-base is self-constructed; full PlayerGui incl.
# service-node attach + scene-scan on dm+0x68 remains the outer fence).
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