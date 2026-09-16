# Frontier SH229 — FULL DataModel ctor drive (un-NOP'd subobject) = measured instance-base redirect, not a DM

Status: single-agent Route-B re-attack, NEW measurement (SH187 only ever ran the PARTIAL NOP'd
build). +1 hermetic sh229 (arm64jit lib, 389/0). Opt-in env `JIT_ROUTEB_DM_REALCTOR_FULL`.
frontier: Route-B "drive its ctor world-build further" line.

## What was open

SH187 drove the REAL DataModel ctor wrapper (0x1023f5ff8 -> bl 0x1023f6038 + subobj-init call
`bl 0x23f6b0c` @ 0x1023f60b8) but ONLY as a PARTIAL build: it NOP'd that subobject call
(0x94000295 -> 0xd503201f) so the straight-line ctor body fell through to the genuine-vptr
writes at 0x23f6130, yielding obj vptr `{0x1067162e8, 0x1067163a0, 0x1067163f8}` GENUINE MATCH.
SH187 explicitly noted the un-NOP'd ctor stalls inside `bl 0x23f6b0c` and never tried driving it
to completion — the operator's "drive the DM ctor world-build further" line. SH189 recon asserts
the subobject builds the DM's INTERNAL 361-entry class/instance index (bl 0x2374c90, x0=obj+0x2a0,
w1=0x169, x2=&stack-pair) — a MORE-complete DM than the NOP'd partial.

## What we did

Added `JIT_ROUTEB_DM_REALCTOR_FULL` (seed modifier on the real-ctor guard; default-inert; requires
the base `JIT_ROUTEB_DM_REALCTOR=1`). Under FULL the guard does NOT apply the SH187 NOP — it drives
the ctor leaving `bl 0x23f6b0c` live so the subobject/index build runs — and the drive read-back
additionally dumps obj+0x1f0 + the obj+0x2a0 index-build region. Hermetic test asserts the guard
is env+region gated (sh229).

## Measured (real libroblox.so, canonical --v2boot ladder, EXIT 124, 0 SIGSEGV/SIGABRT, reproducible baseline)

- PARTIAL (SH187, NOP'd) at this HEAD re-verified: `obj vptr set = 0x1067162e8,0x1067163a0,0x1067163f8 GENUINE MATCH` — planted into current-DM holder 0x106391908.
- FULL (SH229, un-NOP'd): `REAL DM ctor wrapper DROVE ok ret x0=0x7fd7834c1361; obj vptr set = 0x1067147c8,0x1067bdb20,0x106796dc0 (genuine=false)`; `obj+0x1f0 vptr=0x106796dc0; obj+0x2a0 index-build = 0x0,0x0,0x0`.

## Interpretation (measured, not re-tread)

The FULL ctor does NOT stall/crash this cycle (SH187's "stalls inside 0x23f6b0c" note is a
different transient — here the wrapper returns) — **but it does not advance the DM**: the un-NOP'd
subobject ctor redirects the object construction to the **instance-base vptr family
0x106796dc0** (the SH190c instance base — a DIFFERENT class, the Instance/creatable base, NOT the
derived RBX::DataModel), and the 361-entry index region (obj+0x2a0) stays all-zero (the index build
did not populate). So the partial NOP actually selects FOR the derived genuine-DM vptr set; running
the subobject produces a base-class-shaped object, which is upstream of the DataModel, not more
DataModel.

## Honest verdict

This converts the operator's "drive the full ctor" re-attack angle into a MEASURED result: the
un-NOP'd full ctor yields the instance-base family, not the derived DM — the SH187 NOP was
load-bearing for genuine-DM production, and there is no evidence the full ctor gets closer. This is
consistent with the standing Route-B structural gate (SH209/218/223/224/228 unchanged). The PARTIAL
drive remains the correct genuine-DM manufacture path and is UNREGRESSED (re-verified GENUINE MATCH
this cycle). No production path edited in the default (env-off) configuration.

## Code / verify

- `crates/arm64jit/src/jit.rs`: real-ctor guard gains the FULL env branch + FULL-mode read-back,
  default-inert. Hermetic `sh229_dm_real_ctor_full_mode_env_and_region_gated` (1 passed).
- Repro: `runs/capture_sh229_real_ctor_full.sh` (FULL) and `runs/capture_sh187_real_ctor.sh`
  (PARTIAL baseline, UNREGRESSED).
- `cargo test --workspace` green; `cargo build --workspace` green. recon-v3 render plane
  re-verified at HEAD (24 task-driven frames, 194 node pops, no json abort, EXIT 124).

## Next (honest, single-agent)

Route-B live-DM = structural gate UNCHANGED. The FULL-ctor variant is measured as a base-class
redirect (do-not-re-tread as "more DM"). SH174 capture-latch arming at a real make_shared = the
single forward hook. recon-v3 deliverables stay shipped + verified.