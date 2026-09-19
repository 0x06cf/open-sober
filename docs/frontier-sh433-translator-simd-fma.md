# SH433 — SIMD FP multiply-accumulate hermetics (translate.rs Fmla, FmlaEl)

Date: Sep 19, 2026 · hermes-worker · single-agent (cone suppressed)

## What

Hermetic coverage of the SIMD FP multiply-accumulate codegen family — the
single most render-heavy translate.rs surface (matrix/vertex/lighting
transforms accumulate as SIMD FMAs). decode.js + runtime were pinned but the
byte EMISSION had no direct hermetics. SH433 adds 4 deterministic exact-byte
pins (synthetic `Inst` → `translate()` → `CodeBuf`, zero pc, no image/env,
parallel-safe; subsequence-window asserts for the multi-lane bodies) noting the
discriminators a byte error silently corrupts.

## Pins

1. **Fmla .2s add** — per-lane `movd xmm0=Vn[l]; movd xmm1=Vm[l]; mulss
   xmm1,xmm0` (F3 0F 59 C8 → product into xmm1); reload `movd xmm0=Vd[l];
   xmm0,xmm1` (F3 0F 58 C1). The product DIRECTION is the
      load-bearing point: because `mulss` targets xmm1, the subsequent `addss`
   xmm0,xmm1` computes `Vd += Vn*Vm` — giving fmls the CORRECT sign (Vd −
   Vn·Vm, never Vn·Vm − Vd). Pinned product-before-accumulate ordering.
2. **Fmla .2s sub (fmls)** — same layout, `subss xmm0,xmm1` (F3 0F 5C C1). The
   0x5C-vs-0x58 accumulate-opcode discriminator; a subtrahend swap corrupts sign.
3. **Fmla .2d (double)** — double lane width uses movq load/store (F3 48 0F 7E /
   66 48 0F D6) and mulsd/addsd (F2 0F 59/58), NEVER the .2s single-precision
   F3+movd/mulss path. A width flub silently halves/squares transform math.
4. **FmlaEl .2s (by-element)** — broadcasts `Vm.el[idx]` into xmm2
   (`movd xmm2,eax` = 66 0F 6E D0) once, then per-lane `mulss xmm1,xmm2`
   (F3 0F 59 CA). The broadcast-target (rm=xmm2) vs the 3-operand Fmla's
   full-Vm-lane multiply (mulss xmm1,xmm0 = C8) is the discriminator.

## Files

- `crates/arm64jit/src/translate.rs` — 4 new hermetics in `translate::tests`
  (pure `#[cfg(test)]`, +4 tests); translator core body byte-untouched;
  jit.rs/elfjit.rs/session.rs unchanged.
- This doc.

## Honest status

NOT a DM (DM-root [0x106a68818]=0 under the complete substrate; Route-B live-DM
structural gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface coverage
completion on the SIMD FP multiply-accumulate family (the matrix/vertex/lighting
math), continuing the SH427-432 translator-core lineage. No re-treads (distinct
from the SimdVLog/SminMax/SimdSatAdd integer-lane family of SH432). Workspace
green (arm64jit lib 560/0 incl. 4 new sh433 pins, was 556; workspace 741
passed/0 failed; cargo build --example elfjit OK). Recon-v3 deliverables
re-verified green at HEAD (24 real task frames, 194 pops, 0 json abort, 0
crash).