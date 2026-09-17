# SH236 — StartLuaAppDM's receiveCall dispatch MEASURED soft-returning before the marshaler (block-entry-definitive)

Session: Sep 17, 2026 (hermes-worker). Real libroblox.so (host `~/.cache/open-sober/robbox/libroblox.so`,
109,193,800 B). +1 hermetic `sh236_startluaappdm_receivecall_dispatch_softreturns_before_marshaler`
(real-image guard family as sh235; skip-if-absent). Workspace green (389 arm64jit + all crates;
elfjit examples 81/0). No production code path edited — pure recon + regression net. Single-agent (cone suppressed).

## Why (a genuinely-open angle, not a re-tread)

SH235 pinned the EC world's SOLE headless front-door: StartLuaAppDM entry 0x1023efe2c -> receiveCall
dispatch -> `bl 0x1023f1210` @0x1023f075c -> the 9-arg marshaler -> EC world 0x102e24598. SH235's verdict
left that gate as a static statement: "gated by StartLuaAppDM's receiveCall dispatch switch on live
controller state." Nobody had MEASURED where StartLuaAppDM's own flow terminates headlessly — i.e. whether
it even reaches 0x1023f075c, or diverts earlier in the dispatch. This cycle closes that gap with a
block-entry-definitive runtime measurement of StartLuaAppDM's OWN body.

## Measured (real libroblox.so, canonical completing --v2boot ladder, EXIT 124, 0 crash)

Region-watch on the FULL StartLuaAppDM body [0x1023efe2c, 0x1023f0800) plus the marshaler
[0x1023f1210, 0x1023f1300). On a clean completing run (full 9-rung ladder Ok end-to-end, incl.
`StartLuaAppDM returned Ok(0x5621099b6a40)` real heap, V2Start/V1AppStart/V2UpdateSurface/SendAppEvent all
Ok(0x3e8)):

- StartLuaAppDM's body enters exactly **14 distinct block-entry pcs**, the LAST being **0x1023f01e4**:
  `1023efe2c efeb0 efedc eff4c effa0 effac effc0 effc8 effd0 efffc f0008 f0020 f00f8 f01e4`.
- The **marshaler-call block at 0x1023f075c** (the `bl 0x1023f1210`, SH235's sole EC front-door) is
  **NEVER entered** (not among the 14; flow soft-returns before reaching it).
- The marshaler region [0x1023f1210,0x1023f1300) = **0 hits**.
- StartLuaAppDM then returns `Ok(0x5621099b6a40)` — benign soft-return — and the ladder continues on.

### Interpretation (block-entry-definitive)
Because every function/fragment is a distinct `cached_block` entry, non-appearance in the region-watch is
provable-non-execution (the SH228/SH231/SH232 methodology). The dispatch terminates headlessly **inside
helper fn 0x1023f00f8** (`sub sp,#0x70` = 0xd101c3ff; reachable from the entry dispatch tail 0x1023efed8
`blr x8`), which reads two guest-stack flag bytes [sp+8]/[sp+32] and benign-returns — the crab that stops
StartLuaAppDM short of building StartAppParams and reaching 0x1023f075c. This is the exact spot a future
drive must satisfy to fall through to the marshaler + EC world. Note helper#1 0x1023eff4c (`sub sp,#0x180` =
0xd10603ff) is also entered (a bigger sub-body); the flow passes through it then into 0x1023f00f8 and stops.

## Code (sh236, real-image guard family as sh235, skip-if-absent)

Byte-pins: StartLuaAppDM entry 0x1023efe2c=0xd10183ff (SH232 re-pin), entry dispatch-bit 0x1023efed8`
blr`=0xd63f0100, helper#1 prologue 0x1023eff4c=0xd10603ff, dispatch-helper prologue 0x1023f00f8=0xd101c3ff,
last-entered block 0x1023f01e4=0x394023e8, and re-checks the marshaler-call `bl` target 0x1023f075c ->
0x1023f1210 (SH235 pin). Window/4-alignment for all six. No production path edited.

Repro: `runs/capture_sh236_sladm_marshaler_unreached.sh` (region-watch full StartLuaAppDM body +
marshaler; expected = driving StartLuaAppDM + Ok soft-return + max entered pc 0x1023f01e4 + 0 marshaler hits).

## Verdict (honest, do-not-over-claim)

Does NOT manufacture a DataModel, does NOT lift the Route-B live-DM structural gate (SH209/218/223/224/
228/231/232/235 unchanged). It CONVERTS SH235's static "dispatch switch gates the EC front-door" into a
measured, located mechanism: StartLuaAppDM's receiveCall dispatch soft-returns in helper 0x1023f00f8
(last entered block 0x1023f01e4) before the marshaler call — so the next drive's exact obstacle is
satisfying that helper's stack-flag path (a live-state/fabricatable-graph class, matching SH235's
doctrine), not a static seed. Standing forward hooks unchanged (SH174 capture-latch arming at a real
make_shared<DataModel>; recon-v3 deliverables stay shipped + verified).

## Re-verify
`cargo test -p arm64jit --example elfjit sh236` = 1 passed (real-image guard).
`cargo test -p arm64jit --examples` = 81/0. `cargo test --workspace` green. `cargo build --workspace` EXIT 0.
Tree clean at end of commit.