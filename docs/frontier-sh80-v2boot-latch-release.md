# SH80 — Route-B latch release: v2boot ladder now drives nativeInitializeNativeFlags first

## Result (honest: wall ADVANCE, not a breakthrough)

Implemented the first step of docs/recon-routeB-globaltinit-unblock.md against
the real libroblox.so: the `--v2boot` ladder now drives
**`nativeInitializeNativeFlags` (0x10232048c) as rung 0** — on the engine's OWN
flags-loaded write chain (0x2320cec -> 0x2320f2c -> the `strb #1,[0x72739d4]`
latch setter at file 0x22474e8) — so the gameGlobalInit gate latch is set through
**JIT-translated guest code** (safe RW-map write) instead of the fragile raw host
write (which SIGSEGVs: the latch page is not host-writable directly).

Pinned the chain read-only: the latch setter at file 0x22474e8
(`adrp x9,7273000; strb w19,#1,[x9,#2516]`) writes guest 0x10672739d4, and the
gate read at 0x224fc6c (`ldrb [x8,#984]` in the same fn family) throws
"Can't initialize the TaskScheduler before flags have been loaded" when 0.

JNI value registry: `jni_call_int_method` now returns **getFlagsCount => 1**
(was 0 -> nativeInitializeNativeFlags bailed immediately), so the engine walks
its flags chain far enough to reach the latch write + beyond.

## Empirical (real libroblox.so, runs/sh80-v2boot-r0.txt)

`--v2boot` now logs:
```
[elfjit:v2boot] after boot start: [0x106829ea8] = 0x0
[elfjit:v2boot] driving nativeInitializeNativeFlags @ guest 0x10232048c
```
and **executes real engine code past the SH55 rung-1 park** — the ladder
reaches a point no prior cycle did. It then SIGSEGVs at guestpc 0x10624f46c
(file 0x624f46c, fault 0x30 = null deref, x0=0) — a DIFFERENT, downstream
harness-bootstrap fault (task-scheduler/device init), NOT the historical
infinite nanosleep park. This confirms the latch gate is the correct first
blocker and that releasing it un-sticks the park (the run no longer hangs at
"driving nativeGameGlobalInit", it crashes *further in* — progress of one gate).

## What changed

- examples/elfjit.rs: `--v2boot` rungs array 6->7, rung 0 = nativeInitializeNativeFlags
  (0x10232048c); comments document the recon chain.
- src/jni.rs: `jni_call_int_method` adds `getFlagsCount => 1`; extended the
  `jni_auto_value_params_getters_resolve_via_fn_table` regression.

Workspace **509/0**, example **22 pass**. The raw-host-write variant was removed
(it faulted: latch page not host-writable). V2BOOT_SEED_LATCH env no longer
used (replaced by the guest-native rung 0).

## Honest scope / next

Advancing one gate: nativeGameGlobalInit's park is confirmed latch-gated and
the latch is now set via engine code. The run then hits a downstream
pre-existing boot fault (0x624f46c) — the standing wall is a sequence of these,
not a single latch. Next (ranks from the recon): wire the NativeHelper
`gameActivity_*` callbacks (onFlagsLoaded/onEngineInitialized/onAppReady/
onDidLogInReceived/onGameLoaded) into the RegisterNatives registry so
StartLuaAppDM's Lua session can actually construct the first login/home
GuiObject tree; or clear the 0x624f46c boot fault by inspecting its caller's
missing device/scheduler object and seeding it like the latch. The taskv4
producer vector [0x106829ea8] remains framework-glue seeded.