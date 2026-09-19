#!/bin/bash
# SH348 forward-measurement: cross the SH285 persistence-lane wall by leaf-`ret`ing
# initStorageManagerNative (JIT_ROUTEB_LSM_INIT_SKIP=1) so the Session-CTOR continuation
# advances past the LSM lane toward app-start 0x2bd2058 / the app-shell ctor. Default-INERT.
# Full SH285-B / SH343-346 ladder env + LSM_NODES; region-watch on the app-shell ctor band
# (0x102207b50..0x102209000) + ScriptContext Lua loader (0x101f1d8ac..0x101f1d940) + governor
# tail, to see whether the Session load reaches the app-shell/SceneGraph step it never reached.
set -u
cd "$(dirname "$0")/.."
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_ENG5_QMUTEX_FREE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_INIT_SKIP=1"
SL="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-engine3 --v2boot-session-engine9"
R="JIT_REGION_WATCH=0x102207b50-0x102209000,0x101f1d8ac-0x101f1d940,0x102e9fa80-0x102ea3b40"
WANT="${SH348_RUNS:-3}"
for i in $(seq 1 "$WANT"); do
  timeout 40 env $BASE $R ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 $SL > runs/sh348-run$i.txt 2>&1
  echo "run$i EXIT=$? lsm_skip=$(grep -acE 'SH348 leaf-.*initStorageManagerNative' runs/sh348-run$i.txt) \
    term=$(grep -aoE 'guestpc=0x[0-9a-f]+' runs/sh348-run$i.txt | tail -1) \
    fault=$(grep -aoE 'fault=0x[0-9a-f]+' runs/sh348-run$i.txt | tail -1) \
    appshell=$(grep -aoE 'appshell=[0-9]+' runs/sh348-run$i.txt | tail -1) \
    scriptctx=$(grep -aoE 'scriptctx=[0-9]+' runs/sh348-run$i.txt | tail -1) \
    gov=$(grep -aoE 'gov=[0-9]+' runs/sh348-run$i.txt | tail -1)"
done
echo "=== region-watch detail (last run) ==="
grep -aE "region hit|appshell|scriptctx|gov" runs/sh348-run$WANT.txt | tail -25
echo "=== app-shell-ctor band hits (0x102207b50) present? ==="
grep -aE "region hit .*0x1022[0-9a-f]" runs/sh348-run$WANT.txt | grep -c "0x1022"
echo "=== ScriptContext loader hits (0x101f1d8ac) present? ==="
grep -aE "region hit .*0x101f1d" runs/sh348-run$WANT.txt | grep -c "0x101f1d"
echo "=== governor tail hits (0x102e9fa80) present? ==="
grep -aE "region hit .*0x102e9fa" runs/sh348-run$WANT.txt | grep -c "0x102e9fa"
echo "=== DM-root / MH_* (last run) ==="
grep -aiE "DM-root|MH_FLAGS_LOADED|MH_ENGINE_INITIALIZED|MH_APP_READY|0x106a68818" runs/sh348-run$WANT.txt | tail -10