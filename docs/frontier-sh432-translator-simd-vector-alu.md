# SH432 — SIMD/vector-ALU codegen hermetics (translate.rs SimdVLog, SminMax, SimdSatAdd)

Date: Sep 19, 2026 · hermes-worker · single-agent (cone suppressed)

## What

Hermetic coverage of the SIMD/vector-ALU codegen families in the arm64→x86
translator core — the byte emission that carries real rendered geometry/color
lane math. `decode.rs` pins decode and `jit.rs` pins runtime, but the EMISSION
between them was unpinned for these three families. SH432 adds 6 deterministic
exact-byte pins (via `tr_bytes`: synthetic `Inst` → `translate()` → `CodeBuf`,
zero pc, no image/env, parallel-safe) asserting the semantically-critical
discriminators a single byte error silently corrupts.

## Pins

1. **SimdVLog AND** — whole-buffer pin: `movdqu xmm0,[rbx+0x120]` (Vn) →
   `movdqu xmm1,[rbx+0x130]` (Vm) → `pand xmm0,xmm1` (`66 0F DB C1`) →
   `movdqu [rbx+0x110],xmm0` (Vd slot @ VECTOR_BASE 0x110 + vt*16).
2. **SimdVLog op** — after the two identical 16-byte movdqu loads, the /r-based
   opcode discriminates AND/ORR/EOR/BIC: `66 0F DB`/`EB`/`EF`/`DF` + operand
   bytes C1 / C1 / C1 / C8 (BIC's `pandn xmm1,xmm0` stores via RCX). A /r-flub
   maps `Vd&Vm` to `Vd^Vm` or worse.
3. **SminMax signed .2s max** — signed lanes `movsxd` (`48 63 C0/C9`) then
   `cmp rcx,rax; cmovg rax,rcx` (`48 39 C1` / `48 0F 4F C1`). cmovg 0x4F is the
   max-wins condition; a flub to cmovl returns the wrong lane.
4. **SminMax unsigned .8b min** — zero-extend (`0F B6` movzx), NO `48 63`
   anywhere, then `cmp; cmovb rax,rcx` (`48 0F 42 C1`). cmovb 0x42 (unsigned
   less-than) vs cmovl 0x4C (signed) is the signed/unsigned discriminator; a
   sign-extend flub turns a 0x80 lane from +128 to -128 and breaks the bound.
5. **SimdSatAdd signed sqadd .4s** — sign-extends lanes, then clamps overflow to
   `smax` (`mov r10,0x7FFFFFFF` = `49 BA FF FF FF 7F 00 00 00 00`; `cmp r10,rax;
   cmovg rax,r10` = `4C 39 D0` / `49 0F 4F C2`) and underflow to `smin` (`mov
   r10,0x80000000` = `49 BA 00 00 00 80 FF FF FF FF`; `cmovl` = `49 0F 4C C2`).
   The two CONSTANT+cmov pairings are the bounds discriminators.
6. **SimdSatAdd unsigned uqsub .2h** — zero-extend word (`0F B7` movzx), NO
   `48 63`, clamp floor `mov r10,0` (`49 BA 00..00`) then `cmp rcx,rax; sub
   rax,rcx; cmovb rax,r10` (`48 39 C1` / `48 29 C8` / `49 0F 42 C2`). The zero
   constant + cmovb (42) vs the signed smax/smin pair is the discriminator.

## Files

- `crates/arm64jit/src/translate.rs` — 6 new hermetics in `translate::tests`
  (pure `#[cfg(test)]`, +6 tests); translator core body byte-untouched;
  jit.rs/elfjit.rs/session.rs unchanged.
- This doc.

## Honest status

NOT a DM (DM-root [0x106a68818]=0 under the complete substrate; Route-B live-DM
structural gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface coverage
completion on the SIMD/vector-ALU families (the geometry/color math path),
continuing the SH427/428/429/430/431 translator-core lineage. No re-treads
(distinct from move/add/logic/bcond SH427, LdStrImm SH428, pair/FP SH429,
mul/div/branch SH430, logic-imm/mul-high SH431). Workspace green (arm64jit lib
556/0 incl. 6 new sh432 pins, was 550; workspace 737 passed/0 failed; cargo
build --example elfjit OK).