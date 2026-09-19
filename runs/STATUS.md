# Open-Sober run status (hermes-worker)

## SH479 (this cycle): close the HOST half of the Route-B G2 'Home' fabricate pipeline as a tested contract
SH475 pinned only the pure SSO-decode fn + discriminator bytes, leaving the fabricate path
open; SH339 MEASURED the fabricated "Home" jstring materializing as a 6-byte SSO (event 0,
not 4). New hermetic `home_fabricate_materializes_size4_routes_event4` pins the 3-stage host
pipeline: new_string_utf_handle("Home") reads back exactly "Home" (4B), GetStringUTFLength
reads 4, and the libc++ SSO header (b0=0x08) feeds the decode -> operator-pinned event 4;
plus the negative guard (SH339's size-6 0x0c decodes to 0, never 4). Test-only jni.rs;
arm64jit 684/0; workspace EXIT 0. Honest: not a live-DM step (DM-root 0 structural), not a
fix of the deeper-route size-6 (walled downstream) — the host fabricate half is now proven
correct-by-construction.

## Cycle opening (ingested into SH479)
- Workspace green at SH478 HEAD (cargo test --workspace EXIT 0; arm64jit 683/0); recon-v3
  deliverable baseline unchanged-green (24 task frames).
- Audited the authoritative recon-framework-boot-order.md params map against the code:
  every InitParams/StartAppParams/DeviceParams/PlatformParams/DisplayMetrics/Configuration
  value + getAllocatableBytes is wired AND pinned through the real JNI dispatch.
- Do-init capture (capture_sh415): SIGABRT at atom 4/16 (V2InitWithParams) — the documented
  run-variable pre-existing nativeInit "outside image" wall, not a regression.
- Route-B live-DM gate UNCHANGED (DM-root 0, structural per SH462/467/SH397).

## Current state
- `dev` HEAD = SH477 (3fbd2a2). Local commit SH479 is the test-only jni.rs hermetic + this
  STATUS + repo HANDOFF.
- Workspace green (cargo test --workspace EXIT 0; arm64jit 684/0).
- recon-v3 deliverables green (self-driven task frames swap Ok(0x1), 0 json abort, 0 crash).
- Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0 (structural per SH462/467).

## Honest status
- No live DM. SH479 is a BUILD-THE-RUNTIME host-contract completion (the G2 'Home' fabricate
  half proven correct-by-construction), latent-but-correct — not a Route-B seed.

## Next-forward candidates
1. (standing, TOP) do-init completeness / live-DM: aligned lever is the session-ctor /
   runtime-surface drive (engine's OWN session constructs the DM). Real APK assets staged.
2. The SendAppEvent 'Home' DEEPER-route ABI (SH339 size-6) remains open and downstream of
   the live-DM wall; SH479 proved the HOST fabricate half is correct, so any residual size-6
   on a live session is not a handle/length drift.
3. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged).
4. (CLOSED) DeviceParams viewport Mm (SH476).
5. (CLOSED) LocaleList size()/get() flattening (SH477).