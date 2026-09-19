#!/bin/bash
# SH366: reproducible artifact — ENTER the guest app-command DISPATCHER process_cmd
# (guest 0x102bcd6e4) to deliver APP_CMD_INIT_WINDOW through the engine's real
# window-attach path (the SESSION-CTOR window/GL-surface precondition the operator
# names for initEngine_). SH39b only region-watched the glue LOOP (0 hits); SH365
# measured the app-command drain dead (guest never entered ALooper). Nobody had EVER
# ENTERED the bounded process_cmd dispatcher (the infinite ALooper loop can't be
# jit_run; this dispatcher CAN). The rung drives it FIRST on the ladder so the
# measurement is captured even if a LATER run-variable persistence-lane wall (the
# pre-existing SH353-class ~2/8 no-full-completion) crashes the run after the drive.
#
# Predicate: process_cmd returns Ok AND the INIT_WINDOW case-body marker [inner+9]==1
# (the case body does `ldr x0,[x20,#64]; strb w8,#1,[x20,#9]; bl window-attach`) — the
# engine's OWN APP_CMD_INIT_WINDOW handler EXECUTED headlessly. Retries (SH352
# precedent) for a confirmed-clean FULL completion: EXIT 124, 0 crash, marker set.
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/home/hermes-worker/runs/sh366-glue-cmd.txt}
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
    SOBER_ANDROID_ROOT=/tmp/sober_sh366_root$i \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
    --v2boot-set-filesdir --v2boot-glue-cmd \
    > "$LOG" 2>&1
  local_exit=$?
  crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  entered=$(grep -acE "INIT_WINDOW case body marker \[inner\+9\]=1" "$LOG")
  drive=$(grep -acE "process_cmd returned Ok" "$LOG")
  echo "attempt $i: exit=$local_exit crash=$crash drive=$drive marker=$entered"
  if [ "$local_exit" -eq 124 ] && [ "$crash" -eq 0 ] && [ "$entered" -ge 1 ]; then confirm=1; break; fi
  rm -rf /tmp/sober_sh366_root$i
done
echo "=== confirmed-green glue-cmd entry: $confirm ==="
echo "=== dispatch + marker (final run) ==="
grep -aE "glue-cmd" "$LOG" | head -6
echo "=== context (DM gate unchanged) ==="
grep -aE "SH155 DM-root probe" "$LOG" | tail -1
rm -rf /tmp/sober_sh366_root* 2>/dev/null
echo "RESULT=confirm:$confirm"
exit 0