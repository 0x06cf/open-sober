# SH230 — Byte-pin the type-4 dispatch site (the recon-v3 self-driven-frame load-bearing opcodes)

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed). Date
Sep 16 2026. Production change: **extend the sh211 hermetic** in
`crates/arm64jit/examples/elfjit.rs` to also byte-pin the four type-4 dispatch
opcodes. Workspace green (verified below). No behavioral/default change — pure
regression net; a gap closed in an already-shipped deliverable.

## WHY (a genuine open gap, not a re-tread)

The recon-v3 immediate-priority deliverable `SELF-DRIVEN FRAMES`
(`docs/recon-selfdrive-seed-jsonfix.md` §A) rides two load-bearing sites:
1. the two drain idle heartbeats (file 0x2856f24 / 0x2856f68) — patched from
   `mov w4,#2/#3` to `mov w4,#4` so every idle drain dispatch routes through the
   type-4 vector; and
2. the type-4 **dispatch site itself** (file 0x2853784): `adrp x8,6829000; ldr
   x3,[x8,#3752]; cbz x3,0x2853af0; ...; br x3` — the exact instruction stream
   that `br's` into the seeded `0x106829ea8` vector (the `type4_frame_thunk`
   host-thunk or the real-guest frame-fn seed) to present a task-driven frame.

SH211 byte-pinned (1) and the wiring **cells**, but NOT (2) — the dispatch site's
own opcodes were never guarded on the real image. A silent drift there (a
disassembly/patch error or in-image shift) would break the self-driven-frame
deliverable at the exact byte the harness patches and seeds through, with no
loud failure. This closes that gap in the same real-image guard family
(sh116b/sh200/sh201/sh211/sh222/sh228 pattern).

## WHAT IS PINNED (added to sh211, verified against the real libroblox.so = 109,193,800 B)

| file vaddr | guest | word (LE) | meaning |
|---|---|---|---|
| 0x2853784 | 0x102853784 | 0xd001fea8 | `adrp x8,6829000` (page of the type-4 vector) |
| 0x2853788 | 0x102853788 | 0xf9475503 | `ldr x3,[x8,#3752]` (= the `0x106829ea8` vector word) |
| 0x285378c | 0x10285378c | 0xb4001b23 | `cbz x3,0x2853af0` (null-vector skip — inert degenerate) |
| 0x28537b8 | 0x1028537b8 | 0xd61f0060 | `br x3` (tail-call into the seeded vector = the dispatch) |

Byte-pins are skipped when the image is absent (guard family convention; only the
VPS keeps it). No `catch` clause — a 4-byte read at these offsets is within the
`.text` section, so lack of the image is the only skip condition.

## VERIFY

- `cargo test -p arm64jit --example elfjit sh211_routeb_wiring` — 1 passed, incl.
  the real-image byte pins (`sh211 real-image opcode anchors verified`).
- `cargo test -p arm64jit --examples` — 76/76 passed.
- `cargo test --workspace` green (389 arm64jit + all crates + examples).
- recon-v3 render plane re-verified green at this HEAD before the edit
  (capture_taskv4_frame.sh = 24 task-driven frames, present #19..#23 swap Ok(0x1),
  195 node pops, no json abort, 0 SIGSEGV/SIGABRT, EXIT 124).
- SH210 wiring re-verified green at this HEAD AFTER the confirm (3/3 clean EXIT
  124; arm=3 xid=3 appevent=3 filesdir=3 dmprobe=3 — the ENTIRE latent Route-B
  wiring: capture latch + G1 + G2 + G3 + DM probe, all ARMED).

## TREE

- crates/arm64jit/examples/elfjit.rs — sh211 extended with the 4 dispatch-site
  byte-pins (test-only, no default code path).
- docs/frontier-sh230-type4-dispatch-bytepins.md — this doc.
- HANDOFF.md / STATUS.md — updated.

## STANDING (unchanged)

Route-B live-DM world-build = the structural gate (SH209 + ~30 measured negatives
do-not-re-tread). This cycle neither manufactures a DM nor adds a seed — it hardens
the already-shipped recon-v3 self-driven-frame deliverable so the load-bearing
dispatch opcodes cannot silently drift before a real session advances. The single
forward hook stays SH174 capture-latch arming at a real make_shared. recon-v3
deliverables stay shipped + verified. Cookie persistence stays committed-as-is
(SH177), not extended (operator: Route B is the front).