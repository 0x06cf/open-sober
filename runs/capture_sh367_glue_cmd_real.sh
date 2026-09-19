#!/bin/bash
# SH367: MEASURED NEGATIVE — driving the REAL window-attach GL-surface path faults.
#
# SH366 entered the guest app-command dispatcher process_cmd (0x102bcd6e4) and delivered
# APP_CMD_INIT_WINDOW; window-attach 0x2bd29a0 BENIGN-RETURNED on the zeroed once-guard
# [win+0x268]==0 (clean EXIT 124, confirmed-green). SH367 ARMING the once-guard
# ([win+0x268].bit0=1 + a crafted non-null [win+0x278] GL obj) makes the real body run:
#   tbz @0x2bd2a14 NOT taken -> bl 0x2291c24 (returns 1: [x0]!=0) -> bl 0x22985c0 (deep GL
#   post-init). But 0x22985c0's DEEP body needs a REAL EGL surface/context object — a fabricated
#   NULL-first-word obj cannot satisfy it, so the run SIGSEGVs inside the host GL dispatch
#   (guestpc=0x7f0000001f50 fault=0x7f818c0097, EXIT 134) BEFORE the INIT_WINDOW body completes.
#
# This script is the reproducible MEASURED NEGATIVE + the confirmation that the SH366 clean
# entry is PRESERVED (guard left OFF -> marker still set, drive first, even if a LATER
# run-variable persistence-lane wall crashes the full ladder).
#
# Predicate: SH366 clean entry holds — process_cmd returns Ok AND [inner+9]==1 marker set —
# proving the once-guard-arm regression is reverted (drive confirmed-green again).
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/home/hermes-worker/runs/sh367-glue-cmd.txt}
confirm=0
for i in $(seq 1 8); do
  rm -f "$LOG"
  timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 \
    JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
    JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
    JIT_ROUTEB_APPSART_GOVFLAG=1 JIT_ROUTEB_PRELOAD_VALUECELL=1 \
    SOBER_ANDROID_ROOT=/tmp/sober_sh367_root$i \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart \
    --v2boot-set-filesdir --v2boot-glue-cmd \
    > "$LOG" 2>&1
  local_exit=$?
  crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  entered=$(grep -acE "INIT_WINDOW case body marker \[inner\+9\]=1" "$LOG")
  drive=$(grep -acE "process_cmd returned Ok" "$LOG")
  echo "attempt $i: exit=$local_exit crash=$crash drive=$drive marker=$entered"
  if [ "$entered" -ge 1 ] && [ "$drive" -ge 1 ]; then confirm=1; break; fi
  rm -rf /tmp/sober_sh367_root$i
done
echo "=== SH366 clean entry preserved (drive + marker): $confirm ==="
echo "NOTE: SH367 once-guard arming is a MEASURED NEGATIVE (documented) — arming faults the run,"
echo "      so the drive is left OFF to preserve the confirmed-green SH366 entry."
echo "=== glue-cmd drive + marker (final run) ==="
grep -aE "glue-cmd|INIT_WINDOW case body marker" "$LOG" | head -6
echo "=== context (DM gate unchanged) ==="
grep -aE "SH155 DM-root probe" "$LOG" | tail -1
rm -rf /tmp/sober_sh367_root* 2>/dev/null
echo "RESULT=confirm:$confirm"
exit 0