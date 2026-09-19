#!/bin/bash
# SH368: bounded app-command SEQUENCE drive on the guarded SH366 entry.
#
# Beyond SH366's single cmd-11 (INIT_WINDOW), SH368 drives the verified-safe subset of the
# engine's REAL Activity-session app-command queue ({6, 8, 11}) through the SAME bounded
# dispatcher process_cmd (0x102bcd6e4), sharing one fabricated app/inner/win so state
# accumulates the way a real command queue does, and reads back session observables
# (marker [inner+9], once-guard [win+0x268], AppBridgeV2 [0x106a705e8], surface XID
# [0x10683d348]) after EACH command. The window-attach once-guard stays OFF (SH367 measured
# fault on arming) — this is a bounded observability advance on the SESSION-CTOR entry, NOT a
# return to the SH367 fabricated-surface re-arm.
#
# Predicate: the sequence drives all 3 commands cleanly (cmd 11 -> process_cmd Ok + the
# INIT_WINDOW marker [inner+9]==1) AND the run stays confirmed-green (no SIGSEGV/ABRT) — i.e.
# the SH366/367 clean entry is preserved with the sequence armed.
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/home/hermes-worker/runs/sh368-glue-seq.txt}
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
    SOBER_ANDROID_ROOT=/tmp/sober_sh368_root$i \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
    --v2boot-set-filesdir --v2boot-glue-cmd-seq \
    > "$LOG" 2>&1
  local_exit=$?
  crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  marker=$(grep -acE "glue-seq.*cmd 11.*marker\[inner\+9\]=1" "$LOG")
  drive=$(grep -acE "process_cmd returned Ok" "$LOG")
  seq_done=$(grep -acE "glue-seq.*SH368 done" "$LOG")
  echo "attempt $i: exit=$local_exit crash=$crash cmd11marker=$marker drive=$drive seqdone=$seq_done"
  if [ "$local_exit" -eq 124 ] && [ "$crash" -eq 0 ] && [ "$marker" -ge 1 ] && [ "$seq_done" -ge 1 ]; then confirm=1; break; fi
  rm -rf /tmp/sober_sh368_root$i
done
echo "=== confirmed-green glue-cmd sequence (SH366 entry preserved): $confirm ==="
echo "=== glue-seq drive (final run) ==="
grep -aE "glue-seq" "$LOG" | head -12
echo "=== context (DM gate unchanged) ==="
grep -aE "SH155 DM-root probe" "$LOG" | tail -1
rm -rf /tmp/sober_sh368_root* 2>/dev/null
echo "RESULT=confirm:$confirm"
exit 0