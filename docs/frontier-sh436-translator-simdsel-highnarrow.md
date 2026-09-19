# SH436 — Hermetic coverage of the SIMD bitwise-select + high-narrow families

Family: `translate.rs` `SimdSel`, `SimdHighNarrow`.

## What was uncovered

decode.rs pins decode, jit.rs pins runtime, but the byte EMISSION between them
was unpinned for the 3-input bitwise select (bsl/bit/bif) and the narrowing
add/sub (addhn/subhn/raddhn/rsubhn) that drive blend masks and byte-level
color/normal packing in rendered content. SH436 pins the discriminators a byte
error silently corrupts (4 exact-byte hermetics, `#[cfg(test)]` only).

## The 4 pins

1. `SimdSel` op=0 (BSL): Vd = (Rn & Rd) | (~Rd & Rm). Full-buffer pin of the
   exact operand order — load Rn@0x120 (xmm0), Rm@0x130 (xmm1), Vd@0x110
   (xmm2) — then `pand xmm0,xmm2` (Rn&Rd), `pandn xmm2,xmm1` (~Rd&Rm; note the
   dst=xmm2/rm=xmm1 order), `por xmm0,xmm2`, store Vd. Pins the BSL operand
   register mapping exactly.
2. `SimdSel` op=1 (BIT/BIF): Vd = (~Rm & Rn) | (Rd & Rm). Window discriminator
   vs BSL: op=1 LEADS with `pandn xmm0,xmm1` (~Rm&Rn) then `pand xmm2,xmm1`
   (Rd&Rm), never BSL's Rn&Rd first. A transposed mask picks the wrong source
   vector. (Genuine finding while pinning: the bare pandn xmm0,xmm1 is a 4-byte
   op — 66 0F DF C1 — so a 5-byte window underneath it misses; the encoder is
   correct.)
3. `SimdHighNarrow` addhn (no round): add src lanes (`add rax,rcx`), `shr
   rax,16` (dst_bits for dst_esize=2) to take the high half — with NO round-
   carry add before the shift; Q=0 then zeroes the upper half of Vd (`mov
   rax,0` + `mov [Vd+8],rax`). Pins the shr-by-dst_bits + the Q=0 upper-half
   clear.
4. `SimdHighNarrow` raddhn (round): the round-carry `mov r10,0x8000` (=
   1<<(dst_bits-1)) is added BEFORE the narrowing shift — the round-carry
   presence is the raddhn-vs-addhn discriminator, and its position before the
   shr is pinned.

## Method

Synthetic `Inst` -> `translate()` -> `CodeBuf.as_slice()` (zero-pc 0x1000 =
deterministic). `[RBX]=CpuState`; vector slot v[t] = VECTOR_BASE(0x110) + t*16.
Deterministic, no image, no env, parallel-safe. (A one-off `dump_sh436`
example was used to establish two exact byte forms during development and
removed before commit — the tree ships only the `#[cfg(test)]` additions.)

Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
codegen-surface coverage completion on the bitwise-select + high-narrow
families, continuing the SH427-435 translator-core lineage. No re-treads
(distinct from SH432 SminMax/SimdSatAdd, SH434 shifts, SH435 fcvt, SH433 Fmla).

Files: this doc + `crates/arm64jit/src/translate.rs` (`#[cfg(test)]` only;
production translator core byte-untouched; jit.rs/elfjit.rs/session.rs
unchanged). Workspace green (arm64jit lib 577/0 incl. 4 new sh436 pins; cargo
test --workspace EXIT 0).