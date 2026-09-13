#!/bin/bash
# SH64: reproducible artifact — the ENGINE's OWN per-node PRESENT walker
# (mid-loop entry 0x105b2eec0, x19=R preset) drives a POPULATED scene list:
# each 0x28-stride node's render-obj vt[+24] is a REGISTERED HOST THUNK
# (walker_item_draw_thunk, dispatched via host_call_at with ZERO block-cache
# mutation) that draws a real Mesa colored clear — closing the SH64 nested-
# jit_run recompile-desync SIGSEGV. The walker then swaps via the real
# ctx-vt[+24] on the currency-owning renderinit thread.
#
# RENDERWALKER_NODES=N lays N nodes (default 3). Requires --renderthunk so
# real_ctx is the recovered real engine ctx.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh64-renderwalker.txt
rm -f "$LOG"
timeout 90 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=3 RENDERWALKER_WINDOW_MS=3000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 \
  --renderinit 0x105b3a280 --renderthunk --renderframe --renderwalker \
  --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== walker frame 0 (expect registered host-thunk per-item draw + Ok) ==="
grep -E "renderwalker\] frame #0" "$LOG" | head
echo "=== per-node host-thunk draws (expect nodes * frames; 9 for 3x3) ==="
grep -cF "item draw #" "$LOG"
echo "=== walker Ok ret === (engine mid-loop ran + real swap)"
grep -cF "present walker Ok(ret=0x1)" "$LOG"
echo "=== drained ==="
grep -E "renderwalker\] drained" "$LOG"
echo "=== crash/json-overflow (must be 0) ==="
grep -icE "SIGSEGV|SIGABRT|string length overflow" "$LOG"