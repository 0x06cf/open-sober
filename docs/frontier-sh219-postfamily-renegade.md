# SH219 — SH203 "post-family gate" re-measured at HEAD = NOT reproduced in 28 runs (stale classification) + flag-manager/post-family words pinned as regression anchors

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed per operator
Sep-15). Date Sep 16 2026. Workspace green (arm64jit 387/0 lib + elfjit example 68/0
incl. new sh219 test + all crates, 0 failures). +1 hermetic. No production code path
changed (pure regression net + stale-verdict correction). Commit `<SH219COMMIT>`.

## WHY THIS CYCLE — a stale Route-B verdict worth re-measuring

SH203 (Sep 16) classified the "post-family fault" — a NULL-singleton pthread_mutex_lock
(crash `SIGSEGV pc=0x7f00000022b0` host-call slot, `lr=0x102b53a78`, `x0=0x28`) reached
only AFTER the V2 singleton family is cleared — as a **deterministic 4/12 live-world-build
gate**, using the argument "enclosing fragment file 0x2b53a64 is a vtable/computed-dispatch
target with 0 direct bl callers."

SH205 later **proved that exact "0 direct bl callers" method unreliable** on a sibling
fault (the flag-manager lockshare): it counted callers of the helper instead of the
actual caller and dismissed a seedable fault as a migration gate. The post-family site
was **never re-analyzed with the GSDSP guest-stack tool** (JIT_GUEST_STACK_DUMP=1) that
SH205 built precisely to find cross-called host-thunk callers. And two fixes that could
have closed it — SH116b (flag-manager lock fix) and SH217 (SH161b governor-tail window)
— landed **after** SH203's measurement. So this cycle re-measures whether the "post-family
gate" is still a real, live fault at HEAD.

## MEASURED — post-family fault = NOT reproduced across 28 fresh runs at HEAD

Two batches under the canonical V2_ONDEMAND completing ladder (nativeInitFlags →
gameGlobalInit → do-init → governor → V2Confirm → V2Start → V1 AppStart →
V2UpdateSurface → SendAppEventOnAppReady), both with the SH174-runbook canon env:

- Batch 1 (16 runs, GSDSP on): 15 clean EXIT 124 / 1 fault at guestpc **0x10284f490**
  (the do-init `__call_once` blr-x2 region, known SH55/64 flake class; GSDSP shows a
  guest→guest .text fault, non-seedable). Zero post-family hits.
- Batch 2 (12 runs, SH203's EXACT env — timeout 110, no GSDSP): 10 clean / 2 faults at
  guestpc **0x106240c24** (FMOD/AAudio crash-A, SH212/SH213) and **0x1021dea94**
  fault=0x20 (SH208 singleton-vtable host-ptr class). Zero post-family hits
  (checked for both the 0x102b53a78 lr marker and the 0x7f00000022b0 host slot).

**0 / 28 post-family reproductions** vs SH203's 4/12 (33%) at its pre-SH116b/SH217 HEAD.
The post-family `pthread_mutex_lock(x0=0x28)`-on-a-NULL-singleton signature is the SAME
family SH116b fixed (both are mutex-lock-on-NULL+0x28 singleton objects). Two independent
interpretations, both consistent with the closure: (a) SH116b's flag-manager singleton
patch absorbed the post-family site (same NULL+0x28 mutex singleton class), or (b) SH217's
window correction moved the dispatch such that the receiveCall region is no longer reached
at the ile that faulted. Under either interpretation the SH203 verdict "post-family gate =
live-world-build structural gate" is **no longer measurably alive at HEAD**.

## HONEST BOUNDARY (do not over-claim)

- A non-reproduction across 28 runs is strong evidence but not a proof the fault cannot
  recur (it may be a rarer residual, e.g. 1-in-many rather than SH203's 4-in-12). It does
  NOT manufacture a DataModel and does NOT change the Route-B structural gate (the
  do-init once-lambda still yields the strcmp intern 0x400000b, not a live DM — SH196;
  SH155 probe still once-guard=0x1 / liveDM=false at the headless idle).
- Fault classes that DID fire (0x106240c24 FMOD, 0x1021dea94 SH208, 0x10284f490 SH55/64)
  are all the documented non-seedable live-object/host-heap classes; none is a fixed-.bss
  singleton, so no new seed is warranted. No production code change this cycle.

## CODE — +1 hermetic regression net (sh219, real-image guard family as sh213/sh211/sh116b/sh200)

`sh219_postfamily_frag_and_flagmanager_words_pinned` (crates/arm64jit/examples/elfjit.rs,
skip-if-absent real libroblox.so guard): byte-pins
- the SH116b flag-manager load slot (file 0x2320a24 `adrp x8,7273000` = 0xf0027a88,
  0x2320a2c `mov x4,x3` = 0xaa0303e4, 0x2320a30 `ldr x8,[x8,#2480]` = 0xf944d908 →
  the [0x10672739b0] singleton global), and
- the SH203 post-family fragment (file 0x2b53a64 `bl JNICallProtocol_receiveCall+0x558`
  = 0x940140e4, 0x2b53a6c `stp x29,x30,[sp,#-16]!` = 0xa9bf7bfd, 0x2b53a74
  `bl pthread_mutex_lock@plt` = 0x94de0a4f) + guest=file+0x100000000 transform +
  4-alignment for all 6 .text sites. If either set of load-bearing words drifts on a
  future libroblox.so, the test fails before a session re-classifies the gate against a
  stale byte layout.

## TREE / VERIFY
- New hermetic: sh219 (elfjit example 68/0). Workspace: cargo test --workspace green
  (arm64jit lib 387/0 + all crates, 0 failures).
- recon-v3 render plane + SH174 capture-latch re-verified green at THIS HEAD:
  capture_taskv4_frame.sh = 24 task-driven frames (`present #19..#23 swap Ok(0x1)`,
  distinct colors), 196 node pops, no json abort, 0 crash; SH167/SH169 capture-latch
  armed+DELEGATING (ACTIVE hook 0x1067daaf0 → engine's own 0x1021ebaf4). Forward hook intact.

## STANDING (unchanged, do-not-re-tread)
- Route-B live-DM world-build = structural gate (SH196/203/204/209/218 do-not-re-tread).
  This cycle does NOT manufacture a DM; it removes a possibly-FALSE "live-world-build
  gate" from the closure set by showing the post-family fault is not observed at HEAD.
- SH174 capture-latch arming *(0x106391908) at a real make_shared<DataModel> = the single
  forward hook. recon-v3 immediate-priority deliverables (self-driven frames + JSON-ZERO
  fix) stay shipped + verified.

## FILES
- crates/arm64jit/examples/elfjit.rs — +sh219 hermetic (regression net only).
- docs/frontier-sh219-postfamily-renegade.md (this file).
- Repro: the two batches described above (logs discarded; 0/28 post-family is the result).