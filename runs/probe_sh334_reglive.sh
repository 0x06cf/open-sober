#!/bin/bash
# SH334 — LIVE answer to STATUS candidate #1 ("does the app-start MAIN-path registration-walk
# register 'App'?") WITHOUT the post-ladder dump. The SH332/333 post-ladder dump() that reads the
# service-registry/DM-root NEVER fires because the run dies at the FMOD/AAudio wall (0x106240cb8)
# or the LSM reader (0x101dcab68) first. JIT_ROUTEB_REG_LIVE=1 installs a default-inert block-entry
# guard (routeb_registry_live_guard in crates/arm64jit/src/jit.rs) that snapshots the registry count +
# entry names + DM-root + once-slot + tier-2 controller-name cell at the EXACT moment the
# DM-controller ctor's name->service lookup (fn 0x2168798, entry 0x102168798) runs — so candidate #1
# is answerable from a live readout that survives the later crash.
#
# MEASURED (this cycle): service-registry-count[0x106fe2f08]=0 entries=[] DM-root[0x106a68818]=0x0
# once-slot[0x106a68408]=0x0 fixidx0=0x0 tier2-cell[0x106fe4f78]="" at the lookup — i.e. on the
# SH332-style MAIN-path run the registry is EMPTY when the DM-ctor lookup runs: "App" is NOT
# registered before the run-variable crash. Candidate #1's "open but UNCHANGED" status is now
# resolved with a live measurement: the registration-walk (0x21e2a90) fires but does not complete a
# registry write before the (run-variable) FMOD/LSM wall kills the run.
set -u
cd "$(dirname "$0")/.."
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 \
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
  JIT_ROUTEB_APPSART_408SEED=1 JIT_ROUTEB_REG_LIVE=1"
LOG=/tmp/sh334.txt; rm -f "$LOG"
timeout 150 env $BASE JIT_REGION_WATCH="0x1021e2a90-0x1021e2b40,0x102168798-0x102168840" \
  $BIN $SO 0x2173ff4 --jni --startapp 0x258b144 --v2boot --v2boot-session >"$LOG" 2>&1
EX=$?
echo "EXIT=$EX"
echo "reglive: $(grep -a 'routeb-reglive' "$LOG" | tail -1)"
echo "crash:  $(grep -aoE 'guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+' "$LOG" | tail -3 | tr '\n' ' ')"
echo "--- region hits ---"; grep -aoE "region hit at guest pc=0x[0-9a-f]{8}" "$LOG" | sort -u | sed 's/^/  /'
exit 0