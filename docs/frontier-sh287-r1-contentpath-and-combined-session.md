# Frontier SH287 — R1 content-path staging + combined SESSION-CTOR measurement at HEAD

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert.

## tl;dr

Two forward artifacts at the newest HEAD (5886ef0, SH286):

1. **R1 content-path staging is now a tested artifact.** Every Route-B content
   *gate* (flags-loaded 0x72739d4, flags-latch 0x106a683e8, governor union-init
   guards 0x106a63da0/0x106a63d70, loader settings 0x106ba3350) was already
   seeded in code — but the actual **content module** the CoreScripts loader
   consumes was UNLANDED: `$SOBER_ANDROID_ROOT` only had `databases/session.db`,
   no `files/scripts/CoreScripts/`. This cycle adds the hermetic proof that the
   engine's loader path
   `/data/user/0/com.roblox.client/files/scripts/CoreScripts/AppShell.lua`
   resolves through fsmap to a real on-disk module that survives a fresh boot
   (`r1_synthetic_corenodes_stages_persistent_module`, fsmap suite 8/8) and
   stages the module at the real persistence root. This is the content gate
   that turns a nonzero R+0x180/0x188 probe into rendered home UI the instant
   do-init completes a live DM. Latent-but-correct (honest dependency: the
   engine only *issues* the rbxasset://scripts/CoreScripts request after
   do-init owns a live DM, which never happens headlessly yet).

2. **A genuinely-new combined-state measurement** (never run together before):
   the full SEP-17 SESSION-CTOR stack — `--v2boot-session` lifecycle +
   `--v2boot-surface-handoff` (G1 XID) + `--v2boot-send-appevent` ('Home' in x5,
   G2) + `--v2boot-session-signed` (client-settings) — ON ONE ladder alongside
   the settings-state drive. Measured (2x A / 2x B, real libroblox.so):
   - G1 wired XID lands: `surface-handoff: wired_xid=0x200000
     [0x10683d348]=0x200000` (V2UpdateSurfaceAppWithPlatformParams returned Ok).
   - SendAppEventOnAppReady drives with 'Home' in x5 and terminates at the
     **SH270 preload-overrides live-object wall** `guestpc=0x102bb803c`
     (x20=nativePreloadFlagOverrides return=0) — the known, already-classified
     SH174/204 live-object gate, NOT a new fencepost.
   - **Stacking the session rungs SUPPRESSES the engine9 settings-state drive**
     (0 state=9 markers in the combined B runs; state9ok fires 3/3 only under
     the SH285-isolated command). The settings-state self-drive needs its own
     single engine9 drive; the combined env perturbs it (SH248b-class latency
        shift, not a regression of either rung).

Repro: `runs/capture_sh287_full_session.sh` (A/B combined). Runs kept
locally (sh287-a{1,2}/b{1,2}.txt, gitignored as run captures).

## Honest (do-not-over-claim)

- The R1 module is staged + tested, but does NOT render anything yet: the
  engine's CoreScripts loader / data plane is never reached headlessly
  (asset-trace fired ZERO rbxasset and fsmap ZERO remap requests across both
  combined runs — terminal always the pre-load live-object wall). This is the
  exact handoff'd honest dependency: R1 fires only when do-init completes.
- Does NOT manufacture a DataModel; DM-root[0x106a68818]=0; MH_* stay false;
  Route-B live-DM structural gate UNCHANGED. SH174 capture-latch stays the
  single forward hook.
- SH285-B's crossed LSM reader/pop terminal (0x101db1b08) stays SH286-classified
  (live-object, do-not-re-drive-repair); no new seed manufactured into it.

## Verify

- `cargo test --workspace` EXIT 0 (400 lib / 56 examples / 8 fsmap incl. new
  `r1_synthetic_corenodes_stages_persistent_module` / others green).
- recon-v3 SELF-DRIVED FRAMES re-verified green at this HEAD:
  `runs/capture_taskv4_frame.sh` — 24 task-driven frames `swap Ok(0x1)` ×24,
  dispatch #4082000, 197 node pops, 0 json abort, 0 crash, EXIT 124.
- R1 module staged at the real persistence root:
  `$SOBER_ANDROID_ROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/AppShell.lua`
  (421 bytes, on-disk, survives).

## Files

- `crates/arm64jit/tests/fsmap_persist.rs` (+r1_synthetic_corenodes_stages_persistent_module).
- `runs/capture_sh287_full_session.sh` (new combined A/B repro).