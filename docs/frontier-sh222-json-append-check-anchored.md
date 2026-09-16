# SH222 — recon-v3 JSON-ABORT append-check site anchored against the real image

## Why
The recon-v3 "JSON-ABORT" deliverable (docs/recon-selfdrive-seed-jsonfix.md §B) is
`JIT_JSON_ZERO_FIX` — a pc-gated host hook at guest `0x102355d40` that forces the
leaking guest-stack libc++ `std::string` length to 0 (SSO empty) so
`RBX::json::Writer` never reaches its "string length overflow" throw helper
(`0x1025fb6bc`). This is an IMMEDIATE-PRIORITY, load-bearing hook: the recon-v3
self-driven frame plane (24 real task-driven frames, 195 node pops, EXIT 124,
0 crash) depends on it firing. Yet it had **no real-image byte-pin** in the
hermetic guard family (sh211/213/219/221 all pin their sites; this one only had a
unit test on the gate function). A silently drifted constant would stop the fix
from engaging without a loud failure.

## The pin (new hermetic `sh222_recon_v3_json_append_check_site_anchored`)
Byte-pinned against the real libroblox.so (109,193,800 B; skip-if-absent same
guard family as sh211/sh213/sh219):
- the block-entry the hook gates on — file `0x2355d40` = `0xa9bd7bfd`
  (`stp x29,x30,[sp,#-48]!`, the fn prologue of the RBX::json::Writer append
  bound-check), + guest `0x102355d40` transform + 4-alignment;
- the capacity cell the check compares the length against — guest `0x107275648`
  (file `0x7275648`) + window + 8-alignment;
- the throw helper the fix prevents reaching — file `0x25fb6bc` = `0xd10543ff`
  (`sub sp,sp,#0x150`, fn prologue of the overflow throw), + guest `0x1025fb6bc`.

## Verify
- `cargo test --example elfjit sh222_recon_v3_json_append_check_site_anchored` → ok
- Full surface: `cargo test --workspace` (567/0) + `cargo test --examples` (70/0
  incl. sh222).
- recon-v3 plane re-verified unregressed at HEAD before this edit (24 task-driven
  frames, 195 node pops, no json abort, 0 crash, EXIT 124).

## Standing (do-not-re-tread, unchanged)
Route-B live-DM world-build = structural gate (SH209/218/220). SH174 capture-latch
arming `*(0x106391908)` at a real make_shared = the single forward hook. recon-v3
immediate-priority deliverables stay shipped + verified. No production path
edited — pure regression net on the JSON-ABORT hook's load-bearing sites.