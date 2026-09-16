#!/usr/bin/env bash
# SH199: cross the V2InitWithParams world-build gate byte [0x106a70568] (env
# JIT_ROUTEB_SETWORLDBUILD) so fn 0x102ea3b14 (operator-new(0x18)+ctor 0x2eacce4+
# nativeAppBridgeAppStart__ 0x2365960) can execute. Region-watch that fn + body.
# NOTE: the V2Init rung may stop at the SH198 run-variable outside-image flake
# before reaching the gate block (pre-existing; baseline env-OFF identical).
# default-inert (JIT_ROUTEB_SETWORLDBUILD=1), single jit_run ladder, EXIT 124 expected.
set -u
LOG="${1:-/tmp/sh199-worldbuild.txt}"
ROOT=/home/hermes-worker/runs/open-sober
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_SETWORLDBUILD=1 \
  JIT_REGION_WATCH="0x102ea3a84-0x102ea3be0" \
  "$ROOT/target/debug/examples/elfjit" \
  /home/hermes-worker/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
echo "exit=$?"
