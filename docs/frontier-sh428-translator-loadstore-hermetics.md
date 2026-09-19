# SH428 — hermetic coverage of the load/store codegen families (translate.rs LdStrImm + the cmn carry path)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, ~196 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 524/0
incl. 9 new sh428 hermetics; cargo build --workspace OK, 0 errors). Production
code ONLY in translate.rs tests (`#[cfg(test)]` addition); translator core body
byte-untouched, jit.rs/elfjit.rs/session.rs unchanged. Pure test coverage — no
production path / guest byte / JIT-hook-default touched.

## The gap

SH427 pinned the move-wide/add-sub/carry/logic/bcond translator families.
SH428 continues the same coverage lineage on the LOAD/STORE emission — the
single most load-bearing translation surface (every guest memory access flows
through `Inst::LdStrImm`). decode.rs pins decode, jit.rs pins runtime, but the
exact bytes for `ldr/str` were untested. These pin the critical semantic
discriminators a byte error would silently corrupt:

- **XZR-source store** (`str xzr,[..]`): the source register field of a store
  reads x31 as XZR=zero, NEVER the SP slot. `str xzr` (extremely common:
  compilers zero-init stack slots / objects with it) must emit `mov rax,0`,
  not `mov rax,[rbx+0xf8]`. A regression here would store the stack pointer
  into memory — silent memory corruption. Pinned 64-bit and 32-bit.
- **Zero- vs sign-extending loads**: `ldr w`/`ldrb`/`ldrh` zero-extend (8b/0f b6
  + full-64 store), while `ldrsw`/`ldrsh`/`ldrsb` sign-extend (movsxd / shl+sar).
  Pinned `ldr w0,[x1]`, `ldrsb x0,[x1,#1]` (shl/sar 56), `ldrsw x0,[x1,#4]`
  (movsxd 63 c0), and the scaled-offset addressing (lea rdx,[rdx+off]).
- **Scaled-offset form**: `imm` is scaled by `size` — imm=2,size=8 -> lea 16;
  imm=1,size=4 -> lea 4. Pinned.
- **`cmn x0,#imm` (shifted, S=1) carry-borrow flag pack**: the qemu-verified
  `x > 0xffffffffffff0000` case compiles to `cmn x,#0x10000; b.ls` — the ADD
  carries (ARM C=1) but the stored C must be the borrow-convention !carry, so
  translate emits `cmc`(f5) before the nzcv pack. Pinned: load x0, add imm32,
  cmc, then the C-store `89 93 08 01 00 00` + caller-reg restores.

## The 9 hermetics

Exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice().
RBX = CpuState base; slot g = [RBX+g*8]. Deterministic, no image, no env,
parallel-safe (all local buffers). Test names + pinned output:
- sh428_ldr_64_scaled_offset_zero: `mov rdx,[rbx+0x08]; mov rax,[rdx];
  mov [rbx],rax`
- sh428_str_64_reads_source_slot: `mov rdx,[rbx+0x08]; mov rax,[rbx];
  mov [rdx],rax`
- sh428_str_xzr_stores_zero_not_sp: `mov rdx,[rbx+0x08]; mov rax,0;
  mov [rdx],rax` (XZR=0, never SP)
- sh428_str_xzr_32_stores_zero_not_sp_w32: same but `mov [rdx],eax` (89 02)
- sh428_ldr_32_zero_extends: `mov eax,[rdx]` (8b 02) then full-64 store
- sh428_ldrsb_scaled_offset_sign_extends: lea 1, movzx byte, shl/sar 56
- sh428_ldrsw_scaled_offset_sign_extends_word: lea 4, mov eax,[rdx], movsxd
  (48 63 c0)
- sh428_ldr_64_scaled_offset_16: lea 16 then 64-bit load/store
- sh428_cmn_shifted_carry_borrow_pack: load x0, add rax,imm32(0x01000000),
  cmc(f5), C-store `89 93 08 01 00 00`, reg-restores

## Cross-validation

Every byte string captured from the live encoder and decoded by hand; the
XZR/SP, sign-extend, and carry-borrow spans cross-referenced against the
qemu-verified comments in translate.rs. Deterministic, no binary required.

## Honest

NOT a DM (the SH415 do-init probe under the complete substrate still reports
DM-root [0x106a68818]=0x0 — Route-B live-DM structural gate UNCHANGED). This is
BUILD-THE-RUNTIME codegen-surface coverage completion on the load/store family,
continuing the SH427 translator-core lineage. Pure `#[cfg(test)]`: production
path byte-identical (proven by the unchanged green suite). No re-treads
(distinct from SH427's move/add/logic/bcond pins; distinct from SH423-426).

- Files: docs/frontier-sh428-translator-loadstore-hermetics.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).