#!/bin/bash
# SH404: trace the do-init FALL-THROUGH branch (0x2206fac nested worker -> app-shell band) to
# find where it terminates. SH403 measured the StartApp boot body -> pipe -> do-init DEEP body
# runs, but the do-init MAIN dispatch `br x1` @0x2206e24 (-> vt[+48]=0x10258b5d8) gets 0 region
# hits — the run takes a DIFFERENT fall-through path: 0x2206e30 bl 0x102206ebc -> 0x2206e34
# (operator-new) -> 0x2206ebc/0x2206f04 -> 0x2206fac (a nested worker prologue sub sp,#0x60).
# Region-watch the full fall-through chain [0x102206fac,0x102209000) (nested worker -> app-shell
# ctor band) + the main dispatch body 0x10258b5d8 + DMCONT, to name where the fall-through lands.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh404-fallthrough.txt
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
  JIT_REGION_WATCH=102206c40-102207000,102207000-102209000,10258b5d8-10258d000,102bd1d68-102bd2600,102baeeec-102baf000 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh404_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== do-init fall-through worker (0x102206fac..0x102207000) block-entry pcs ==="
grep -aoE "pc=0x102206[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn
echo "=== app-shell ctor band (0x102207000..0x102209000) entry pcs ==="
grep -aoE "pc=0x102207[a-f0-9][0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn | head -25
echo "=== main dispatch body (0x10258b5d8) + DMCONT + pipe ==="
for pc in 0x10258b5d8 0x102bd1d68 0x102baeeec; do echo "  $pc: $(grep -acE "pc=$pc " "$LOG")"; done
echo "=== SH361 dispatch marker ==="
grep -a "doinit-dyn" "$LOG" | tail -2
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
echo "  guestpc terminals: $(grep -aoE 'guestpc=0x[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
rm -rf /tmp/sober_sh404_root
exit 0