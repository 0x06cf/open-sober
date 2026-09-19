# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this cycle: SH378 = clean readback of the single SH174 DM-allocation forward hook on the
furthest-advancing env (SH377 crossing+GOVFLAG+PRELOAD+PACK_SKIP, SendAppEventOnAppReady returns Ok). Capture-ONLY
latch (no DELEGATE — closes SH344's unread record) shows 0 validated make_shared<DataModel> AND the trail never even
installs (OP_NEW_WRAPPER not hit installably); terminal = the closed LSM pool-pop lane 0x101d9a528. The forward hook
does not fire at the farthest reach. Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false).

## Current state

- `dev` HEAD (local): SH378 (probe + frontier doc + ledger; capture_sh378_dmcap_advancing.sh; capture log at
  /home/hermes-worker/runs/sh378-dmcap-advancing.txt, outside repo).
- Workspace green (cargo test --workspace EXIT 0, 436 passed/0 failed in arm64jit lib). elfjit.rs under 1MiB hook.
- recon-v3 deliverables CONFIRMED green this cycle (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames,
  swap Ok(0x1), 197 node pops, 0 json abort, 0 crash; JIT_JSON_ZERO_FIX present).
- Route-B live-DM structural gate UNCHANGED: DM-root [0x106a68818]=0, MH_* all false.

## What advanced this cycle

- **SH378**: clean readback of the single SH174 forward hook at the farthest reach. SH344's capture used DELEGATE=1
  (disruptive, died bad_function_call before a clean readback). This cycle runs CAPTURE-ONLY on the SH377 advancing
  env: `SendAppEventOnAppReady returned Ok(0x107273d50)` (the farthest send-appevent reach IS achieved) but the trail
  never installs and `[validated]` = 0 — no make_shared<DataModel> allocates even at the furthest-forward write.
  Terminal drains to 0x101d9a528 (closed LSM pool-pop family). Confirms the Route-B live-DM wall independent of the
  SH344 delegation artifact.
- Maps-completion: the SH377 terminal 0x10284cf5c (a create-once flag setter) was caller-attributed this cycle — all
  its callers pass FIXED bss globals (adrp 6dd4000/7273000/683c000...), never NULL, so the x0=0 fault is a
  run-variable create-once fence in the persistence family, not a seedable global. Re-affirms the LSM-family closure.

## Honest status

- Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward observer. Even at the
  farthest SendAppEventOnAppReady-return reach, no DM allocates and control drains into the closed persistence lane.
  Both recon-v3 deliverables re-verified green at HEAD this cycle.

## Next-forward candidates

1. (PRIMARY, standing) SESSION half remains THE wall: do-init must own a live DM (SH184/185). The governor+preload
   walls are SH269/SH307-armed and pass; the residual is the SH350 pack-lane (measured-closed — do NOT re-drive
   LSM sub-call skips, SH349/350/358/373/375/377/378 stand).
2. R1 content half staged+armed+serviceable (SH351/352/354); fires the moment a live DM drives the loader.
3. Do NOT re-drive LSM skips; do NOT re-attack the EC reader-gate (SH356/374); do NOT re-attack 0x258b5d8/SetInitParams
   (SH362/375); do NOT re-arm window-attach once-guard (SH367); do NOT re-enter ALooper loop (SH365); do NOT re-arm
   JIT_DM_ALLOC_CAPTURE_DELEGATE (SH344/378).