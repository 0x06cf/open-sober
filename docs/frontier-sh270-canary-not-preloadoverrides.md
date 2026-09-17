# Frontier SH270 — SendAppEventOnAppReady post-advance wall pinned to nativePreloadFlagOverrides (corrected attribution)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Route-B live-DM
structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
Default-inert; the temporary SH270 drive rung and a mis-directed SH271 canary
rung were built+measured and BOTH REVERTED (not shipped). +1 CORRECTED hermetic
sh270. Workspace green.

## Why (reconciling SH269 vs an over-correction)

SEP-17 SESSION-CTOR drive. SH269 measured SendAppEventOnAppReady (with the
GOVFLAG seed) advancing to SIGSEGV guestpc=0x102bb803c (`ldr x8,[x20]`) and
attributed x20=0 to "the preload-overrides object = never-constructed live
object". This cycle (SH270 draft) attempted to construct that object for real by
driving the lazy-singleton getter + its own ctor — and while constructing it IS
possible, the attempt exposed that the SH270 DRAFT's own label correction was WRONG
(it claimed the wall was a stack-canary deref). Full re-derivation below.

## Measured (real libroblox.so)

1. **The wall's TRUE x20 source.** The wall's real enclosing fn is entry
   0x102bb7fd4 (`sub sp,#96`). Its body reaches:
      `bl 0x102dae640`    @0x102bb801c   (nativePreloadFlagOverrides)
      `mov x20,x0`        @0x102bb8024   (x20 = the GETTER's return)
      `ldr x8,[x20]`      @0x102bb803c   (wall; faults fault=0x0 when return=0)
   0x102dae640 is a `b 0x2dae5f0` thunk into nativePreloadFlagOverrides.
   **SH269's original attribution (x20 = nativePreloadFlagOverrides return) was
   CORRECT.** The getter returns 0 headlessly → the wall derefs [0]→fault=0x0.
2. **nativePreloadFlagOverrides 0x2dae5f0 is a lazy Meyers singleton.** Entry does
   `bl 0x1057816f0` (guard-acquire helper) then `tbz w0,#0`:
   - if guard „not done" → falls to `bl 0x101df8ff8` @0x2dae624 = the engine's OWN
     ctor that zero-builds the preload-overrides object IN-PLACE at fixed-.bss
     base 0x106d2dd20 (guard via __cxa family 0x10284ce54/0x10284cf5c).
   - if guard „done" → the load path reading [0x106a64d78] (adrp 0x106a64000
     @0x2dae600 + ldr [x8,#431_i] @0x2dae604).
3. **The object IS engine-constructible headlessly** — driving the ctor
   0x101df8ff8 DIRECTLY returns Ok(0x106d2dd20) (non-NULL, first measured headless
   construction). Driving the getter 0x2dae5f0 returns Ok(0x0).
4. **Wiring is INERT.** Writing the constructed object base 0x106d2dd20 into
   [0x106a64d78]/[0x106a64d98] does NOT change the wall: SendAppEventOnAppReady
   still faults at the IDENTICAL guestpc=0x102bb803c fault=0x0. So the getter's
   effective return at the wall does not come from those cells (or is re-zeroed),
   closing the seed-wire lever.

## Correction of the SH270 draft's own mislabel (IMPORTANT)

The first version of this cycle over-corrected: I saw `adrp x20,0x67d1000; ldr
x20,[x20,#0x6f0]` = [0x1067d16f0] near the wall and declared the wall a
STACK-CANARY deref. That was WRONG:
- The wall's REAL enclosing fn is 0x102bb7fd4, and ITS x20 is set by
  `mov x20,x0` @0x102bb8024 (the getter return), NOT by a canary prologue load.
- 0x102bb786c's x20 adrp belongs to a DIFFERENT earlier fn that ends at a `ret`
  before the wall.
- 0x102bb7fe8/0x102bb7ff4 load [0x1067d16f0] into **x21** (not x20) as THIS fn's
  own stack-protector (the canary of fn 0x102bb7fd4) — real, but unrelated to the
  wall deref and untouched by it. A canary init rung (SH271, built+measured
  REVERTED) confirmed inert: it did NOT move the wall.

## Verdict (do-not-re-tread)

- **SH269's attribution stands: guestpc=0x102bb803c derefs x20 = nativePreload-
  FlagOverrides' return (0 headlessly).** The blocker is the getter returning 0,
  i.e. the preload-overrides singleton's effective value is not produced on the
  session path even though its ctor can build the object in isolation.
- **Do NOT re-drive a seed/wire into [0x106a64d78]/[0x106a64d98]** — MEASURED inert
  (the getter's return at the wall is unaffected). The advance here needs the
  getter's once-guard + value cell to reflect a real construction produced during
  the session, not a host wire.
- Keep the corrected hermetic so a future cycle does NOT re-misattribute the wall
  to a canary/different-fn register.

## Honest (do-not-over-claim)
Does NOT manufacture a DataModel; DM-root stays 0; MH_* stay false; Route-B
live-DM gate UNCHANGED. New+measured this cycle: (a) the wall's TRUE x20 source
chain pinned (bl 0x102bb801c→0x102dae640→0x2dae5f0, mov x20,x0, ldr [x20]); (b)
the preload-overrides object IS engine-constructible in isolation (ctor Ok
0x106d2dd20); (c) wiring is inert; (d) the draft's canary mislabel corrected
(x21 self-protector vs x20 getter-return, distinct fns).

## Code / verify / artefacts
- elfjit.rs hermetic `sh270_preload_wall_is_canary_cell_pinned` (name kept;
  now the CORRECTED attribution — real-image guard, skip-if-absent: wall fn
  prologue 0x102bb7fd4=0xd10183ff, wall bl 0x102bb801c=0x9407d989, mov
  0x102bb8024=0xaa0003f4, wall 0x102bb803c=0xf9400288 + post 0x102bb8044, thunk
  0x102dae640=0x17ffffec, getter 0x2dae5f0/0x2dae5fc=0x36000140/0x2dae624=
  0x97c12a75, ctor 0x101df8ff8=0xa9be7bfd).
- Verify: `cargo test -p arm64jit --example elfjit sh270` = 1 passed.
- Repro logs (SH270 preload/wire, SH271 canary) runs/sh27x*.txt (gitignored).
- Commit: local `dev` only.