# SH467 — instruction-level static grounding of the Route-B once-slot store (+ fresh-HEAD re-verification)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverable re-verified
GREEN at the fresh HEAD dfd7a4c FIRST (capture_taskv4_frame.sh attempt 1: **24 real
task-driven frames** `present swap Ok(0x1)`, dispatch #2201000, 0 json abort,
0 crash, exit 124/stable; artifact at runs/sh60-taskv4-frame.txt). This matters
because SH462 touched translate.rs (default-inert emit observer) and SH464/465/466
touched session.rs — all production code that must not have broken the task-frame
path; the fresh-head capture confirms it did not. Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 677->678 incl. 1 new sh467 hermetic; cargo build
--workspace OK). Production code UNCHANGED — test-only session.rs addition.

## The finding

SH462 measured at the STORE level that pc=0x102206d74 (nativeGameGlobalInit's
once-lambda, file 0x2206d74) writes the sentinel 0x400000b into once-slot
[0x106a68408], and that NO guest store ever reaches DM-root [0x106a68818]. SH467
adds the instruction-level static chain that PRODUCES that value and proves why
DM-root is structurally unreachable there, verified by disassembling the real
libroblox.so (aarch64-linux-gnu-objdump, segment map offset 0x232ed4 -> vaddr,
guest = file vaddr + 0x100000000):

- **The once-lambda** (file 0x2206d60-0x2206d88): `adrp x23, 0x6a68000` (=guest
  0x106a68000), then `str x0, [x23, #1032]` at file 0x2206d74 writes ONLY
  [x23+0x408]=once-slot [0x106a68408]. Its x0 is the return of `bl 0x2173b3c`.
  The only other visitor in the lambda is [x23+0x410]=once-guard [0x106a68410]
  (adrp x0,0x6a68000 + #0x410; bl 0x284cf5c) then `b 0x2206c88`.
- **The helper 0x2173b3c is NOT a DM-constructor.** It is a `strcmp`-dispatch
  settings shim: `adrp x1, 0x29bbcb; bl strcmp` (literal at file 0x29bbcb =
  "GPU"), then tail `b 0x61e30bc` (guest 0x10161e30bc — a large
  FMOD/telemetry-class dispatcher) with w3=cset eq. So the 0x400000b the
  store-watch caught in the once-slot is a SETTINGS-dispatch return, refining
  SH381's "Execute sentinel" label for THIS value: this lambda runs a first-call
  settings init, not the DM world-build. The real DM builder is a DIFFERENT,
  real-session-owned path.
- **DM-root [x23+0x818]=[0x106a68818] is structurally absent from the lambda** —
  the store only touches +0x408 (once-slot). This is the instruction-level
  confirmation of SH462's store-level measure: no guest store can synthesize
  [0x106a68818] here; it is produced only by an upstream session ctor the JIT
  cannot drive (SH397/SH462/463).

## What changed

- session.rs: new hermetic `sh467_once_lambda_static_chain_grounds_dmroot_gap`
  that pins the verified chain as constants (once-lambda store pc 0x102206d74;
  x23 base 0x106a68000; once-slot +0x408; once-guard +0x410; DM-root +0x818
  gap; helper 0x102173b3c; "GPU" literal 0x10029bbcb; helper tail 0x10161e30bc;
  store-vs-do-init-entry distinct sites 0x2206d74 != 0x2206db8). Pure, no
  env, no image, parallel-safe — same provenance as the SH463 address-map pin,
  so future recon starts from a verified chain and a mis-transcribed address
  fails loudly.
- Test-only; no production line touched; default-inert runtime byte-identical.

## Honest

NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root 0 under
the complete substrate). This refines the mental model of the once-slot sentinel
(the "0x400000b Execute" label is more precisely a settings-dispatch return; the
once-lambda is not the DM builder) and re-verifies the recon-v3 runtime
deliverable at a fresh HEAD after a production-code window. No re-treads, not a
render-plane visual, no DM seed. The builder of a live DM remains the
real-session-owned upstream ctor (SH397/462/463).

Files: docs/frontier-sh467-once-lambda-static-chain.md + crates/arm64jit/src/
session.rs (test-only). Commit (SH467).