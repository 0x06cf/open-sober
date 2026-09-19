# SH452 — Hermetic coverage of the SIMD PAIRWISE-ADD-LONG codegen family (translate.rs SimdAdalp — saddlp/uaddlp/sadalp/uadalp Vd.Td, Vn.Ts)

## Summary
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 638/0 incl. 3 new sh452 pins, was 635; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs 1,048,390
B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit 7a75a4f +3.

## What SH452 pins
`SimdAdalp` (pairwise-ADD-LONG — sum each adjacent pair (2i,2i+1) of
src_esize-byte elements into a dst lane of DOUBLE width 2*src_esize; the acc
forms sadalp/uadalp **add** into the existing dst, the plain uadalp/saddlp
overwrite) had zero direct byte tests. SH452 pins the exact emit (rd=1 rn=2;
Vn@0x130 Vd@0x120) with 3 exact-byte / window pins:

1. **THE double-width dst store (the load-bearing pin).** saddlp V1.2s,V2.2s
   (se=4 np=2, acc=false): each pair mov_load32(RAX,[0x130]) (8b 83 — the [2i]
   source) + mov_load32(RCX,[0x134]) (8b 8b — the [2i+1] source into RCX) +
   `add rax,rcx` (48 01 c8) + `mov [rbx+0x120],rax` (48 89 83 — the DOUBLE-width
   8-byte dst). The 4-byte-sources-in / 8-byte-dst-out IS the pairwise-add-LONG
   widening: a flub that stores only the low 4 bytes drops the pair-carry and a
   flub reading a 4B source for an 8B lane corrupts the sum. Both pair stores
   counted (0x120, 0x128) as 64-bit forms. Two pairs: (0x130,0x134)->0x120 and
   (0x138,0x13c)->0x128.

2. **Word-pair narrowing + signed-byte sign-EXTEND.** uadalp V1.4s,V2.4h
   (se=2 signed=false): movzx_word (0f b7 83 / 0f b7 8b) + add + 32-bit store
   (89 83). saddlp V1.4h,V2.8b (se=1 signed=true): the byte sources are
   SIGN-extended via movsx_byte_mem (48 0f be) before the add — pinned against
   the zero-extend (0f b6) that would flip a negative element — then the byte-
   pair sum lands in a 2-byte dst (66 89 83).

3. **THE accumulate (acc) discriminator.** uadalp V1.2s,V2.2s (acc=true): reads
   the existing dst lane into R10 (mov_load64 = 4c 8b 93) and ADDS it (4c 01 d0)
   before storing — the plain saddlp (acc=false) never touches the dst. The
   R10-load + add-r10 presence is the acc-vs-overwrite discriminator; positional
   assert the accumulate read precedes the store, negative assert the non-acc
   form has no R10 load.

## Method
Deterministic: synthetic `Inst::SimdAdalp` -> `translate()` -> `CodeBuf.as_slice()`
(zero-pc 0x1000). [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16.
Emission captured precisely with a one-off probe test (eprintln dump, removed
before commit) so the pins match the real emission; the se=4 full-buffer
assert_eq matched the capture first try. Dev-time fixes during authoring (all
corrected): the R10 load and the 2B/1B stores are 7 bytes (4c 8b 93 + dw /
66 89 83 + dw), so those windows are windows(7) not windows(6), and a raw
`!windows(6)==[89 83..]` negative is ambiguous because the 64-bit store's tail
contains it — replaced with a counted 64-bit-store assertion. 3 pins,
parallel-safe, no image/env.

## Honesty
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the pairwise-add-long family, adjacent to SH451
(narrowing shift) and SH440 (SimdSum8 uaddlv horizontal). No re-treads (distinct
pairwise-pair-adjacent form). Commit 7a75a4f +3.