#!/bin/bash
# SH274 A/B repro: the ROUTE-B RECON V3 NEXT-3 do-init seed VALUES
# (JIT_ROUTEB_DOINIT_NEXT3). SH156 only mapped the three ctor-global pages; the
# specific values that clear the do-init/app-shell ctor SEGVs (thread-init
# singleton [0x1067333aa0] -> 0x20 buffer, telemetry once-cell [0x106dcd380] ->
# -1, map-page [0x10673336d8].bit0 -> 1) were never written. Implemented as
# (a) an in-crate block-entry guard (fires in [0x102206c40,0x102213000)) AND
# (b) a direct ladder seed at the SH156 page-map synthesis point (the GOVFLAG
# pattern — the ctor body blocks are JIT-cached by the earlier StartLuaAppDM
# rung so the entry guard alone is partially latent).
# MEASURED: A base = terminal 0x101db1d04 (LSM insert-leaf wall), regionpcs 169;
# B +NEXT3 = all 3 seeds fire headlessly (pc=0x1022076f8), regionpcs 168,
# terminal UNCHANGED at 0x101db1d04. The do-init ctor SEGVs are not the binding
# floor on the full app-start ladder — the standing LSM wall is.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh-doinit-next3-ab.txt
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1"
AFLAGS="--jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent"
timeout 200 env $BASE JIT_REGION_WATCH=0x102206000-0x102214000,0x1021dde34-0x1021df00 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $AFLAGS \
  > $LOG-a.txt 2>&1
echo "A(base) EXIT=$? fired=$(grep -c 'routeb-doinit-next3' $LOG-a.txt) term=$(grep -oE 'guestpc=0x[0-9a-f]+|SIGSEGV' $LOG-a.txt | tail -2 | tr '\n' ' ') regionpcs=$(grep -oE 'region hit at guest pc=0x[0-9a-f]{8}' $LOG-a.txt | sort -u | wc -l)"
timeout 200 env $BASE JIT_ROUTEB_DOINIT_NEXT3=1 JIT_REGION_WATCH=0x102206000-0x102214000,0x1021dde34-0x1021df00 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $AFLAGS \
  > $LOG-b.txt 2>&1
echo "B(+NEXT3) EXIT=$? fired=$(grep -c 'routeb-doinit-next3' $LOG-b.txt) term=$(grep -oE 'guestpc=0x[0-9a-f]+|SIGSEGV' $LOG-b.txt | tail -2 | tr '\n' ' ') regionpcs=$(grep -oE 'region hit at guest pc=0x[0-9a-f]{8}' $LOG-b.txt | sort -u | wc -l)"
echo "== B do-init next3 seed lines =="
grep "routeb-doinit-next3" $LOG-b.txt | head