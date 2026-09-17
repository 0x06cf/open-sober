#!/bin/bash
# SH243: A/B the NativeDataModelManager getter cell. The getter 0x102174c04's
# `adrp x8,7275000; add x8,+#0x550; ldar x0,[x8]` reads GUEST 0x107275550 — NOT the
# 0x102727550 seeded by SH165-240 (8 cycles). FORWARD (this binary) seeds BOTH cells,
# so StartLuaAppDM returns Ok(M) (the fabricated manager) instead of baseline Ok(0x3e8).
# Success markers: `StartLuaAppDM returned Ok(<heap>)` (a host pointer, M-shaped; NOT
# 0x3e8), plus `[routeb-dmforce:SH243] ALSO seeded getter's TRUE read cell 0x107275550`.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh243-cell48-ab.txt
rm -f "$LOG"
timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH243 true-cell seed fired? ==="
grep -E "ALSO seeded getter|SH243.*cells" "$LOG" | head -3
echo "=== StartLuaAppDM return (want Ok(<host heap>), NOT Ok(0x3e8)) ==="
grep -oE "StartLuaAppDM returned Ok\([^)]*\)" "$LOG" | tail -1
echo "=== crash? ==="
grep -icE "SIGSEGV|SIGABRT" "$LOG" || true