# SH92 — substitute non-family map/this at the INSERT entry (clears the OTel registrar loop)

## Result (wall advance: the ladder runs the FULL OTel/pb_defaults registration with NO crash; the registrar loop gate is CLEARED)

Read-only recon deleg_03083bd6 (disasm-verified) root-caused the post-SH91 fault
(guestpc 0x1029b3828, fault==rip==host-heap/stack — execution jumped INTO a host
stack address): the OTel registrar loop (file 0x29b3814) reads its INSERT map from
`[0x106838380]` via `ldr x0,[x21,#896]`, and that slot was holding **0x1067da308** —
the `.data` descriptor-table base (16-byte-strided records), a NON-coherent object,
NOT the SH88-seeded empty map. Why it slipped through: SH88's substitution predicate
is `v < 0x100000000` (sub-image tag); 0x1067da308 is IN-IMAGE (>= base) so it passes,
the guest stores it into the registry slot, and INSERT's +0x18-repair also skips it
(`[map+0x10]`=[0x67da318]=0x00a80000003f0060 is not an in-image family hash ->
`continue`). INSERT then ran with a garbage map; its `+0x30` stack-spill slot became
the code-fetch target (crash raw[] decodes 1:1 against INSERT's prologue spills).
A new instance of the SH86b "foreign object passed as map" class.

**Fix (jit.rs routeb-map-op hook):** extend substitution to the INSERT entry
(0x1029f3e70): overwrite a candidate x0/x19 with `routeb_substitute_map()` iff it is
a NON-FAMILY map/this —
  m==0 or m<0x100000000        -> invalid/sub-image  -> substitute (SH88/92)
  else [m+0x10] not in {0x1029b4a84 (span), 0x102a25dec (string)} -> non-family -> substitute
A real family map's +0x10 IS one of those hashes and is never overwritten; the
seeded substitute itself has +0x10==SPAN_HASH so it is never re-substituted
(idempotent). Gated by JIT_ROUTEB_HASHFIX (--v2boot), product-safe (register overlay
only, never repoints +0x00).

**Verified:** SH92 substitution fires ~1.25M times and the SIGSEGV at 0x1029b3828 is
GONE on every run. The ladder now runs the FULL OTel/pb_defaults descriptor
registration without faulting: `nativeInitializeNativeFlags` returns cleanly
(stopped: pc 0xd3d outside image — a benign return), `nativeGameGlobalInit` runs to
timeout (EXIT 124, stable idle) with the render thread producing frames
(renderframe-drive frame-fn + post-frame swap Ok(0x1)) CONCURRENTLY. RUN3 shows the
whole boot path running StartApp through the post-run phase with frames. No more
abort/SEGV in the registration path. Workspace **516/0** (new regression
`routeb_hashfix_substitutes_nonfamily_map_at_insert_entry`). Product path unregressed
(exit 124, persist byte-exact, 0 crash). 4 pre-existing tests that "failed" in the
same run were environmental ("No space left on device" — /tmp filled by old trace
dumps); they pass after /tmp cleanup. The substitution is gated behind
JIT_ROUTEB_HASHFIX (--v2boot path).

## Repro

`runs/capture_v2boot_sh82.sh`. Expect `SH92 substituted ... non-family map/this`
lines (many), `SH88 substituted`/`SH91` lines, and NO SIGSEGV — the ladder runs the
registration to completion / timeout (exit 124) with frames swapping.

## Next (ranked)

1. nativeGameGlobalInit STILL does not RETURN (the Main thread parks in the nanosleep
   poll at 0x10284d114/0x10284d134, never fed the next init step — the standing
   gameGlobalInit park, SH82). With the OTel registration now completing, drive the
   park-feed: either the ROUTE-B latch [0x72739d4] bit0=1 (TaskScheduler flags-loaded
   gate) needs the fill, or the next globalinit step must be injected. Goal (a) from
   the recon-routeB doc: gameGlobalInit RETURNS so rung 2 nativeUpdateAdapterInit
   (0x10221c3ec) runs and rungs 2-6 install the type-4 producer vector [0x106829ea8].
2. Then wire the NativeHelper callbacks (onFlagsLoaded -> onEngineInitialized ->
   onAppReady -> onDidLogInReceived -> onGameLoaded) so StartLuaAppDM advances the
   session and the Lua app-shell builds REAL GuiObjects.
Standing structural wall (real self-constructed login/home session) unchanged.