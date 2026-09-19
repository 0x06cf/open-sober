# SH459 — hermetic coverage of the SATURATING NARROWING-SHIFT codegen (translate.rs SatNarrowShift — sqshrn/uqshrn/sqshrun Vd.T, Vn.T, #imm)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 663/0 incl. 3 new sh459 pins, was 660; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- SatNarrowShift (the shift-then-SATURATING-narrow — each src element shifted
  right by `imm` then clamped to the DST element's range, the color-channel /
  narrow-packing path that must not wrap) had zero direct byte tests (STATUS
  next-forward #5 named `ShrAcc2`, and the saturating-narrow family was
  adjacent-unpinned to SH451's narrowing-shift). SH459 pins the exact emit
  (rd=1 rn=2; src@0x130 dst@0x120) with 3 exact-byte/window pins:
  (1) THE sqshrn .4H lane-0 full emit (load-bearing): mov_load32 + `shl rax,32`
  (48 c1 e0 20) + `sar rax,32` (48 c1 f8 20 = the 32-bit SIGN-EXTEND) + `sar
  rax,6` (48 c1 f8 06, arithmetic — signed src) + clamp cmp 0x...8000 (48 39
  c8) + cmovl 48 0f 4c c1 + cmp 0x7fff + cmovg 48 0f 4f c1 + 16-bit store (66
  89); plus the q=false upper-half-zero (mov rax,0 + mov [0x128],rax);
  (2) THE signed-vs-unsigned shift + clamp discriminator: signed src uses shl+
  `sar rax,#` (48 c1 f8, arithmetic) with the shl/sar-64 sign-extend and clamps
  to [-0x8000, 0x7fff]; unsigned uses a bare `shr rax,#` (48 c1 e8, logical,
  NO shl/sar sign-extend) and clamps to [0, 0xffff] — the E8-vs-F8 shift byte
  AND the clamp constants are the semantic (a flub clamps to the wrong bound or
  shifts the wrong direction, corrupting every narrowed lane);
  (3) THE q=true full-16B + byte-width discriminator: uqshrn .16B (src_esize=2
  dst_esize=1 q=true) zero-extends via movzx (0f b7 83) + `shr rax,4` + clamps
  to [0, 0xff] + BYTE stores (88 83) across Vd (0x120..0x12f) with NO
  upper-half-zero (q=true fills the whole register) — the q-false square-zero
  is absent and the byte-vs-word store is the dst_esize width discriminator.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission (one lane-0 slice width dev-fix: 35 bytes, not
  44). No real binary/env needed; parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the saturating-narrow family, distinct
  from SH451's plain narrowing-shift (which does NOT clamp) and SH434's plain
  shift (which does NOT narrow). No re-treads.
- Files: docs/frontier-sh459-translator-satnarrowshift.md + crates/arm64jit/
  src/translate.rs (`#[cfg(test)]` only).