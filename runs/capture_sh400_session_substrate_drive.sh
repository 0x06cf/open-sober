#!/bin/bash
# SH400: SEM-casing the substrate DATA into an executable session drive (SEP-18
# BUILD-THE-RUNTIME). SH399 shipped the 16-atom ordered ROUTEB_SESSION_SUBSTRATE
# as data whose only consumer was a hermetic address-pin test; this run drives it
# through the NEW session::drive_routeb_session_substrate library driver
# (--v2boot-session-drive), which walks the ordered table and jit_run's each atom
# with per-atom ABI args, reporting Ok/Err + MH_* after every step.
#
# This is the cause-not-symptom drive the operator's SEP-17 session-ctor directive
# names (drive the engine's REAL Activity/AppBridge lifecycle natives in order),
# now assembled from ONE table instead of scattered --v2boot-* rungs. It does NOT
# manufacture a DM (a live DataModel is still the output of a real do-init that
# only an upstream session constructs) — it IS that drive's ordered host-side
# runtime, testable headlessly.
#
# Measure: how many of the 16 substrate atoms jit_run returns Ok(v!=0) for on the
# real libroblox.so, which atom stops (the first SESSION wall), and the MH_* /
# AppBridgeV2 observables after each. Entry 0x2173ff4 = the ladder entry point
# (same envelope as SH397 capture, which is the furthest-advancing full env).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh400-session-substrate-drive.txt
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
  SOBER_ANDROID_ROOT=/tmp/sober_sh400_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the ordered substrate drive (SH400) — per-atom Ok/Err ==="
grep -E "session-drive" "$LOG" | grep -E "\[[0-9]+/16\]|returned Ok|stopped|substrate complete"
echo "=== how many atoms returned non-zero Ok (the drive's verdict) ==="
grep -aoE "substrate complete: [0-9]+/16 atoms returned non-zero Ok" "$LOG"
echo "=== session observables final (MH_* / AppBridgeV2) ==="
grep -E "MH_FLAGS_LOADED|MH_ENGINE_INITIALIZED|MH_APP_READY|AppBridgeV2|app ready" "$LOG" | tail -6
echo "=== data-persistence / DM markers ==="
grep -oE "DM-root probe|once-slot|make_shared|\[validated\]" "$LOG" | sort | uniq -c
echo "=== terminal / crash ==="
grep -aoE "SIGSEGV|SIGABRT|bad_function_call|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh400_root
exit 0