# Frontier SH232 — EC-world CALLER MECHANISM: the in-rung callers never translate on the ladder

Status: single-agent Route-B re-attack, NEW runtime mechanism measurement. +1 hermetic `sh232`
(elfjit example, 78/0). Repro `runs/capture_sh232_ec_caller_mechanism.sh`.
frontier: SH231 located the REAL ExperienceController DM-creation world (bodies 0x102e1c650..
0x102e25200, __func vtable band 0x63981d8..0x6399c00) and measured it headless-UNREACHED
(region-watch 0 hits). SH231a added a static caller-side scan: **241 direct bl/b callers** into
that region, including two inside LADDER rungs. This cycle measures the caller MECHANISM at
runtime.

## Why this is a genuinely new measurement (not a re-confirmation)

SH231a's caller-side finding was STATIC: a region has 241 callers, but a reached caller would
translate its `bl` target as a fresh block and fire the watch, so 0 target hits meant none executes.
That leaves one gap: **are the caller bodies themselves entered as blocks and just diverted
in-body, or do the enclosing rungs complete BEFORE the caller's own block is ever translated?**
SH228 established block-entry-DEFINITIVE reasoning for functions (a distinct fn = its own block
entry); this cycle applies the same lens one level up — to the caller blocks in the ladder rungs.

The two in-rung EC callers:
- 0x1023f1294 (in `nativeAppBridgeStartLuaAppDM` entry 0x1023efe2c) call chain marshals then
  `bl 0x2e24598` — the EC lambda world's entry (file 0x2e24598 = a9ba7bfd stp,#96 prologue).
- 0x1023cfd68 (in `nativeAppBridgeV2InitWithParams` deep branch prologue 0x23cfafc)
  `bl 0x2e24468` — the other EC lambda body (file 0x2e24468 = d10183ff sub,#0x60).

## Method (runtime, real libroblox.so, completing ladder)

Watched five regions in one run (JIT_REGION_WATCH comma-ranges):
1. EC target world 0x102e1c650..0x102e25200 (SH231's region)
2. StartLuaAppDM EC-arg block 0x1023f12d0..0x1023f1300 (the marshalling just before bl 0x2e24598)
3. V2Init deep EC-caller block 0x1023cfd40..0x1023cfd70 (before bl 0x2e24468)
4. V2Init mid-body 0x1023cfb40..0x1023cfc7c (deep-branch body containing #3)
5. governor-tail control 0x102e9fa80..0x102ea3b40 (ladder-completion control, SH197/204/209)

Env: the canonical completing-ladder set (JIT_DRIVE_LIFECYCLE/RENDERINIT/RM_SEED/HASHFIX/
JSON_ZERO_FIX/SETFIX/SH115_SINGLETON_PATCH/V2_ONDEMAND), --v2boot ladder with surface-handoff +
send-appevent, EXIT 124, timeout 175s/run.

## Measured (fresh, completing runs only)

```
run 4: EXIT=124 ec_target=0 sldm_ecblock=0 v2deep=0 v2mid=0 govtail=2     <- COMPLETING
run 5: EXIT=124 ec_target=0 sldm_ecblock=0 v2deep=0 v2mid=0 govtail=2     <- COMPLETING
(runs 1-3,6 = EXIT 139/134 = the known run-variable SH198/SH55 V2Init singleton-vtable flake,
 never completing; excluded from the conclusion.)
```

**On BOTH completing runs, the governor-tail control fired (ladder walked the full tail to
0x102ea30dc) but ALL FOUR EC caller/body blocks stayed at 0 hits.** Combined with SH231's
0 target hits, this is now a fully-observed caller+region mechanism: **StartLuaAppDM returns
Ok(heap addr — its benign soft-return / SH155-observed Ok(0x55a01d785a20)) and V2InitWithParams
its own soft-return BEFORE either reaches its EC-call block, so the `bl 0x2e24468`/`bl 0x2e24598`
are never even translated into blocks on the completing ladder.** The ladder log confirms the
sequence: `driving StartLuaAppDM` -> `StartLuaAppDM returned Ok(0x55a01d785a20)` -> SH155
once-slot=0x400000b (intern, NOT a DM) -> drive V2StartAppWithParams -> outside-image stop.
The rungs complete upstream of the EC world.

## Honest boundary (do-not-over-claim)

- CLOSES SH231a's residual: "callers exist but none reached" is now a measured runtime mechanism
  (the caller blocks themselves are never translated), not a static inference.
- Does NOT manufacture a DataModel, does NOT lift the Route-B live-DM structural gate (SH209/
  218/223/224/228/231 unchanged). The EC world stays constructible only inside a real
  experience/app-launch session where StartLuaAppDM actually advances past its benign return.
- Reproducibility note: the ladder is ~1/3 flaky (SH198/SH55 V2Init singleton-vtable stop) — the
  measurement's conclusion rests on the govtail-positive (completing) runs only, exactly as
  SH209/223/231 did. Non-completing runs are excluded by the control, not by selection bias.

## Code

- +1 hermetic `sh232_ec_callers_pinned_to_ladder_rungs` (real-image guard family as sh232/sh231,
  skip-if-absent): byte-pins the two EC-call `bl` words (0x1023f1294=0x9428ccc1 -> 0x2e24598;
  0x1023cfd68=0x942951c0 -> 0x2e24468), their enclosing-fn prologues (StartLuaAppDM entry
  0x1023efe2c=0xd10183ff, EC-arg helper 0x1023f11f4=0xa9bf7bfd, V2Init deep-branch 0x23cfafc=
  0xa9bc7bfd), bl imm26 sign-extended target cross-checks, + 4-alignment/window guards. A drifted
  caller address or a reassembled `bl` now fails loudly instead of silently reading the wrong
  block. Repro `runs/capture_sh232_ec_caller_mechanism.sh` (watches all five regions incl. the
  caller bodies, reports per-run, states the completing-only conclusion).
- No production path edited.

## Verify

- `cargo test -p arm64jit --example elfjit sh232` -> 1 passed (anchors on real libroblox.so).
- Full `cargo test --workspace` green at HEAD (re-verified before this cycle; elfjit 77->78/0).

## Next (honest, single-agent)

Route-B live-DM = structural gate UNCHANGED. The EC world is now located (SH231), its in-rung
callers pinned + their non-execution mechanism measured (SH232). The remaining forward hook stays
SH174's capture-latch arming *(0x106391908) at a real make_shared<DataModel> inside a genuine
session — nothing headless reaches it. recon-v3 immediate-priority deliverables stay shipped +
verified. The caller pin makes any future session that DOES advance StartLuaAppDM past its
benign return fail-loud if it drifts off the EC path.