# Frontier SH308 — SendAppEventOnAppReady's pipe now drives the do-init app-shell ctor through its audio tail

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed).
Confirmed forward reach of the SH307 lever (already committed at HEAD 47c25a3),
measured fresh at the newest HEAD on real libroblox.so.

## What was measured (this cycle)

Re-running the SH307 A/B canonical ladder with JIT_REGION_WATCH on the
SendAppEventOnAppReady pipe/do-init/EC/governor bands:

- **A (SH307 off, GOVFLAG on):** SendAppEventOnAppReady still dies at the
  standing wall `guestpc=0x102bb803c` (EXIT 139, x20=0). **Zero** do-init /
  app-shell-ctor / FMOD / StartAppWithParams region hits — the construction
  never runs on this path.
- **B (JIT_ROUTEB_PRELOAD_VALUECELL=1, the SH307 forward):** `SendAppEventOnAppReady
  returned Ok(0x107273d50)`, 0 SIGSEGV, ladder completes clean (LADDER_DONE=1,
  EXIT 124). And its app-bridge pipe now actually drives construction:
  - do-init **0x102206c40 entered** + its body walks ~26 block-entries to
    **0x10220703x** (once-lambda / flags / app-data-model register),
  - the **app-shell ctor 0x102207b50 runs 31 blocks** through **0x102208eac**,
  - its **FMOD/AAudio audio-iterate tail** (0x5fb30b4, entered via the app-shell
    tail `b 0x5fb30b4` at 0x102208ebc) executes **0x105fb310c..0x105fb31b4**
    — the audio-init body (sub sp,#0x30, registry init at 0x6dcb160, canary),
    not just the empty-early-return,
  - **nativeAppBridgeV2StartAppWithParams (0x10258b144) is reached** (18 region
    hits through 0x10258b5a0) — the app-bridge pipe converges on the real
    StartApp path.

These do-init/app-shell-ctor/StartAppWithParams executions are FIRSTS on the
SendAppEventOnAppReady path (baseline A: 0 hits everywhere).

## Honest (do-not-over-claim)

- Does NOT manufacture a DataModel: DM-root [0x106a68818] stays 0, MH_* all stay
  false, post-do-init continuation 0x1023eff4c / ScriptContext loader 0x101f1d8ac
  / CoreScript cells remain 0 hits. The app-shell ctor runs but the session half
  of the pipe (governor, ScriptContext, Lua) is still gated on a live DM.
- `w19-event=0x0` after the rung (the 'Home' discriminator needs w19=4). Across
  every run logged (46) w19 is only ever 0x0 or 0x1, never 0x4 — the fabricated
  "Home" jstring still does not resolve to a recognized event (the Step-2
  real-jstring ABI gap remains open; noted, not solved here).
- SH307 is the executor of this reach; SH308 pins it as a regression anchor.
  Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the
  single forward hook.

## Verify / files

- `cargo test -p arm64jit --example elfjit -- sh308` = 1 passed (real-image
  pins: app-shell ctor 0x102207b50=0x14000001, ctor tail 0x102208ebc=0x14f6a87e,
  FMOD iterate 0x105fb30b4=0xd10183ff / empty-check 0x5fb30e0 / audio-body
  0x105fb3174, StartAppWithParams 0x10258b144=0xd103c3ff; in-window + 4-aligned).
- `cargo test --workspace` EXIT 0; `cargo build --workspace` EXIT 0.
- elfjit.rs held under the 1MB pre-commit hook (margin ok).
- Repro runs/sh308-rw.txt, runs/sh308-rw2..rw5.txt (region-watch captures),
  A/B in runs/sh307-appev-A.txt / runs/sh307-appev-B.txt.
- Commit: local `dev` only (operator pushes).