#!/bin/bash
# SH408 probe: is the SH285/LSM "unconstructible manager" terminal in part caused by the
# UNSEEDED app files-dir global [0x10726d600]? SH407 showed the do-init MAIN arm runs the app-start
# body end-to-end then drains into the SH341 pool-pop persistence lane (190 pops, SETFIX armed, still
# EXIT 139), with NO files-dir seeding in the env. A real host provides the app files dir before the
# LocalStorageManager session ctor runs. This adds --v2boot-set-filesdir (seeds [0x10726d600] as a
# valid libc++ string "/data/user/0/com.roblox.client/files") + --v2boot-r1-stage, and region-watches
# the REAL LSM ctor (0x1db0dfc), initStorageManagerNative (0x1db1050), the SH285 reader (0x1d99e30)
# + reader wall pc 0x101db1b08, the append leaf (0x1d9a15c), and the pool-pop (0x1d9a5a0) to see
# whether seeding the files-dir lets the real storage session construct and cross the persistence lane.
set -u
cd "$(dirname "$0")/.."
LOG=/home/hermes-worker/runs/sh408-filesdir-lsm.txt
rm -f "$LOG"
timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_GETTER_TAIL_RET=1 JIT_ROUTEB_DM_CONT_M48_SEED=1 JIT_ROUTEB_CONT_APPNAME_SEED=1 \
  JIT_ROUTEB_APPSART_JAR_SEED=1 JIT_ROUTEB_APPSART_ONCE_SEED=1 JIT_ROUTEB_APPSART_ADAPTER_SEED=1 \
  JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 JIT_ROUTEB_APPSART_LSM_NODES=1 JIT_ROUTEB_APPSART_GOVFLAG=1 \
  JIT_ROUTEB_PRELOAD_VALUECELL=1 JIT_ROUTEB_DOINIT_EMPTYVEC=1 \
  JIT_ROUTEB_LSM_KEYTRACE=1 JIT_ROUTEB_LSM_KEYFIX=1 \
  JIT_ROUTEB_DOINIT_DYN_TRACE=1 \
  JIT_ROUTEB_DONEPATH_MAIN=1 \
  JIT_ROUTEB_LIFECYCLE_EARLYRET=1 JIT_ROUTEB_SETTINGS_SSO_SEED=1 \
  JIT_REGION_WATCH=101db0dfc-101db1000,101db1050-101db2000,101d99e30-101d9a000,101d9a15c-101d9a200,101d9a5a0-101d9a610,10258b5d8-102590000,102bd1d68-102bd2600 \
  SOBER_ANDROID_ROOT=/tmp/sober_sh408_root \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-session-drive --v2boot-set-filesdir --v2boot-r1-stage \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== filesdir seeded + verified? ==="
grep -a "v2boot-setfilesdir" "$LOG" | head -3
echo "=== R1 staged? ==="
grep -a "\[r1\]\|STAGED\|v2boot-r1" "$LOG" | head -6
echo "=== region hits: LSM ctor / initStorage / SH285 reader / append / pool-pop / app-body / DMCONT ==="
grep -a "region-watch] region hit" "$LOG" | grep -aoE "guest pc=0x[0-9a-f]+" | sort -u
echo "=== SH285 reader wall pc 0x101db1b08 hits ==="
grep -ac "101db1b08" "$LOG"
echo "=== SH285 0xff..ff / append base fault still present? ==="
grep -aoE "guestpc=0x[0-9a-f]+|fault=0x[0-9a-f]+" "$LOG" | sort -u | head
echo "=== pool-pop count ==="
grep -ac "SH341 pop write-site" "$LOG"
echo "=== last 3 ==="
tail -3 "$LOG"
rm -rf /tmp/sober_sh408_root
exit 0