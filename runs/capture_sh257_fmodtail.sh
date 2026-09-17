#!/bin/bash
# SH257 probe: does the app-shell ctor's trailing FMOD/audio iterate 0x5fb30b4
# currently fire on the DMCONT ladder, and does its early-return path
# (0x5fb3134 canary-check -> ret) ever execute so the init body COMPLETES
# instead of being absorbed by the audio tail?
#
# SH239 measured the app-shell/global-init ctor 0x102207b50 body runs 61 blocks
# to 0x102208eac then `b 0x5fb30b4` at 0x102208ebc (tail-call into the FMOD
# iterate). The iterate has an empty-container early-return:
#   0x5fb30d8 ldp x8,x9,[x0,#8]   ; [x0+8]=begin [x0+16]=end
#   0x5fb30dc cmp x8,x9
#   0x5fb30e0 b.eq 0x5fb3134      ; begin==end -> skip body
#   ...audio init body...
#   0x5fb3134 canary-check -> ret -> returns to global-init CALLER (completion)
# So [x0+8]==[x0+16] at entry = the tail becomes a no-op and the init body
# RETURNS (goes from "absorbed by sound tail" to "constructs -> returns").
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh257-fmodtail.txt
rm -f "$LOG"
timeout 130 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_REGION_WATCH=0x102208e80-0x102208ec0,0x105fb30b4-0x105fb3160,0x102206c40-0x102207000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== do-init body + FMOD tail region hits =="
grep -oE "region hit at guest pc=0x10220[0-9a-f]{2,}|region hit at guest pc=0x105fb30[0-9a-f]{2}" "$LOG" | grep -oE 'pc=0x[0-9a-f]+' | sort -u
echo "== FMOD iterate early-return reached? (0x5fb3134/0x5fb3154 ret) =="
grep -cE "region hit at guest pc=0x105fb3134" "$LOG"
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash" "$LOG" || true
echo "== StartLuaAppDM / ladder flow =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)" "$LOG" | tail -1