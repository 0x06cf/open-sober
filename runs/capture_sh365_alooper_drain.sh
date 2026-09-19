#!/bin/bash
# SH365: reproducible artifact for the APP-COMMAND DRAIN measurement.
# SH264/276 drove lifecycle NATIVES directly (initAppShellReporter/setActive/
# nativeInitClientSettings/nativeActivity_onEngineSettingsReceived) and
# *suspected* the android_app glue main loop (guest 0x102bcd5d0) "busy-spins"
# rather than consuming the host app-command FIFO. Nobody pinned it as measured
# 0-consumption on the completing ladder. SH365 adds always-on counters to the
# ALooper shims (ALooper_addFd/ALooper_pollOnce entries + post_app_command), so
# the dead-letter is MEASURED without JIT_TRACE.
#
# Predicate: addfd==pollonce==0 with posted>0 == the guest glue loop never
# reaches the ALooper lifecycle shims -> the window/GL-surface APP_CMD_INIT_WINDOW
# precondition the SESSION-CTOR directive names for initEngine_'s "Engine settings
# is null" hard-assert is never delivered (independent of the direct-native drive).
set -u
cd "$(dirname "$0")/.."
LOG=${1:-/home/hermes-worker/runs/sh365-alooper-drain.txt}
confirm_green=0
for i in $(seq 1 3); do
  rm -f "$LOG"
  timeout 40 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
    JIT_ROUTEB_PRELOAD_VALUECELL=1 \
    SOBER_ANDROID_ROOT=/tmp/sober_sh365_root$i \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-set-filesdir \
    > "$LOG" 2>&1
  local_exit=$?
  crash=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG")
  drain=$(grep -aE "SH365 drain-stats" "$LOG" | tail -1)
  alarms=$(grep -aE "\[elfjit:appcmd\] post|SH365 drain-stats" "$LOG" | wc -l)
  echo "attempt $i: exit=$local_exit crash=$crash appcmds=$alarms"
  echo "  $drain"
  # confirmed-green = clean completing run (exit 124) with the drain-stats line.
  if [ "$local_exit" -eq 124 ] && [ "$crash" -eq 0 ] && [ "$alarms" -ge 4 ]; then confirm_green=1; fi
  rm -rf /tmp/sober_sh365_root$i
done
echo "=== confirmed-green completing-ladder drain measurement: ${confirm_green} ==="
echo "=== final-run app-command post + drain-stats ==="
grep -aE "\[elfjit:appcmd\]|SH365 drain-stats" "$LOG" | tail -6
echo "=== anativewindow (surface) entry (context) ==="
grep -aE "anativewindow\]" "$LOG" | head -2 || echo "(no anativewindow trace)"
echo "=== DM-root probe (Route-B gate context) ==="
grep -aE "SH155 DM-root probe" "$LOG" | tail -1 || echo "(no SH155 probe)"
exit 0