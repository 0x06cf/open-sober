# SH434 — Hermetic coverage of the SIMD shift-and-accumulate codegen families

Family: `translate.rs` `SimdShl`, `SimdShr`, `SimdShrAcc`.

## What was uncovered

decode.rs pins decode, jit.rs pins runtime, but the byte EMISSION between them
was unpinned for the integer shift/rounding families. These are load-bearing:
vertex-index math, packed-color lane shifting, and the shift-based rounding
every image/geometry path leans on. SH434 pins the discriminators a byte error
silently corrupts (6 exact-byte hermetics, `#[cfg(test)]` only).

## The 6 pins

1. `SimdShl` esize=8 shift=1 — full 2-lane buffer: `mov_load64 [Vn0x120] ->
   shl rax,1 (48 C1 E0 01) -> mov_store64 [Vd0x110]`, second lane at +8
   (0x128 -> 0x118). Asserts shl emits the C1/E0 opcode, never shr(E8) or
   sar(F8) — a shift-direction flub moves every lane the wrong way.
2. `SimdShr` signed (sshr) esize=8 shift=2 — full 2-lane buffer with `sar
   rax,2` (48 C1 F8 02). Pins the F8 (arithmetic) as the signed form.
3. `SimdShr` unsigned (ushr) esize=8 shift=2 — full buffer with `shr rax,2`
   (48 C1 E8 02). Pins the E8 (logical) vs signed's F8 — the signed-vs-
   unsigned opcode discriminator.
4. `SimdShr` esize=4 signed — MUST `movsxd rax,eax` (48 63 C0) before `sar`;
   unsigned MUST NOT (zero-extend + `shr`). A zero-extended negative element
   under an arithmetic shift becomes positive -> sign-bit flub. Window asserts
   both directions.
5. `SimdShr` shift>=esize-bits guard (esize=1, shift=8): unsigned all-zeros
   via `xor rax,rax` (48 31 C0), signed all-ones sign-fill via `sar rax,63`
   (48 C1 F8 3F). x86 imm shifts clamp to 64 bits, so a large guest shift must
   be synthesized; the guard is the discriminator between the two fills.
6. `SimdShrAcc` unsigned esize=8 shift=1 (usra) — full lane-0 buffer: load Vn ->
   `shr` -> load Vd accumulator into RCX AFTER the shift -> `add rcx,rax`
   (48 01 C1) -> re-store Vd. Pins the accumulate ordering (read-before-add,
   add after the shift) that separates usra/ssra from a plain overwrite shift.

## Method

Synthetic `Inst` -> `translate()` -> `CodeBuf.as_slice()` (zero-pc 0x1000 =
deterministic). `[RBX]=CpuState`; vector slot v[t] = VECTOR_BASE(0x110) + t*16.
Deterministic, no image, no env, parallel-safe.

Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
codegen-surface coverage completion on the integer shift families, continuing
the SH427-433 translator-core lineage. No re-treads (distinct families).

Files: this doc + `crates/arm64jit/src/translate.rs` (`#[cfg(test)]` only;
production translator core byte-untouched; jit.rs/elfjit.rs/session.rs
unchanged). Workspace green (arm64jit lib 566/0 incl. 6 new sh434 pins, was
560; cargo test --workspace EXIT 0). jit.rs 1,048,390 B < 1MiB hook.