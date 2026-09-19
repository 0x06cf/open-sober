# Frontier SH401 — the do-init → governor reach through the GENUINE AppBridgeV2 singleton
# (build-the-runtime, SEP-18; the frontier's named "next gate" after SH400)

Date: 2026-09-19, hermes-worker, single-agent. Workspace green before/after
(`cargo test --workspace` EXIT 0; arm64jit lib 450 passed/0 failed incl the new
sh401 hermetic).

## Why this cycle

SH400 measured (2/2) that the ordered session-substrate drive constructs the
**AppBridgeV2 singleton to its genuine relocated vtable 0x1063a3410** from atom
[5/16] StartLuaAppDM onward — the FIRST time any cycle measured that singleton
non-zero. The frontier doc's explicit next step was: *"drive DEEPER past the
AppBridgeV2 singleton — the governor vt[+0x18]=0x102e9fa84 (guest 0x102e9fa84)
the singleton makes reachable."* SH401 executes exactly that drive and MEASURES
the governor reach for the first time.

## What SH401 measured (real libroblox.so, region-watch on the session drive)

With the same env as SH400 (the furthest-advancing full ladder + the SH400
`--v2boot-session-drive` ordered substrate that self-constructs AppBridgeV2 to its
genuine vt), JIT_REGION_WATCH on the do-init worker / governor body / DMCONT:

```
[region-watch] hit pc=0x1023eff4c   (do-init worker entry, sub sp,#0x180 prologue)
[region-watch] hit pc=0x1023effa0   (do-init GetOrCreate -> bl 2367270)
[region-watch] hit pc=0x1023effac   (do-init `ldr x8,[x19,...]; blr [vt+0x18]` @0x23effbc prelude)
[region-watch] hit pc=0x102e9fa84   *** THE REAL GOVERNOR BODY *** (frame stp a9ba7bfd)
[region-watch] hit pc=0x102e9fb58   (governor after its MODERN dispatch `blr x9` @0x2e9fb54)
[region-watch] hit pc=0x10258c6e4   *** StartAppWithParams entry (make-call `bl 0x258c6e4`) ***
[region-watch] hit pc=0x10258c7b4   (StartAppWithParams body past `bl 1d96768` alloc)
EXIT=124 stable, crash-signals=0. Reproducible 2/2.
```

That is the frontier's named forward landed as a measurement: the ordered session
drive now **executes the real governor** (0x102e9fa84, SH157-router/gov dispatch +
SH159c/e/gov-tail patches applied) and its make-call **into StartAppWithParams
0x258c6e4**, both of which were historically "empty headlessly" (SH378/379:
"governor ... 0 hits", "app-shell ctor ... 0 hits"). Previously the governor body
was reached only via the SH159-patched inert leaf retire; now the genuine vt makes
the do-init `blr [vt+0x18]` land on the real governor entry.

## Honest

- Does NOT manufacture a DataModel. DM-root [0x106a68818]=0, MH_* stay false
  (they fire only when a live DM completes do-init). AppBridgeV2 [0x106a705e8] =
  genuine vt 0x1063a3410 (from SH400, unchanged).
- The governor executes and reaches StartAppWithParams, but **DMCONT 0x102bd1d68
  (the engine-init continuation) is still NOT reached** (0 region hits this run)
  — it is now the next standing gate, one step past StartAppWithParams's entry.
- This is a MEASURE (region-recorded reach) + a hermetic byte-pin, not a new DM.
  It confirms the SESSION-CTOR substrate is exercising real engine session code
  deeper than ever before.

## Next

Drive StartAppWithParams 0x258c6e4's body (now reachable) toward DMCONT 0x102bd1d68
— the next SESSION-CTOR gate on the do-init → DM world-build line. Keep the same
ladder + SH400 substrate env; region-watch the StartAppWithParams interior and the
DMCONT entry. R1 content stays staged/armed (SH351/352/354; fires the instant a
live DM drives the loader). Do-not-re-tread unchanged (SH285/385/395-398 family,
setDataModelToCurrent SH388, EC reader-gate, 0x258b5d8/SetInitParams SH362/375,
window-attach real SH367, ALooper SH365, governor-gates full-ladder SH379, once-
lambda store SH381).