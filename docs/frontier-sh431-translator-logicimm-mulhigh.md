# Frontier SH431 — hermetic coverage of the LogicImm (bitmask-immediate) + MulHigh codegen families (translate.rs)

## Why this cycle

SH430 pinned the MulDiv/MulLong/ClzCls arithmetic + B/Cbz/Tbz control-flow. The
two adjacent families it did NOT cover — `Inst::LogicImm` (AND/ORR/EOR/ANDS with
the encoded bitmask immediate, incl. the `mov xD,#imm` = ORR xD,xzr,#imm alias)
and `Inst::MulHigh` (umulh/smulh, the high-64 half of the 128-bit product) — had
ZERO direct byte tests. These are load-bearing for constants (the mov-alias is
how compilers load many immediates) and for wide arithmetic (umulh/smulh are the
high half of every 64x64 multiply). Pure `#[cfg(test)]` addition; translator core
byte-untouched.

## What landed

`crates/arm64jit/src/translate.rs` (`#[cfg(test)] mod tests`) — 4 deterministic
exact-byte pins (synthetic `Inst` -> `translate()` -> `CodeBuf::as_slice()`,
zero-pc 0x1000, [RBX]=CpuState base, slot g = [RBX+g*8]):

1. `sh431_logicimm_orr_xzr_mov_alias_materializes_mask` — `mov x0,#7` = ORR
   x0,xzr,#7 (op=1, rn==31): rn==31 reads as XZR (zero), so RAX is a plain
   `mov rax,0` — never the SP slot; the bitmask immediate is materialized in RCX
   (`mov rcx,7`), `or rax,rcx`, store. Pins the bitmask-immediate materialization
   + the xzr-not-SP read of rn==31 (and asserts no `[rbx+0xf8]` SP access).
2. `sh431_logicimm_ands_w32_flag_setting_nzcv` — ANDS w0,w1,#5 (op=3, sf=false):
   `and rax,rcx` then store_nzcv (the pushfq + 4-bit NZCV pack ending with the
   C-store `89 93 08 01 00 00` to [rbx+0x108], caller pop `5a 59 58`), then the
   32-bit zero-extend (`mov eax,eax`) + store. Pins that the flag-setting ANDS
   form is the ONLY LogicImm op that emits the nzcv pack (and/orr/eor don't).
3. `sh431_mulhigh_umulh_unsigned_high_half_in_rdx` — umulh x0,x1,x2: one-operand
   UNSIGNED `mul rcx` (48 f7 e1, /4); high half lands in RDX, stored via
   `mov [rbx],rdx` (48 89 13). Pins the /4 (mul) discriminator.
4. `sh431_mulhigh_smulh_signed_high_half_in_rdx` — smulh x0,x1,x2: same load
   pair, but the SIGNED one-operand `imul rcx` (48 f7 e9, /5) puts the high half
   in RDX. A flub of /4-vs-/5 silently corrupts the high half of every signed
   wide multiply.

Arm64jit lib tests 546 -> 550.

## Measured (real libroblox.so)

- recon-v3 deliverables re-verified green at this exact HEAD (SH430 cycle:
  capture_taskv4_frame.sh attempt 1: 24 real task-driven frames
  `present swap Ok(0x1)`, 194 node pops, 0 json abort, 0 crash, EXIT 124).
- Workspace green: `cargo test --workspace` EXIT 0 (arm64jit lib 550/0 incl. 4
  new sh431 pins); `cargo build --example elfjit` OK.

## Honest

NOT a DM (DM-root [0x106a68818]=0 under the complete substrate). Route-B live-DM
structural gate UNCHANGED. BUILD-THE-RUNTIME codegen-surface coverage completion
(bitmask-immediate logic + high-widen multiply), continuing the SH427/428/429/430
translator-core lineage. No re-treads (distinct families).

## Files

- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only; core body byte-untouched)
- docs/frontier-sh431-translator-logicimm-mulhigh.md (this file)