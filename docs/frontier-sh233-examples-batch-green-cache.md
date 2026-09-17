# SH233 — `cargo test --examples` parallel batch made genuinely green (real-image guard collision fixed)

## The defect (measured, reproducible)

The documented gate `cargo test -p arm64jit --examples` failed **5 of 78 tests on every
run** — the real-image guard family (`sh222`/`sh223`/`sh224`/`sh225`/`sh226`/`sh227`/`sh228`/
`sh231`/`sh232`, whichever members collided that run) panicked with `left: 0` against a non-zero
expected word, plus one `load real libroblox.so: Failed to read segment into JIT image: Bad
address (os error 14)`.

**It was NOT a code regression.** Each of the 5 failed tests passes in 0.08 s when run filtered
(e.g. `cargo test -p arm64jit --example elfjit sh232`), because a filtered run is a *fresh
process* that loads the image exactly once.

## Root cause

`libloader::elf::load_elf_image` maps the 109 MB `libroblox.so` PT_LOADs into **ONE contiguous
anonymous region at a FIXED guest base `0x100000000`** — the required property for the
identity-addressing translator (guest == host, so ADRP/ADR globals dereference the right host
pointer). The mapping is **deliberately leaked** for the one-shot production run (documented
in elf.rs). Inside a single process that holds the batch (`cargo test --examples` runs all 78 in
parallel threads):

1. Each real-image test calls `load_elf_image` → each `mmap` targets the *same* fixed `0x100000000`.
2. Only the FIRST mmap can acquire the base; every concurrent/subsequent one fails (mmap rejects
   the occupied region) → `host_addr_of` yields 0 → the word assertions read `left: 0`; the load
   that gets the clean failure surfaces as `EFAULT` / `Bad address (os error 14)`.

So the batch was silently red while every individual real-image test (and the ledger's
"examples green" / "shXXX 1 passed" filtered claims) was correct. Any future real-image test added
would hit the same wall in batch.

## The fix (test-harness only, zero production change)

Added a per-process `OnceLock<LoadedElf>` cache `load_real_image()` in the `sh115_tests` module
and routed the **8** real-image test load sites (sh221/sh223/sh224/sh225/sh226/sh227/sh231/sh232)
through it. Exactly ONE `mmap` at `0x100000000` happens per test process; all guard tests share the
same image and their assertions are **unchanged**. The production jit_run load (elfjit.rs main,
line 6340) and the four `std::fs::read` sites (sh116b/sh212-family, plain file reads, no collision)
are untouched.

`cargo build --workspace` (EXIT 0), `cargo test --workspace` (0 failed), and the full
`cargo test -p arm64jit --examples` batch are all green at the same HEAD.

## Verified at commit

- `cargo test -p arm64jit --examples` → **78 passed; 0 failed** (was 5 failed every run).
- `cargo test -p arm64jit --example elfjit sh232|sh231|sh228|sh227|sh226` → 1 passed each (unchanged semantics).
- recon-v3 SELF-DRIVEN FRAMES re-verified green at this HEAD (`runs/capture_taskv4_frame.sh`):
  type4_frame_thunk seeded into `[0x106829ea8]`, both heartbeats w4=4, RENDERCTX published,
  **24 real task-driven frames, `swap Ok(0x1)` ×24, distinct colors, no json abort, 0 crash**,
  dispatch #4654000 reached the thunk.