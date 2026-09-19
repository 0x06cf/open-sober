# SH446 — hermetic coverage of the SIMD by-element FMUL codegen family (translate.rs SimdFmulEl — fmul Vd.T, Vn.T, Vm.T[L])

## frontier
SH445 pinned the FP compare->mask family. This closes SimdFmulEl — the
by-element single-precision FMUL (each lane of Vd = Vn[lane] * Vm[L] for one
fixed element index L), a common vector-by-element multiply on vertex/weight/
color paths. It is DISTINCT from SH433's FmlaEl (which is a multiply-ACCUMULATE
into the dst); SimdFmulEl is a plain multiply with no accumulate term.

## gap being closed
SimdFmulEl had no direct byte tests. Its load-bearing discriminator is the
**rd==rm broadcast-once guard**: the element Vm[L] must be loaded into xmm2
EXACTLY ONCE up front, BEFORE the lane loop. When rd==rm (e.g. a compiler
writing a self-multiply), the first lane's store to Vd overlaps the element
source, so a per-lane re-read of the element corrupts every lane after 0 with
a clobbered value — a silent wrong vector. SH433's FmlaEl ABI note (SH399)
captured the broadcast form; the plain-FMUL variant was untested.

## emit pinned (rd=1 rn=2 rm=4; Vd@0x120 Vn@0x130 Vm@0x150)
single (esize=4), per lane:
`mov eax,[rbx+m-el]` (8b 83 d32) -> `movd xmm2,eax` (66 0f 6e d0) — ONE time up
front — then per lane `mov eax,[rbx+Vn+4l]` (8b 83) -> movd xmm0,eax (66 0f 6e
c0) -> `mulss xmm0,xmm2` (F3 0F 59 C2) -> movd eax,xmm0 (66 0f 7e c0) -> `mov
[rbx+Vd+4l],eax` (89 83 d32). m-el = f(rm) + index*es (rm=4 -> 0x150; idx1 ->
0x154).

double (esize=8), 1 lane: `movq xmm2,[rbx+m-el]` (f3 48 0f 7e **93** — the rm
reg-field=2 makes modrm 0x93, vs the single's movd 66 0f 6e) -> movq_load Vn
(f3 48 0f 7e 83) -> `mulsd xmm0,xmm2` (F2 0F 59 C2) -> movq_store Vd (66 48 0f
d6). Width discriminator: double uses mulsd (f2) + movq (66 48 0f ...), never
the single mulss (f3 0f 59 c2).

## discriminator facts
1. broadcast total: exactly ONE `movd xmm2,eax` (66 0f 6e d0) regardless of
   lane count (2s -> 1, 4s -> 1); lane product count = lane count (2s -> 2, 4s
   -> 4) via mulss xmm0,xmm2 (f3 0f 59 c2).
2. element addressing: source = Vm base + index*es (rd=1 rm=4 -> Vm@0x150;
   index scales the element byte offset — 0x150 idx0, 0x154 idx1).
3. **rd==rm clobber guard**: when rd==rm the broadcast (66 0f 6e d0) must
   precede the overlapping Vd store (89 83 <Vm base disp>), and the element
   source is read from memory EXACTLY ONCE — a per-lane re-read after the
   clobbering store corrupts lanes >0. Both pinned.
4. double width: mulsd (f2 0f 59 c2) + movq store (66 48 0f d6), never single
   mulss.

## tests
4 deterministic exact-byte pins in `crates/arm64jit/src/translate.rs`
(`#[cfg(test)]` only; translator core body byte-untouched):
- `sh446_fmulel_2s_full_buffer_broadcast_once` — full 2-lane .2s byte buffer +
  broadcast-once count + element-at-0x150 + per-lane mulss count.
- `sh446_fmulel_4s_index_and_lane_advance` — .4s (q=true) idx=1: element at
  0x154, still one broadcast, 4 lane products, dst lanes 0x120..0x12c.
- `sh446_fmulel_2d_double_broadcast_and_mulsd_width` — .2d esize=8 idx=1:
  movq xmm2 @0x158 + mulsd + movq store + mulss-absence width discriminator.
- `sh446_fmulel_rd_eq_rm_broadcast_captures_before_overlap_store` — the guard:
  broadcasting 66 0f 6e d0 precedes the overlapping Vd@0x150 store; element
  source read from memory exactly once.

arm64jit lib 613/0 (was 609). Deterministic, no image/env, parallel-safe.
Emission established precisely with a one-off eprintln dump (removed before
commit) so pins match the real emission.

## honest
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM structural gate UNCHANGED). BUILD-THE-RUNTIME
codegen-surface coverage completion on the by-element FMUL family, distinct
from SH433 (FmlaEl multiply-accumulate). No re-treads.