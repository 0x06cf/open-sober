# Open-Sober run status (hermes-worker)

Updated this cycle (SH476): wired the PlatformParams viewport{Width,Height}Mm display
surface — getScreenPhysicalSizeInMillimeters (Java static) -> Point -> x/y int FIELDS,
scoped to a dedicated Point object so generic x/y reads stay honest.

## Current state

- `dev` HEAD: (SH476). recon-v3 immediate-priority deliverables re-verified GREEN at this
  HEAD first (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames, 197 node
  pops, 0 json abort, 0 crash, EXIT 124).
- Do-init/Route-B baseline: DM-root [0x106a68818]=0 (LIVE DM=false), structural SH462/467.
- SH476: slot 114 (CallStaticObjectMethod) was the shared NULL stub, so
  `getScreenPhysicalSizeInMillimeters` died at NULL and the whole viewport-Mm chain never
  reached a field read. Now: dedicated viewport Point object, `x`=338 / `y`=190 served by
  GetIntField ONLY on that object (a plain x/y on any other object stays 0). Hermetic pins
  the full dispatch chain + the scoping negative.

## This cycle's advance

- Production code only in jni.rs (off the 1MiB hooks; jit.rs/elfjit.rs untouched). arm64jit
  lib 681->682 incl. 1 new SH476 hermetic. Workspace green.

## Honest status

- No DM (DM-root [0x106a68818]=0, LIVE DM=false) — Route-B live-DM structural gate
  UNCHANGED (SH462/467). SH476 is a BUILD-THE-RUNTIME session-content completion (the last
  remaining recon-named display value: physical screen size in mm), not a Route-B seed.

## Next-forward candidates

1. (standing, TOP) do-init completeness / live-DM: aligned lever is the session-ctor /
   runtime-surface drive (engine's OWN session constructs the DM). The real APK assets are
   staged so a completed do-init's first rbxasset/AAssetManager request has REAL content.
2. The SendAppEvent 'Home' fabricate path (SH339): a real 4-byte "Home" SSO reaching the
   discriminator (currently the fabricated jstring materializes as size 6 -> event 0).
   Fixing the fabricate side is an open question downstream of the live-DM wall.
3. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged).
4. (CLOSED by SH476) DeviceParams viewport{Width,Height}Mm — the
   getScreenPhysicalSizeInMillimeters->Point->x/y FIELD path is now wired + hermetic.