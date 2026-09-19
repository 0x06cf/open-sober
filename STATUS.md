# Open-Sober run status (hermes-worker)

Updated 2026-09-21, this cycle: SH407+SH408 — two probes decisively characterized the do-init
MAIN-arm terminal. SH407: the app-start MAIN body [0x10258b5d8,0x10258bbb0] now runs END-TO-END
(biggest app-start reach on record; SH362-404 called it unreachable — all 14 block-entry pcs fire),
then drains into the SH341 pool-pop persistence lane (190 valid-key pops, SETFIX fired); DMCONT
0x102bd1d68 = 0 real hits from the MAIN arm (region-watch distinction vs the spec-string count).
SH408: same env + files-dir/R1 rungs (which did NOT fire — unreached) shows LSM init ADVANCES
through initStorageManagerNative + crosses the SH285 reader (0x101db1b08 now 0 hits = old fault
terminal bypassed) then next-faults at fault=0x0 in the opnew/insert band — the standing unbound
whack-a-mole, one fencepost deeper, not a DM advance. New hermetic sh407 byte-pins the full
app-start body span + the LSM-init deepen (arm64jit lib 455->456).

## Current state

- `dev` HEAD: SH407 (app-start MAIN body full-span hermetic + frontier doc; workspace green,
  456 arm64jit lib tests; elfjit.rs untouched, jit.rs + hermetic only).
- Workspace green (cargo test --workspace EXIT 0 re-confirmed this cycle).
- recon-v3 deliverables green (24 real task frames swap Ok(0x1), producer INERT, JSON fix).

## Session's advances (SH406 -> SH407+SH408)

- **SH406**: extended SH269 GOVFLAG seed to the SH405 MAIN-arm app-start continuation; the
  0x10258b5d8 body's continuation PASSES the NULL-controller deref (0x1025f501c) and drains into
  the SH341 persistence lane; DMCONT 0.
- **SH407** (this cycle): measured the MAIN-arm terminal — the app-start body runs END-TO-END
  (0x10258b5d8..0x10258bbb0 all 14 block-entry pcs fire, nothing past), then 190 valid-key
  pool-pops (SETFIX fired) and EXIT 139 (host-side). DMCONT = 0 real hits; the MAIN arm drains to
  the standing LSM persistence lane, not do-init-completion (closes SH405's frontier question).
- **SH408** (this cycle): same env + files-dir/R1 rungs (seed unreached) — LSM init ADVANCES
  through initStorageManagerNative + crosses the SH285 reader (0x101db1b08 = 0) then faults
  fault=0x0 in opnew/insert band (whack-a-mole one deeper; SH385 node value needs a real LSM ctor).

## Honest status

- Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2
  genuine vt 0x1063a3410 unchanged). No DM manufactured; DMCONT unreached from the MAIN arm
  (reachable only via the settings-state/SH371 env, where it also terminates at the LSM lane SH372).
- The app-start MAIN body is now fully cleared end-to-end (the biggest SESSION-CTOR reach on
  record); the persistence lane is measured-returned at two successive fenceposts (reader crossed,
  opnew/insert fault). Only a real session ctor (LocalStorageManager, do-init completion) crosses
  it — the standing SESSION-CTOR / BUILD-THE-RUNTIME deliverable.

## Next-forward candidates

1. (standing, TOP — Route B) The persistence lane is measured-returned at two fenceposts (SH407/408).
   The app-start body is clear; the forward is the real session-compat runtime surface (SH400
   substrate + a real LSM/EGL/onAppReady host drive) so the engine SELF-constructs its
   LocalStorageManager + DM. Do NOT re-drive LSM skips (SH349/350/358) or re-manufacture the map
   (SH396).
2. DMCONT 0x102bd1d68 = 0 from the MAIN arm; reachable only via the SH371 settings-state env.
3. R1 content half staged+armed (SH351/352/354); fires the instant a live DM drives the loader.
4. Do NOT re-tread: setDataModelToCurrent (SH388), LSM crossings/composed (SH385/393/396),
   SH285/SH341 family (SH349/350/358/373/395-398), EC reader-gate (SH355/356/374), 0x258b5d8/
   SetInitParams (SH362/375), window-attach real (SH367), ALooper (SH365), governor-gates (SH379),
   once-lambda store (SH381), app-cmd 1/13/15/17/18 (SH393).
5. Do NOT run the SH174 latch without JIT_DM_ALLOC_CAPTURE_DELEGATE=1 (SH395).