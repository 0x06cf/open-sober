# SH441 — hermetic coverage of the SCALAR-REDUCTION + WIDEN codegen families
## (translate.rs FMaxV `fmaxv/fminv` / WidenShl `shll/shll2`)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (24 real task-driven frames `present swap Ok(0x1)`, 0 json
abort, 0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
594/0 incl. 4 new sh441 pins, was 590; cargo build --workspace + --example
elfjit OK). Production code ONLY in translate.rs `#[cfg(test)]` addition
(translator core body byte-untouched; jit.rs 1,048,390 B < 1MiB hook
unchanged; elfjit.rs/session.rs unchanged). Commit 3af34d9.

## What SH441 pins

FMaxV (`fmaxv/fminv Sd, Vn.4s`) and WidenShl (`shll/shll2`) had no direct
byte tests. SH441 pins the exact emitted x86 with rd=1, rn=2 (Vn base
= VECTOR_BASE 0x110+2*16 = 0x130; Vd base = 0x110+1*16 = 0x120):

1. **FMaxV fmaxv full buffer** — reduce Vn's 4 single lanes into scalar Sd:
   `mov eax,[rbx+0x130]` + `movd xmm0,eax` (Vn.0), then for i=1..3
   `mov eax,[rbx+0x130+4i]` (0x134/0x138/0x13c) + `movd xmm1,eax` +
   `maxss xmm0,xmm1` (f3 40 0f 5f c1), then `movd eax,xmm0` +
   `mov [rbx+0x120],eax`. The 0x5f (maxss) vs 0x5d (minss) opcode is the
   fmaxv-vs-fminv discriminator; the REX.B `40` byte is pinned.
2. **FMaxV fminv** — identical ladder but every reduce is `minss` (0x5d):
   asserts zero `maxss` (0x5f) present. A max/min cross silently picks the
   wrong lane extrema on a rendered color/height reduction.
3. **WidenShl shll signed vs unsigned** — dst_esize=4, nlanes=2 (src_esize=2),
   REVERSE iteration writes dst lane1 first (0x124 from src 0x132) then lane0
   (0x120 from 0x130). signed uses `movsx_word_mem` (`48 0f bf`), unsigned uses
   `movzx_word_mem` (`0f b7`, NO REX.W 0x48) — the REX.W presence is the
   signed-vs-unsigned discriminator a sign-extended lane picks the wrong value.
4. **WidenShl shll2 upper half byte widen** — dst_esize=2 (src_esize=1),
   nlanes=4, upper => src base = Vn + uh(8) = 0x138. Reverse iter src
   0x13b..0x138, dst lanes 0x126/0x124/0x122/0x120 (2 apart). signed byte widen
   `movsx_byte_mem` (`48 0f be`) + the 16-bit `66 89` store. Asserts the
   upper-half base (0x138, never the low 0x130) and the 16-bit store width
   (66-prefixed, never bare 32-bit 89).

Each test asserts the exact full emit AND the semantic discriminators
(specific lane displacements, opcode byte, REX.W presence/absence, upper-half
base, store width). Deterministic: synthetic `Inst` -> translate() ->
CodeBuf.as_slice() (zero-pc 0x1000), [RBX]=CpuState, vector slot
v[t]=VECTOR_BASE(0x110)+t*16. No image, no env, parallel-safe. Trunk verified
by a one-off eprintln dump (removed before commit) so pins match the REAL
emission.

## Honest

NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the scalar-reduction + widen families, continuing the
SH439/440 translator-core lineage (distinct from SH440 popcnt/sum8, SH437
lanecopy, SH439 fcvtzu). No re-treads.

## Files

- docs/frontier-sh441-translator-fmaxv-widenshl.md (this file)
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)