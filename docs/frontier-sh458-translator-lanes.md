# SH458 — hermetic coverage of the SIMD lane-SELECT copy codegen (translate.rs SimdLaneS — mov Sd/Dd, Vn.T[idx])

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 660/0 incl. 3 new sh458 pins, was 657; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- SimdLaneS (the scalar lane-SELECT — copy one element Vn.T[idx] into the
  DEST FP (vector) slot's low bytes, the scalar-result half of a lane-extract)
  had zero direct byte tests (STATUS next-forward #5 named `SimdLaneS`). SH458
  pins the exact emit (rd=1 rn=2; Vn@0x130 Vd@0x120) with 3 exact-byte/window
  pins:
  (1) THE esize=8 index=0 full-buffer (load-bearing): mov rax,[0x130] (48 8b
  83, src = Vn + 0*8) + mov [0x120],rax (48 89 83, dst = Vd slot);
  (2) THE index-moves-source-dst-fixed discriminator: index advances ONLY the
  SOURCE by +esize (index=1 -> 0x138 for esize=8) while the DEST stays at
  f(rd)=0x120 — a flub that also advances the dest (or misses the source
  advance) copies the wrong element; negative asserts the dest never moves;
  (3) THE esize=4 width + index stride: mov_load32 (8b 83) + mov_store32 (89
  83), index strides source by +4 (0x130 -> 0x134) — the 89-vs-48 89 store
  (32-vs-64-bit) is the esize width discriminator, negative no mov_load64 /
  mov_store64.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; esize=8 full-buffer assert_eq matched first try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the lane-select-copy family, distinct
  from SH437's lane-COPY (which copies WITHIN a register across slots); this is
  extract-into-dest-low-bytes. No re-treads.
- Files: docs/frontier-sh458-translator-lanes.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only).