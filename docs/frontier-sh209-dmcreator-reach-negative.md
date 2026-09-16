# SH209 — ROUTE-B DM-CREATOR REACHABILITY: fresh measured negative at the post-SH202 completing-ladder state

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed, operator
directive Sep 15). Date Sep 16 2026. Workspace green (cargo test --workspace EXIT 0).
Repro: runs/capture_sh209_dmcreator_reach.sh; sample log runs/sh209-run6-clean.txt
(gitignored).

## WHY THIS CYCLE

The operator's return-to-Route-B directive names the live-DataModel construction path
(ExperienceController / initializeLuaAppWithDataModel / NativeDataModelManager) as the
re-attack target. SH164 (pre-SH202) measured the NativeDataModelManager DM-creator region
at **0 hits** — but at that HEAD the ladder always stopped first at the V2Init outside-image
flake (SH198/SH55), so "the DM creator is not reached *on this completing path*" was never
cleanly separable from "the ladder never got that far." SH202 (V2_ONDEMAND) changed the
state: the FULL V2 ladder now completes clean ~8/12. This cycle re-measures the DM-creator
region reachability on the completing path — the operator's named front, at the newest
state — with a calibrated in-run positive control.

## METHOD

Region-watch (JIT_REGION_WATCH, multi-range, SH197 parser confirmed) on three regions in
every run of the canonical completing ladder:
- DM-creator region A: [0x102bd1a30, 0x102bd1d08) = NativeDataModelManager
  getFlagsFromEngine_/initEngine_ bodies (SH164).
- DM-creator region B: [0x102bd21d4, 0x102bd2600) = initializeLuaApp_/startLuaApp_ bodies (SH164).
- POSITIVE CONTROL: governor tail [0x102e9fa80, 0x102ea3b40) — the SH197/204-proven region
  that fires on every completing ladder run.

Cmd: `elfjit ... --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent`
with the canonical Route-B env (JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED/HASHFIX/
JSON_ZERO_FIX/SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1).

## RESULT (10 runs: 4 + 6)

- **3/10 runs COMPLETED CLEAN (EXIT 124, 0 crash) AND fired the governor control** (run-2,5,6).
- In every one of those control-positive runs: **DM-creator region A & B both = 0 hits.
  Total across all 10 runs: dmcreator_pcs = 0.**
- Governor-tail control hit **25 distinct pcs per clean completing run**, walking the whole
  tail: 0x102e9fa84 (entry) → ... → 0x102e9fcc4/0x102e9fdc8 (governor tail blocks the
  routeb_tail_dispatch_guard seeds) → 0x102e9fe0c (the SH161b 0x3c12c0 b.eq site) →
  0x102ea3084 → 0x102ea30d0 → **0x102ea30dc (stack-canary `ret` thunk)** — the exact
  SH197/204/198 terminal. The completing ladder ends cleanly at the governor-tail canary-ret.
- The 7 non-completing runs crashed at the known pre-existing run-variable SH198/SH55
  singleton-vtable host-pointer site (identical signature: guestpc 0x102b9dee0 fault=0x10,
  same as SH205/206/208 listed) OR the 0x10284ce54 do-init-class flake — NOT at any
  DM-creator pc, and NOT a region-watch-induced regression (region-watch is a compare+print).

## CONCLUSION

**Fresh measured negative at the post-SH202 state.** Even now that `V2_ONDEMAND` lets the full
ladder complete clean (gov-tail reaches its canary-ret terminal on 3/3 control-positive runs),
`NativeDataModelManager`'s DM-creator bodies (`getFlagsFromEngine_`/`initEngine_`,
`initializeLuaApp_`/`startLuaApp_`) are **never entered** headlessly. The governor tail's
terminal dispatch (0x102ea30dc canary-ret) returns before any DM-construction body on its
successor edge runs. This closes the SH164 "…could not be separated from the ladder stopping
first" residual at the completing state, and reconfirms Route-B live-DM world-build = the
structural gate at the NEWEST evidence (SH202+ completing path), not just the pre-SH202 path.

## TREE / VERIFY

- New repro script: runs/capture_sh209_dmcreator_reach.sh (bash, N arg, prints per-run
  dmcreator_pcs + govtail_pcs + crashes and a summary verdict).
- Sample clean run log: runs/sh209-run6-clean.txt (gitignored — small enough, but per
  project discipline kept out of git; see the script's own output for the tally).
- `cargo test --workspace` green (EXIT 0) at this commit; SH208 + recon-v3 deliverables
  (self-driven frames / json-zero-fix) unaffected (no production code touched this cycle).

## DO-NOT-RE-TREAD (added)

- Do NOT re-run the DM-creator region-watch negative on the same completing ladder expecting
  a different answer — now measured twice-over (SH164 pre-SH202 + SH209 post-SH202).
- Do NOT static-seed any NativeDataModelManager cell to "force" the DM creator:
  `initEngine_`'s next gate is a NETWORK feature-flag fetch and `startLuaApp_` needs a real
  manager instance (SH164), both live-session-gated; a seed would be cruft against an
  unreached body.
- Standing Route-B front unchanged: live-DM world-build = structural/account-level gate;
  the one dropped measure of forward motion is the SH174 capture-latch observer arming
  *(0x106391908) in-process the instant a real session's make_shared<DataModel> runs.