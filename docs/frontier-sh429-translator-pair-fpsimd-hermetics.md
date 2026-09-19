# SH429 — hermetic coverage of the load/store-PAIR + scalar FP/SIMD codegen families (translate.rs LdStPair, FpLdStImm)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, ~196 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 533/0
incl. 9 new sh429 hermetics; cargo build --workspace OK, 0 errors). Production
code ONLY in translate.rs tests (`#[cfg(test)]` addition); translator core body
byte-untouched, jit.rs/elfjit.rs/session.rs unchanged. Pure test coverage — no
production path / guest byte / JIT-hook-default touched.

## The gap

SH427 pinned move/add/carry/logic/bcond; SH428 pinned the LdStrImm single-load/
store surface. SH429 continues the same coverage lineage onto the two families
that move REAL rendered geometry/vertex data: the load/store-PAIR (LdStPair —
`ldp`/`stp`, every function-prologue push/pop and geometry chunk) and the
scalar FP/SIMD immediate (FpLdStImm — `ldr/str d/s`, texture/vertex float data).
Neither had direct byte tests. These pin the semantically-critical
discriminators a byte error would silently corrupt:

- **stride-16 vector slot for FP d/s-pairs** — the Session-99 BUGFIX. `ldp
  d29,d28` (FP/vector pair) writes each reg to its 16-byte guest vector slot at
  VECTOR_BASE + vt*16, NOT vt*8. A stale vt*8 stride wrote d29 to 0x1f8/0x1f0
  instead of 0x2e0/0x2d0 and the follow-on fmadd read stale slots (a real
  structfield.elf -O2 corruption, 128 vs 52). Pinned: d29 -> [rbx+0x2e0], d28
  -> [rbx+0x2d0] (VECTOR_BASE=0x110 + 29*16/28*16). A regression to *8 would
  fail these byte-exact.
- **32- vs 64-bit pair width + zero-extension**: 32-bit `ldp w0,w1` does
  `mov eax,[rdx]` (zero-extends into full RAX) then a full-64 store; the second
  lane steps +esize (4 for W, 8 for X).
- **offset-form raw-byte immediate**: `ldp x0,x1,[x8,#imm]` applies imm to the
  access address RAW (not scaled); lane1 at imm+8. Pinned imm=2 -> [rdx+2] and
  [rdx+0xa].
- **scalar FP/SIMD xfer lower-N-bytes slice**: `ldr s0,[x1]` stores only the
  low 4 bytes of the d0 vector slot (`mov [rbx+0x110],eax`), `ldr d0` the low 8
  (`mov [rbx+0x110],rax`) — upper lanes preserved (ARM scalar semantics).

## The 9 hermetics

Exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice().
[RBX]=CpuState; GPR slot=[RBX+g*8] (g=8->0x40); vector slot vt=VECTOR_BASE+
vt*16 (0x110 base). Deterministic, no image, no env, parallel-safe (local
buffers). Test names + pinned output:
- sh429_ldp_64_pair_offset_zero / sh429_stp_64_pair_offset_zero: pair load/
  store, lane1 at [rdx+8]
- sh429_ldp_64_pair_offset_imm_raw_bytes: imm is RAW byte offset (lane0
  [rdx+2], lane1 [rdx+0xa])
- sh429_ldp_32_pair_zero_extends: `mov eax,[rdx]` + full-64 store, lane1 at +4
- sh429_stp_dpair_stride16_vector_slots / sh429_ldp_dpair_stride16_vector_slots:
  FP d-pair land in stride-16 vector slots (d29=0x2e0, d28=0x2d0) — pins the
  Session-99 bugfix
- sh429_ldr_d_scalar_into_vector_base / sh429_str_d_scalar_from_vector_base /
  sh429_ldr_s_scalar_32_into_vector_base: fp_scalar_xfer 64/32/32-into-slot
  (low 4 vs low 8 bytes, upper lanes preserved)

## Cross-validation

Every byte string captured from the live encoder and decoded by hand; the
stride-16 vector-slot arithmetic cross-referenced against the Session-99
BUGFIX comment in translate.rs (d29=0x2e0, d28=0x2d0 = VECTOR_BASE 0x110 +
vt*16). Deterministic, no binary required.

## Honest

NOT a DM (the SH415 do-init probe under the complete substrate still reports
DM-root [0x106a68818]=0x0 — Route-B live-DM structural gate UNCHANGED). This is
BUILD-THE-RUNTIME codegen-surface coverage completion on the pair + FP/SIMD
scalar families, continuing the SH427/SH428 translator-core lineage. Pure
`#[cfg(test)]`: production path byte-identical (proven by the unchanged green
suite). No re-treads (distinct from SH427's move/add/logic/bcond pins, SH428's
LdStrImm pins, SH423-426).

- Files: docs/frontier-sh429-translator-pair-fpsimd-hermetics.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).