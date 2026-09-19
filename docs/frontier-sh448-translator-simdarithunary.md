# SH448 — hermetic coverage of the SIMD integer ARITH-UNARY codegen family (translate.rs SimdArithUnary — neg/abs Vd.T, Vn.T) — 4 exact-byte pins

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 baseline: 24 real task-driven frames `present swap
Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 622/0 incl. 4 new sh448 pins, was 618; cargo
build --workspace OK + cargo build --example elfjit OK). Production code ONLY
in translate.rs `#[cfg(test)]` addition (translator core body byte-untouched;
jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs 1,048,392 B / session.rs
unchanged).

## What was unpinned
SimdArithUnary (the per-lane integer unary — `neg` op 0 / `abs` op 1) had no
direct byte tests (decode pins decode, jit pins runtime, but the byte EMISSION
was uncovered; STATUS next-forward #5 named `SimdFpUnary/SimdArithUnary` a
remaining family). SH448 pins the exact emitted x86 (rd=1, rn=2; Vn@0x130,
Vd@0x120) with 4 hermetics:

1. `neg V1.2s V2.2s` (op 0, esize 4) full-buffer — per-lane a 32-bit
   zero-extending load (`mov eax,[0x130]` = 8b 83), then `neg rax` (48 f7 d8),
   then 32-bit store (89 83). neg MUST NOT movsxd (48 63 c0) — the 32-bit load
   zero-extends, which is correct for neg (two's-complement wrap). Full-buffer
   exact.

2. `abs V1.2s V2.2s` (op 1) full-buffer — the `(x ^ (x ar>> w-1)) - (x ar>> w-1)`
   idiom: esize=4 MUST `movsxd rax,eax` (48 63 c0) FIRST (the lane is signed),
   then `mov rcx,rax` (48 89 c1) + `sar rcx,31` (48 c1 f9 1f — imm = esize*8-1)
   + `xor rax,rcx` (48 31 c8) + `sub rax,rcx` (48 29 c8), then 32-bit store.
   abs MUST NOT be a bare `neg rax` (48 f7 d8). Signed-vs-zero extension is the
   semantic — a zero-extended negative lane would never become positive.

3. esize width discriminator — esize=8 (q=true) neg loads full 64-bit (`mov
   rax,[0x130]` = 48 8b 83) + neg + 64-bit store (48 89 83, lane1 at +8 0x128);
   esize=1 abs loads via `movsx_byte_mem` (48 0f be) + `sar rcx,7` (imm 07 =
   esize*8-1) + byte store (88 83, lane1 at 0x121, advance +1). The sar imm
   (esize*8-1) and the load/store width are the discriminator a flub silently
   corrupts (wrong sign mask or wrong lane width).

4. THE abs-vs-neg discriminator count (4-lane .4s): neg emits exactly 4 `neg
   rax` (48 f7 d8) and ZERO movsxd; abs emits exactly 4 movsxd (48 63 c0) and
   ZERO neg. A transposed op silently abs()'s a negate or negates an absolute.

## Deterministic
Synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc 0x1000);
[RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission established
precisely with a one-off probe dump (captured neg/abs across esize 1/4/8, q
true/false; removed before commit) so pins match the real emission. 4
exact-byte + window + count/negative asserts. No image, no env, parallel-safe.

## Honest
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the integer neg/abs family, continuing the SH427-447
translator-core lineage. No re-treads (distinct from SH447 FP unary — this is
the integer GPR neg / sar-xor-sub absolute-value).

## Files
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)
- this doc: docs/frontier-sh448-translator-simdarithunary.md