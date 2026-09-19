# Open-Sober run status (hermes-worker)

Updated 2026-09-19, this session: SH350 (crossed SH349+1 terminal, 3rd persistence-lane fencepost)
then SH351 — staged the R1 synthetic CoreScript content path (Route-B marker half): a hand-authored
Luau ScreenGui module + real loader gates, latent-but-correct. Workspace green (arm64jit 418/0).

## Current state

- `dev` HEAD: SH351 (stage_r1_core_scripts + fsmap::staging_root + page_writable_rw guard + 2 sh351
  hermetic + elffjit --v2boot-r1-stage). Workspace green (arm64jit lib 418/0; elffjit example 157/0;
  cargo test --workspace exit 0; recon-v3 frame plane green).
- Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0, MH_* all false.

## What advanced this session

- SH350 (prior commit): crossed the SH349+1 terminal 0x101d9a708 with a BOUNDED single-caller
  name-pack skip (JIT_ROUTEB_LSM_PACK_SKIP); ladder advances to 175 LSM pool-pops, then the same
  run-variable live-object family — 3rd fencepost that LSM sub-call-whack-a-mole is UNBOUNDED.
  Persistence lane permanently closed.
- SH351 (this commit): staged the R1 content path. `stage_r1_core_scripts` writes a synthetic
  CoreScript (ScreenGui+TextLabel) to SOBER_ANDROID_ROOT/data/user/0/com.roblox.client/files/
  scripts/CoreScripts/{AppShell.lua,CoreScripts.lua} (both inferred candidates) + arms the real
  loader gates (flags-loaded/latch, governor union-init guards, loader-settings), each page-
  writable-guarded. +2 hermetic tests proving the module lands at the correct fsmap mirror.
- Measured (confirm, not implemented): the messageBus "experience-launch" topic is a Java-side
  runtime string (SH347's never-firing cb is NOT a topic-string bug); onAppLuaWillStart is an
  internal lambda, not an export.

## Honest status

- Route-B live-DM structural gate UNCHANGED. SH351's R1 rung is LATENT: the full ladder still
  terminates run-variable at the persistence lane before the post-ladder rung, so staging is
  staged-and-tested but not yet exercised in a completing ladder. It arms the content half of the
  Route-B marker the instant a live DM owns a session. SH174 capture-latch stays the single
  forward hook.

## Next-forward candidates

1. (PRIMARY, Route-B) The SESSION half remains THE wall: do-init must own a live DataModel (the
   four-stacked SH184/185 closure; session-ctor / StartLuaAppDM). R1 content is now staged (SH351)
   so the marker fires the moment the DM exists.
2. onAppLuaWillStart (dataModel-bindings live binder) stays migration-gated; not seedable.
3. Do NOT re-drive LSM sub-call skips (3 fenceposts of measured evidence it is unbounded).