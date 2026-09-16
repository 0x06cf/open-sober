# SH228 — engine-init dispatcher sub/continueAfterFlagsLoaded_ NEVER enter as blocks (closes SH166(a) definitively + corrects SH226's mechanism)

Status: single-agent Route-B re-attack, measurement closure + hermetic pin. +1 hermetic sh228
(elfjit example). frontier: Route-B. No production path edited.

## What was open

SH166 explicitly left question (a) **unclosed**: "whether continueAfterFlagsLoaded_ (the
engine-init continuation that would drive the app-shell) truly executes under DMCONT is not
decidable from region-watch alone" — because a `blr`-to-computed-target inside a single traced
chain can run without firing a fresh block-entry log. SH226 then CLAIMED the pipeline
"benign-completes via its 2nd-frame (sub_2bd8dac) -> 0x102bd9058 soft-return, never reaching
the blr at 0x102bd8e28" — a mechanism assertion made without pinning whether sub_2bd8dac
itself is even entered.

## Measured (real libroblox.so, one completing --v2boot ladder run, DMCONT=1, 4 region windows)

```
region hit at guest pc=0x102bd1b98   (fnB engine-init entry)        FIRES
region hit at guest pc=0x102bd8ce8   (engine-init dispatcher)        FIRES
(0x102bd8dac .. sub_2bd8dac)                                        NEVER
(0x102bd1d68 .. continueAfterFlagsLoaded_)                          NEVER
```

The two that never fire are **separate functions called via `bl` (sub, 0x2bd8dac, reached by an
UNCONDITIONAL `bl 0x2bd8d60 -> 0x2bd8dac`) and `blr` (continueAfterFlagsLoaded_, 0x102bd1d68,
via vt[+0x1f0])**. This JIT creates a fresh block entry for each distinct function target
(cached_block is per-pc; fnB and the dispatcher each logged their own entry). A separate-function
entry that never appears as a block is therefore **block-entry-definitive: it is never entered** —
not region-watch-blind, and not a trace-merge artifact.

## Two consequences (both forward, neither re-confirmation)

1. **SH166(a) is CLOSED as a definitive NEGATIVE.** The vt[+0x1f0] dispatch at 0x2bd8e28 never
   runs; continueAfterFlagsLoaded_ is unreachable from this path even with the fabricated
   manager's vt[+0x1f0] routed to the REAL guest 0x102bd1d68 (DMCONT=1 measured).
2. **SH226's completion mechanism is CORRECTED.** SH226 asserted the dispatcher "benign-completes
   via its 2nd-frame (sub_2bd8dac)". But sub_2bd8dac never enters as a block — so the dispatcher
   completes through the resolve/leaf paths (the `blr vt[+0xf8]` / `blr vt[+0x108]` leaf calls
   that precede the unconditional `bl sub`), NOT through sub. i.e. one of the +0xf8/+0x108 leaf
   returns diverts control away from the (unconditional) `bl sub`; sub and the +0x1f0 dispatch
   are skipped. The DMCONT continuation lever is therefore WIRED+ARMED but inert because the
   dispatcher stops before the continuation, not because "flags aren't loaded" per the older
   framing.

## Honest reading (unchanged structural gate)

This does NOT manufacture a DM and does NOT lift the Route-B live-DM structural gate. It removes
one remaining "can't-decide" residual (SH166(a)) from the DMCONT accounting and corrects a
mechanism claim (SH226's sub completion), so the next Route-B drive on this line starts from
"the dispatcher diverts at a leaf before sub" rather than re-arguing reachability. It also pins
the unconditional-bl-sub + the +0xf8/+0x108 leaves as the divergence window for anyone hunting
the leaf that diverts.

## Code

- `crates/arm64jit/examples/elfjit.rs` hermetic
  `sh228_engineinit_dispatcher_sub_never_fires_blocks` (real-image guard family as
  sh225/sh226/sh227): byte-pins the unconditional `bl sub` (0x2bd8d60=0x94000013) + mov x0,x20,
  sub entry (0x2bd8dac=0xd104c3ff) + +0x1f0 dispatch chain (0x2bd8e18/0x2bd8e20/0x2bd8e28), the
  two pre-bl leaves (0x2bd8d24/0x2bd8d2c and 0x2bd8d38/0x2bd8d50), continueAfterFlagsLoaded_
  prologue (0x102bd1d68=0xa9ba7bfd), + the 4 site guest-transforms/alignment. So a drifted
  constant fails loudly instead of silently re-measuring 0 region hits.

## Verify

- `cargo test -p arm64jit --example elfjit sh228` passes (real-image guard: 1 passed, others
  filtered; skips clean if the .so is absent).
- Full `cargo test --workspace` green (re-verified at HEAD before this cycle's re-check; the
  recon-v3 render plane re-verified 24 task-driven frames / 194 pops / no json abort / EXIT 124).
- Repro: `runs/capture_sh228_dispatcher_divert.sh` (the 4-window region-watch showing
  fnB+dispatcher fire, sub+continuation never).

## Next (honest, single-agent)

Structural Route-B gate UNCHANGED. The dispatcher-divert-before-sub is now measured; the next
genuine lever is a real session's make_shared<DataModel> (SH174 capture latch) or a genuine
flags-loaded engine-init state — neither a headless seed. recon-v3 render plane + SH210 wiring +
cookie persistence unregressed.