# SH455 — hermetic coverage of the SCALAR 2-SOURCE FP MULTIPLY codegen (translate.rs FmulScalar — fmul/fnmul Sd/Dd, Sn, Sm)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 650/0 incl. 4 new sh455 pins, was 646; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- FmulScalar (the scalar 2-source FP multiply `Dd = (+/-)(Dn * Dm)` — the
  same product core as SH454's Fma3 but with NO accumulate into a 4th operand)
  had zero direct byte tests (STATUS next-forward #5 named `FmulScalar` / the
  scalar-fp remainder). SH455 pins the exact emit (rd=1 rn=2 rm=3; Dn@0x130
  Dm@0x140 Dd@0x120) with 4 exact-byte/window pins:
  (1) THE fmul double full-buffer (load-bearing): movq_load xmm0=[0x130]
  (f3 48 0f 7e 83, Dn) + movq_load xmm1=[0x140] (f3 48 0f 7e 8b, Dm) +
  `mulsd xmm0,xmm1` (f2 0f 59 c1) + movq_store [0x120] (66 48 0f d6 83);
  full-buffer assert_eq; negative asserts NO mulss and NO accumulate addsd
  (a pure multiply must not accumulate);
  (2) THE fnmul/double negate discriminator: the single neg flag flips the
  sign via `pxor xmm0,xmm1` (66 0f ef c1) after loading the 64-bit sign
  constant 0x8000_0000_0000_0000 into RCX (48 b9 .. 00 00 00 00 00 00 00 80)
  + `movq xmm1,rcx` (66 48 0f 6e c9), sign-flip BEFORE the store (positional),
  and the pxor must be ABSENT when neg=false (control);
  (3) THE fmul single width: swaps to mov_load32 RCX (8b 8b) + `movd xmm0,ecx`
  (66 0f 6e c1) + `mulss xmm0,xmm1` (f3 0f 59 c1) + `movd ecx,xmm0` (66 0f 7e
  c1) + 32-bit store (89 8b 0x120) — the F3-mulss-vs-F2-mulsd prefix IS the
  single-vs-double width discriminator; negative asserts no mulsd and no
  movq_store;
  (4) THE fnmul/single negate WIDTH: uses the 32-bit sign constant
  0x8000_0000 (48 b9 00 00 00 80 00 00 00 00) + `movd xmm1,ecx` (66 0f 6e c9)
  + pxor — the 32-bit-vs-64-bit sign const (and movd-vs-movq) is the negate
  width discriminator, negative assert no movq xmm1,rcx.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; fmul_double full-buffer assert_eq matched the
  captured buffer first try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the scalar 2-source FP multiply
  family, the complement of SH454 (scalar 3-source FMA) — the pure fmul/fnmul
  with the pxor-sign-flip negate distinct from Fma3's 0-sub. No re-treads.
- Files: docs/frontier-sh455-translator-fmuscalar.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only).