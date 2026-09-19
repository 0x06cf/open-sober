# SH450 — Hermetic coverage of the SIMD FP COMPARE-TO-LITERAL-ZERO codegen family (translate.rs VecFpCmpZero — fcmeq/fcmgt/fcmge/fcmlt/fcmle Vd.T, Vn.T, #0.0)

## Summary
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 630/0 incl. 4 new sh450 pins, was 626; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs 1,048,390
B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

## What SH450 pins
`VecFpCmpZero` (the per-lane FP compare-to-literal-`#0.0` mask builder — the
zero-comparison cousin of SH445's two-operand VecFpCmp; a lane becomes all-
ones if `Vn op 0.0` holds, else 0) had zero direct byte tests. SH450 pins the
exact emit (rd=1 rn=2; Vn@0x130 Vd@0x120) with 4 exact-byte tests:

1. **THE zero-operand discriminant (the load-bearing pin).** Each lane first
   materializes the `+0.0` second operand with `pxor xmm1,xmm1` (66 0f ef c9) —
   a FRESH zero every lane — then mov_load32 (8b 83) + `movd xmm0,eax`
   (66 0f 6e c0) + comiss (40 0f 2f c1) + the setcc + movzx (0f b6 c0) + `neg
   rax` (48 f7 d8) + 32-bit store (89 83). This is what distinguishes the
   zero-operand form from SH445's two-operand VecFpCmp, which LOADS Vm — a lone
   setcc byte cannot tell them apart (same comiss + same cc), so the pxor is
   the ONLY reliable differentiator. A flub that reuses a stale/shared register
   instead of re-zeroing per lane compares against garbage.

2. **THE cc-map is the semantic** (like SH445): op 0 fcmeq = sete 0f 94, op 1
   fcmgt = seta 0f 97, op 2 fcmge = setae 0f 93, op 3 fcmlt = setb 0f 92, op 4
   fcmle = setbe 0f 96 — a wrong cond silently picks the wrong comparison
   (eq-into-0 becomes gt-into-0). Negative asserts pin that a transposed op
   must NOT emit the wrong cc.

3. **Double-path width discriminator** (op 0, esize=8): 1 lane (8/8), uses
   movq_load (f3 48 0f 7e) + comisd (66 40 0f 2f c1) + 64-bit store (48 89 83) —
   and must NOT emit the single-path movd (66 0f 6e c0).

4. **Lane-advance math for the full 4s q=true form**: 4 lanes, Vd@0x120/0x124/
   0x128/0x12c, Vn@0x130/0x134/0x138/0x13c, 4 pxor xmm1,xmm1 counted — the q
   bit doubles the lane count and confirms the +esize advances across all
   vectors, with a negative assert that the 4s-q form stays single-path (no movq).

## Method
Deterministic: synthetic `Inst::VecFpCmpZero` -> `translate()` ->
`CodeBuf.as_slice()` (zero-pc 0x1000). [RBX]=CpuState; vector slot
v[t]=VECTOR_BASE(0x110)+t*16. Emission captured precisely with a one-off probe
test (eprintln dump, removed before commit) so the pins match the real
emission — the pxor-per-lane + exact cc bytes came from that capture, and the
2s full-buffer assert_eq matched first try.

## Honesty
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the compare-vs-literal-zero family, adjacent to SH445
(two-operand VecFpCmp) and SH449 (integer SimdCmpZero). No re-treads (distinct
forms). Commit dc84448 +4.