# Frontier SH387 — byte-anchor the DataModelServices current-DM getter ABI (the SEP-15 re-attack cone door)

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start (cargo test --workspace EXIT 0, 621/0; arm64jit lib 441/0) and at end
(after sh387: lib 442/0, full workspace gate re-run green).

## Why this cycle

The operator's SEP-15 ROUTE-B directive names a FRESH recon cone as the primary
re-attack target once the persistence track closes:

> re-attack the live-DataModel construction with a FRESH recon cone aimed at the
> ExperienceController / initializeLuaAppWithDataModel / DataModelServices::
> setDataModelToCurrent path (SH163 flagged 'next seed must target
> ExperienceController').

SH163 flags that cone by STATIC judgment; SH172/178/180 document the
setDataModelToCurrent current-DM holder in PROSE (CUR_DM_HOLDER at jit.rs
0x106391908, as a `0x2dbcc10 target`). But NO hermetic ever pinned the actual
0x2dbcc10 ABI on the real binary — the cone the operator wants re-attacked sits
nowhere in the test suite. SH387 pins it (SH386 "pin the named cone's bytes"
discipline), so the mapping the re-attack depends on is reproducible and
byte-groundable, not comment-anchored.

## The pins (real-image hermetic sh387, file-offset == guest-0x100000000)

- 0x2dbcc10 (guest 0x102dbcc10) — the current-DM GETTER, a pure leaf
  `adrp x0,6391000` (0xb001aea0) / `add x0,x0,#0x908` (0x91242000) / `ret`
  (0xd65f03c0). It returns the guest ADDRESS 0x106391908 (file 0x6391908 + 0x1_0000_0000)
  — the current-DM holder the harness's CUR_DM_HOLDER (jit.rs) plants a
  manufactured DM into (SH180/181).
- 0x2dbcc1c (guest 0x102dbcc1c) — the real BODY, a state-setter NOT a DM ctor:
  `sub sp,#0x40` (0xd10103ff), `stp x29,x30,[sp,#32]` (0xa9027bfd), `adrp
  x20,67d1000` (0xb001d0b4) + `ldr x20,[x20,#1776]` (0xf9437a94 = canary got
  0x67d16f0); dispatches via bl 0x24e3e98 / 0x2417d58 (persistence-family, NOT
  a DataModel construct). The DEBUG build's frame + stack cannotary read confirm
  a real function (this is the app's debug stack-cookie prologue family).

## What it means

The setDataModelToCurrent door the operator named as the cone is now byte-anchored:
(1) its GETTER returns exactly the current-DM holder the harness already seeds
(SH180/181 manufactured DM at CUR_DM_HOLDER), so the "plant a manufactured DM, the
setter consumes it" premise already has its exact target; (2) its BODY is a
persistence-family state-setter (dispatches through frames that reach the
measured-closed LSM/storage code), NOT a DataModel ctor — consistent with the
standing funnel (every re-opened door funnels to a closed lane). This is a
pin/verify contribution: it grounds the ONE cone the operator explicitly wants
re-attacked, so the next session's re-attack starts from verified bytes instead of
re-deriving the mapping.

## Honest

Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false, AppBridgeV2 [0x106a705e8]=0). recon-v3 immediate-priority
deliverables re-verified green at HEAD this cycle: (1) capture_taskv4_frame.sh attempt 1
= 24 real task-driven frames `present swap Ok(0x1)`, 196-197 node pops, 0 json abort,
0 crash, EXIT 0; (2) JIT_JSON_ZERO_FIX len-clamp present (jit.rs:6390); (3) the
session-gated producer stays INERT on bare boot (SH382/386 inverse control). SH174
capture-latch stays the single forward observer. Do-not-re-tread unchanged (the
experience-controller cone remains OPEN for a real re-attack armed with these pins;
all measured-closed lanes still stand: setDataModelToCurrent BODY is persistence-family,
do NOT re-drive it expecting a ctor).

## Files

- crates/arm64jit/src/jit.rs: +hermetic `sh387_dmservices_current_dm_getter_abi_pinned`
  (arm64jit lib 441->442). jit.rs ~1,019,xxx B (<1MiB hook).
- Repro: `cargo test -p arm64jit --lib sh387` (passes on this box against the real
  libroblox.so; skip-if-absent elsewhere).