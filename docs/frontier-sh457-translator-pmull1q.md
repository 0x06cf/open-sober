# SH457 — hermetic coverage of the POLYNOMIAL 64x64 MULTIPLY codegen (translate.rs Pmull1q — pmull/pmull2 Vd.1Q, Vn.1D, Vm.1D)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 657/0 incl. 3 new sh457 pins, was 654; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- Pmull1q (the 64x64 carry-less / polynomial multiply -> 128-bit — the
  carry-less form used by hash/checksum/reduce paths) had zero direct byte
  tests (STATUS next-forward #5 named `Pmull1q`). SH457 pins the exact emit
  (rd=1 rn=2 rm=3; Vn@0x130 Vm@0x140 Vd@0x120) with 3 exact-byte/window pins:
  (1) THE pmull low64 full-buffer (load-bearing): movq_load xmm0=[0x130]
  (f3 48 0f 7e, low64 of Vn) + movq_load xmm1=[0x140] (f3 48 0f 7e, low64 of
  Vm) + `pclmulqdq xmm0,xmm1,0x00` (66 0f 3a 44 c1 00) + movdqu_store [0x120]
  (f3 0f 7f, the 128-bit result); full-buffer assert_eq;
  (2) THE pmull2 (hi=true) high-half discriminator: hi selects the UPPER
  elements (bytes 8..15) so the sources advance +8 to 0x138/0x148 (vs hi=
  false's 0x130/0x140) while the pclmulq imm (0x00) and the Vd store (0x120)
  stay identical — the hi flag is the ONLY thing that moves the sources; a
  flub reading the low half multiplies the wrong polynomial elements; negative
  asserts hi=true never reads 0x130/0x140, controls hi=false reads them;
  (3) THE pclmulq imm + opcode lock: the carry-less semantic is the 3-byte
  PCLMULQDQ opcode 66 0f 3a 44 with the imm in the following /r ib — imm must
  be 0x00 (low64 x low64), must NOT be 0x01/0x10 (cross-half), and must NOT
  degrade to an integer `imul` (48 0f af) which would add instead of XOR-carry.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; pmull low64 full-buffer assert_eq matched first
  try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the polynomial-multiply family (the
  SSE4.2 PCLMULQDQ carry-less form, distinct from SH454/455's FP multiply).
  No re-treads.
- Files: docs/frontier-sh457-translator-pmull1q.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only).