#!/bin/bash
# SH4xx-next: reproduce the canary stack-smash wall with BOTH watches armed —
# the store-watch NAMES the guest store that writes a foreign HOST pointer
# (0x700000000000..0x800000000000) into a frame canary window, and the host-
# RETURN watch NAMES the host fn that RETURNED that same 0x7f... pointer into
# guest x0. Running both on one boot lets you correlate by value: grep the same
# 0x7f... token in the [host-return-watch] and [canary-store-watch] lines = the
# named host producer of the canary-smashing pointer (SH103 bridge-sanitize dir).
# Both are default-inert instruments (JOIN the 3-gate crossing env; add no guest
# bytes). Stands on the standing nativeGameGlobalInit / app-shell ctor wall.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh-host-return-leak-watch.txt
rm -f "$LOG"
timeout 55 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_CANARY_STORE_WATCH=1 JIT_HOST_RETURN_WATCH=1 JIT_TRACE=0 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot \
  --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderframe-drive --renderframe-seedgles --persist-roundtrip \
  --kicker 0x106863af8 \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== named canary-window writers (is_canary_slot=true) ==="
grep "is_canary_slot=true" "$LOG" | sed -E 's/.*pc=0x([0-9a-f]+) dst=.*/\1/' | sort | uniq -c | sort -rn | head
echo "=== host fn producers of foreign host-pointer returns (by value) ==="
grep -o "hostfn=[^ ]*" "$LOG" | sed 's/^hostfn=//' | sort | uniq -c | sort -rn | head
echo "=== CORRELATED: same 0x7f... value in BOTH a host-return and a canary-store ==="
grep -Eo "0x7[0-9a-f]{12,}" "$LOG" | grep -E "^(0x7f|0x7e)" | sort -u > /tmp/hrw_vals.txt
for v in $(cat /tmp/hrw_vals.txt); do
  r=$(grep -c "host-return-watch.*$v" "$LOG" 2>/dev/null || true)
  s=$(grep -c "canary-store-watch.*val=$v " "$LOG" 2>/dev/null || true)
  if [ "$r" -ge 1 ] && [ "$s" -ge 1 ]; then
    echo "VALUE $v : host-return $r x, canary-store $s x"
    grep "host-return-watch.*$v" "$LOG" | head -2
  fi
done
echo "=== the LAST canary-window store before the abort (candidate clobber) ==="
grep -E "is_canary_slot=true" "$LOG" | tail -1
echo "=== stack-smash present? ==="
grep -c "stack smashing" "$LOG" || echo 0