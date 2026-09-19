# SH445 — hermetic coverage of the SIMD single-precision FP COMPARE->mask codegen family (translate.rs VecFpCmp — fcmeq/fcmgt/fcmge/facgt/facge)

## frontier
SH444 pinned the SIMD 2-src FP arithmetic (VecFpArith). The adjacent
compare->mask family (VecFpCmp: fcmeq/fcmgt/fcmge Vd.T, Vn.T, Vm.T, plus the
abs variants facgt/facge) produces the all-ones/all-zero per-lane comparison
masks every shader-like branch / blend / color-decision lane leans on — and had
ZERO direct byte tests.

## gap being closed
VecFpCmp had no byte coverage. Its whole job is to turn each lane compare into
an ALL-ONES (matched) or 0 (missed) mask; a wrong setcc condition byte picks
the wrong comparison, and a wrong width (single vs double) silently corrupts
the mask. DISCOVERY (SH445 vs SH444): the emitter method `CodeBuf::comiss`/
`comisd` ALSO always emit the 0x40 REX (like maxss/minss) — the single compare
comiss(0,1) = `40 0F 2F C1`, the double comisd(0,1) = `66 40 0F 2F C1`.

## emit pinned (rd=1 rn=2 rm=3; Vd@0x120 Vn@0x130 Vm@0x140)
single (esize=4, .2s q=false / .4s q=true), per lane +4:
`mov eax,[rbx+Vn+4l]` (8b 83 d32) -> movd xmm0,eax (66 0f 6e c0) -> `mov
eax,[rbx+Vm+4l]` -> movd xmm1,eax (66 0f 6e c8) -> comiss xmm0,xmm1 (40 0f 2f
c1) -> `set?cc al` -> movzx eax,al (0f b6 c0) -> neg rax (48 f7 d8 =>
+1 -> 0xffff_ffff all-ones) -> `mov [rbx+Vd+4l],eax` (89 83 d32).

cc (the semantic discriminator): fcmeq = sete `0F 94 C0`, fcmgt = seta `0F 97
C0`, fcmge = setae `0F 93 C0`. (lt/le are gt/ge with Vn/Vm swapped by the
decoder.)

double (esize=8, .2d): `movq xmm0,[rbx+0x130]` (f3 48 0f 7e 83) -> `movq
xmm1,[rbx+0x140]` (f3 48 0f 7e 8b) -> comisd (66 40 0f 2f c1) -> sete ->
movzx -> neg -> `mov [rbx+0x120],rax` (48 89 83, 64-bit). Width discriminator:
the double path uses movq_load (f3 48 0f 7e) exclusively — it never emits the
single path's movd (66 0f 6e c0).

abs (facgt/facge): clears each lane's sign bit BEFORE the compare — merges
0x7fffffff (mov rax,imm, 48 b8 ff ff ff 7f ..), movq xmm2,rax (66 48 0f 6e d0),
pand xmm0,xmm2 (66 0f db c2) + pand xmm1,xmm2 (66 0f db ca), THEN comiss. The
pand-before-compare ordering is the abs-vs-plain discriminator.

## tests
3 deterministic exact-byte pins in `crates/arm64jit/src/translate.rs`
(`#[cfg(test)]` only; translator core body byte-untouched):
- `sh445_veccmp_2s_fcmeq_allones_mask_full_buffer_and_cc` — full 2-lane .2s
  fcmeq exact byte buffer + the neg-(1->all-ones) mask fact + cc ladder
  (sete 94 / seta 97 / setae 93, cross-absent) + lane-advance + the
  always-REX 0x40 comiss.
- `sh445_veccmp_2d_comisd_width_discriminator` — full .2d fcmeq (esize=8)
  byte buffer + comisd 66 40 0F 2F + movq_load + 64-bit store + the
  movd-absence width discriminator.
- `sh445_veccmp_abs_signmask_pand_before_compare` — the abs sign-clear (0x7f…
  const + movq xmm2 + pand both lanes) and the pand-before-comiss ordering.

arm64jit lib 609/0 (was 606). Deterministic, no image/env, parallel-safe.

## honest
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM structural gate UNCHANGED). BUILD-THE-RUNTIME
codegen-surface coverage completion on the compare->mask family, adjacent to
SH444 (VecFpArith 2-src FP arithmetic). No re-treads.