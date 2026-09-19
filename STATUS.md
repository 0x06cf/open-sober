# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this cycle: SH378 + SH379 closed two never-run intersections — 
(378) the SH174 DM-allocation capture latch (CAPTURE-ONLY) is silent even on the furthest-advancing
skip-appstart env (SendAppEventOnAppReady returns Ok; 0 validated make_shared<DataModel>, trail never
installs, terminal = LSM pool-pop 0x101d9a528); (379) the GOVFLAG+PRELOAD+PACK_SKIP governor gates are
INEFFECTUAL on the FULL ladder (app-start driven) — governor/DM-creator/setDataModelToCurrent/
ScriptContext 0 hits, drains to the same closed persistence lane. Confirms the Route-B live-DM wall is
path-independent and still structural. recon-v3 deliverables re-verified green.

## Current state

- `dev` HEAD (local): SH379 (probes + frontier docs + ledger). Workspace green (cargo test --workspace
  EXIT 0, 616 passed/0 failed, arm64jit 436/0). elfjit.rs under 1MiB hook.
- recon-v3 deliverables CONFIRMED green this cycle (capture_taskv4_frame.sh attempt 1: 24 real
  task-driven frames swap Ok(0x1), 197 node pops, 0 json abort, 0 crash; JIT_JSON_ZERO_FIX present).
- Route-B live-DM structural gate UNCHANGED: DM-root [0x106a68818]=0, MH_* all false, AppBridgeV2
  [0x106a705e8]=0x0.

## What advanced this cycle

- **SH378**: clean readback of the single SH174 forward hook at the farthest reach (closes SH344's
  unread DELEGATE record). Capture-ONLY (no disruptive DELEGATE) on the SH377 advancing skip-appstart
  env: SendAppEventOnAppReady returns Ok(0x107273d50) but the trail never installs and `[validated]`=0 —
  no make_shared<DataModel> even at the furthest-forward write. Also caller-attributed the SH377
  terminal 0x10284cf5c: all its callers pass FIXED bss globals (adrp 6dd4000/7273000/683c000...), never
  NULL, so the x0=0 fault is a run-variable create-once fence in the persistence family (not seedable).
- **SH379**: the governor-crossing gates (GOVFLAG+PRELOAD+PACK_SKIP) only matter on the skip-appstart
  send-appevent env (SH376/377); on the FULL app-start-driven ladder they are INEFFECTUAL — the run
  drains into the LSM pool-pop lane (0x101d9a528) before the governor is reached (0 gov/DM-creator/
  ScriptContext hits). Refines SH376's corrected terminal sequence as path-specific; both paths converge
  on the same closed persistence lane.

## Honest status

- Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward observer.
  Both recon-v3 deliverables re-verified green. Two new never-run intersections measured negative — the
  wall is confirmed path-independent (full ladder and skip-appstart env both drain into the closed LSM
  family before any live DM / governor / Lua).

## Next-forward candidates

1. (PRIMARY, standing) SESSION half remains THE wall: do-init must own a live DM (SH184/185). The
   governor/preload walls pass only on the skip-appstart env; the residual is the SH350/378/379 pack-lane
   (measured-closed — do NOT re-drive LSM sub-call skips, SH349/350/358/373/375/377/378/379 stand).
2. R1 content half staged+armed+serviceable (SH351/352/354); fires the moment a live DM drives the loader.
3. Do NOT re-drive LSM skips; do NOT re-attack EC reader-gate (SH356/374); do NOT re-attack 0x258b5d8/
   SetInitParams (SH362/375); do NOT re-arm window-attach once-guard (SH367); do NOT re-enter ALooper
   (SH365); do NOT re-arm JIT_DM_ALLOC_CAPTURE_DELEGATE (SH344/378); do NOT expect governor gates to
   change the full-ladder terminal (SH379).