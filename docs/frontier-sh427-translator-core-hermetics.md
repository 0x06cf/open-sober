# SH427 — hermetic coverage of the arm64→x86 translator CORE (translate.rs)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, ~196 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 515/0
incl. 13 new sh427 hermetics; cargo build --workspace OK, 0 errors). Production
code ONLY in translate.rs tests (`#[cfg(test)]` addition); the translator core
body is byte-untouched, jit.rs/elfjit.rs/session.rs unchanged. Pure test
coverage completion — no production path / guest byte / JIT-hook-default touched.

## The gap

`translate(Inst -> CodeBuf)` is the actual byte-emission mapping between the
decoder and the runtime: every AArch64 instruction, once decoded, is turned into
x86-64 machine-code bytes by this one `match` (translate.rs line ~605). It is
the single most load-bearing encoder-driving surface in the JIT — yet it had
ZERO direct hermetics. decode.rs pins decode (insn word -> Inst, 76 tests) and
jit.rs pins runtime behavior (295 tests), but the STEPS BETWEEN them — how an
`Inst` value becomes concrete x86 bytes — were only ever exercised implicitly by
whether a translated block ran, exactly the SH423-426 coverage-lineage hole
(those pinned fsmap/boot/signals/x86 emitters; this pins the arm64-side
translator that drives them). A byte error here decodes-ambiguously or faults at
runtime with no code-level anchor.

## What SH427 pins (13 deterministic hermetics, exact-byte)

No real binary, no image, no env — synthetic `Inst` values run through
`translate()` and byte-asserted against the emitted `CodeBuf`. RBX = CpuState
base; guest reg g at `[RBX+g*8]`; SP slot = 31*8 = 248 = 0xf8.

- **movz x0,#0x1234 (64-bit)**: `mov r32-imm` shortcut (mov_eax_imm32) + store
  slot0 — pins the zero-extending imm32 path choice.
- **movn w0,#2 (32-bit, opc=2)**: `NOT(imm)` TRUNCATED to 0x00000000fffffffd
  (W-dest zero-extends). The qemu-verified comment's exact regression — a bare
  `mov r64,imm` would sign-extend to 0xfffffffffffffffd — pinned byte-exact.
- **movk x0,#2,lsl#16 (64-bit)**: read-modify-write — load [rbx], `and` with the
  clear mask 0xffffffffffff0000, `or` with 0x20000, store. Pins the historical
  full-replace corruption fix (0x8bb1 would have become 0x20000).
- **sub sp,sp,#0x20 (64-bit)**: rd==31 is SP for ADD/SUB immediate — load/`sub`/
  store [rbx+0xf8]. Every function prologue.
- **cmp wzr,#0x10 (32-bit, s=1, rd=31)**: rn==31 read as XZR=0 (NOT SP) AND no
  SP writeback (comparison discards); the C-bit(nzcv) pack `89 93 08 01 00 00`
  (mov [rbx+0x108],edx) + caller-reg restores `5a 59 58` wrap the tail.
- **neg x6,x6 (bit21=0 shifted form)**: rn==31 is XZR=0 (never SP) — `mov rax,0`,
  load rm slot6, sub, store slot6.
- **sub sp,sp,x1 (bit21=1 extended form)**: rn==31 AND rd==31 are SP — load/
  sub/store the 0xf8 slot, load rm from slot8.
- **adds w0,w1,w2 (32-bit, S=1)**: 32-bit zero-extend of both operands, 32-bit
  `add eax,ecx`, the `cmc` (f5) borrow-convention carry complement for the ADDS
  non-subtract case, nzcv pack, then a zero-extended store (shl/shr 32 + `mov
  [rbx],rax`).
- **adc x0,x1,x2**: TRUE_C = !stored_C, so `cmc`(f5) then native `adc rax,rcx`
  (48 11 c8) then store.
- **sbc x0,x1,x2**: SBB consumes C_s = (1-TRUE_C), so native `sbb rax,rcx`
  (48 19 c8) directly, no cmc, then store.
- **bic x0,x1,x2 (op=4)**: load rn/rm, `not rcx` (48 f7 d1), `and`, store.
- **orr x0,xzr,x1**: rn==31 loads as zero (NOT SP) then `or`, store.
- **BCond b.eq rel**: nzcv loaded into EFLAGS then `je rel32` (`0f 84 00 00 00
  00`, the rel32 placeholder patch_rel32 fixes up).

## Cross-validation

Each exact byte string was captured from the live encoder and then decoded by
hand (and where ambiguous, cross-referenced against the qemu-verified comments
in translate.rs for the historical-regression families: movn 32-bit truncation,
movk read-modify-write, neg/sub-sp XZR-vs-SP, adds cmc, adc/sbc carry-borrow
convention). The pins document intent AND catch a future regression — e.g. a
real `mov r64,imm` sneaking into the movn-w path, or the SP-slot writeback
appearing in `cmp wzr`. Deterministic, no binary required, runs in `cargo test`
parallel with no shared-static races (all use local buffers).

## Honest

NOT a DM (the SH415 do-init probe under the complete substrate still reports
DM-root [0x106a68818]=0x0 — Route-B live-DM structural gate UNCHANGED). This is
BUILD-THE-RUNTIME codegen-surface coverage completion: the translator core is
the last big decode-driven surface without self-tests, now pinned byte-exactly.
Pure `#[cfg(test)]`: production path byte-identical (proven by the unchanged
green suite). No re-treads (distinct from SH423 fsmap / SH424 boot / SH425
signals / SH426 x86-emitter).

- Files: docs/frontier-sh427-translator-core-hermetics.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).