#!/bin/bash
# SH251b-batch: confirm reproducibility of the NEW downstream Route-B fencepost reached only under
# the class-desc-registered + genuine-DM CONSUMER+DISPATCH path. SH251b run 1 showed PlayerGui+ScreenGui
# class-descriptors register headlessly (SH189 getters DROVE OK, desc counter/vtable-family populated)
# then advanced into a large ctor 0x102b9eca0 (sub sp,#0x480) that SIGSEGVs at guestpc=0x102b9ecbc
# reading [this+16] with this=NULL (fault=0x10). Confirm it's stable (not a flake) across N runs.
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh251b-consumer-dispatch-batch.txt
rm -f "$LOG"
for i in 1 2 3; do
  printf "===== RUN %s =====\n" "$i" >> "$LOG"
  timeout 160 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 \
    JIT_ROUTEB_DM_REALCTOR_CONSUMER=1 JIT_ROUTEB_DM_REALCTOR_DISPATCH=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_DM_SERVICES=1 \
    JIT_REGION_WATCH=0x102dbcc10-0x102dbce90,0x10201fda4-0x102020000,0x102b9ec00-0x102b9ee00 \
    JIT_DUMP_PC=0x102b9ecbc \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    >> "$LOG" 2>&1
  echo "EXIT=$? (run $i)" >> "$LOG"
done
echo "===== AGGREGATE ====="
echo "realctor GENUINE: $(grep -c 'GENUINE MATCH' "$LOG")"
echo "PlayerGui getter DROVE: $(grep -c 'PlayerGui class-register GETTER .* DROVE ok' "$LOG")"
echo "ScreenGui getter DROVE: $(grep -c 'ScreenGui class-register GETTER .* DROVE ok' "$LOG")"
echo "new ctor fencepost crashes (0x102b9ecbc): $(grep -c 'guestpc=0x102b9ecbc' "$LOG")"
echo "ctor region hits: $(grep -oE 'region hit at guest pc=0x102b9ec[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
echo "get-or-create returns: $(grep -oE 'get-or-create consumer .*DROVE ok ret x0=0x[0-9a-f]+' "$LOG" | sort -u)"
echo "deregister/shutdown progress after fencepost: $(grep -cE 'service container|class-desc counter|ScreenGui desc' "$LOG")"
echo "EXITs: $(grep -oE 'EXIT=[0-9]+' "$LOG" | tr '\n' ' ')"
echo "DEFAULT-fence crash sites (other than new one): $(grep -oE 'guestpc=0x102[0-9a-f]+' "$LOG" | grep -vE '0x102b9ecbc' | sort | uniq -c | sort -rn | head -5)"