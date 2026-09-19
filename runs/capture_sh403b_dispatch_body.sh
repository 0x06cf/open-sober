#!/bin/bash
# SH403b: does the do-init MAIN dispatch BODY (0x10258b5d8, SH362's measured-never-executes
# gate) now EXECUTE through the StartApp boot body -> app-bridge-pipe -> do-init chain?
# SH362 measured: no headless env ever ENTERED 0x10258b5d8 (every run faulted at SH285 first).
# SH403 showed the StartApp boot body + pipe + do-init deep body all run clean EXIT 124, the
# SH361 dispatch fires (container+32 non-NULL -> vt[+48]=0x10258b5d8). Region-watch 0x258b5d8
# body + the SH285 persistence-lane terminal + DMCONT to see if the do-init dispatch body itself
# is now entered (a genuine SH362-gate advance) and where the run terminates.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh403b-dispatch-body.txt
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
  JIT_REGION_WATCH=10258b5d8-10258d000,102bd1d68-102bd2600,102206c40-102207000 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh403b_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH362 gate: do-init MAIN dispatch body (0x10258b5d8) block-entry pcs ==="
grep -oE "pc=0x10258b[5-9a-f][0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn | head
echo "=== DMCONT (0x102bd1d68) ==="
echo "  DMCONT: $(grep -acE "pc=0x102bd1d68 " "$LOG")"
echo "=== do-init deep body pcs (0x102206c40..0x102207000) ==="
grep -aoE "pc=0x10220[67][0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn | head -14
echo "=== SH361 dispatch marker ==="
grep -a "doinit-dyn" "$LOG" | tail -2
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
echo "  guestpc terminals: $(grep -aoE 'guestpc=0x[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
rm -rf /tmp/sober_sh403b_root
exit 0