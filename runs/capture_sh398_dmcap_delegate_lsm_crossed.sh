#!/bin/bash
# SH398: genuinely-never-run intersection — the WORKING SH395/397 DELEGATE observer
# (JIT_DM_ALLOC_CAPTURE_DELEGATE=1, the only mode that installs over the engine's
# real allocator hook 0x1021ebaf4) ON TOP of the SH285-CROSSING env (SH373's
# LSM_APPEND_SKIP + SH385's LSM_PACK_SKIP + KEYFIX), which advances the do-init
# dispatch ladder PAST the standing SH285 persistence lane (guestpc=0x101db1b08 ->
# crossed) into the further persistence-family fenceposts (0x101d9a708 pack helper /
# governor NULL-DM 0x102ea0b9c).
#
# SH397 ran DELEGATE on a dispatch env WITHOUT the SH285-crossing skips, so the run
# terminated AT the SH285 lane before any deeper reach — the observer could only
# report what the pre-lane reach did. SH373/385 ran the crossings WITHOUT the
# observer, so "thousands of pool-pops with no make_shared" was inferred, never
# observed. This composes the two: measure, with a working observer, whether ANY
# validated make_shared<DataModel> allocation fires anywhere PAST the crossed SH285
# lane, including transiently in the deeper governor/pack/SetInitParams region.
#
# Verdict produced by this probe:
# - 0 validated make_shared<DataModel> even past the crossed lane -> the strongest
#   closure yet (working observer + furthest reach past the persistence fence).
# - >0 validated -> a live DataModel allocation exists deeper; Route B advances.
#
# Probe-only: no production Rust / guest byte / JIT-hook-default touched.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh398-dmcap-delegate-lsm-crossed.txt
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
  JIT_ROUTEB_LSM_APPEND_SKIP=1 JIT_ROUTEB_LSM_PACK_SKIP=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh398_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH285 crossed? (0 hits = crossed; >0 = run died at the persistence lane, probe inconclusive-but-indicative) ==="
echo -n "sh285-hits="; grep -cE "guestpc=0x101db1b08" "$LOG"
echo "=== append-skip / pack-skip fired? (crossing arms) ==="
grep -acE "append-skip|pack-skip|SH349|SH350" "$LOG"
echo "=== the do-init MAIN dispatch (sh361 dyn trace) ==="
grep -E "routeb-doinit-dyn" "$LOG" | head -2 || echo "(no SH361 trace this run)"
echo "=== DM-capture DELEGATE latch install (trustworthy observer active) ==="
grep -aE "routed CRT operator-new ACTIVE|prev_hook|capture trail|FIRST call" "$LOG" | head -6
echo "=== validated make_shared<DataModel> count (THE verdict) ==="
echo -n "validated="; grep -acE "\[validated\]" "$LOG"
grep -aE "\[validated\]" "$LOG" | head -5
echo "=== all allocation bytes observed (dedup) ==="
grep -aoE "bytes=0x[0-9a-f]+" "$LOG" | sort | uniq -c | head
echo "=== terminal / crash signal ==="
grep -aoE "SIGSEGV|SIGABRT|bad_function_call|bad_weak_ptr|guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "DM-root probe|SendAppEventOnAppReady returned|MH_APP_READY|governor|SetInitParams" "$LOG" | tail -6
rm -rf /tmp/sober_sh398_root
exit 0