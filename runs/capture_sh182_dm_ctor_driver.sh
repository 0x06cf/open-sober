#!/bin/bash
# SH182 repro: host-DRIVE the manufactured DM through its REAL app-shell ctor
# [V+0x30]=0x1057d6ef4 (genuine primary DataModel vtable 0x1067162f0).
# SH181 planted a genuine-vptr manufactured DM into the current-DM holder but no
# headless consumer dispatched its vtable (region-watch = 0 'entered region').
# SH182 (recon deleg_661626bb): the app-shell ctor's one fault point is the
# stack-canary global file 0x67d16f0 (guest 0x1067d16f0), value 1 in a bare boot
# -> `ldr x8,[x21]` at 0x57d6f10 SEGV. The driver seeds that canary to a stable
# word + host-drives the ctor with x0=manufactured DM + x1=zeroed descriptor
# (PATH A: clean zero-touch survival no-op). Success: the ctor region
# [0x1057d6ef4,0x1057d7100] now shows 'entered region' (SH181 = 0) AND the
# guard logs 'DROVE ok' -> the manufactured DM ENTERED AND RETURNED through
# real relocated engine code headlessly. Ladder stays clean, EXIT 124.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh182-dm-ctor-driver.txt
rm -f "$LOG"
timeout 200 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_CTOR_DRIVER=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_REGION_WATCH=0x1057d6ef4-0x1057d7100 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== the SH182 ctor-driver fire (the deliverable) ==="
grep -E "routeb-dmctor" "$LOG"
echo "=== did the DM app-shell ctor region EXECUTE? (SH181 was 0; want >=1) ==="
grep -c "entered region" "$LOG"
echo "=== ladder completion ==="
grep -E "ladder done|joined cleanly|SendAppEventOnAppReady returned|StartLuaAppDM returned" "$LOG" | head -5
echo "=== real crashes (want 0) ==="
grep -icE "SIGSEGV|SIGABRT|stack smashing" "$LOG"