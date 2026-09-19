# Frontier SH394 — never-run composition measured: FULL safe app-command table drive (SH393) + SH174 DM-allocation capture latch on the furthest-advancing SH378 env — the capture latch never even ARMS, 0 validated make_shared<DataModel>, terminal still the standing SH285 persistence-lane wall

Date: Sep 21, 2026, hermes-worker. Single-agent (cone suppressed). One new probe
`runs/capture_sh394_dmcap_fulltable.sh` (never-run intersection). No production Rust /
guest byte / JIT-hook-default touched (reuses only existing default-inert guards). Workspace
green at start and end (cargo test --workspace EXIT 0).

## Why this cycle

Recon-v3 immediate-priority deliverables re-verified green at this exact HEAD first:
- capture_taskv4_frame.sh (SH391 guard armed): confirmed-green attempt 1, **24 real
  task-driven frames `present swap Ok(0x1)`**, 196 node pops, 0 json abort, 0 crash.
- capture_sh304_session_producer.sh: session-gated producer correctly INERT (0 GATED,
  lone present #0 = render-plane warmup).
- JIT_JSON_ZERO_FIX present at 0x102355d40.

SH393 established the FULL safe app-command table drive (engine's OWN process_cmd consuming
all 15/20 safe APP_CMD cases in lifecycle order, cmd 11 INIT_WINDOW marker fires). The SH174
DM-allocation capture latch is the SINGLE forward observer for Route-B. But these two were
NEVER composed: SH393's capture did NOT set JIT_DM_ALLOC_CAPTURE; SH378 ran the latch on the
SH377 advancing env WITHOUT the full-table drive. SH394 runs the genuinely-never-composed
intersection.

## MEASURED (real libroblox.so, full-table + furthest env, attempt 1 of 3)

```
[elfjit:glue-full] SH393 done: 15/15 commands returned Ok, 0 stopped; AppBridgeV2 selftransition 0x0->0;
   surface XID 0x200000->2097152; flags=0x0 dmreg=0x0
[elfjit:appevent-w19] SH339 handle 0x7f7fecfb1bf0 bytes=[486f6d6500...] ascii="Home"
[elfjit:v2boot] SendAppEventOnAppReady returned Ok(0x107273d50) w19-event=0x0
[validated] count = 0
capture trail never installed (no 'routed ... capture trail' lines)
terminals: SIGABRT SIGSEGV fault=0x0 fault=0x3e900127d54 guestpc=0x101d9a528 (LSM pool-pop write-site, EXIT 134)
post-lifecycle: MH_FLAGS_LOADED=false MH_ENGINE_INITIALIZED=false MH_APP_READY=false AppBridgeV2[0x106a705e8]=0x0
```

- **The capture latch never ARMS**: zero `routed ... capture trail` lines, so the operator-new
  wrapper (0x102a0d9b8) is never enterinstallably on this composition — a strict strengthening
  of SH378 (where the trail at least installed, just never validated). The only `bytes=` hit is
  the SH339 w19 jstring readback ("Home"), not an allocation.
- **0 validated make_shared<DataModel>**: the single forward observer stays silent even with the
  full command queue delivered on the furthest-advancing world-build env.
- **Terminal unchanged**: the run completes the full-table drive + SendAppEventOnAppReady (Ok),
  then drains into the standing SH285/0x101d9a528 LSM pool-pop write-site family (EXIT 134) —
  the same measured-closed persistence lane every Route-B arm converges on.

## Interpretation

Composing the engine's full own-command-queue drive with the DM-allocation capture latch on the
furthest env does NOT route any dispatch to make_shared<DataModel>. This closes the last "the
full command table might change the allocation picture" loophole: the SH174 capture latch not
only refuses to validate, it never even installs on this composition. The engine's command
dispatcher (even 15/15 safe consumption) does not self-manufacture a DataModel — consistent with
SH366/368/393's reading that AppBridgeV2/surface/DM "move only when a live session/do-init builds
the DM world." Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false).

## Honest

Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED. This is a
map-completion on a genuinely-never-run composition (strengthens SH378), not a DM advance.
SH174 capture-latch stays the single forward observer.

## Do-not-re-tread (unchanged closures, all still stand)

SH393's cmd 1/13/15/17/18 (measured live-object-deref), LSM skips (SH349/350/358/373),
EC reader-gate (SH355/356/374), 0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367),
ALooper (SH365), governor gates full-ladder (SH379), -9 string (SH380), map-header (SH248h),
once-lambda store (SH381), SH267 node-cell (SH385), setDataModelToCurrent (SH388),
LSM-manufactured wiring (SH385).

## Files

- runs/capture_sh394_dmcap_fulltable.sh (new probe, committed).
- Log /home/hermes-worker/runs/sh394-dmcap-fulltable.txt (outside repo).
- Commit: local dev only (operator pushes).