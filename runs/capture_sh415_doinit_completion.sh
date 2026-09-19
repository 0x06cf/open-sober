#!/bin/bash
# SH415: measure the do-init COMPLETION under the complete ordered session
# substrate (SH400 + all SH409-414 surfaces: onAppReady lifecycle drive, login
# gate, DM binder, nativeAppBridgeAppStart, G3 content surface). The new
# session::probe_doinit_completion now REPORTS the four EXECUTE-DO-INIT-GATES
# live-DM markers right after the two do-init-reaching atoms (StartLuaAppDM
# 0x1023efe2c + V2StartAppWithParams 0x10258b144), so do-init completion is a
# per-atom observable of the runtime, not a one-off probe guess. This run
# answers: has the FULL current substrate moved the engine toward owning a live
# DataModel (vs DM-root stuck at 0 on every SH400-414 run)?
#
# Same env as capture_sh400 (SH397-proven furthest-advancing ladder) + the new
# substrate surfaces that SH409-414 wired into drive_routeb_session_substrate.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh415-doinit-completion.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh415_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the ordered substrate drive (SH400) — per-atom Ok/Err ==="
grep -E "session-drive" "$LOG" | grep -E "\[[0-9]+/16\]|returned Ok|stopped|substrate complete" | tail -20
echo "=== how many atoms returned non-zero Ok === "
grep -aoE "substrate complete: [0-9]+/16 atoms returned non-zero Ok" "$LOG"
echo "=== SH415 EXECUTE-DO-INIT live-DM probe (after StartLuaAppDM + V2StartAppWithParams) ==="
grep -E "EXECUTE-DO-INIT live-DM probe" "$LOG"
echo "=== any probe reporting LIVE DM? ==="
grep -cE "EXECUTE-DO-INIT live-DM probe.*LIVE DM = true" "$LOG"
echo "=== session observables (MH_* / AppBridgeV2) ==="
grep -E "MH_FLAGS_LOADED|MH_ENGINE_INITIALIZED|MH_APP_READY|AppBridgeV2" "$LOG" | tail -6
echo "=== data-persistence / DM markers ==="
grep -oE "DM-root probe|once-slot|make_shared|\[validated\]" "$LOG" | sort | uniq -c
echo "=== terminal / crash ==="
grep -aoE "SIGSEGV|SIGABRT|bad_function_call|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh415_root
exit 0