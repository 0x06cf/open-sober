#!/bin/bash
# SH335 — extend the SH334 reglive guard to fire on COUNT TRANSITIONS (not once-per-run): the
# DM-ctor name->service lookup (0x2168798) runs once per service registration on the bus route,
# so SH334's once-fire guard only ever captured the first (empty count=0) readout. This probe
# expects a 0..12 count sequence dump, proving the registry holds ONLY the task-scheduler family
# (Thread/Spawn/Yield/Close/Sleep/Sched/UNKNOWN) and "App" is NEVER a registry entry at any lookup,
# while once-slot transitions 0x0->0x400000b ('Execute', the fast-path's only match) at count=12.
# SH334 measured the MAIN path registry=0 at the lookup; the bus route is the only one that
# populates the registry (12 services, SH315/316/318) — this closes that missing timing-accurate readout.
set -u
cd "$(dirname "$0")/.."
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_REG_LIVE=1"
LOG=/tmp/sh335.txt; rm -f "$LOG"
timeout 150 env $BASE \
  $BIN $SO 0x2173ff4 --jni --startapp 0x258b144 --v2boot --v2boot-session \
  --v2boot-skip-appstart --v2boot-session-bus >"$LOG" 2>&1
EX=$?
echo "EXIT=$EX"
echo "reglive: $(grep -a 'routeb-reglive' "$LOG" | tail -1)"
echo "once:    $(grep -aoE 'once-guard\[0x106a68410\]=0x[0-9a-f]+|once-slot\[0x106a68408\]=0x[0-9a-f]+' "$LOG" | tail -2 | tr '\n' ' ')"
echo "bus:     $(grep -aoE 'MessageBus.subscribe (returned|stopped)[^ ]*' "$LOG" | tail -1)"
echo "crash:   $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+' "$LOG" | tail -3 | tr '\n' ' ')"
exit 0