#!/bin/bash
# SH384: reproducible artifact — DRIVE the GENUINE LocalStorageManager ctor 0x1db0dfc
# through REAL engine code with a coherent (zeroed, NULL-tolerant) container, manufacturing
# a real vtable-owning manager object — the MIGRATION-directive manufacture lever, which
# SH383 only byte-PINNED. Prior persistence cycles were SKIP (SH348 leaf-ret, SH349/350
# sub-call-skip) or SEED (SH267/285 map/nodes); NONE drove the genuine ctor. Disasm-driven:
# the outer ctor's `cbz x0` @0x40 (NULL container sub -> benign) + inner ctor's `cbz x9`
# @0x2c -> `mov x19,xzr` @0x78 make the zeroed-container drive clean, and the inner stack
# canary reads the same global twice (self-consistent, no patch needed). If the drive
# returns Ok with in-image vtable words on `this`, the manager is genuinely manufactured.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh384-lsm-ctor-manufacture.txt
MAX=${SH384_RETRY_MAX:-4}
run_once() {
  rm -f "$LOG"
  timeout 150 env \
    JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
    JIT_ROUTEB_LSM_CTOR_MANUFACTURE=1 \
    JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
    JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
    JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
    JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
    JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
    JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
    > "$LOG" 2>&1
  local exit=$?
  # SUCCESS = the drive REPORTED a "DROVE ok" manufacture (or a clean drive err = measured),
  # and 0 SIGSEGV on the ctor line (a guest fault inside the drive is still reported Err).
  local drove=$(grep -cE "routeb-lsm-ctor\] SH384" "$LOG")
  local crash=$(grep -icE "SIGSEGV guestpc=0x1db0dfc|SIGABRT" "$LOG")
  echo "attempt: exit=$exit drove-lines=$drove crash-on-ctor=$crash"
  [ "$drove" -ge "1" ] && [ "$crash" -eq "0" ]
}
won=""
for i in $(seq 1 "$MAX"); do
  echo "=== attempt $i/$MAX ==="
  if run_once; then won=$i; break; fi
done
echo "=== SH384 LSM-ctor manufacture drive marker ==="
grep -E "routeb-lsm-ctor\] SH384" "$LOG" || echo "(no SH384 marker — drive never reached StartLuaAppDM)"
echo "=== manufacture result (DROVE ok / in-image vt) ==="
grep -E "DROVE ok|drive err" "$LOG" | tail -3
echo "=== terminal guestpcs ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
echo "=== session markers ==="
grep -E "MH_APP_READY|DM-root probe|continueAfterFlagsLoaded|LocalStorageManager|SendAppEventOnAppReady returned" "$LOG" | tail -6
echo "won-attempt=${won:-none}"