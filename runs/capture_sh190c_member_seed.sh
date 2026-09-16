#!/bin/bash
# SH190c (Route-B PlayerGui member-seed): the real PlayerGui derive body headlessly, with the
# call-site NOP + string-member zeroing. This is the next gate past SH190c's "derive body faults
# at the post-write member" — seeding [obj+0x40,obj+0x100) as valid EMPTY SSO so the derive's
# copy-assign into obj+0x60/0x70 doesn't deref mempool-garbage long-form pointers.
# OPT-IN DIAGNOSTIC (crashes EXIT 134 at the next unseeded member by design — not default).
# SUCCESS MARKERS (from routeb-dmins lines):
#   obj vptr = 0x106648950  => full PlayerGui self-construct (observe [obj+0]=0x106648950)
#   no SIGSEGV / EXIT 124   => derive body ran to completion headlessly.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh190c-member-seed.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 JIT_ROUTEB_DM_INSTANCE_NOP=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT (124 = derive body no-segfault / full run; 134 = next unseeded member gate; 139 = SH55/64 clone-worker flake)"
echo "=== SH190c member-seed drive (the frontier probe) ==="
grep -E "routeb-dmins|SIGSEGV|SIGABRT" "$LOG"