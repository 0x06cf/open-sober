# Frontier SH270 — the SendAppEventOnAppReady post-advance wall is the STACK CANARY, not a preload-overrides object (label correction)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Route-B live-DM
structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
This is a MEASURED LABEL CORRECTION + closure of a tangent — SH269 parked the
session-ctor drive at a wall it attributed to a "preload-overrides live object";
fresh disasm + a decisive A/B show that attribution is WRONG (it is the
SH182/SH256 stack-canary pointer deref). Default-inert; the temporary SH270
drive rung was built, measured, and REVERTED (not shipped). +1 hermetic sh270.
Workspace green.

## Why (a genuinely open attribution worth correcting)

SEP-17 SESSION-CTOR directive: drive the REAL engine session (cause-not-symptom),
don't re-drive single-object seeds. SH269 measured the SendAppEventOnAppReady
rung advancing past the governor NULL-controller deref (0x102ea0b9c, GOVFLAG
seed) to SIGSEGV guestpc=0x102bb803c and attributed x20=0 to "the
preload-overrides object [0x106a64d98] = never-constructed live object
(SH174/204 class)". If that were true, the next forward lever would be to
construct/fabricate that object. This cycle tests that premise directly: can the
preload-overrides object be built by driving the engine's own lazy-singleton
getter/ctor?

## Measured (real libroblox.so, full SH269 seed set + GOVFLAG)

1. **nativePreloadFlagOverrides (0x2dae5f0) is a lazy Meyers singleton.** Entry:
   `bl 0x1057816f0` (guard-acquire helper) then `tbz w0,#0`; on the
   not-yet-initialized path it falls to `bl 0x101df8ff8` (0x2dae624) = the
   engine's OWN constructor that zero-builds the object IN-PLACE at fixed-.bss
   base 0x106d2dd20 (guard via the __cxa family 0x10284ce54/0x10284cf5c).
   Driving the getter directly: **returns Ok(0x0)** (its guard is already
   latched-initialized headlessly with a NULL object, so it takes the LOAD path).
2. **Driving the ctor 0x101df8ff8 DIRECTLY: returns Ok(0x106d2dd20) — non-NULL.**
   The object IS constructible headlessly by engine code (cause-not-symptom would
   work). But [0x106a64d78]/[0x106a64d98] stay 0 after (the ctor does not store
   into those consumer cells).
3. **DECISIVE A/B: wiring the constructed object base (0x106d2dd20) into
   [0x106a64d78]/[0x106a64d98] did NOT move the wall** — the SendAppEventOnAppReady
   rung STILL SIGSEGVs at the IDENTICAL guestpc=0x102bb803c fault=0x0. This proves
   x20 there does NOT come from those cells.
4. **Fresh disasm of the actually-enclosing fn (entry 0x102bb785c, `sub sp,#0x50`):
   x20 is loaded at PROLOGUE** via
   `adrp x20,0x67d1000` (d001e0d4 @0x102bb786c) + `ldr x20,[x20,#0x6f0]`
   (f9437a94 @0x102bb7878)  =>  x20 = **[0x1067d16f0]**, then the wall
   `ldr x8,[x20]` @0x102bb803c derefs it. **[0x1067d16f0] is the SH182/SH256
   STACK-CANARY pointer global** (SH256's "map root" false positive: fn
   prologues read x20=[0x1067d16f0] as the canary pointer, the epilogue re-reads
   [x20] vs the saved canary, so writing it deterministically trips __stack_chk_fail).

## Verdict (do-not-re-tread)

- **SH269's "preload-overrides object [0x106a64d98]" attribution at guestpc=
  0x102bb803c is WRONG.** The wall is a NULL deref of the stack-canary pointer
  [0x1067d16f0] (0 headlessly: the canary value only lives when a real session
  initializes it). The preload-overrides singleton is REAL and CONSTRUCTIBLE by
  the engine's own ctor — but it is a separate concern, NOT what feeds this wall.
- **Do NOT re-drive a preload-overrides seed to cross 0x102bb803c** — it is inert
  (measured) and the real blocker is the canary-pointer cell, which is unseedable
  by the SH256 proof (a fabricated canary → __stack_chk_fail). The session-ctor
  drive's next genuinely-forward step remains upstream: a session that
  initializes the process canary / constructors headlessly, i.e. the same
  structural cause the SEP-17 directive targets — NOT this seed.
- Measured closure of the tangent, saving a future wasted cycle.

## Honest (do-not-over-claim)
Does NOT manufacture a DataModel; DM-root stays 0; MH_* stay false; Route-B
live-DM gate UNCHANGED. What IS new+measured: (a) the preload-overrides singleton
is engine-constructible headlessly (first time its real ctor returned a live
object base 0x106d2dd20), and (b) the SendAppEventOnAppReady post-advance wall is
correctly identified as the SH182/SH256 stack-canary pointer deref, correcting
SH269's mislabel with two independent measurements (decisive A/B + fresh disasm).

## Code / verify / artefacts
- elfjit.rs hermetic `sh270_preload_wall_is_canary_cell_pinned` (real-image
  guard, skip-if-absent: wall-window prologue 0x102bb785c=0xd10143ff, x20-load
  chain 0x102bb786c/0x102bb7878, wall 0x102bb803c=0xf9400288 + post 0x102bb8044,
  getter 0x2dae5f0/0x2dae5fc/0x2dae624, ctor 0x101df8ff8, canary cell
  [0x1067d16f0] host-resolvable=.bss).
- Verify: `cargo test -p arm64jit --example elfjit sh270` = 1 passed.
- Repro (during development, A/B): drove getter 0x2dae5f0 (Ok(0x0)), ctor
  0x101df8ff8 (Ok(0x106d2dd20)), wired into [0x106a64d78/98] -> identical
  0x102bb803c fault. Logs runs/sh270-preload*.txt (gitignored).
- Commit: local `dev` only.