#!/bin/bash
# SH247: the SH245-active continuation (continueAfterFlagsLoaded_ 0x102bd1d68)
# pervasively NULL-allocs headlessly (bad_alloc from MANY operator_new sites,
# variants 0x1db1a38 + 0x1d96768 returning NULL for size>0xa when
# [0x10727570c].bit0 is clear). SH246 scoped one closure site (boxed 0x28) but the
# wall is pervasive. SH247's lever: make both variants' size-gate `b.ls SMALL`
# (taken ~ size<=0xa) UNCONDITIONAL, so EVERY size walks the working descriptor/
# allocator path (0x1db1ab4/0x1d96824 -> scudo classifier -> real tail 0x1db1c60)
# that demonstrably returns real memory headlessly for tiny sizes. This does NOT
# touch the bit0 flag (whose =1 real-alloc route is a measured 3/3 regression).
# A/B: off (expect bad_alloc, continuation dies) | on (JIT_ROUTEB_OPNEW_SIZE_GATE=1).
with_arm() {
  local ARM="$1"
  local EXTRA=""
  [ "$ARM" = on ] && EXTRA="JIT_ROUTEB_OPNEW_SIZE_GATE=1"
  local LOG=/home/hermes-worker/runs/open-sober/runs/sh247-${ARM}.txt
  rm -f "$LOG"
  timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
    $EXTRA \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    > "$LOG" 2>&1
  local EXIT=$?
  echo "[$ARM] EXIT=$EXIT"
  echo "== SH247 patch line (if on) =="
  grep -E "SH247" "$LOG" | tail -3
  echo "== op_new fast-path reached (0x1021db1a38/..68) AND continuation region 0x102bd1d68 hits =="
  grep -oE "region hit at guest pc=0x102bd[0-9a-f]{4}" "$LOG" | sort -u | tail -20
  echo "== bad_alloc / crash / terminate counts =="
  grep -icE "SIGSEGV|SIGABRT|terminate|bad_alloc|stack smashing|alloc" "$LOG" || true
  echo "== lasterr / last reached continuation pc =="
  grep -oE "pc=0x102bd[0-9a-f]{4}" "$LOG" | sort -u | tail -6
  echo
}
echo "########## OFF (baseline: expect NULL-op_new bad_alloc death) ##########"
with_arm off
echo "########## ON (JIT_ROUTEB_OPNEW_SIZE_GATE=1: route all sizes to descriptor path) ##########"
with_arm on