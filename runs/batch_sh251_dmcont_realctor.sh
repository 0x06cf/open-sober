#!/bin/bash
# SH251 (fresh Route-B re-attack, single-agent): the operator's SEP-15 directive = re-attack the
# live-DM path via the DYNAMIC DM-ctor trace rather than static seed. The routeb_dm_real_ctor_drive_guard
# (SH187) manufactures a GENUINE-vptr DM at StartLuaAppDM entry and plants current-DM holder 0x106391908 —
# but prior DMCONT continuation batches (SH248c..f) ran WITHOUT JIT_ROUTEB_DM_REALCTOR, so app-start's
# live-object map construction always saw holder=0. This batch plants the genuine DM DURING the ladder
# and measures whether app-start (nativeAppBridgeAppStart -> live host-heap map build, SH248g wall 0x1021dde34)
# changes behaviour / advances / reaches the DataModelServices setDataModelToCurrent registry fan-out.
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh251-dmcont-realctor-batch.txt
rm -f "$LOG"
for i in 1 2 3; do
  printf "===== RUN %s =====\n" "$i" >> "$LOG"
  timeout 150 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x102339000-0x102339500,0x1021dde00-0x1021ddf00,0x1022dbcc0-0x1022dbd40 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
    >> "$LOG" 2>&1
  echo "EXIT=$? (run $i)" >> "$LOG"
done
echo "===== AGGREGATE ====="
echo "realctor drive fires: $(grep -c 'GENUINE MATCH' "$LOG")"
echo "holder planted: $(grep -c 'SH187b: planted constructed DM' "$LOG")"
echo "holder value during appstart: $(grep -oE 'holder\[0x106391908\]=0x[0-9a-f]+' "$LOG" | sort | uniq -c)"
echo "map-wall appstart region hits: $(grep -oE 'region hit at guest pc=0x1021dde[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
echo "setDataModelToCurrent region hits (0x2dbcc0): $(grep -c '0x1022dbcc0' "$LOG")"
echo "appstart advance hits: $(grep -oE 'region hit at guest pc=0x1023390[0-9a-f]{2}' "$LOG" | sort -u | tr '\n' ' ')"
echo "EXITs: $(grep -oE 'EXIT=[0-9]+' "$LOG" | tr '\n' ' ')"
echo "crash sites: $(grep -oE 'guestpc=0x102[0-9a-f]+' "$LOG" | grep -vE '0x102bd|0x1023390|0x1021dde' | sort | uniq -c | sort -rn | head -6)"