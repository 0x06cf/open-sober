# SH451 — Hermetic coverage of the SIMD NARROWING-SHIFT codegen family (translate.rs SimdShrn — shrn/shrn2/rshrn/rshrn2 Vd.T, Vn.U, #imm)

## Summary
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 635/0 incl. 5 new sh451 pins, was 630; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs 1,048,390
B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit 7a75a4f +2.

## What SH451 pins
`SimdShrn` (the NARROWING shift — shift each DOUBLE-width source element right
by `shift`, truncate [shrn] or round-half-up [rshrn: add 1<<(shift-1)] to a
HALF-width dest element; shrn2 writes the dest high half) had zero direct byte
tests. SH451 pins the exact emit (rd=1 rn=2, non-alias so permute_source is a
no-op; Vn@0x130 Vd@0x120) with 5 exact-byte / window pins:

1. **THE narrowing width pair (the load-bearing pin).** shrn V1.2s,V2.2d,#16
   (esrc=8 shift=16): per-lane `mov rax,[rbx+0x130]` (48 8b 83 — the 64-bit
   source, +esrc*8 bytes/lane) + `shr rax,16` (48 c1 e8 10) + `mov
   [rbx+0x120],eax` (89 83 — the 32-bit dest store). The width pair — 8B source
   in, 4B dest out — IS the semantic: a flub that stores the full 64-bit source
   into a 4B dest or truncates a 4B source corrupts every lane. Negative assert:
   the 8to4 form must NOT emit a 64-bit store (48 89 83).

2. **THE `upper` high-half destination offset.** shrn2 (upper=true) emits
   byte-identical code except the dest base shifts +8 (dst_off=8): stores land
   at 0x128/0x12c instead of 0x120/0x124. The upper flag is the ONLY thing that
   moves the destination base — pinning the 0x128 store + asserting 0x120 is
   NOT written isolates it; the source lane addressing is unchanged at +8.

3. **THE rshrn round-add is the round-vs-no-round discriminator.** rshrn
   V1.2s,V2.2d,#15 (round=true) emits `add rax, 1<<(shift-1)=0x4000` (48 81 c0
   00 40 00 00, the imm32 add) BEFORE the `shr rax,0x0f` (48 c1 e8 0f); shrn
   (round=false) has NO such add. Pinned by window + a positional assert that
   the round-add PRECEDES the narrowing shift — a flub that shifts without
   rounding truncates instead of rounding-half-up.

4. **The O-word narrowing width (4to2).** shrn V1.4h,V2.4s,#8 (esrc=4 shift=8):
   4 half-word lanes, each mov_load32 (8b 83) + `shr rax,8` (48 c1 e8 08) +
   16-bit `mov [..],ax` (66 89 83, the 66-prefixed 2B store). Distinct store
   width/switching vs the 8to4 form's 32-bit store; lane1 source at 0x134
   counted (exactly once), lane3 dest at 0x126; no 64-bit store.

5. **The narrowest-width byte narrowing (2to1).** shrn V1.8b,V2.8h,#4 (esrc=2
   shift=4): 8 byte lanes, each movzx_word_mem (0f b7 83 — UNSIGNED, zero-
   extend the 2B source) + `shr rax,4` (48 c1 e8 04) + byte store (88 83,
   0x120..0x127). Negative assert: shrn is unsigned, so the source must NOT be
   sign-extended (no 48 0f bf movsx).

## Method
Deterministic: synthetic `Inst::SimdShrn` -> `translate()` -> `CodeBuf.as_slice()`
(zero-pc 0x1000). [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16.
rd=1 rn=2 avoids the self-alias `permute_source` snapshot so the emitted bytes
are the clean per-lane body. Emission captured precisely with a one-off probe
test (eprintln dump, removed before commit) so the pins match the real
emission; the 8to4 full-buffer assert_eq matched first try. One dev-time fix:
the 2B/1B stores are 7 bytes (66+89/88+83+dw), so those windows are windows(7),
not windows(6). 5 exact-byte / window pins, parallel-safe, no image/env.

## Honesty
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the narrowing-shift family, adjacent to SH434
(SimdShl/Shr/ShrAcc element-wise same-width shift) — this is the HALVING-width
form, distinct. No re-treads. Commit 7a75a4f +2.