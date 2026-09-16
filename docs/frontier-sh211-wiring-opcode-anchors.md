# SH211 — Regression anchors for the Route-B wiring + recon-v3 render-plane opcode sites

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed). Date
Sep 16 2026. Production change: **one new hermetic test** in
`crates/arm64jit/examples/elfjit.rs` (`sh211_routeb_wiring_and_render_plane_opcode_anchors`).
Workspace green (verified below). No behavioral/default change — pure regression net.

## WHY

The standing front (SH209/SH210/SH206) is a fully-armed-but-latent Route-B wiring
suite whose every marker is verified on the real binary by exact guest addresses.
The SH174 migration runbook, the recon-v3 self-driven frame plane, and the G1/G2/G3
surface wiring all depend on precise `.text` byte sites and cell addresses. If any
one of those constants silently drifts (a future disassembly error, a shifted patch
site, a mis-based cell), the fetch/seed would target the wrong byte and the whole
wiring would fail at the exact moment of a real app-launch (the migration instant)
— precisely when it must not. This cycle locks the load-bearing sites into the same
real-image guard family the project already uses (sh116b, sh200, sh201).

## WHAT IS PINNED

Byte-pinned against the real `libroblox.so` (109,193,800 bytes) when present
(skipped elsewhere, same guard as sh201):

| file vaddr | guest | word | meaning |
|---|---|---|---|
| 0x2856f24 | 0x102856f24 | 0x52800044 | type-4 drain idle heartbeat w4#2 original (`mov w4,#2`) — recon-v3 patches it to `mov w4,#4` (0x52800084) |
| 0x2856f68 | 0x102856f68 | 0x52800064 | type-4 drain idle heartbeat w4#3 original (`mov w4,#3`) — same patch |
| 0x2bb47c4 | 0x102bb47c4 | 0x52800093 | SendAppEventOnAppReady 'Home' path discriminator (`movz w19,#4`) — SH206 pin |
| 0x2bb47cc | 0x102bb47cc | 0x52800033 | SendAppEventOnAppReady ALT discriminator (`movz w19,#1`) — SH206 pin |

Always (hermetic, no image): the guest = file + 0x100000000 transform + 4-alignment
for those four `.text` sites, and 8-aligned in-window placement for the eight
standing wiring **cells** (`.bss`/`.data`, no on-disk opcode, but a mis-based/
typo'd constant is caught): type-4 producer vector [0x106829ea8], G1 surface-XID
cell [0x10683d348], G3 files-dir string [0x10726d600], governor router flag
[0x106a70880], SH174 capture-latch arm [0x106391908], flags-loaded latch
[0x106a683e8], do-init once-guard [0x106a68410], DM-root [0x106a68818].

## VERIFY

- `cargo test -p arm64jit --example elfjit sh211` — 1 passed, incl. the real-image
  byte pins (`sh211 real-image opcode anchors verified on libroblox.so`).
- `cargo test -p arm64jit --example elfjit` — 66/66 passed.
- `cargo test --workspace` green at this commit (re-verified).

## TREE

- crates/arm64jit/examples/elfjit.rs — new sh211 test (test-only, no default code path).
- docs/frontier-sh211-wiring-opcode-anchors.md — this doc.
- HANDOFF.md / STATUS.md — updated.

## STANDING (unchanged)

Route-B live-DM world-build = the structural gate (SH209/210/206 reconfirm). This
cycle adds no seed and no live-DM progress — it is a regression net so the armed
wiring (SH174 capture latch + G1/G2/G3) cannot drift before it is needed. recon-v3
self-driven frames re-verified green at HEAD before this change (24 task frames,
swap Ok(0x1), 197 pops, no json abort, EXIT 124). Cookie persistence stays
committed-as-is (SH177), not extended (operator: return to Route B).