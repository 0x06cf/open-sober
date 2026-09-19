# SH424 — hermetic coverage of the guest initial-stack builder (`boot.rs`)

**Single-agent (cone suppressed).** Recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort,
0 crash). Workspace green (arm64jit lib 485/0 incl. 3 new sh424 hermetics;
`cargo test --workspace` EXIT 0, 666 passed/0 failed; `cargo build --workspace`
OK, 0 errors). Production code ONLY in `crates/arm64jit/src/boot.rs` (off the
1MiB hooks — jit.rs/elfjit.rs untouched, byte-unchanged); the edit is a pure
`#[cfg(test)]` addition (3 hermetics + a stack-walk helper), no production path /
guest byte / JIT-hook-default changed.

- `boot::layout_initial_stack` is the guest **initial-stack bootstrap**: it lays
  out the kernel's `[argc][argv][envp][auxv, AT_NULL]` word image + arg/env
  strings at the top of the guest stack for a remote-loaded aarch64 ELF, and
  patches a zero-valued `AT_RANDOM` slot to a freshly xorshift-filled 16-byte
  block so the glibc stack canary gets real entropy without a `getrandom`
  syscall. It is **load-bearing for real boots** (memory note): a garbage
  initial stack makes glibc `_start` read bogus `AT_HWCAP`/`AT_HWCAP2` and
  IFUNC-dispatch into SVE/SME opcodes (`__libc_arm_za_disable` + `str za`) the
  JIT cannot decode — the module's whole purpose is to keep hwcap minimal
  (`HWCAP_FP | HWCAP_ASIMD` only) so libc takes scalar paths.
- **Genuine gap closed:** `boot.rs` had **ZERO tests** of its own logic — the
  module's correctness (16-byte sp alignment, argc/argv/envp framing, auxv
  layout, AT_NULL termination, AT_RANDOM patch-to-nonzero-in-bounds entropy)
  was only ever exercised implicitly when a full boot reached `_start`. SH424
  delivers the module's own hermetic pair, mirroring the SH423 fsmap coverage
  pattern (a runtime-surface module with an untested core).
- **+3 hermetics** (deterministic, no real binary, no env):
  1. `layout_initial_stack_builds_parseable_word_image_aligned` — 64KiB host
     buffer, 1 argv0 + 3 env entries + 6-entry auxv; asserts sp 16-byte-aligned
     and in-bounds; walks the word image the way glibc `_start` does and
     round-trips each env `CStr`; every non-AT_RANDOM auxv (type, value) present.
  2. `at_random_zero_slot_patched_to_nonzero_in_bounds_entropy` — argc==0 branch
     (no argv0), AT_RANDOM zero slot must be patched to an in-bounds non-zero
     pointer with ≥1 non-zero entropy byte; argc==0 / argv empty verified.
  3. `no_at_random_leaves_auxv_untouched_and_terminates` — auxv with no
     AT_RANDOM slot stays byte-identical; walk terminates at AT_NULL.
- **Honest:** NOT a DM (DM-root [0x106a68818]=0 under the complete SH415
  substrate probe — once-guard bit0=1, once-lambda let to run, AppBridgeV2
  genuine vt 0x1063a3410, yet DM-root stays 0). Route-B live-DM structural gate
  UNCHANGED. This is BUILD-THE-RUNTIME coverage completion in the SEP-18
  direction — hardening the host-side session substrate the engine boots on
  top of — NOT a re-tread and NOT a DM-seed. recon-v3 deliverables unchanged-green.

Files: `crates/arm64jit/src/boot.rs` (+3 hermetics). Commit (pending).