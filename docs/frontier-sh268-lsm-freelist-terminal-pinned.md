# Frontier SH268 — LSM free-list/pop terminal MECHANISM pinned (the SH267 crossed-state guard gap closed)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed — no subagents).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. No production path edited (a new hermetic regression test only).
Default-inert state unchanged: the only seed relevant here (`JIT_ROUTEB_APPSART_LSM_NODES=1`)
is SH267's, already committed. Workspace green (cargo build + cargo test --workspace exit 0).

## Why (the guard gap)

SH267 crossed the LSM INSERT-leaf wall (fault=0x0 at guestpc 0x101db1d04) by
giving each of the 0x2000 sub-slots a real leaked node cell so the insert's
atomic-OR (0x2b9ea40 `ldset`) lands in valid memory. Its +1 hermetic (`sh267`)
byte-pins ONLY the insert-leaf mechanism + the reader — but the NEW terminal that
SH267 *reached* by crossing it — the LSM **free-list/pop** path — was never
pinned. That is a regression-guard gap: if the image drifts (or a future seed
advances the lane), the crossed state could move without any hermetic failing
loudly. SH268 closes exactly that gap.

## The pin (fresh disasm + fresh run, real libroblox.so)

The app-start lane, with the FULL SH267 seed set, now terminates:

    SIGSEGV fault=0x101d968e4 guestpc=0x101d9a528
      x19=host-heap  x20=x1=0x101d968e4  lr=0x101d9a6b8  lsm_map_global=host addr

Re-verified fresh this cycle: `/tmp/sh268_fresh.txt`, EXIT 139, same
`guestpc=0x101d9a528 fault=0x101d968e4` (3 clean runs — deterministic).

Mechanism (fn `sub_1d9a4e0` tail 0x1d9a528, the free-list/pop):
- reached via `mov x0,x19; bl 0x1d9a528` at 0x1d9a6b4/0x1d9a6b0 (x19 = held key);
- decodes the two-level map (adrp 726f000 -> ldr [x8,#2240] = base; bucket; sub);
- pop tail: `ldr x8,[x0,#24]; str x1,[x0,#24]; str x8,[x1]` @0x1d9a568 writes the
  free-list link into `*x1` where x1 == the map **key**;
- the key 0x101d968e4 = file 0x1d968e4 sits inside the R-E exec LOAD segment
  [file 0x0,0x62d8190) prot write:off -> the str faults.

This is exactly the SH249/SH258 proven-unwritable live-object class, one full
fencepost past the insert leaf. Per SH249's segment proof, no seed / repair /
count-clamp / dynamic-ctor lever in this JIT can reach it — so it is a REGRESSION
pin of the current known frontier, NOT a forward gate. Do NOT re-drive
single-object seeds into it (same SEP-17 directive that says stop re-driving
seeds into the standing 0x1021dde34 wall).

## Code / verify
- `sh268_lsm_freelist_terminal_mechanism_pinned` (crates/arm64jit/examples/elfjit.rs,
  hermetic real-image guard, skip-if-absent): byte-pins free-list/pop entry +
  bucket/sub decode + the fatal link-store tail 0x1d9a568 + the caller
  `mov x0,x19; bl` at 0x1d9a6b0/0x1d9a6b4, and asserts the write target
  0x101d968e4 is in-image AND its file offset lands within the R-E exec segment
  [0x0,0x62d8190) (write:off).
- Verify: `cargo test -p arm64jit --example elfjit sh268` = 1 passed; full
  `cargo test --workspace` green (examples count bumps by one from 102/0).

## Honest (do-not-over-claim)
- Does NOT cross the free-list wall, does NOT manufacture a DM, does NOT lift the
  Route-B live-DM gate. The LSM/persistence lane is still parked at the standing
  live-object write-off wall (now one fencepost deeper than SH260 parked it).
- What IS new: the SH267 crossed-state's second half (the terminal it unlocked)
  is now fail-loudly pinned, so neither a binary drift nor an over-claimed future
  "crossing" can silently pass. This is the same pure-hardening discipline as
  SH251b/256/260/267: lock measured reality tightly, advance honestly.