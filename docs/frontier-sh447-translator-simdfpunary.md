# SH447 — hermetic coverage of the SIMD FP UNARY codegen family (translate.rs SimdFpUnary — fneg/fabs/fsqrt Vd.T, Vn.T) — 5 exact-byte pins

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 baseline: 24 real task-driven frames `present swap
Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 618/0 incl. 5 new sh447 pins, was 613; cargo
build --workspace OK + cargo build --example elfjit OK). Production code ONLY
in translate.rs `#[cfg(test)]` addition (translator core body byte-untouched;
jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs 1,048,392 B unchanged;
session.rs unchanged).

## What was unpinned
SimdFpUnary (the per-lane 1-source FP unary — `fneg`, `fabs`, `fsqrt`) had no
direct byte tests (decode pins decode, jit pins runtime, but the byte EMISSION
was uncovered; the STATUS next-forward #5 list named `SimdFpUnary/SimdArithUnary`
as a remaining family). SH447 pins the exact emitted x86 (rd=1, rn=2; Vn@0x130,
Vd@0x120) with 5 hermetics:

1. `fneg V1.2s V2.2s` (op 0, esize 4) full-buffer — per-lane: `movq xmm0,[Vn+l]`
   (f3 48 0f 7e 83 — the lane is moved as a full 64-bit bit-pattern) then
   `movq rax,xmm0` (66 48 0f 7e c0), materialize sign const bit31 0x8000_0000 in
   RCX (48 b9 ..00 00 00 80 00 00 00 00), `xor rax,rcx` (48 31 c8 — the FLIP),
   `movq xmm0,rax`, then a 32-bit store `movd eax,xmm0` (66 0f 7e c0) +
   `mov [Vd+l],eax` (89 83). fnEG flips the sign bit (xor); the 32-bit store
   class (89 83) vs the 16-bit 66 89 store is the esize discriminator.

2. `fabs V1.2s V2.2s` (op 1) — the sign CLEARED via `mov rdx,const` (48 ba ..00
   00 00 80 00 00 00 00) + `not rdx` (48 f7 d2) + `and rax,rdx` (48 21 d0). The
   and+not (NEVER the fnEG xor-flip 48 31 c8) is the fabs-vs-fneg discriminator
   — a transposed op silently toggles instead of clearing the sign bit.

3. `fsqrt V1.2d V2.2d` (op 2, esize 8, q=false, 1 lane) full-buffer — `movq
   xmm0,[Vn]` + `sqrtsd xmm0,xmm0` (f2 0f 51 c0) + `movq [Vd],xmm0` (66 48 0f
   d6). fsqrt emits NO GPR sign-bit manipulation at all (no mov rcx const, no
   xor/and) — the absence of the sign-const mov (48 b9 ..) + absence of xor is
   the fsqrt discriminator.

4. esize width discriminator across the family — esize=8 (q=true) loads the
   full 64-bit sign (0x8000_0000_0000_0000 imm ..00*7 80) + 64-bit `movq` store
   (66 48 0f d6) with lane1 at +8; esize=2 loads bit15 (0x8000 imm 00 80 ..) +
   16-bit store (66 89). A width flub applies the wrong sign bit / stores the
   wrong lane width and silently corrupts the vector.

5. q/lane advance (fabs V1.4s, q=true) — loads 0x130/0x134/0x138/0x13c, stores
   0x120/0x124/0x128/0x12c, every lane advances exactly +esize; pins the store
   slots 0x128/0x12c so a stale/non-advancing dst fails.

## Deterministic
Synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc 0x1000);
[RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission established
precisely with a one-off rag-probe eprintln dump (captured fneg/fabs/fsqrt +
lane-width variants; removed before commit) so pins match the real emission. 5
exact-byte + window + negative/order asserts. No image, no env, parallel-safe.

## Honest
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the 1-source FP unary family, continuing the SH427-446
translator-core lineage. No re-treads (distinct from SH446 by-element FMUL,
SH444 2-src scalar FP, SH436 SimdSel/high-narrow).

## Files
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)
- this doc: docs/frontier-sh447-translator-simdfpunary.md