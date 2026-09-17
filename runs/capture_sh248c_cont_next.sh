#!/bin/bash
# SH248c: re-run the DMCONT continuation after the SH248b M+0x48 cap fix.
# Goal: observe the continuation's NEXT fencepost past the -9 bad_alloc at 0x2b50600.
LOG=/home/hermes-worker/runs/open-sober/runs/sh248c-cont-next.txt
rm -f "$LOG"
timeout 150 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x1022338ef4-0x102233a80 JIT_GUEST_STACK_DUMP=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== routeb/M48/seed lines =="
grep -E "routeb|M\+0x48|SH245|SH248|bad_alloc|terminate" "$LOG" | head -30
echo "== continuation region hits =="
grep -oE "region hit at guest pc=0x102bd[0-9a-f]{4}" "$LOG" | sort -u | tail -30
echo "== -9 / bad_alloc / SIGSEGV / SIGABRT / EXIT ==="
grep -inE "bad_alloc|SIGSEGV|SIGABRT|terminate|exception|0x2b506" "$LOG" | head -20