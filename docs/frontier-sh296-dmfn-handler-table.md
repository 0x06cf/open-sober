# Frontier SH296 — DM-construction handler 0x1023f03b4: located in a method-dispatch table, driven headlessly FIRST time

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--v2boot-session-dmfn`).

## tl;dr

SH255 proved the DM-construction fn **0x1023f03b4** (the app-shell session-ctor
front-door that calls `nativeAppBridgeAppStart` 0x2362e98 and the EC-world
marshaler 0x23f1210) is indirect-dispatch-only (0 direct bl/b callers) — `this`
is a "fabricatable-object-graph/live-this" class, never executed headlessly. This
cycle LOCATES its method-dispatch table, then DRIVES it DIRECTLY. Measured: the
fn **executes headlessly for the first time** and, with an empty fabricated
receiver, **benign-returns Ok(0x0)** deterministically 3/3 (the safety epilogue);
with fields [this+120/128/136/144] seeded to the blanket singleton it still
returns Ok(0x0) WITHOUT entering the marshaler/app-start/factory ranges
(region-watch 0 hits). SH255's static judgment -> measured: the deep construction
is gated on GENUINE distinct live sub-objects (SH174/SH204 class), the standing
live-object gate — not reachable by blanket-leaf fabrication.

## The dispatch table (new locating result)

The handler fn 0x1023f03b4 is the SOLE entry at **method-dispatch handler table
vaddr 0x73fa4d0, entry[4]** (slot +0x50). File-offset scan: the value 0x23f03b4
appears exactly ONCE in the binary, at file 0x68ea538 (vaddr 0x73fa538) — a
loader-RELATIVE (packed-RELA) slot. The table is `{marker=0x403, handler_fn,
tag}` tuples exchanging into the 0x639a000 protocol-method name band. Its
handlers span the EC/game region (0x2e27c48/0x2e3b530/0x2e27d14/0x2e281bc/
0x2e28434...), the V2UpdateSurface region (0x25f1868/0x25f2c8c), the EC-world +
`__clone` stub (0x1db2cf0, ×4), and the DM-construction fn 0x23f03b4.
The marshaler (0x1023f1210) and EC world (0x102e24598) have ZERO data-pointer
refs — they are code-reached only.

## What was driven (`--v2boot-session-dmfn`)

Fabricated receiver: leaked 0x200 `this` with a leaf-vtable whose vt[32] is a
registered **ret-0** host leaf (`dmfn_ret0_leaf`) — the entry dispatch
`ldr x8,[x0]; ldr x8,[x8,#32]; blr x8` then reads w0: ret==0 -> the construction
body (0x23f0484), !=0 -> the crashpad-init+return branch. arg1/arg2 = leaked
readable zeroed buffers. Two stages:
- **S1** (empty receiver, 3/3): `dmfn returned Ok(0x0)` — fields all 0, so
  [this+144]=0 -> cbz -> [this+136]=0 -> cbz -> the 0x23f0680 safety epilogue ->
  ret 0. The fn EXECUTES headlessly FIRST time, benign-return.
- **S2** (fields [this+120/128/136/144]=routeb singleton, run-watch): `dmfn
  returned Ok(0x0)`; region-watch on [0x1023f04d4,0x1023f1210), marshaler
  [0x1023f1210,0x1023f1300), nativeAppBridgeAppStart [0x102362e98,0x102362f40),
  factory [0x2b4ea48,0x2b4eac0) = **0 distinct hits** — the deep construction
  (0x23f04d4 factory path -> marshaler -> EC world) is NOT entered with a
  blanket-singleton receiver. The SIGSEGV in the run (guestpc 0x101db1b08) is the
  known run-variable LSM app-start lane (SH285), unrelated.

## VERDICT (do-not-re-tread)

- The DM-construction fn is EXECUTABLE headlessly (S1, measured). It benign-returns
  with an empty or singleton-seeded receiver because the gates ([this+120/128/
  136/144] -> distinct LIVE sub-objects whose virtuals perform real app-bridge
  work, plus the 0x2b4ea48 factory returning a real box) are the SH174/SH204
  live-object class, NOT blanket-leaf fabricatable.
- The EC-world marshaler front-door stands UNCHANGED (live-DM structural gate).
  This converts SH255's static indirect-only judgment into a measured result and
  records the exact next-unsynthesized-object requirement on the SESSION-CTOR
  primary lever: distinct coherent live receivers, not one blanket singleton.

## Verify

- `cargo test -p arm64jit --example elfjit -- sh296` = 1 passed.
- `cargo test -p arm64jit --examples` = green (125/0). `cargo test --workspace` EXIT 0.
- elfjit.rs (1,048,522, 54 under) + HANDOFF.md under the 1MB pre-commit hook
  (prose trimmed this cycle to fit SH296).

## Files

- `crates/arm64jit/examples/elfjit.rs` (`dmfn_ret0_leaf`, DMFN_RET0_ADDR,
  `--v2boot-session-dmfn` rung, +hermetic sh296).
- `runs/capture_sh296_dmfn.sh`, `runs/sh296-dmfn-s1-{1,2,3}.txt`,
  `runs/sh296-dmfn-s2-{1,2,3}.txt`, `runs/sh296-dmfn-s2-watch.txt` (repro + captures).
- `HANDOFF.md`, `runs/STATUS.md` (ledger). `docs/frontier-sh296-dmfn-handler-table.md` (this).