# SH460 — hermetic coverage of the DUP-BROADCAST, FP-IMMEDIATE-BROADCAST, and 128-bit-OR codegen (translate.rs SimdDupGp / SimdFmovImm / SimdOrr16)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 667/0 incl. 4 new sh460 pins, was 663; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- The three broadcast/OR families had zero direct byte tests (STATUS next-
  forward #5 named `SimdDup/FmovImm`). SH460 pins them:
  (1) THE SimdDupGp GPR-broadcast esize discriminator: esize<8 ZERO-EXTENDS
  the Wn GPR (mov eax,eax 89 c0) then ANDs the element mask (48 81 e0 ff ff ff
  ff for esize=4) then stores the same value to every lane; esize=8 loads Xn
  raw (48 8b 43 18, rn=3 slot) with NO zero-extend and NO mask. A missing mask
  leaks the high garbage bits of Wn into every lane (silent broadcast
  corruption); a missing zero-extend leaves a sign-extended value;
  (2) THE SimdDupGp lane-layout: the store count + stride is the q/esize
  discriminator — esize=2 q=false = 4 halfword stores at +2 (0x120..0x126, 66
  89 83) with the 0xffff mask; esize=1 q=true = 16 byte stores at +1 (0x120..
  0x12f, 88 83) with the 0xff mask. A count/stride flub broadcasts into the
  wrong slots;
  (3) THE SimdFmovImm immediate-broadcast: esize=8 materializes the full 64
  bits (48 b8 .. f0 3f) + 64-bit stores; esize=4 materializes the zero-extended
  low 32 bits (48 b8 00 00 80 3f 00 00 00 00 for 1.0f) + 32-bit stores — the
  materialization + store width is the esize discriminator (a flub broadcasts a
  truncated/expanded value);
  (4) THE SimdOrr16 128-bit OR: two passes loading 64-bit halves (0x130/0x138
  from Vn, 0x140/0x148 from Vm) each `or rax,rcx` (48 09 c8) + 64-bit store
  (0x120/0x128). The rm==rn (vmov copy) form is NOT special-cased — it emits
  the same OR (V|V=V) — and the 16-byte OR must NOT degrade to a 128-bit SSE
  `por` (66 0f eb), which would leave the lo/hi granularity wrong.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, GPR slot g=[RBX+g*8], vector slot
  v[t]=VECTOR_BASE(0x110)+t*16. Emission captured with a one-off probe test
  (eprintln dump, removed before commit) so pins match the real emission (one
  window-width dev-fix on the 2-byte mov eax,eax). No real binary/env needed;
  parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the broadcast + 128-bit-OR families,
  distinct from SH437's lane-COPY (per-element move, not broadcast). No
  re-treads.
- Files: docs/frontier-sh460-translator-dup-fmov-orr.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only).