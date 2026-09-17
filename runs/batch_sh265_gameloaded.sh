#!/bin/bash
# Batch: run the OnGameLoaded session-drive up to 3x to catch a post-ladder completion
# (the app-start ladder self-terminates at run-variable live-object walls ~run-variably,
# so post-ladder stages fire only when the ladder completes past them, SH55/64/261-style).
set -u
cd "$(dirname "$0")/.."
for i in 1 2 3; do
  LOG=/home/hermes-worker/runs/open-sober/runs/sh265-gameloaded-run$i.txt
  rm -f "$LOG"
  timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 \
    JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x102330000-0x10233a000,0x10233a000-0x102350000 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded \
    > "$LOG" 2>&1
  EXIT=$?
  echo "RUN$i EXIT=$EXIT game_loaded_printed=$(grep -ic 'SendAppEventOnGameLoaded' "$LOG") SH126b=$(grep -ic 'SH126b' "$LOG") bad_fn=$(grep -ic 'bad_function_call' "$LOG")"
done