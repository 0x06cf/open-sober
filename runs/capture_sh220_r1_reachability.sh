#!/bin/bash
# SH220: R1 content-path reachability — empirical 3-way cross-check closing the
# operator's SH219b delegated task ("log the engine's first rbxasset:// request
# to name the R1 CoreScript file") with a MEASURED answer.
#
# Question: on the canonical completing ladder, does the engine request ANY
# assets/ or rbxasset://scripts/CoreScripts content (so a staged R1 synthetic
# Lua module could be picked up and drive a self-constructed GuiObject)?
#
# Arms JIT_ASSET_TRACE (the SH219b diagnostic) on aassetmanager_open and JIT_REGION_WATCH
# on the CoreScript loader (0x101f1d8ac region, SH131d/SH186) plus the rbxasset path
# (0x10232ed4 region). Runs THREE arms:
#   arm A: harness only (no staged file, assets NOT mounted)
#   arm B: staged synthetic R1 CoreScript at the filesdir mirror (+SOBER_ANDROID_ROOT)
#   arm C: arm B + REAL extracted assets mounted (SOBER_ASSETS_ROOT, incl. R2
#          UniversalApp.rbxm at ExtraContent/models/UniversalApp/)
# Success markers would be: >=1 [asset-trace] open line OR a CoreScript-loader
# region hit. The measured result (all arms): ZERO of both, EXIT 124, 0 crash =>
# the engine requests no content headlessly; the R1 loader never fires; the R1
# path is live-DM-gated (the same Route-B structural wall). No in-image loader
# name can be matched because no load is ever attempted.
set -u
cd "$(dirname "$0")/.."
E="${E:-0}"
ARMROOT=/tmp/sh-r1-root
ROBO=~/.cache/open-sober/robbox/libroblox.so
ASSETS=~/.cache/open-sober/android-env/assets
STAGE=0
for arm in A B C; do
  LOG="/tmp/sh220-arm$arm.txt"
  rm -f "$LOG"
  EXTRA=""
  if [ "$arm" != "A" ]; then
    rm -rf "$ARMROOT"; mkdir -p "$ARMROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts" \
      "$ARMROOT/data/user/0/com.roblox.client/cache"
    cat > "$ARMROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/CoreScripts.lua" <<'LUA'
local PlayerGui = game:GetService("PlayerGui")
local screen = Instance.new("ScreenGui")
screen.Name = "OpenSoberR1"
screen.Parent = PlayerGui
print("[R1] ScreenGui constructed")
LUA
    cp "$ARMROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/CoreScripts.lua" \
       "$ARMROOT/data/user/0/com.roblox.client/cache/"
    EXTRA="SOBER_ANDROID_ROOT=$ARMROOT"
    STAGE=1
  fi
  if [ "$arm" = "C" ]; then
    EXTRA="SOBER_ANDROID_ROOT=$ARMROOT SOBER_ASSETS_ROOT=$ASSETS"
  fi
  A=$?; T=0; R=0; C=0; ATT=1
  # Retry up to 3x taking the first CLEAN (EXIT 124, 0 crash) run — the ladder has a
  # documented run-variable FMOD-AAudio / SH55-64 non-seedable flake (unrelated to
  # this measurement); counts must come from a clean run to be meaningful.
  while [ "$A" != "124" ] && [ "$ATT" -le 3 ]; do
    # shellcheck disable=SC2086
    timeout 110 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
      JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
      JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
      JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 \
      JIT_ASSET_TRACE=1 \
      JIT_REGION_WATCH=0x101f1d8ac-0x101f1db00,0x10232ed4-0x10232f00 \
      $EXTRA \
      ./target/debug/examples/elfjit "$ROBO" 0x2173ff4 \
      --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
      --v2boot-set-filesdir > "$LOG" 2>&1
    A=$?; T=$(grep -acE "\[asset-trace\]" "$LOG"); R=$(grep -acE "CoreScript-loader-region-hit|0x101f1d8ac" "$LOG")
    C=$(grep -aicE "SIGSEGV|SIGABRT" "$LOG"); ATT=$((ATT+1))
  done
  echo "arm $arm: EXIT=$A asset_trace_lines=$T csl_region_hits=$R crash=$C"
done
echo "=== RESULT: if all arms T=0 R=0 => R1 content path is live-DM-gated (no headless loader, no asset request) ==="