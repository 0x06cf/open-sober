#!/bin/bash
# SH218-probe: is the WORLD-BUILD fn 0x102ea3b14 reachable via the surface-handoff rung's OWN
# call site (0x25f5e04 inbound nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams)?
# SH199/204 measured the V2Init call site 0x10236810c as UNREACHED (SETWORLDBUILD latent). Nobody
# examined the SECOND call site at 0x25f5e04. This is a genuinely-new completion/continuation angle.
#
# fn 0x25f5fec (V2UpdateSurface, driven REAL per SH210/202, returns Ok XID):
#   25f5dd0 ldrb w8,[x8,#1384](=[0x106a70568])   <- SAME SH199 world-build gate byte
#   25f5dd4 cbz w8, 25f5e10     (skip world-build if gate==0)
#   25f5e00 mov x0,x19
#   25f5e04 bl  2ea3b14         (WORLD-BUILD: op-new 0x18 + ctor 0x2eacccc + nativeAppBridgeAppStart)
#   25f5e08 mov x1,x20 ; bl 2eacdb4
#
# Region-watch: surface-handoff gate+call region [0x1025f5dd0,0x1025f6110) + world-build body
# [0x102ea3b14,0x102ea3c50) + governor tail [0x102e9fa80,0x102ea3b40). With AND without
# SETWORLDBUILD=1 (which seeds gate [0x106a70568]=1 IF its V2Init gate-block 0x102368100 is entered).
set -u
cd "$(dirname "$0")/.."
for MODE in latent seeded; do
  for i in 1 2 3; do
    LOG="/tmp/sh218-surf-$MODE-$i.txt"
    rm -f "$LOG"
    EXTRA=""
    [ "$MODE" = seeded ] && EXTRA="JIT_ROUTEB_SETWORLDBUILD=1"
    timeout 120 env $EXTRA \
      JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
      JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
      JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
      JIT_REGION_WATCH=0x1025f5dd0-0x1025f6110,0x102ea3b14-0x102ea3c50,0x102e9fa80-0x102ea3b40 \
      ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
      --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
      > "$LOG" 2>&1
    E=$?
    surf_gate=$(grep -oE "region hit at guest pc=0x1025f5(dd|e0)" "$LOG" | sort -u | tr '\n' ' ')
    surf_call=$(grep -cE "region hit at guest pc=0x1025f5e0[04]" "$LOG")
    wb=$(grep -E "region hit at guest pc=0x102ea3[b]" "$LOG" | sort -u | wc -l)
    crash=$(grep -acE "SIGSEGV|SIGABRT" "$LOG")
    setfix=$(grep -ac "SH161b" "$LOG")
    echo "[$MODE] run $i: EXIT=$E surf_gate_hits=$surf_gate surf_worldbuild_call=$surf_call worldbuild_body_pcs=$wb crashes=$crash sh161b=$setfix"
  done
done
echo "=== if worldbuild_body_pcs>=1 in either mode => world-build fn EXECUTES via the surface rung (Route-B forward motion, FIRST reach) ==="
echo "=== if surf_call>=1 but worldbuild_body==0 => the surface rung reaches its world-build call but the cbz gate [0x106a70568] blocks it there ==="