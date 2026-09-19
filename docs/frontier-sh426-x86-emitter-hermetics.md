# SH426 — hermetic coverage of the x86-64 emitter backend (`x86.rs`)

**Single-agent (cone suppressed).** Recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort,
0 crash). Workspace green (arm64jit lib 502/0 incl. 12 new sh426 hermetics;
`cargo test --workspace` EXIT 0, 683 passed/0 failed; `cargo build --workspace`
OK). Production code ONLY in `crates/arm64jit/src/x86.rs` (off the 1MiB hooks —
jit.rs/elfjit.rs untouched, byte-unchanged); the edit is a pure `#[cfg(test)]`
addition (12 hermetics), no production path / guest byte / JIT-hook-default
changed (the source code, pre-rustfmt, was byte-identical except the added
tests module).

- `x86.rs` is the **x86-64 code emitter backend**: `CodeBuf` + the register
  constants + `rex`/`modrm`/`disp_mod` ModRM/SIB resolution + the `mov*` family
  (imm64/imm32/rr64/load64/store64/eax32) + `patch_rel32` for encoding internal
  control-flow. It is *the* final codegen surface every translated block emits
  bytes through before execution — the single most load-bearing encoder in the
  runtime, and it had **ZERO tests**.
- **Genuine gap closed:** an encoder this foundational (REX.W/R/X/B bit layout,
  ModRM reg/rm field placement, disp8-vs-disp32 selection, REX insertion for
  r8-r15, rel32 displacement arithmetic) was only ever validated implicitly by
  whether a translated block ran — a byte error would either decode-ambiguously
  or fault at runtime with no code-level unit anchor. SH426 pins the encoder
  byte-exactly, mirroring the SH423 (fsmap) / SH424 (boot) / SH425 (signals)
  coverage pattern.
- **+12 hermetics** (deterministic, no real binary, no env):
  1. `rex_prefix_bitfields` — W bit + r/x/b high-bit placement (0x40/48 base,
     per-bit 0x04/0x02/0x01; all-high-bits 0x4F).
  2. `modrm_byte_fields` — mod/reg/rm field bit layout.
  3. `disp_mod_classification` — 0/disp8/disp32 selection at the ±128 boundary.
  4. `mov_ri64_encoding` — 48 B8+rd imm64 byte-exact.
  5. `mov_ri64_rex_b_for_high_register` — R8 → REX.B prefix.
  6. `mov_ri32_encoding` — 48 C7 /0 imm32 sign-extended.
  7. `mov_rr64_encoding` — 48 89 /r, high-reg REX.R+B, same-register no-op.
  8. `mov_load64_mem_forms` — mod00/disp8/disp32/negative-disp ModRM forms.
  9. `mov_store64_mem_forms` — store & REX.W.R for a high src register.
  10. `mov_eax_imm32_encoding` — 0xB8+rd imm32 (32-bit zero-extend).
  11. `cqo_cdq_encoding` — signing helpers byte-exact.
  12. `patch_rel32_disp_calculation` — disp = target-(from+off+4), forward &
      backward.
- **Honest:** NOT a DM (DM-root [0x106a68818]=0 under the complete SH415
  substrate probe — once-guard bit0=1, once-lambda let to run, AppBridgeV2
  genuine vt 0x1063a3410, yet DM-root stays 0). Route-B live-DM structural gate
  UNCHANGED. This is BUILD-THE-RUNTIME codegen-surface coverage completion in
  the SEP-18 direction — the tier of coverage the last four cycles
  (SH423-425) have been building systematically — NOT a re-tread and NOT a
  DM-seed. recon-v3 deliverables unchanged-green.

Files: `crates/arm64jit/src/x86.rs` (+12 hermetics). Commit (pending).