# SH454 — Hermetic coverage of the SCALAR 3-SOURCE FP FUSED-MULTIPLY codegen family (translate.rs Fma3 — fmadd/fmsub/fnmadd/fnmsub Dd, Dn, Dm, Da)

## Summary
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 646/0 incl. 4 new sh454 pins, was 642; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs 1,048,390
B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit 7a75a4f +5.

## What SH454 pins
`Fma3` (scalar 3-source FP fused multiply — Dd = Da ± (Dn × Dm), optionally
negated: fmadd/mnsub/fnmadd/fnmsub) had zero direct byte tests. This is the
most render-heavy scalar transform surface (every matrix/vertex/lighting
dot-product path). SH454 pins the exact emit (rd=1 rn=2 rm=3 ra=4; Dn@0x130
Dm@0x140 Da@0x150 Dd@0x120) with 4 exact-byte pins:

1. **THE fmadd product-and-accumulate direction (the load-bearing pin).**
   fmadd d1,d2,d3,d4 (sz=true sub=false neg=false): movq_load xmm0=[0x130] (Dn,
   f3 48 0f 7e) + movq_load xmm1=[0x140] (Dm) + `mulsd xmm0,xmm1` (f2 0f 59 c1,
   prod into xmm0) + movq_load xmm2=[0x150] (Da) + `addsd xmm2,xmm0` (f2 0f 58
   d0 = da + prod) + movq_store [0x120] (66 48 0f d6). The prod direction is
   rn*rm and the accumulate is ra+prod — never prod+ra. Full-buffer assert_eq.

2. **THE add-vs-sub opcode AND the operand ORDER are the semantic.** fmsub
   (sub=true neg=false): `subsd xmm2,xmm0` (f2 0f 5c d0) = da - prod; fnmsub
   (sub=true neg=true): `subsd xmm0,xmm2` (f2 0f 5c c2) = prod - da — the
   signed negation makes fnmsub rn*rm - da, REQUIRING the operands SWAPPED (a
   lone subss in the wrong order negates the wrong term). The 0x58/blob0x5c
   opcode byte plus the c0/c2 operand order discriminate all four FMA combos.

3. **THE fnmadd negate-via-scratch (pxor + 0-sub).** fnmadd (sub=false
   neg=true): addsd xmm2,xmm0 then `pxor xmm3,xmm3` (66 0f ef db, +0.0) + `subsd
   xmm3,xmm2` (f2 0f 5c da = 0 - result) to negate, storing FROM xmm3. The
   pxor+subsd-into-scratch is the negate discriminator — a flub that skips the
   0-sub leaves the sign un-negated. Positional assert the add precedes the
   negate.

4. **THE single-vs-double width discriminator.** fmadd s1,s2,s3,s4 (sz=false):
   swaps to movd_xmm_r32 (66 0f 6e) + `mulss xmm0,xmm1` (f3 0f 59 c1) + `addss
   xmm2,xmm0` (f3 0f 58 d0) + movd_r32_xmm (66 0f 7e) + 32-bit store (89 83).
   The F3-mulss-vs-F2-mulsd prefix is the single-vs-double discriminator;
   negative asserts the single path emits NO mulsd/addsd/movq_store.

## Method
Deterministic: synthetic `Inst::Fma3` -> `translate()` -> `CodeBuf.as_slice()`
(zero-pc 0x1000). [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16.
Emission captured precisely with a one-off probe test (eprintln dump, removed
before commit) so the pins match the real emission; the fmadd double full-
buffer assert_eq matched the capture first try. 4 exact-byte/window pins,
parallel-safe, no image/env.

## Honesty
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the scalar 3-source FP fused-multiply family, distinct
from SH433 (vector Fmla/FmlaEl element-wise accumulate). No re-treads. Commit
7a75a4f +5.