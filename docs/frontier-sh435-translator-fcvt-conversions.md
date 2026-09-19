# SH435 — Hermetic coverage of the FP conversion codegen families

Family: `translate.rs` `Fcvt`, `FcvtTzReg`, `FcvtHalf`.

## What was uncovered

decode.rs pins decode, jit.rs pins runtime, but the byte EMISSION between them
was unpinned for the FP widen/narrow/trunc/FP16 conversions. These are
load-bearing: color-intensity, light, and texture-sample math. SH435 pins the
discriminators a byte error silently corrupts (7 exact-byte hermetics,
`#[cfg(test)]` only).

## The 7 pins

1. `Fcvt` to_d=true (S->D widen) — full buffer: `mov eax,[s1 low32]` ->
   `movd xmm0,eax` (66 0F 6E C0) -> `cvtss2sd xmm0,xmm0` (F3 0F 5A C0) ->
   `movq [d0 low64]`. Pins the F3 (single->double) promote opcode; asserts it
   never emits the F2 cvtsd2ss (which would narrow).
2. `Fcvt` to_d=false (D->S narrow) — full buffer: `movq xmm0,[d1 low64]`
   (F3 48 0F 7E ...) -> `cvtsd2ss xmm0,xmm0` (F2 0F 5A C0) -> `movd eax,xmm0`
   (66 0F 7E C0) -> `mov [s0 low32]`. Pins F2 (double->single) + the 32-bit
   store vs widen's 64-bit movq — the lane-width discriminator.
3. `FcvtTzReg` signed fcvtzs Dd,Dn — full buffer: `movq_load` -> `cvttsd2si
   rax,xmm0` (F2 48 0F 2C C0) DIRECTLY (x86 cvttsd2si is already trunc-toward-
   zero, so no pre-round) -> 64-bit store. Asserts no extra rounding compare.
4. `FcvtTzReg` SIGNED vs UNSIGNED (fcvtzs vs fcvtzu): BOTH truncate via
   cvttsd2si, but the UNSIGNED form appends the clamp `mov rcx,0 / test
   rax,rax (48 85 C0) / cmovs rax,rcx (48 0F 48 C1)` so negatives become 0;
   the signed form stores the raw signed trunc. The cmovs presence is the
   fcvtzu discriminator; also pins the S-source path (movd + cvtss2sd before
   trunc).
5. `FcvtHalf` op=0 (H->S) — full buffer ending in F16C `vcvtph2ps xmm0,xmm0`
   (c4 e2 79 13 c0) + 32-bit store. Pins the 13 (promote) opcode; asserts the
   1d (demote) form is absent.
6. `FcvtHalf` op=1 (S->H) — F16C `vcvtps2ph $0,xmm0,xmm0` (c4 e3 79 1d c0 00)
   demote + movd eax. Pins the 1d (demote) opcode.
7. `FcvtHalf` op=2 (H->D) promotes then widens via cvtss2sd to 64-bit; op=3
   (D->H) narrows via cvtsd2ss before demote. The extra 5A cvtss2sd (after
   promote) vs cvtsd2ss (before demote) distinguishes the .5-width conversions
   from the direct S<->H pair.

## Method

Synthetic `Inst` -> `translate()` -> `CodeBuf.as_slice()` (zero-pc 0x1000 =
deterministic). `[RBX]=CpuState`; vector slot v[t] = VECTOR_BASE(0x110) + t*16.
Deterministic, no image, no env, parallel-safe.

Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
codegen-surface coverage completion on the FP conversion families, continuing
the SH427-434 translator-core lineage. No re-treads (distinct from SH433 Fmla
FP FMA, SH434 integer shifts).

Files: this doc + `crates/arm64jit/src/translate.rs` (`#[cfg(test)]` only;
production translator core byte-untouched; jit.rs/elfjit.rs/session.rs
unchanged). Workspace green (arm64jit lib 573/0 incl. 7 new sh435 pins, was
566; cargo test --workspace EXIT 0).