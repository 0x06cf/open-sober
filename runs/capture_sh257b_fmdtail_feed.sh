#!/bin/bash
# SH257b: FULL-seed DMCONT run watching for do-init COMPLETION.
# Prior runs showed the app-shell init body's FMOD tail 0x5fb30b4 CAN reach its
# early-return 0x5fb3134 (contradicting "absorbed by sound tail"). With the full
# SH248c-f seed set (jar/once/adapter/appname), measure whether the FMOD-return
# lets the init body COMPLETE into POST-do-init (0x1023eff4c), the app-shell
# ctor (0x102207b50), the governor (0x102e9fa84), and the ScriptContext Lua
# loader (0x101f1d8ac) — the ROUTE-B self-construction chain.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh257b-fmdtail-feed.txt
rm -f "$LOG"
timeout 135 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 \
  JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 \
  JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_REGION_WATCH=0x105fb3080-0x105fb3160,0x1023eff4c-0x1023f0000,0x102207b50-0x102208ec0,0x102e9fa80-0x102ea3b40,0x101f1d8ac-0x101f1da40 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "== EVERY region hit pc (sorted) =="
grep -oE "region hit at guest pc=0x[0-9a-f]+" "$LOG" | grep -oE '0x[0-9a-f]+' | sort -u
echo "== FMOD tail early-return (0x5fb3134/0x5fb3154)? =="
grep -cE "region hit at guest pc=0x105fb3134|region hit at guest pc=0x105fb3154" "$LOG"
echo "== post-do-init / app-shell / governor / ScriptContext loader hit counts =="
for r in 0x1023eff4c 0x102207b50 0x102e9fa84 0x101f1d8ac; do
  echo -n "$r: "; grep -cE "region hit at guest pc=$r" "$LOG"
done
echo "== crashes =="
grep -icE "SIGSEGV|SIGABRT|bad_alloc|terminate|stack smash" "$LOG" || true
echo "== ladder tail =="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)|do-init[a-z ]*|app-shell[a-z ]*" "$LOG" | tail -5