#!/bin/bash
# SH219: post-family fault NON-reproduction harness (stale-verdict correction).
# SH203 measured the 'post-family gate' (NULL-singleton pthread_mutex_lock, host-call
# slot 0x7f00000022b0, lr=0x102b53a78, x0=0x28) at 4/12 pre-SH116b/sh217. This script
# re-measures at the CURRENT HEAD under SH203's EXACT env (timeout 110, no GSDSP).
# RESULT as measured 2026-09-16: 10 clean / 2 faults at ONLY the known non-seedable
# classes (0x106240c24 FMOD crash-A, 0x1021dea94 SH208 singleton-vtable) — NEVER the
# post-family site. Combined with a 16-run GSDSP batch = 0/28 post-family reproductions.
# Parses each run for a post-family hit (the 0x102b53a78 lr marker OR host-slot
# 0x7f00000022b0) vs any-other-fault vs clean.
set -u
cd "$(dirname "$0")/.."
N=${1:-12}
clean=0; other=0; postfamily=0
for i in $(seq 1 "$N"); do
  LOG=/tmp/sh219-postfamily-$i.txt
  rm -f "$LOG"
  timeout 110 env \
    JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_SETWORLDBUILD=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  if grep -qaE "102b53a78|0x7f00000022b0" "$LOG"; then
    postfamily=$((postfamily+1)); echo "run $i: POST-FAMILY HIT"
  elif grep -qaE "SIGSEGV|SIGABRT" "$LOG"; then
    other=$((other+1)); echo "run $i: other fault $(grep -aE 'guestpc=' "$LOG" | head -1)"
  else
    clean=$((clean+1)); echo "run $i: clean"
  fi
done
echo "=== SH219 result: $clean clean / $other other-fault / $postfamily POST-FAMILY / $N total ==="
echo "(SH203 baseline was 4/12 post-family pre-SH116b/sh217; 0 post-family with \$N runs = stale verdict)"