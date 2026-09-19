# Frontier SH398 — never-run intersection closed: the WORKING DELEGATE observer (SH395/397) ON the SH285-CROSSED env (SH373/385 append+pack skips) still records 0 validated make_shared<DataModel> — the "no live DM" verdict now rests on the deepest, fully-crossed reach yet (crossed persistence lane + trustworthy observer)

Date: 2026-09-21 (this cycle), hermes-worker, single-agent (cone suppressed). One new probe script
`runs/capture_sh398_dmcap_delegate_lsm_crossed.sh`. No production Rust / guest byte / JIT-hook-default
touched. Workspace green at start and end (cargo test --workspace EXIT 0; recon-v3 deliverables
re-verified green at this exact HEAD: capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames
`present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; capture_sh304 session-gated producer
INERT 3 UNGATED/0 GATED; JIT_JSON_ZERO_FIX present at 0x102355d40).

## Why this cycle

SH397 ran the working DELEGATE observer on the do-init DISPATCH-reaching full ladder, but that env has
NO LSM-crossing skips — every run terminated AT the SH285 persistence lane (guestpc=0x101db1b08)
before any deeper reach, so the observer could only report what the PRE-lane reach did. SH373/385 ran
the crossings (append-skip SH349 + pack-skip SH350 + keyfix) WITHOUT the observer, so the conclusion
"deep pool-pops, no make_shared" was inferred, never observed on the deepest, crossed reach.
This composes the two never-combined halves: a WORKING install observer + a deterministically CROSSED
SH285 lane, measuring whether ANY validated make_shared<DataModel> fires anywhere past the fence.

## Measured (real libroblox.so, do-init dispatch-reaching env + APPEND_SKIP + PACK_SKIP + DELEGATE)

- **SH285 crossed** (sh285-hits=0): `[elfjit:routeb] SH349 ret LSM append byte-copy @0x101d9a15c` +
  `SH350 ret name-pack @0x101d9a708` both fire; no `guestpc=0x101db1b08` terminal anywhere.
- **do-init MAIN dispatch FIRES** (SH361 trace): `container=0x55f75259a6c0 [container+32]=0x55f752b86150
  (non-NULL) -> reach DM-ctor dispatch: [obj]vt=0x10635dde8 vt[+48]=0x10258b5d8`. once-guard=0x101
  once-slot=0x400000b DM-root=0x0.
- **DELEGATE latch INSTALLS**: `[routeb-dmalloc] routed CRT operator-new ACTIVE hook 0x1067daaf0 ->
  capture trail 0x7f00000001e8 (prev_hook 1021ebaf4, default_hook 0x0)`. FIRST call#1..#5 all
  `bytes=0x18` delegated cleanly through the engine's OWN hook (no bad_function_call from delegation).
- **685 allocations observed** — all `0x18` bytes except ONE `0x20040` (at pc 0x1029f4284 = the SH88
  hash-fix substitution site, the pb_defaults registry map — NOT a DM allocation; the set of allocation
  sizes observed spans {0x18, 0x20040}).
- **[validated] make_shared<DataModel> = 0** — the verdict, at the deepest reach yet: past the crossed
  SH285 lane AND with a working, installed, delegating observer.
- Terminal: SIGABRT+SIGSEGV drain to the standing SH285 pool-pop write-site 0x101d9a528 (SH341 family;
  the reached node's value is still a path-independent unconstructed object, SH385/396 class).
- Session observables: serialized ladder shows governor version-gate patched, governor MODERN dispatch
  drives to `bl 0x258c6e4` startAppWithParams, fabricated NativeDataModelManager instance drives
  engine-init 0x102bd1b98 — but no live DataModel allocation validates.

## Interpretation

This is the STRONGEST closure of the "no live DM" verdict on record: a trustworthy-by-construction
observer (SH395) was installed AND firing, the persistence fence that previously swallowed every
dispatch-reaching run was deterministically crossed (SH349/SH350 skips), and STILL zero validated
make_shared<DataModel> anywhere in a 685-allocation reach-through. It is consistent with SH385/SH396
(path-independent seam), SH393 (governor gates), SH397 (observer depth) and corners the sampling gap
that remained: "maybe a make_shared fires once you get past the lane." Measured: no. The persistence
lane crossing lands in the SAME measured-closed family (pool-pop 0x101d9a528) with no DM event under
the only observer that proves the absence.

This is map-completion + observer-depth advance (never-run intersection), NOT a DM advance — honest.
Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0). The
session-gated producer + R1 content path remain latent-but-correct; they fire when a real session
advances (do-init owning a live DM), which SH398 re-confirms no static-composition manufactures.

## Do-not-re-tread (all prior closures stand, unchanged)

SH393 cmd 1/13/15/17/18, LSM skips (SH349/350/358/373), EC reader-gate (SH355/356/374),
0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367), ALooper (SH365), governor gates
full-ladder (SH379), -9 string (SH380), map-header (SH248h), once-lambda store (SH381), SH267
node-cell (SH385), setDataModelToCurrent (SH388), LSM-manufactured wiring (SH385), do NOT run the
SH174 latch WITHOUT DELEGATE expecting an install (SH395), do NOT re-manufacture [0x10726f8c0]-style
coherent empty maps (SH396). SH398 adds: do NOT expect the SH174 DELEGATE observer to find a DM past
the crossed SH285 lane either — measured 0 validated with the lane crossed (probe-only, no production
change, tree clean at SH398).

## Files

- runs/capture_sh398_dmcap_delegate_lsm_crossed.sh (new, committed)
- Log /home/hermes-worker/runs/sh398-dmcap-delegate-lsm-crossed.txt (outside repo, per CLAUDE.md <1MiB)
- HANDOFF.md + runs/STATUS.md updated (SH398 entry)
- Commit: local dev only (operator pushes).