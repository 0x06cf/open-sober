# Frontier SH339 — measured negative: the fabricated "Home" jstring does NOT route to event-code 4 (answers SH308's open Step-2 ABI question)

## Session
Sep 18-19, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert read-only
guard `routeb_appevent_w19_guard` (jit.rs, opt-in `JIT_ROUTEB_APPEVENT_W19`). Workspace
green (590 passed / 0 failed). No production path edited — this is a measurement, not a fix.

## Why (the open question SH308 left)
SH307/308 drove `SendAppEventOnAppReady` (0x102bb463c) past its preload-overrides terminal
into the do-init → app-shell ctor → FMOD audio tail → StartAppWithParams pipe. But SH308
clearly flagged ONE remaining open Step-2 ABI question: whether the fabricated "Home"
jstring actually routes to w19=4 in the engine's event discriminator. The post-return
`se.x[19]` read is unreliable because x19 is callee-saved and restored on return (measured
0x0 every run), so it cannot tell us whether 'Home'→4 or fell through.

## What is measured (4/4 deterministic, real libroblox.so)
New default-inert block-entry guard `routeb_appevent_w19_guard` captures the discriminator
INPUT at its real block-boundary entry 0x102bb46b8 (region-watch-verified), before the event
build overwrites the frame. At that point the parsed 4th jstring's libc++ SSO header sits at
[sp] and the discriminator's branch is decided by its size byte.

Consistent across 4 independent runs (sh339-w19b/g/h/i), e.g.:
```
[elfjit:appevent-w19] ... sp=0x55f35cbdf990 [sp].b0=0xc long=0 size_hdr=0x6 SSO="...garbage..."
[elfjit:appevent-w19] ... handle 0x7f3210037d00 bytes=[486f6d6500..] ascii="Home"
```
Decode: `b0=0x0c` → libc++ SSO short string, size = b0>>1 = **6** (NOT "Home"'s size 4).
The discriminator's `cmp x9,#0xc / #5 / #4` chain therefore falls through to `mov w19,wzr`
= **event-code 0**, not 4. The handle bytes at x19/x5 are `48 6f 6d 65` ("Home") — so the
source string IS correct, but the RBX string materialized at [sp] that the discriminator
reads is NOT "Home". The harness's 'Home' event does not reach the router as 'Home'.

## Honest interpretation (do-not-over-claim)
- ANSWERS SH308's open ABI question with a measured negative: fabricated "Home" does NOT
  route to w19=4; it falls to 0. The event payload built is the default/other event, not Home.
- The do-init pipe forward-reach (SH307/308: app-shell ctor, FMOD audio tail, StartAppWithParams,
  app-data-model count 0→1) is NOT contingent on 'Home' routing — it still fires with event-code
  0 (re-verified in every run: app-data-model-count 0x1). So the Route-B reach is robust to the
  discriminator result; the pipe is driven by the SH126 sync-gate ([0x10683d010]=-1), not by a
  correctly-routed Home event. This actually REFINES the SH307/308 attribution: the do-init
  construction runs regardless of which event name is fed, which is a positive (the session
  advance does not depend on the fragile fabricated-jstring round-trip).
- Does NOT manufacture a DataModel; Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false).
  SH174 capture-latch stays the single forward hook. Source diff is +guard only, default-inert.
- Root-cause of the non-"Home" materialization at [sp] (size 6 vs the source's 4) is NOT
  further pinned this cycle — the measured headline (event-code 0, not 4) is the deliverable;
  nailing why the RBX string at [sp] decodes to size 6 is a follow-up if that matters again.

## Verify / files
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (590 passed / 0 failed).
- elfjit.rs untouched; jit.rs +guard only.
- Repro (baseline SH307 forward arm + the guard):
  `runs/capture_sh339_appevent_w19.sh`
- Captures: runs/sh339-w19{b,g,h,i}.txt (4/4: b0=0xc, size 6, event-code 0, handle="Home").
- Commit: local `dev` only (operator pushes).