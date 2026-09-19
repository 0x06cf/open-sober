# Frontier SH430 — hermetic coverage of the MulDiv/MulLong/ClzCls arithmetic + branch control-flow families (translate.rs)

## Why this cycle

STATUS next-forward #6: the translator-codegen linage SH427/428/429 pinned move/add-logic/bcond, the
LdStrImm load/store surface, and the load/store-pair + scalar FP/SIMD families — but the MUL/DIV/LONG
arithmetic families (`Inst::MulDiv`, `Inst::MulLong`, `Inst::ClzCls`) and the branch control-flow family
(`Inst::B`/B.L, `Inst::Cbz`, `Inst::Tbz`) had ZERO direct byte tests. These two families carry documented
historical-bug discriminators a byte error silently corrupts, and they are in the everyday instruction
mix of any compiled program (function calls = B.L, loops = B/Cbz/Tbz, index math = MADD/UDIV, size math =
smull/umaddl). Pure `#[cfg(test)]` addition to the existing test module; the translator core body is
byte-untouched (jit.rs/elfjit.rs/session.rs unchanged).

## What landed

`crates/arm64jit/src/translate.rs` (`#[cfg(test)] mod tests`) — 13 deterministic exact-byte pins, each a
synthetic `Inst` -> `translate()` -> `CodeBuf::as_slice()` on a zero-pc (0x1000) so fixup targets are
deterministic (RBX = CpuState base; guest reg g = [RBX+g*8]):

**Arithmetic (MulDiv / MulLong / ClzCls):**
1. `sh430_muldiv_madd_64_accumulates_rn_mul_rm_plus_ra` — madd x0,x1,x2,x3 = x0 = x1*x2 + x3. Pin the
   accumulate `+` direction (`add rax,rdi` after imul) as the discriminator against the msub case.
2. `sh430_muldiv_msub_64_subtracts_product_from_ra` — msub x0,x1,x2,x3 = x0 = x3 - x1*x2. **The -48
   BUGFIX**: `n - q*d` compiled to msub returned -48 for 1298-25*50 (should be +48) until the direction
   was fixed to `ra - product`. Pin the FIXED shape: `sub rdi,rax` (RDI = ra - product) then `mov rax,rdi`.
   A regression back to `rn*rm - ra` flips the sub operand order and fails this pin.
3. `sh430_muldiv_mul_xzr_accumulate_skipped_not_sp` — mul x0,x1,x2 = madd with ra==31 (XZR, accumulate 0).
   MUST skip the accumulate entirely, NEVER read the SP slot [RBX+0xf8] (ldg(ra=31) would corrupt the
   product by adding the stack pointer). Pins the 4-instruction shape AND asserts no 0xf8 SP access appears.
4. `sh430_muldiv_udiv_64_unsigned_quotient_in_rax` — udiv: `xor rdx,rdx` (48 31 d2) to zero the RDX:RAX
   high half, then `div rcx` (/6 group-3). The signed/unsigned discriminator is xor-rdx vs cqo+idiv.
5. `sh430_muldiv_sdiv_32_signextends_both_operands` — sdiv w0,w1,w2 (sf=false): sign-extend both 32-bit
   operands (movsxd rax,eax + movsxd rcx,ecx), cqo, `idiv rcx`, then 32-bit zero-ext store (`mov eax,eax`).
6. `sh430_mullong_smull_64_signextends_w32_operands` — smull x0,w1,w2: movsxd both operands, imul (low-64
   of the signed 32x32 product, exact in 64 bits), store.
7. `sh430_mullong_umsubl_negates_product_then_adds_ra` — umsubl: zero-ext both (mov eax,eax/ecx,ecx), imul,
   load ra(3), `neg rax` (48 f7 d8) then `add rax,rdi` => RAX = ra - product. The neg+add pair is the
   umsubl discriminator.
8. `sh430_clz_64_rexw_before_f3_lzcnt` — **the REX.W-order BUGFIX**: clz x0,x1 emits `f3 48 0f bd c0`.
   The REX.W prefix MUST come after F3 and immediately before the 0F opcode; `48 f3 0f bd` makes the CPU
   ignore REX.W and run a 32-bit lzcnt (real repro: clz(0x16136740) returned 3, oracle 35).
9. `sh430_clz_w32_zext_then_32bit_lzcnt` — clz w0,w1: zero-extend first (mov eax,eax), then the 32-bit
   `f3 0f bd c0` (no REX.W), matching W zero-extend semantics.

**Branch control-flow (B / Cbz / Tbz):**
10. `sh430_branch_bl_stores_lr_then_call_placeholder` — bl: save guest LR=pc+4 into slot30 (10-byte
    `mov rax,0x1004` + `mov [rbx+0xf0],rax`), then `call rel32` (e8) with an emitted 4-byte zero placeholder
    (patched later by jit.rs). Pins the exact bytes + the call-fixup metadata (cc=0xfe, target=pc+imm) +
    that the placeholder bytes after the e8 are the trailing zeros.
11. `sh430_branch_b_uncond_jmp_fixup` — b (no link): a single `e9` + fixup cc=0xff (unconditional-jmp
    marker), target=pc+imm (backward 0x1000-0x20 = 0xfe0).
12. `sh430_branch_tbz_single_bit_and_test` — tbz/tbnz x7,#3: load rt(7), `mov rcx, 1<<3`, `test rax,rcx`
    (48 85 c8), then jnz (cc=0x85) — the masked single-bit test that distinguishes tbz from cbz.
13. `sh430_branch_cbz_zero_test_two_operand` — cbz x9: load rt(9), `test rax,rax` (48 85 c0), jz
    (cc=0x84) to pc+imm — the whole-register zero test.

All pins carry the byte-sequence assertions AND the fixup `cc`/`target_pc` where applicable. A `tr_bytes_fx`
helper (returns bytes + fixups) was added alongside the existing `tr_bytes`. Deterministic: no real binary,
no env, parallel-safe (local buffers). Arm64jit lib tests 533 -> 546.

## Measured (real libroblox.so)

- recon-v3 deliverables re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
  ~24 real task-driven frames `present swap Ok(0x1)`, ~196 node pops, 0 json abort, 0 crash).
- Workspace green: `cargo test --workspace` EXIT 0 (arm64jit lib 546/0 incl. the 13 new sh430 pins);
  `cargo build --example elfjit` OK.

## Honest

NOT a DM (DM-root [0x106a68818]=0 under the complete substrate). Route-B live-DM structural gate
UNCHANGED. This is BUILD-THE-RUNTIME codegen-surface coverage completion — the arithmetic + control-flow
families every translated block leans on, pinned the same way SH427/428/429 pinned the rest. The MSUB
-direction and CLZ-REX.W-order pins are the two genuine historical-bug discriminators worth freezing.
No re-treads (distinct families from SH427 move/add/logic/bcond, SH428 LdStrImm, SH429 LdStPair/FP-SIMD).

## Files

- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only; translator core body byte-untouched)
- docs/frontier-sh430-translator-muldiv-clz-branch.md (this file)