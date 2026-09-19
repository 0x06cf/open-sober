# SH461 — hermetic coverage of the SHA crypt HOST-CALL codegen (translate.rs Sha — sha1/sha256 Vd.4S, Vn.4S, Vm.4S)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (`#[cfg(test)]`-only change, so the runtime deliverable is
byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 668/0 incl. 1 new sh461 pin, was 667; cargo
build --workspace + --example elfjit OK). Production code ONLY in the
translate.rs `#[cfg(test)]` block (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).

- Sha (the AArch64 SHA1/SHA256 crypt ops, dispatched to a HOST `guest_sha1stem`
  helper) had zero direct byte tests (STATUS next-forward #5 named `Sha`).
  SH461 pins the host-call thunk shape (rd=1 rn=2 rm=3):
  `mov rdi,rbx` (48 89 df = CpuState* arg0) + `mov rsi,<packed>` (48 be, arg1 =
  mode<<24|rd<<16|rn<<8|rm) + `mov rax,<host-fn>` (48 b8, the helper address —
  a HOST pointer, deliberately NOT pinned as it varies per loader) + `sub rsp,8`
  (48 81 ec 08 = RSP-align to 16 at the SysV call site; the block body runs at
  RSP≡8, host calls need ≡0) + `call rax` (ff d0) + `add rsp,8` (48 81 c4 08).
  The mode nibble in the IMM is the discriminator (mode=3 -> the 4th imm byte is
  0x03). The `sub -> call -> add` RSP-align round-trip is pinned positionally (a
  missing align mis-calls the host helper).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission (one window-width dev-fix: `call rax` is 2 bytes
  ff d0, not 3). No real binary/env; parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under
  the complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the SHA crypt host-call family — the
  last named in STATUS next-forward #5, completing the translate.rs hermetic
  coverage lineage SH427-461. No re-treads.
- Files: docs/frontier-sh461-translator-sha-hostcall.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only).