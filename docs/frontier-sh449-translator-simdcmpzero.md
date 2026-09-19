# SH449 — hermetic coverage of the SIMD compare-vs-zero (mask) codegen family (translate.rs SimdCmpZero — cmeq/cmgt/cmge/cmlt/cmle Vd.T, Vn.T, #0) — 4 exact-byte pins

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 baseline: 24 real task-driven frames `present swap
Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 626/0 incl. 4 new sh449 pins, was 622; cargo
build --workspace OK + cargo build --example elfjit OK). Production code ONLY
in translate.rs `#[cfg(test)]` addition (translator core body byte-untouched;
jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs 1,048,392 B / session.rs
unchanged).

## What was unpinned
SimdCmpZero (the per-lane compare-to-literal-0 mask builder — each lane ->
all-ones if the signed/unsigned int compare against 0 holds, else 0) had no
direct byte tests (STATUS next-forward #5 named `SimdCmpZero` a remaining
family). SH449 pins the exact emitted x86 (rd=1, rn=2; Vn@0x130, Vd@0x120) with
4 hermetics:

1. `cmeq V1.2s V2.2s,#0` (cond 0) full-buffer — per-lane a 32-bit zero-extending
   load (`mov eax,[0x130]` = 8b 83), `test rax,rax` (48 85 c0), `sete al`
   (0f 94 c0), `movzx eax,al` (0f b6 c0), `neg rax` (48 f7 d8) -> 0 or all-ones,
   32-bit store (89 83). eq (cond 0) is UNSIGNED — must NOT movsxd (48 63 c0).
   Full-buffer exact.

2. THE setcc opcode byte is the semantic: cond 0=eq `0f 94` (sete), 1=gt `0f 9f`
   (setg), 2=ge `0f 9d` (setge), 3=lt `0f 9c` (setl), 4=le `0f 9e` (setle). A
   wrong cond silently picks the wrong comparison (a >= 0 becomes a > 0). And
   the signed-vs-unsigned split: signed conds (1-4) MUST movsxd the 32-bit lane
   (48 63 c0) so the x86 64-bit signed compare is correct; eq (cond 0) is
   unsigned and does not.

3. `cmlt V1.8b V2.8b,#0` (cond 3, esize 1) — 8 lanes, each via
   `movsx_byte_mem` (48 0f be — SIGNED), test, `setl` (0f 9c), movzx, neg, byte
   store (88 83), lanes advance +1 (0x130..0x137, 0x120..0x127); asserts the
   8-lane count of setl.

4. `cmeq V1.2d V2.2d,#0` (esize 8, q=true) — full 64-bit load (48 8b 83) +
   test + sete + neg + 64-bit store (48 89 83), 2 lanes at +8, 2 sete, 2 neg
   (every lane's result is 0 or 0xffffffff_ffffffff via the 64-bit neg of 0/1).

## Deterministic
Synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc 0x1000);
[RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission established
precisely with a one-off probe dump (captured each cond + esize 1/4/8, q
true/false; removed before commit) so pins match the real emission. 4
exact-byte + window + cc-map + count/negative asserts. No image, no env,
parallel-safe.

## Honest
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the compare-to-literal-0 mask family, continuing the
SH427-448 translator-core lineage. No re-treads (distinct from SH445 FP
compare->mask VecFpCmp — this is the INTEGER compare-to-zero form).

## Files
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)
- this doc: docs/frontier-sh449-translator-simdcmpzero.md