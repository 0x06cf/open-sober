#!/bin/bash
# SH258 fresh measurement: with the FULL seed set (SH248c-f + SH245 + SH257), do-init
# completes to governor; where does the continuation go after govtail -> app-start
# at THIS HEAD? Watch the post-governor region + map wall + app-start entry + the
# stride-0x2a0 live-array allocator clusters (SH254: 0 hits) to see the exact
# current termination point and whether any previously-zero region now fires.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh258-postgovtail.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_REGION_WATCH=0x102e9fa80-0x102ea3b40,0x1021ddc40-0x1021df00,0x1021f47f0-0x1021f4850,0x102e9fdc8-0x102ea3b40,0x101df48c0-0x101df4a00,0x102330000-0x10233a000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== all region hits (post-governor, map, jar, live-array, app-start) =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | head -60
echo "== distinct region pcs count =="
grep -oE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | wc -l
echo "== signals/crash =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash|guestpc=" "$LOG" || true
echo "== last guest pc lines =="
grep -oE "guestpc=0x[0-9a-f]+|terminated.*|last guest.*" "$LOG" | tail -6
echo "== ladder flow =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)|driving [a-zA-Z]+|nativeAppBridge[A-Za-z]+" "$LOG" | tail -8