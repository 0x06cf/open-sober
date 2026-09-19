#!/bin/bash
# SH407 probe: characterize the do-init MAIN-arm (SH405/406) terminal precisely.
# SH406 left the app-start MAIN arm draining into the SH341 pool-pop persistence loop
# (no crash, log ends mid-pool-pop) — the named open frontier question is whether that
# world-build ADVANCES past the persistence drain into new construction regions (DMCONT,
# app-shell ctor band, governor, scriptctx) or PARKS (spins on the same pcs). This adds
# region-watch over the FULL app-start body extent [0x10258b5d8,0x102590000) + the post-body
# tail, plus DMCONT / app-shell / governor / scriptctx, and a block-entry pc histogram, to
# separate "crawl forward" from "park".
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh407-appstart-main-terminal.txt
rm -f "$LOG"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_ROUTEB_DONEPATH_MAIN=1 \
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
  JIT_REGION_WATCH=10258b5d8-102590000,102bd1d68-102bd2600,102207b50-102209000,102e9fa84-102ea1000,101f1d8ac-101f1f000,101d9a528-101d9a608 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh407_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== distinct region-hit guest pcs (construction gates) ==="
grep -a "region-watch] region hit" "$LOG" | grep -aoE "guest pc=0x[0-9a-f]+" | sort -u
echo "=== region-hit counts by 0x2-level bucket ==="
grep -a "region-watch] region hit" "$LOG" | grep -aoE "0x[0-9a-f]{7,}" | sed -E 's/0x(....).*/\1/' | sort | uniq -c | sort -rn | head -20
echo "=== DMCONT hits (<region>/DMCONT 0x102bd1d68) ==="
grep -ac "102bd1d68" "$LOG"
echo "=== app-start body block-entry pcs (0x10258b5d8..0x102590000) ==="
grep -aoE "pc=0x10258[0-9a-f]+ " "$LOG" | tr -d ' ' | sort | uniq -c | sort -rn | head -40
echo "=== app-start body tail entered? (>=0x10258bbb0) ==="
grep -aoE "pc=0x10258[bc][0-9a-f]+ " "$LOG" | tr -d ' ' | sort -u | head
echo "=== terminal / crash ==="
crash=$(grep -icE "SIGSEGV|SIGABRT|bad_function_call" "$LOG"); echo "crash-signals=$crash"
echo "  guestpc terminals: $(grep -aoE 'guestpc=0x[0-9a-f]+' "$LOG" | sort -u | tr '\n' ' ')"
echo "  last 6 lines:"; tail -6 "$LOG"
echo "=== pool-pop count (SH341) ==="
grep -ac "SH341 pop write-site" "$LOG"
rm -rf /tmp/sober_sh407_root
exit 0