# SH280 — drive the initEngine_ settings-state machine's state=5 body headlessly for the first time

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · extends `--v2boot-session-engine5` (default-inert) · workspace green (578/0, examples 113/0)

## What this is

SH279 crossed the state=3 settings serializer (initEngine_ ==3 branch → full serializer body →
self-drive into app-start depth). But the initEngine_ state machine has TWO more bodies that were
**never driven**: state=5 (0x2bd24b4) and state=9 (0x2bd2668), both reachable from the dispatch at
0x2bd1d10 (b.eq) / 0x2bd1d18 (b.eq). This cycle drives state=5 for the first time.

## The mechanism (fresh disasm + measured)

The state=5 body (0x2bd24b4):
- `ldrb w8,[x19,#72]` + `ldr x9,[x19,#80]` + csel/cbnz (0x2bd2528..0x2bd2544) = the SAME app-name
  guard class as SH279's Gate 2. With the SH279 manager ([this+0x48] LONG "Home", so cap bit0=1 →
  csel picks cap>>1 = nonzero) the guard is satisfied → skips the NULL-store fault.
- `ldr x20,[x19,#64]` = [this+0x40] = the SH279-seeded settings-config object.
- builds a stack std::pair from x23/[config-derived] and `bl 0x2bce0d4` with x0=config, w2=1.
- 0x2bce0d4: `ldr x0,[x0,#56]` (reads [config+56], the config's sub-object), then `tbz w2,#0` —
  w2=1 → NOT taken → `mov w2,wzr; b 275a0c4` (GlobalInit-reentry).

The GlobalInit-reentry 0x275a0c4:
- once-guard [0x6a68410] gate (already latched by the session drive) → takes the body
- `bl 220671c` (gameGlobalInit sub) + the config+56-derived x21
- **fault** at guestpc=0x10275a148/0x154 `ldr x0,[x21,x8]` fault=0x0, because x21=[config+56]=0 on
  our fabricated zeroed config → this is the settings-config CONTENT live-object field, the
  structural gate (SH174/204 class) reached from the engine's own settings-state path.

## Measured (real libroblox.so, full SH279 seed env, runs/capture_sh280_state5_direct.sh)

**Deterministic 3/3**: `SH280 driving initEngine_ state=5 body DIRECT` fires, and every run
terminates at guestpc=0x10275a148 (SIGSEGV → SIGABRT, EXIT 134). This is the FIRST headless
execution of the state=5 settings body and the FIRST time the engine's OWN session-state machine
advances 3→5→6 (state written to [this+16]=6) into the GlobalInit-reentry.
A/B baseline (engine3 only, no engine5): unchanged terminal guestpc=0x101db1d04 (SH260 LSM wall)
— the state=5 crossing is attributable to the new rung.

## Honest framing

This is cause-not-symptom on the SEP-17 SESSION-CTOR primary lever: the engine's OWN settings
state machine now executes a body it never reached before and advances one machine-step past
SH279's LSM wall (to the settings-config content wall). It does NOT manufacture a live DataModel
(MH_FLAGS_LOADED=false, MH_APP_READY=false, DM-root 0 — see runs/sh280-e5-1.txt) and the config
content wall is the SH174/204 live-object structural gate, not seedable by a constant. SH174
capture-latch stays the single forward hook. Unlike the state=3 serializer (which self-drives and
never returns), driving state=5 DIRECTLY as its own jit_run is the correct pattern — a chained
re-dispatch after the =3 body is latent (measured: the =3 body dies at the LSM wall before
returning).

## Next (honest, single-agent)

The state=9 body (0x2bd2668) is the same class and also undriven — it does NOT add a new
state-transition beyond state=5's (both dispatch config via 0x2bce0d4 w2=1 → 275a0c4), so it is
not re-attacked as a fresh lever. The config-content wall ([config+56] sub-object) is structural.
Route-B live-DM gate UNCHANGED.

## Verify

`cargo test -p arm64jit --example elfjit sh280` = 1 passed (real-image pins: state=5 body
prologue/app-name guard/state->6, config dispatch 0x2bce0d4 ldr+mov+b, GlobalInit-reentry prologue
+ bl 220671c + fault site ldr [x21,x8]); examples **113/0** (was 112); `cargo build --workspace` +
`cargo test --workspace` EXIT 0 (578). elfjit.rs kept under the 1MB pre-commit hook (1,048,530 B).
Repro runs/capture_sh280_state5_direct.sh + runs/sh280-e5-{1,2,3}.txt + runs/sh280-baseline.txt.
Single-agent, default-inert.