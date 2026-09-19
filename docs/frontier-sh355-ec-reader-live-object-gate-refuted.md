# Frontier SH355 — EC-world reader gate: soft-return premise REFUTED (no `ret` in the block); the reader is gated on a live-object slot, not a compile-block artifact

Date: Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). One new hermetic
`sh355_ec_reader_block_no_softreturn_gate_is_live_object_slot` (arm64jit lib, real-image
byte pins), plus recon-v3 self-driven-frame deliverable re-verified green. No production
code path edited (test-only). elfjit.rs unchanged (holds <1MB hook).

## What this corrects

sh301/sh302's doctrine (frontier-sh302-ec-readergate-inert.md:59) asserted the EC-world
body "provably soft-returns BEFORE the reader at 0x2e246f4", and sh302's NEXT GATE told the
next session to "make the single compiled block spanning [0x2e245f4..0x2e247dc] actually
FALL THROUGH to 0x2e246f4 — requires understanding the block's real early-exit". The
test-suite record therefore told a future cycle the reader is behind an internal early
soft-return (a compile/control-flow artifact) and to hunt that exit.

Fresh disasm of the real block REFUTES that premise:

- There is NO `ret` (0xd65f03c0) anywhere in [0x2e245f4, 0x2e246e0) — sh355 scans the whole
  window and asserts zero. The only way out to the reader is the REAL data-dependent branch
  `cbz x0, 0x2e246f4` at 0x2e246dc.
- The gate load is `ldr x8,[x29,#104]` @0x2e246b0 then `ldr x0,[x8,#32]` @0x2e246d8 — i.e.
  the branch is taken iff `[[x29,#104]+0x20]==0`. That is a CLOSED-LOOP live-object slot
  (SH174/SH204 class: an unconstructed caller-frame object whose [+0x20] is host garbage),
  NOT a compile-block "internal early-exit".
- The interior string-assign `bl 0x2b504e4` @0x2e24690 returns to 0x2e24694 = a REAL block
  boundary (sh301 already proved interior bls open block entries), so the reader-gate block
  starts at 0x2e24694 — 0x2e246f4 is reachable as a normal block fall-through / cbz target.

## Consequence for the next frontier (re-scoping the do-not-re-tread)

sh302's seed (write entry_sp+8 to a zeroed buffer, so [+0x20]==0) is mechanism-CORRECT but
only fires if the SAME frame that reaches 0x2e246dc is the one seeded (entry-timing, not
block-cache; the "block already cached" hypothesis in sh302 is not what gates it). The real
residual is the live-object POINTER VALUE at [x29,#104]+0x20 — to cross the reader legally we
must hand a coherent object whose [+0x20]==0 to the actual construction entry (a value seed
that FABRICATES the live object), not hunt a phantom compile-block early-exit. This does NOT
manufacture a DataModel; Route-B live-DM gate is UNCHANGED (DM-root 0, MH_* false; canonical
full-ladder probe re-ran this cycle: EXIT 134 at the known run-variable live-object arm
guestpc 0x10284cf5c, 0 region hits, MH_* false). SH174 capture-latch stays the single forward
hook.

## Also this cycle

recon-v3 SELF-DRIVEN FRAME deliverable (recon-selfdrive-seed-jsonfix.md §A,
capture_taskv4_frame.sh) re-verified green at this HEAD: attempt 1, 24 real task-driven
presented frames, `present swap Ok(0x1)` monotonic, 197 node pops, 0 SIGSEGV/ABRT, exit 124
(stable idle), no json abort. Deliverable (1) confirmed reproducible.

## Verify / files

- `cargo test -p arm64jit --lib -- sh355` = 1 passed.
- `cargo test --workspace` EXIT 0 (600/0 across all crates).
- `cargo build --workspace` EXIT 0.
- Repro: objdump -d `aarch64-linux-gnu-objdump -d --start-address=0x2e24598 --stop-address=0x2e247e0
  libroblox.so` (the sh355 pins are literal disasm-derived words).
- No production path edited; no new capture script (test + doc only).
- Commit: local `dev` only (operator pushes).