# Frontier SH356 — frame-accurate EC reader-gate re-attack (SH355-fwd): fire INSIDE the reader-gate block using the LIVE x[29], addressing SH302's measured-inert entry-timing residual

Date: Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). New default-inert
guard `routeb_ec_world_reader_gate_frame_guard` (jit.rs, opt-in
`JIT_ROUTEB_EC_READERGATE_FRAME=1`) + hermetic test + run-loop wiring. No production
path runs with the guard (env-gated). This implements the exact prescription SH355's
do-not-re-tread correction names as the ONLY legitimate re-attack of the EC reader.

## Why this is the genuinely-new step (not a re-tread)

SH302 seeded the EC reader-gate from block-entry pc 0x102e24598 by computing the slot as
`entry_sp + 8` and measured it INERT (the reader's V2Init/StartLuaAppDM bl-targets still 0/5)
because the block was already cached / the seed lands on the wrong frame (entry-timing, not
block-cache). SH355's fresh disasm pinned WHY: the reader-gate block actually STARTS at
0x2e24694 — a REAL block boundary opened by the interior string-assign `bl 0x2b504e4`
@0x2e24690 returning to 0x2e24694. The gate load `ldr x8,[x29,#104]` @0x2e246b0 reads the
LIVE frame pointer of the frame that actually reaches `cbz x0` @0x2e246dc, so a seed must be
applied INSIDE that block from the live x[29], not from an sp-derived guess at a different
block's entry.

SH355's exact words for the next-artifact: "the residual is the live-object pointer value at
[x29,#104]+0x20, i.e. we must hand a coherent object whose [+0x20]==0 to the actual
construction entry — a value seed that fabricates the live object, not a compile/early-exit
fix." SH356 is precisely that value seed, made frame-accurate.

## What shipped (jit.rs)

`routeb_ec_world_reader_gate_frame_guard(state, pc)`:
- Fires at pc == 0x102e24694 (reader-gate block entry) OR pc == 0x102e246b0 (the `ldr
  x8,[x29,#104]` gate-load instruction) — both INSIDE the reader-gate block, using the LIVE
  state.x[29].
- Computes the slot = x[29] + 104 (the exact caller-frame object slot `ldr x8,[x29,#104]`
  reads @0x2e246b0).
- Writes a FABRICATED zeroed 0x40 buffer (so [obj+0x20]==0 -> `cbz x0, 0x2e246f4` TAKEN ->
  the reader 0x2e246f4 and its real V2Init 0x1023c5538 + StartLuaAppDM 0x1023f1654 become
  reachable).
- Seeds a NULL slot too (a NULL would make 0x2e246d8 `[0+32]` fault; handing a coherent
  object is strictly better).
- Idempotent (OnceLock on the zeroed buffer), default-inert (env-gated).

Wired into the run-loop dispatch immediately after the SH302 guard (jit.rs run_loop).

Hermetic `sh355_frame_accurate_ec_reader_gate_seeds_live_frame_slot` asserts: inert without
env; pc-gated to only 0x102e24694/0x102e246b0 (NOT entry 0x102e24598); seeds [x[29]+104]
with a [+0x20]==0 object; seeds a NULL slot; idempotent.

## Measure (real binary, capture_sh302_readergate.sh + JIT_ROUTEB_EC_READERGATE_FRAME=1, 3/3)

**The frame-accurate guard fired 0/3; the reader remains closed. This is a decisive
measured negative, NOT a re-tread:**

- All entry-pc seeds fire 3/3 (routeb-sh298 arg1, routeb-sh299 arg0vt, routeb-sh300
  realsession, routeb-sh302 entry reader-gate) at the EC-world entry block 0x102e24598.
- `[routeb-sh355] FRAME-ACCURATE EC reader-gate` NEVER fires in any of 3 runs. Since the
  frame-accurate guard fires only when the reader-gate BLOCK at 0x102e24694/0x102e246b0 is
  actually ENTERED (live x[29] captured from that executing frame), 0 fires is the proof
  that block is never entered headlessly — not a value problem, a reachability problem.
- Run sequence: EC entry (seeds fire) -> SH296 dmfn returns Ok -> control DIVERTS into
  app-start (routeb-sh248e once-cell + routeb-sh248f lifecycle adapter fire) -> SIGSEGV at
  the SH285 persistence-lane live-object wall guestpc=0x101db1b08 (fault 0xff..ff). EXIT 134.
- The reader (0x2e246f4) sits behind a block (0x2e24694) that the EC body does not fall
  through to in this harness — control leaves the EC world into the app-start/persistence
  path and dies at SH285 before any reader-gate continuation block is ever translated.

**Conclusion vs. sh302/sh355:** sh355 reasoned the reader-gate block starts at 0x2e24694
and that entry-frame-accurate seeding (sh302's entry_sp+8 was wrong-frame) would open it.
The frame-accurate variant eliminates the wrong-frame concern — same result (0 fires). The
measurement therefore proves the gate is BEYOND a slot value: the reader-gate block is never
entered at all (the EC body diverges into app-start / the SH285 wall first). This is the
SH174/204-class unconstructed-live-object attribution, confirmed at one level deeper than
sh302 (which could only say "seed fires, reader-targets 0/5"; we show the target block is
0-fires even with the exact correct frame). This CLOSES the SH355-named lever with evidence:
do NOT re-attack the EC reader by fabricating [x29,#104]+0x20 — the block is unreachable
(control diverges to the persistence lane), not value-gated.

## Honest

Does NOT construct a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). The frame-accurate guard is implemented, hermetic-test-clean,
and MEASURED INERT (0/3) on the real binary — a closure that refines sh302's verdict from
"wrong frame" to "block never entered (control diverges to SH285 persistence-lane wall)".

## Verify

- `cargo test -p arm64jit --lib -- sh355_frame_accurate` = 1 passed.
- `cargo test --workspace` EXIT 0; `cargo build --workspace` EXIT 0 (after completing-ladder
  capture finishes).
- Repro: capture_sh302_readergate.sh + `JIT_ROUTEB_EC_READERGATE_FRAME=1`.
- Commit: local `dev` only (operator pushes).