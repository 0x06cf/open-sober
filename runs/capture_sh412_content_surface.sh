#!/bin/bash
# SH412: the G3 content surface as a first-class driven substrate step. The SH400
# ordered substrate (--v2boot-session-drive) formerly drove G1 (V2UpdateSurface) +
# G2 (SendAppEventOnAppReady "Home") but left G3 (the engine's OWN files-dir libc++
# std::string [0x10726d600] + the R1 CoreScript content) reachable ONLY via the
# --v2boot-set-filesdir / --v2boot-r1-stage elfjit rungs, which SH407/408 MEASURED
# never fire on the reaching full-ladder env. SH412 wires drive_content_surface into
# the ordered substrate right after the MessageBus.subscribe atom (same post-bus step
# that drives the binder + nativeAppBridgeAppStart), so the content surface is a
# first-class driven runtime step on the SAME ladder thread.
#
# Measure: (a) the substrate still completes with an Ok count (no regression from the
# new step, which is inert-by-construction), (b) the SH412 files-dir SEEDED read-back
# fires inside the ordered drive, and (c) the R1 content staging runs (staged N
# candidates) — the "G3 content gate" now actually executes headlessly. No DM is
# manufactured (DM-root stays 0) — this is the content surface the engine draws FROM
# the instant a completed do-init owns a live DM.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh412-content-surface.txt
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
  SOBER_ANDROID_ROOT=/tmp/sober_sh412_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the ordered substrate drive (SH400) — per-atom Ok/Err ==="
grep -E "session-drive" "$LOG" | grep -E "\[[0-9]+/16\]|returned Ok|stopped|substrate complete"
echo "=== SH412 content surface (the new first-class step) ==="
grep -E "SH412" "$LOG"
echo "=== how many atoms returned non-zero Ok (the drive's verdict) ==="
grep -aoE "substrate complete: [0-9]+/16 atoms returned non-zero Ok" "$LOG"
echo "=== session observables final (MH_* / AppBridgeV2) ==="
grep -E "MH_FLAGS_LOADED|MH_ENGINE_INITIALIZED|MH_APP_READY|AppBridgeV2" "$LOG" | tail -4
echo "=== DM markers (must stay 0 — honest, no manufacture) ==="
grep -oE "DM-root probe|make_shared|once-slot" "$LOG" | sort | uniq -c
echo "=== terminal / crash ==="
grep -aoE "SIGSEGV|SIGABRT|bad_function_call|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
rm -rf /tmp/sober_sh412_root
exit 0