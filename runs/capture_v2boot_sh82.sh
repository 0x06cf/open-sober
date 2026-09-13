#!/bin/bash
# SH82 repro: --v2boot ladder now UNPARKS rung 1 — the GlobalInit thread-dispatch
# main-thread-id cell [0x106863a68] is seeded with the ladder thread's own
# pthread_self, so nativeGameGlobalInit takes the match path and runs its real
# do-init instead of parking in the 0x2207648 completion spin forever. Expect the
# ladder to ADVANCE past rung 1 and fault FURTHER at guestpc 0x1021daf78 (the next
# gate) rather than parking (pre-SH82: exit 124, never prints "after
# nativeGameGlobalInit").
# SH83 repro: same --v2boot ladder; JIT_ROUTEB_HASHFIX=1 repairs the string hash-map's
# garbage +0x18 hash-fn-2 slot at the insert ENTRY (guest 0x1029f3e70, x0 = the map) so
# the nativeGameGlobalInit do-init registration insert's dispatch `ldp x1,x8,[x19,#16]`
# falls back to the real single string-hash (blr x1, 0x102a25dec) instead of `blr x8`
# into unmapped memory (SH82b fault guestpc 0x1029f3f7c). Expect the ladder to advance
# past that gate (fault moves further in, next gate ~0x1029f3f84 bucket probe).
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/open-sober/runs/sh83-v2boot-regtab.txt
rm -f "$LOG"
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500 \
  JIT_ROUTEB_HASHFIX=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles \
  --persist-roundtrip --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== SH82 seed (must appear once, before rung 1) ==="
grep -E "seeded main-thread-id" "$LOG"
echo "=== SH83 hash-map +0x18 repair (must appear) ==="
grep -E "routeb-hashfix" "$LOG"
echo "=== rung 1 — ADVANCED past the SH82b reg-table gate? ==="
grep -E "nativeGameGlobalInit|after nativeGameGlobalInit" "$LOG"
echo "=== old SH82b fault should be GONE ==="
grep -oE "guestpc=0x1029f3f7c" "$LOG" | head -1
echo "=== any NEW fault (next gate) ==="
grep -oE "guestpc=0x[0-9a-f]+" "$LOG" | sort -u | head -5
echo "=== product baseline (no --v2boot) sanity ==="
grep -oE "persist\] live datastore" "$LOG" | head -1