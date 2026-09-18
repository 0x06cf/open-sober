#!/bin/bash
# SH327 probe: force AppStarted factory construction deterministic -> confirm the 0x1025f5300
# gate ADVANCES one fencepost (construction lands, field-copy completes, fault moves to the
# V2StartAppWithParams params-obj +0x140 member at 0x25f5328). Same base as probe_sh324_terminal_regs.sh.
cd /home/hermes-worker/runs/open-sober
BASE="JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_DONEPATH_MAIN=1 JIT_ROUTEB_LIFECYCLE_EARLYRET=1 \
  JIT_ROUTEB_SETTINGS_SSO_SEED=1 JIT_DUMP_REGION=0x10225500-0x10225680 JIT_DUMP_PC=0x102e89150,0x1025f52f0"
BIN=./target/debug/examples/elfjit
SO=~/.cache/open-sober/robbox/libroblox.so
ARGS="--jni --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus"
LOG=/tmp/sh327-probe.txt; rm -f "$LOG"
timeout 150 env $BASE $BIN $SO 0x2173ff4 $ARGS > "$LOG" 2>&1
echo "EXIT=$?"
echo "first_sigsegv=$(grep -o 'guestpc=0x[0-9a-f]*' "$LOG" | head -1)"
echo "SH327_construct_patch=$(grep -c 'SH327 force AppStarted construct' "$LOG")"
echo "construction_write=$(grep -c 'DUMPPC pc=0x102e89150' "$LOG")"
echo "gate_read_cell_nonnull=$(grep -o 'appstart0x106a6f480\]=0x[1-9a-f][0-9a-f]*' "$LOG" | head -1)"
echo "fault_reads_params_member=$(grep -c 'guestpc=0x1025f5300' "$LOG")"
echo "--- near-fault gate registers ---"
grep -A2 'appstart0x106a6f480' "$LOG" | grep -E 'x19=|x20=' | head -4