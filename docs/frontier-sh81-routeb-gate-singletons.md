# SH81 — Route-B crash chain cleared: dispatch-gate force + singleton-record seed

## Result (wall ADVANCE: v2boot ladder now runs CRASH-FREE, rung 0 completes)

Two complementary patches remove the SIGSEGV chain that has halted the
`--v2boot` ladder since SH80. Real libroblox.so now runs the ladder **without a
single SIGSEGV/SIGABRT** and exits **124 clean** (stable idle), with the full
productized render + persist baseline green in the same run. `nativeInitializeNativeFlags`
(rung 0) **returns**; `nativeGameGlobalInit` (rung 1) is then driven (still
parking in the run window — next gate, see below).

Recon: read-only subagent deleg_5ebaa5f9 (disasm of the same binary).

## Root causes (both gated dispatch-singleton mechanisms)

1. **Dispatch accessor low-bit gate.** The engine's generated dispatch stubs
   all do `bl 21730ec; tbz w0,#0,<fb>`. Accessor `21730ec` returns `query & 1`
   (its tail `and w0,w0,#1` at file 0x2173124). On a headless boot the "subsystem
   initialized?" query returns 0, so every site takes a FALLBACK singleton-lookup
   path (`6249e9c`/`6249eb8` -> `2b9dee0(&0x6829a48/&0x6829a68)`) whose lazy-created
   object has a NULL vtable -> `ldr x8,[x8,#48]; blr x8` SIGSEGV (SH80 crash at
   guest 0x10624f46c).
2. **Direct singleton fallback.** Callback `0x624f4bc` reaches the SAME singletons
   through its own `cbz x1,0x624f4f0` branch (independent of the gate), so forcing
   the gate alone is insufficient: after the gate force the crash merely MOVED to
   guest 0x10624f500 at that direct path's `ldr x8,[objB]`.

## The fixes (neither weakens anything; both idempotent)

**Gate force** (`routeb_patch_dispatch_gate`): patch file 0x2173124 LE u32
`0x12000000` (`and w0,w0,#1`) -> `0x52800020` (`mov w0,#1`), then
`block_cache_drop_region(0x1021730ec,0x102173138)`. Since the mask already
collapsed w0 to bit0, forcing 1 changes nothing observable except that every gated
site takes its CLEAN DIRECT path (e.g. 0x624f41c -> `1db1050 / 224d550 / 224d5b4 /
224d600 / 240a1b0`, the last StartLuaAppDM-adjacent) instead of the singleton
fallback.

**Singleton-record seed** (`routeb_seed_task_singletons`): `2b9dee0`'s create path
(0x2b9e030) does `memcpy(new,[rec+24],[rec+0]); store` for each `.data` singleton
record; with the records all-zero it produced an all-zero 8-byte stub (NULL
vtable). Seed both records (`{+0 size=0x28, +8 allocclass=8(pow2), +16 ticket
(lazy), +24 src=host template}` at guest 0x106829a48 / 0x106829a68) so the
accessor returns a coherent object whose `+0` is a leaked vtable whose every slot
is `routeb_singleton_leaf` (a registered `register_host_call_auto` thunk returning
its first arg, never dereferencing).

Both are applied in the `--v2boot` block before the ladder thread spawns, so the
seed precedes the ladder's first accessor call (per-thread registry means it
creates fresh from our template regardless of any earlier main-thread access).

## Empirical (runs/sh81-v2boot-gate.txt, exit 124)

```
[elfjit:routeB] patched dispatch-gate `and w0,w0,#1` 0x102173124 (12000000) -> `mov w0,#1` (52800020)
[elfjit:routeB] benign singleton virtual registered at 0x7f00000001c0
[elfjit:routeB] seeded dispatch singletons .data 0x106829a48/0x106829a68 ...
[elfjit:v2boot] after boot start: [0x106829ea8] = 0x0
[elfjit:v2boot] driving nativeInitializeNativeFlags @ guest 0x10232048c
[elfjit:v2boot] after nativeInitializeNativeFlags: [0x106829ea8] = 0x0   <- RETURNED (rung 0 done)
[elfjit:v2boot] driving nativeGameGlobalInit @ guest 0x102206404
SIGSEGV/SIGABRT count = 0 ; EXIT=124 (stable idle, timeout)
```
Productized baseline intact in the same run: persist 45B byte-exact,
taskv4 present #0 swap Ok(0x1), triangle centroid RGBA(255,0,0,255),
textured quad BL=RED/BR=GREEN/TR=WHITE/TL=BLUE, 3 quad-loop frames, renderinit
Ok, 0 crash.

## Next (ranked)

`nativeGameGlobalInit` (rung 1) still does not return in the ~50 s window after the
latch is set — it appears to park (nanosleep poll) beyond the flags latch. Per
recon-routeB the intended full ladder is ONE jit_run ordering: latch ->
nativeInitializeNativeFlags -> nativeGameGlobalInit -> setTaskSchedulerBackgroundMode
-> V2InitWithParams -> StartLuaAppDM -> V2StartAppWithParams. Options:
(a) give rung 1 longer / check what gameGlobalInit waits on after the flags latch
(TaskScheduler init on the real main thread vs the ladder thread); (b) wire the
NativeHelper `gameActivity_*` callbacks into RegisterNatives so StartLuaAppDM can
construct the first GuiObject tree once reached; (c) drive rungs 2-6 (the
no-GlobalInit probe `--v2boot-r246`) now that the crash chain is gone, to see if
any later bridge native installs the type-4 producer vector. Standing structural
wall: type-4 vector [0x106829ea8] still framework-glue seeded.

Workspace **510/0** (+1 regression `routeb_dispatch_gate_force_reroutes_clean_path`),
example **22 pass**.