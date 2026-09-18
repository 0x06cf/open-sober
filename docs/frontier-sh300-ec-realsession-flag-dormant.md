# Frontier SH300 — EC-world realsession flag seed: dormant-by-measurement, latent-but-correct prep

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `JIT_ROUTEB_EC_REALSESSION=1`, wired alongside the SH298/298b/299 EC guards).

## tl;dr

The SH299 EC-world line (dmfn 0x1023f03b4 completing through the genuine
ExperienceController world 0x102e24598) was disassembled one more gate past the
SH299 dispatch to find its REAL engine-init branch. At `0x2e246f4` the EC body
reads the writable .bss byte **guest [0x106d31e28]** (`adrp 6d31000; ldrb
w8,[x8,#3624]; cbz w8,0x2e2472c`): flag=0 (its headless value) -> the benign
singleton builder `bl 23c1b0c` and SKIPS the real engine init fns entirely;
flag=1 -> `bl 23c5538` (real /nativeAppBridgeV2InitWithParams AppBridge-V2
singleton factory) + `bl 23f1654` (nativeAppBridgeStartLuaAppDM body +0x1828)
then a vt[+16] dispatch. SH300 implements a default-inert block-entry guard that
seeds `[0x106d31e28]=1` at the EC-world entry 0x102e24598 (idempotent —
`routeb_ensure_writable` maps the .bss page). **MEASURED dormant-by-measurement:
the flag fires and persists, but the EC body that reads it never runs headlessly
— the dmfn benign-returns upstream of the app-request build, so neither the real
V2Init (23c5538) nor the StartLuaAppDM body (23f1654) is ever entered.** No
terminal change (still the SH285-B LSM wall). Kept as latent-but-correct prep
(the SH265/266 precedent), NOT a dead-end-claim.

## Why the flag is dormant (measured, not assumed)

- The EC world 0x102e24598 compiles from entry as a SINGLE straight-line block
  (`JIT_REGION_WATCH`, `JIT_DUMP_PC`, and a `0x102e24598-0x102e25200` full-guess
  region all show only the entry pcs 0x102e24598 + 0x102e245f4 as block-entry
  transitions; the interior 0x2e24678..0x2e24744 are mid-block and cannot fire a
  block-entry probe). So the flag branch at 0x2e246f4 is unreachable by any
  entry-level instrumentation *once the body is a single compiled block*.
- Full `JIT_TRACE=1` pc-set diff (ON vs OFF the SH300 flag, real libroblox.so,
  same drive env): the EC-region pc set is IDENTICAL (only the two entry pcs) in
  both arms — the flag seed provably does not add or change any EC-world block
  that runs. The dmfn returns `Ok(0x...5410)` (= dmthis+0x30, unchanged) either
  way.
- Consequence: the app-request build (where the realsession branch lives) is the
  STANDING EC-body continuation — the same "construction body benign-soft-returns
  before the marshaler/EC continuation" class the SH299 NEXT-GATE note flagged.
  The flag is correct and is THE lever that would light those fns up, but the
  drive cannot reach the reader. Same honesty class as SH265/266 ("wired +
  measured-latent-but-correct"), explicitly NOT the SH248h/256 revert class
  (those failed to fire / corrupted state; this one fires cleanly and is
  default-inert, zero baseline perturbation).

## SH300 code

- `routeb_ec_world_realsession_guard` (jit.rs, opt-in `JIT_ROUTEB_EC_REALSESSION=1`):
  fires at EC-world entry block 0x102e24598; when `[0x106d31e28]==0` writes 1 via
  `routeb_ensure_writable` (idempotent; never touches a non-zero byte).
- Wired in the block-entry dispatch right after `routeb_ec_world_arg0_vt_guard` (SH299).
- +1 hermetic `sh300_ec_world_realsession_guard_is_env_pc_gated_and_seeds_flag`
  (env-gated / pc-gated / seeds canonical .bss cell / idempotent).

## HONEST (do-not-over-claim)

- No DM manufactured (MH_* false, DM-root 0). Route-B live-DM gate UNCHANGED.
- The realsession flag is a REAL lever (verified cell, verified branch mapping,
  verified idempotent seed) but its reader never runs headlessly yet — when the
  EC-body continuation becomes driveable, this flag is what switches it to the
  real V2Init/StartLuaAppDM path.
- SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays
  the single forward hook.
- Cause-not-symptom SESSION-CTOR: the EC-world real-init path is now mapped and
  armed (was "the branch exists" -> now "the branch exists, is correctly seeded,
  and is dormant only because its reader is not yet reached").

## Verify

- `cargo test -p arm64jit --lib -- sh300` = 1 passed (hermetic).
- `cargo test --workspace` EXIT 0 (workspace green).
- SH300 arm live run (real so): `[routeb-sh300] seeded EC-world realsession flag
  [0x106d31e28]=1 at pc=0x102e24598` fires; terminal unchanged SH285-B LSM wall
  0x101db1b08 (baseline parity).

## NEXT GATE (for the next frontier session)

The EC-body app-request build (0x2e24678..0x2e25200, incl. the now-armed real
V2Init/StartLuaAppDM branch) remains un-driven: the dmfn drives fn 0x1023f03b4 to a
benign `Ok(dmthis+0x30)` without descending into that continuation. Options: (a)
drive the dmfn DEEPER so the SH299 `cbnz w0 @0x2e2465c` path continues past the
v-branch into 0x2e24678..0x2e25200 (the marshaller 0x1023f1354 likely soft-returns
first — pin WHICH sub-path returns and whether a live object there advances it),
or (b) accept that the EC body needs a REAL session object (SH174/SH204
fabricatable-object-graph class) and keep grinding the SESSION-CTOR Activity drive.
Do-not-re-tread broad static seeds (SH174/204/248h/256 doctrine); keep single-drive
discipline (stacking with the app-start drive suppresses the EC continuation).