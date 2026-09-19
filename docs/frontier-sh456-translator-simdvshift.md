# SH456 — hermetic coverage of the SIMD VARIABLE-SHIFT codegen (translate.rs SimdVShift — ushl/sshl/urshl/srshl Vd.T, Vn.T, Vm.T)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 654/0 incl. 4 new sh456 pins, was 650; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- SimdVShift (the per-lane VARIABLE shift where each count lane C is a SIGNED
  esize-bit value: C>=0 left-shifts, C<0 right-shifts by -C, |C|>=B zeroes /
  sign-fills) had zero direct byte tests — SH434 pinned only the CONSTANT-
  shift SimdShl/SimdShr/SimdShrAcc. SH456 pins the exact emit (rd=1 rn=2 rm=3;
  V@0x130 C@0x140 Vd@0x120) with 4 exact-byte/window pins:
  (1) THE sshl .2d full-buffer (load-bearing): bbits==64 so there is NO wmask
  and NO out-of-range clamp (x86's shl/sar mask CL to the low 6 bits = the
  exact 0..63 ARM range); per lane mov rax,[0x130] + mov rcx,[0x140] + `test
  rcx,rcx` (48 85 c9) + js (0f 88) right + LEFT `shl rax,cl` (48 d3 e0) + jmp +
  RIGHT `neg rcx` (48 f7 d9) + `sar rax,cl` (48 d3 f8) + mov [0x120],rax;
  full-buffer assert_eq + negative no shr (E8);
  (2) THE ushl .2s sign-dispatch: the COUNT C is SIGN-extended (movsxd rcx,
  ecx 48 63 c9) so a high-bit-set C makes `test`+`js` (0f 88) take the right
  path — a zero-extend would turn a negative count huge-positive and wrongly
  take the left path; the left path guards C>=32 -> 0 and ANDs the esize=4
  wmask 0xffffffff back (48 21 d0); 32-bit store (89 83);
  (3) THE signed-vs-unsigned RIGHT-shift opcode (the semantic): sshl/srshl
  uses `sar rax,cl` (48 d3 f8, arithmetic — sign-extends the value first via
  movsxd rax,eax 48 63 c0), ushl/urshl uses `shr rax,cl` (48 d3 e8, logical,
  NO value sign-extend); signed out-of-range right sign-fills (jns 0f 89 ->
  mov rax,0xffffffff) vs unsigned plainly mov rax,0;
  (4) THE urshl rounding-bias before shift: the round=true in-range right path
  adds 1<<(k-1) to V BEFORE the final shift — `mov rdx,1` + `shl rdx,cl`
  (48 d3 e2) + `shr rdx,1` (48 c1 ea 01) + `add rax,rdx` (48 01 d0) then `shr
  rax,cl`; the add-precedes-shift position (and its ABSENCE in the non-
  rounding form) is the round discriminator — a skip truncates instead of
  rounding half-up.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; sshl_d8 full-buffer assert_eq matched first try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the variable-shift family, the
  dynamic-count complement of SH434's constant shifts. No re-treads.
- Files: docs/frontier-sh456-translator-simdvshift.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only).