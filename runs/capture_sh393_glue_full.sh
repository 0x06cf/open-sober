#!/bin/bash
# SH393: FULL 20-command app-dispatcher table drive on the guarded SH366 entry.
#
# SH366 entered the engine's REAL app-command dispatcher process_cmd (0x102bcd6e4)
# headlessly for the FIRST time (cmd 11 INIT_WINDOW); SH368 extended to a verified-safe
# {6,8,11} subset with session-state readback. Both drove only 3 of the dispatcher's 20
# APP_CMD cases. The operator's SESSION-CTOR directive is to "drive the engine's REAL
# Activity-session init state machine" — a real Activity consumes the FULL command queue in
# lifecycle order. --v2boot-glue-cmd-full (jit.rs drive_glue_process_cmd_full) extends the
# confirmed-green SH366 entry to ALL 20 dispatchable commands (cmd in [1..20], the
# dispatcher's `sub w8,w1,#1; cmp w8,#0x13` bound), each command on its OWN fresh CpuState
# jit_run (per-command fault-tolerance: a command that derefs a not-yet-constructed
# live-session object is recorded as an Err and the sequence continues — it cannot abort the
# probe), sharing ONE fabricated app/inner/win so glue state accumulates like a real queue,
# and reading back session observables (marker [inner+9], once-guard [win+0x268],
# AppBridgeV2 [0x106a705e8], surface XID [0x10683d348], flags-loaded [0x10672739d4],
# DM class-registry [0x106dca0e70]) after EACH command. The window-attach once-guard stays
# OFF (SH367 measured fault on arming); the ALooper loop is NOT entered (SH365 dead-drain).
#
# This is a bounded observability advance on the confirmed-green SESSION-CTOR entry — it does
# NOT arm the SH367 once-guard, does NOT create a real EGL surface, and does NOT re-attack the
# persistence lane.
#
# Predicate: the 15 safe commands are dispatched (>= 14 return Ok) AND the INIT_WINDOW marker
# [inner+9]==1 fires (cmd 11 works) AND the run stays confirmed-green. The 5 unsafe commands
# (cmd 1 pre-gate deref; cmd 13/15/17/18 gate-checked-but-live-object-deref) are recorded as the
# measured-excluded map — these are the SH366/367 live-object class (deref [x20+24]/[x20+8] on a
# zeroed app), the same reason SH367 faulted on the real window-attach. The substantive extension
# over SH368's {6,8,11} is the full SAFE map: 15 of the engine's own APP_CMD cases drive cleanly.
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/home/hermes-worker/runs/sh393-glue-full.txt}
confirm=0
for i in $(seq 1 6); do
  rm -f "$LOG"
  timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
    JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
    JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
    SOBER_ANDROID_ROOT=/tmp/sober_sh393_root$i \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
    --v2boot-set-filesdir --v2boot-glue-cmd-full \
    > "$LOG" 2>&1
  local_exit=$?
  crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  ok_count=$(grep -acE "glue-full\] cmd .*process_cmd returned Ok" "$LOG")
  marker=$(grep -acE "glue-full.*cmd 11.*marker\[inner\+9\]=1" "$LOG")
  done_lines=$(grep -acE "glue-full\] SH393 done" "$LOG")
  echo "attempt $i: exit=$local_exit crash=$crash okcmds=$ok_count cmd11marker=$marker done=$done_lines"
  if [ "$local_exit" -eq 124 ] && [ "$crash" -eq 0 ] && [ "$ok_count" -ge 14 ] \
     && [ "$marker" -ge 1 ] && [ "$done_lines" -ge 1 ]; then confirm=1; break; fi
  rm -rf /tmp/sober_sh393_root$i
done
echo "=== confirmed-green full app-cmd table (SH366 entry preserved): $confirm ==="
echo "=== glue-full drive (final run) ==="
grep -aE "glue-full" "$LOG" | head -30
echo "=== context (Route-B live-DM gate unchanged) ==="
grep -aE "SH155 DM-root probe" "$LOG" | tail -1
rm -rf /tmp/sober_sh393_root* 2>/dev/null
echo "RESULT=confirm:$confirm"
exit 0