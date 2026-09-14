# SH156 — Route-B do-init now EXECUTES the real GlobalInit construction (live DM-root seed + .bss page-remap fix)

## Outcome
The GlobalInit do-init's match dispatch (guest 0x102206db8) no longer just
soft-returns. With an opt-in JIT_ROUTEB_DM_SEED=1 rung the harness seeds a live
object at DM-root [0x106a68818] whose vtable is a real dispatch vtable, pins the
dispatch vtable+0x30 to a real ctor, and remaps the once-faulting .bss page.
The do-init's real GlobalInit ctor chain now RUNS: its awaited once-guard
[0x106a64d70] self-latches 0->1 for the first time (2/2 runs), and the engine
reaches the AppBridgeV2 governor dispatch (guest 0x102e9fa84 ->
nativeAppBridgeStartAppWithParams) per disassembly. Workspace green; real-binary
combined ladder runs CLEAN (EXIT 0, 24 real task-driven frames, 0 crash).

## The gate (disassembly-back by 3 recons)
The GlobalInit do-init (0x102206c40) ends in a match dispatch (0x2206df4):
  `ldr x0,[<obj>,#32]` -> if 0: benign Ok(0x3e8) soft-return (no session node)
  `ldr x8,[x0]` (vtable) -> `ldr x1,[x8,#48]` (+0x30) -> `br x1(obj)`
Two recons disagreed on WHICH dispatch object/vtable (caller-dependent):
- deleg_eeec00a2: DM-root [0x106a68818] holds the object; vtable = the real
  dispatch vtable 0x10635cce0 whose +0x30 slot (0x10635cd10, RELATIVE addend
  0x2207b50) = the genuine global-init ctor 0x102207b50 (a __call_once wrapper
  over a ~18-installer construct chain; zero `this` derefs = non-faulting).
- deleg_94c0be26: on the StartLuaAppDM path the dispatched object is the
  caller's own stack union {slot0=0x635dd68 table, slot32=sp}; table+0x30 =
  0x23eff4c (a real engine-boot body -> AppBridgeV2 governor 0x2e9fa84).
The harness's seed covers the DM-root/vtable interpretation; the empirical
marker (ctor-guard self-latch) confirms REAL global-init ctor code executes.
Recon docs: deleg_fcc6cdc6 / deleg_eeec00a2 / deleg_94c0be26 + committed
docs/recon-sh156-startluaappdm-postdoinit.md.

## The .bss page-remap fix (the concrete next-gate)
The GlobalInit ctor chain reads globals on .bss pages that the engine's boot
remapping leaves UNMAPPED (the SH116 class). Empirically the ctor's
`ldr x0,[0x7285000+0xfb0]` faults at guest 0x1067285fb0 (3/3 before the fix).
FIX (elfjit.rs): new `guest_page_mapped` (/proc/self/maps check) +
`routeb_map_guest_page` — when a guest page is genuinely unmapped, mmap a fresh
zeroed anon RW page there (MAP_FIXED, guest==host); ALREADY-mapped pages are
left untouched (never clobber file-backed content). Applied (guardedly) to the
ctor's globals: flags latch [0x7285fb0], loadLocalFlags arg [0x7285fb8],
once-guard2 [0x6c347c0], telemetry [0x6dcd380]/[0x6dca000]/[0x6dce218],
thread-mutex [0x7333aac], clock-sched [0x6ed9000].

## The flags-byte gate is NOT a gate (recon correction)
recon deleg_69ab5272: [0x7285fb0] needs value word condition (byte0>=6 &&
byte1>=3, i.e. 0x306) ONLY to take a telemetry path; that path is reached only
when the thread-init (0x2207df8) returns negative, which a healthy run does not.
So we do NOT pre-seed [0x1067285fb0]=0x306; mapping the page to readable (0) is
correct (the ctor skips telemetry and continues).

## Success markers (real binary, JIT_ROUTEB_DM_SEED=1)
- `SH155 post-StartLuaAppDM: ... ctor-guard[0x6a64d70]=Some(1) ctor-flags[0x7285fb0]=Some(0)`
  — the GlobalInit dispatch ctor's OWN __call_once completed and self-latched its
  once-guard (previously always 0 / never reached).
- once-guard[0x6a68410]=0x1 (do-init __call_once, unchanged from SH155),
  once-slot[0x106a68408]=0x4000, app-data-model counter advanced.
- Full ladder EXIT 0 / 24 real task-driven frames swap Ok(0x1) / ladder done /
  SendAppEventOnAppReady / V1 AppStart / surface-handoff. 0 SIGSEGV/SIGABRT.
- Reproducible: runs/sh156-r3a.txt, sh156-r3b.txt (2/2 EXIT 0).

## Repro command
```
timeout 100 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_DM_SEED=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 --persist-roundtrip --kicker 0x106863af8
```

## Hermetic tests added (elfjit.rs example tests, +5, 61 total)
- sh156_routeb_dm_root_object_builds_live_dispatch_obj — object[0]=0x10635cce0
  dispatch vtable; pins vtable+0x30/ctor 0x102207b50/DM-root addresses.
- sh156_routeb_dm_root_object_rejects_null_buf
- sh156_guest_page_mapped_detects_mapped_page
- sh156_routeb_maps_unmapped_guest_page — maps 0x1067285fb0 anon RW, verifies
- sh156_routeb_map_already_mapped_is_noop — already-mapped page untouched (no clobber)

## Scope honesty
Opt-in (JIT_ROUTEB_DM_SEED=1); default product path unaffected (env off). The
do-init now executes real GlobalInit construction instead of soft-returning —
a verifiable forward step down the ONE-NEXT-UNSYNTHESIZED-OBJECT loop — but the
engine is now at the NEXT frontier marker: AppBridgeV2 governor startup
guest 0x102e9fa84 -> nativeAppBridgeStartAppWithParams (recon deleg_94c0be26),
which is where the next cycle's probe lands. It does NOT yet render engine
self-constructed login/home GuiObjects (structural Lua/app-shell wall remains
downstream of the governor).