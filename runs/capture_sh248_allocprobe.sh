#!/bin/bash
# SH248: measure the real allocator tail (0x623fe1c) size-class free-list state
# headlessly. Route B is allocator-gated: sizes <=0xa succeed but the DMCONT
# continuation / StartLuaAppDM's own construction / op_new all return NULL for
# sizes 0x28/0x20 when [0x10727570c].bit0 is clear (headlessly clear). SH247 proved
# the working descriptor path cannot serve >0xa (3 mechanisms). SH248 measures WHY
# at the allocator entry: base(x0)/size(x1)/align(x2) + the size-class node
# (base+round8(size)+232) and its free-list head [+8]/count[+16] — direct evidence
# of whether small classes are pre-bootstrapped (real free-lists) vs 0x28 empty
# ("size-class region not set up"), i.e. whether a free-list seed is feasible.
#
# Uses the SH247 size-gate patch + SH245 continuation-active env so >=0x28 requests
# actually reach the allocator tail, plus JIT_ROUTEB_ALLOC_PROBE=1 to log.
LOG=/home/hermes-worker/runs/open-sober/runs/sh248-allocprobe.txt
rm -f "$LOG"
timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 \
  JIT_ROUTEB_OPNEW_SIZE_GATE=1 JIT_ROUTEB_ALLOC_PROBE=1 \
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== allocprobe summary (distinct size->base/freehead classes) =="
grep -E "\[allocprobe\]" "$LOG" | sed -E 's/pc=0x[0-9a-f]+ //' | sort | uniq -c | sort -rn | head -40
echo "== allocprobe lines matching size>=0x18 (the failing class zone) =="
grep -E "\[allocprobe\]" "$LOG" | grep -vE "size=0x[0-8] " | head -30
echo "== continuation region hits =="
grep -oE "region hit at guest pc=0x102bd[0-9a-f]{4}" "$LOG" | sort -u | tail -10
echo "== crash/terminate/exception =="
grep -icE "SIGSEGV|SIGABRT|terminate|bad_alloc|alloc" "$LOG" || true