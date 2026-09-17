#!/bin/bash
# SH251b (Route-B re-attack continuation): drive the class-desc-REGISTERED + genuine-DM CONSUMER+DISPATCH
# path to the new ctor fencepost 0x102b9eca0 (NULL this). Marks: SH189 PlayerGui/ScreenGui class-register
# getters DROVE ok + new ctor SIGSEGV 0x102b9ecbc (fault=0x10). Single run.
BASE=/home/hermes-worker/runs/open-sober
LOG=$BASE/runs/sh251b-consumer-dispatch.txt
rm -f "$LOG"
timeout 170 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 \
  JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 \
  JIT_ROUTEB_DM_REALCTOR_CONSUMER=1 JIT_ROUTEB_DM_REALCTOR_DISPATCH=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  JIT_ROUTEB_DM_SERVICES=1 \
  JIT_REGION_WATCH=0x102dbcc10-0x102dbce90,0x10201fda4-0x102020000,0x102e1c650-0x102e25200 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
EXIT=$?
echo "EXIT=$EXIT"
echo "=== realctor + consumer drive ==="
grep -E "routeb-realctor|GENUINE|get-or-create|create-path|MNFT|manufactur" "$LOG"
echo "=== genuine-DM dispatch region hits (setDataModelToCurrent / PlayerGui register / EC world) ==="
grep -oE "region hit at guest pc=0x10[0-9a-f]+" "$LOG" | sort -u | tr '\n' ' ' | head -c 1500; echo
echo "=== service container / class registry built? (probe via dmservices lines) ==="
grep -E "routeb-dmservices|routeb-dmins|class-registry|service.*container|register" "$LOG" | head
echo "=== crashes ==="
grep -icE "SIGSEGV|SIGABRT|guestpc=" "$LOG"
echo "=== ladder end ==="
grep -E "ladder done|returned|EXIT" "$LOG" | tail -5