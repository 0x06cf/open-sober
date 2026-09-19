# SH444 — hermetic coverage of the SIMD single-precision FP TWO-SOURCE arithmetic codegen family (translate.rs VecFpArith)

## frontier
SH443 pinned the byte-reverse (SimdRev) family. STATUS next-forward #5 names
the remaining unpinned translate.rs Inst families; this closes the SIMD
single-precision FP 2-src arithmetic (VecFpArith): fadd/fsub/fmul/fdiv
(op 0..3) + fmax/fmin/fmaxnm/fminnm (op 4..7) on Vd.2s/.4s lanes, and the
frecps/frsqrts (op 8/9) reciprocal-estimate helpers.

## gap being closed
SH433 pinned the FP multiply-ACCUMULATE (Fmla/FmlaEl — a vector-3/4-operand
dst accumulate). The plain per-lane 2-src arithmetic — every geometry/color
lane's add/sub/mul/div/max/min — had ZERO direct byte tests. The whole family
shares one movd-in / opcode / movd-out store shape; the ONLY semantic bit is
the middle opcode byte. A flub (say emitting mulss where fsub was wanted, or
swap max/min) silently wrong-computes every rendered pixel/color/vertex lane.

## emit pinned (rd=1 rn=2 rm=3; Vd@0x120 Vn@0x130 Vm@0x140)
Per lane (offsets +4*lanes): `mov eax,[rbx+Vn+4l]` (8b 83 d32) -> `movd xmm0,
eax` (66 0f 6e c0) -> `mov eax,[rbx+Vm+4l]` -> `movd xmm1,eax` (66 0f 6e c8)
-> the FP op -> `movd eax,xmm0` (66 0f 7e c0) -> `mov [rbx+Vd+4l],eax` (89 83
d32).

- addss (op 0) = F3 0F **58** C1
- subss (op 1) = F3 0F **5C** C1
- mulss (op 2) = F3 0F **59** C1
- divss (op 3) = F3 0F **5E** C1
- maxss (op 4/6) = F3 40 0F **5F** C1
- minss (op 5/7) = F3 40 0F **5D** C1

## discriminator facts (the exact forms a wrong emit silently corrupts)
1. opcode byte 58/5C/59/5E/5F/5D = add/sub/mul/div/max/min. Cross-asserted
   (fsub must NOT emit addss, etc).
2. **maxss/minss ALWAYS emit the REX prefix `0x40`** (the `x86::rex()` is
   called unconditionally in `CodeBuf::maxss/minss`, where the plain
   add/sub/mul/div omit it below GPR 8). So the emitted bytes are
   `F3 40 0F 5F C1` / `F3 40 0F 5D C1`, NOT `F3 0F 5F C1`. This is a genuine
   (harmless, mandatory-but-ineffective) REX.B for registers <8 — the real
   emission, and a useful presence/absence discriminator vs the non-REX
   add/sub/mul/div. (First attempt assumed REX=0x41; reading `rex()` confirmed
   bit3 of the reg is clear -> 0x40.)
3. fmaxnm/fminnm (op 6/7) deliberately reuse the maxss/minss opcodes (the NaN
   edge differs from x86 only on NaN inputs; a documented approximation).
4. frecps (op 8): the product path is DIFFERENT — `mulss(xmm0,xmm1)` (F3 0F 59
   C1) then the INVERTED sub `subss xmm1,xmm0` (F3 0F **5C C8**, dst=1 rm=0)
   computing 2.0 - prod, plus the f32 2.0 (0x40000000) imm. Inversion vs the
   plain fsub's F3 0F 5C C1 (dst=0 rm=1) is the discriminator.
5. frsqrts (op 9): same 2.0-prod base but adds `movd xmm3,eax` (66 0f 6e d8)
   of 0.5f (0x3f000000) + `mulss xmm1,xmm3` (F3 0F 59 CB).
6. lane addressing: Vn/Vm/Vd all advance exactly +4 per lane (0x130/0x140/0x120
   -> 0x134/0x144/0x124) — a stale/fixed stride silently reuses lane 0.

## tests
3 deterministic exact-byte pins in `crates/arm64jit/src/translate.rs`
(`#[cfg(test)]` only; translator core body byte-untouched; jit.rs/elfjit.rs/
session.rs unchanged). `tr_bytes()` synthetic Inst -> translate() (zero-pc
0x1000); [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16:
- `sh444_vecfparith_2s_add_full_lane_addressing` — full 2-lane .2s fadd exact
  byte buffer + lane-advance asserts.
- `sh444_vecfparith_opcode_discriminators_add_sub_mul_div_max_min` — the six
  opcode bytes + the always-REX-0x40 max/min fact + cross-absence.
- `sh444_vecfparith_frecps_inverted_sub_and_frsqrts_half_scale` — the op 8/9
  inverted-sub / 0.5-scale helpers.

arm64jit lib 606/0 (was 603). Deterministic, no image/env, parallel-safe.

## honest
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM structural gate UNCHANGED). BUILD-THE-RUNTIME
codegen-surface coverage completion on the 2-src scalar-FP family, distinct
from SH441 (cross-lane FMaxV reduction) and SH433 (Fmla accumulate). No
re-treads.