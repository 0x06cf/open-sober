#!/bin/bash
# SH344b: does the SH344-reached NativeDataModelManager -> nativeAppBridgeAppStart
# continuation now drive the APP-SHELL CTOR band [0x102207b50..0x102209000]?
# SH340 measured that band 77 hits deep on the send-appevent skip-appstart path (but
# governor silent). SH344 showed the FULL SH343-deepened ladder now reaches the manager
# line (fnB 0x102bd1b98 -> real continueAfterFlagsLoaded_ -> nativeAppBridgeAppStart).
# This measures whether the app-shell ctor band fires on the full ladder too, and whether
# it progresses past the SH156-marked ctor-calleeps toward governor-owned Lua.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh344b-appshell.txt
rm -f "$LOG"
timeout 150 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_REGION_WATCH=0x102207b50-0x102209000,0x1023eff4c-0x1023f0100,0x101f1d8ac-0x101f1d940 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-surface-handoff --v2boot-send-appevent --v2boot-send-game-loaded --v2boot-session-bus \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== region hits by band ==="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | awk '{print $NF}' | cut -c1-12 | sort | uniq -c | sort -rn
echo "=== app-shell ctor distinct pcs ==="
grep -oE "guest pc=0x102207[0-9a-f]+|guest pc=0x102208[0-9a-f]+|guest pc=0x1022090[0-9a-f]+" "$LOG" | sort -u | head
echo "=== post-do-init / ScriptContext hits ==="
grep -cE "guest pc=0x1023eff4c|guest pc=0x101f1d8ac" "$LOG"
echo "=== terminal + session markers ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' '; echo
grep -E "DM-root probe|app-data-model-count|ladder done|once-guard" "$LOG" | tail -6