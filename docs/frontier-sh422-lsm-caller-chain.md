# SH422 — name the caller CHAIN into the SH285/SH341 persistence lane

Date: Sep 2026 (hermes-worker). Single-agent (cone suppressed).

## Problem

Every headless do-init / app-start arm (SH404/405/407/408) drains into the
standing SH285/SH341 LSM persistence lane, and the loop's instrumentation has
only ever logged the TERMINAL guestpc (0x101db1b08 reader fault / 0x101d9a528
pool-pop write-site). The call path INTO the lane — which do-init callee reaches
the LocalStorageManager persistence code — is never named. The operator's
MIGRATION directive says keep manufacturing the DMCONT/PATH-B line "INSIDE this
JIT" and hunt its continuation; naming the caller chain that reaches the lane is
the missing first datum for that hunt. (`0x101d9a528` is not in-image in the
fault-bearing read region; only the terminal pc is ever repursed.)

## Deliverable

A reusable guest frame-pointer (bp) chain walker + a one-shot, default-inert
block-entry guard fired at the persistence-lane READER entry `0x101d99e30`
(guest == file 0x1d99e30 + 0x100000000) that walks the live aarch64 x[29] chain
from the CpuState and logs the saved-lr caller chain.

- `session::bp_chain_walk(fp, max)` — pure walker. aarch64 grows DOWNWARD, so
  each caller's saved-fp is at a strictly HIGHER address; the walk collects
  saved-lrs and stops on a non-domain fp, a non-ascending fp (chain edge or
  cycle), or `max` frames. Mem-fault-safe by trust convention (reads only under
  the mapped-domain check `>= 0x100000000 && top16 != 0xffff`), exactly like
  `read_visible_u64` / the SH420 canary shim.
- `session::routeb_lsm_bt_guard(state, pc)` — fires ONCE (first reader entry)
  per run. Env gate `JIT_ROUTEB_LSM_BT=1`; test override `set_lsm_bt_test` /
  `lsm_bt_test_override` for the hermetic (mirrors shims::STACKCHK pattern).
- Wired into the jit.rs block-entry dispatch next to the LSM keytrace/keyfix
  guards (single line; prose condensed elsewhere to keep jit.rs under the 1MiB
  hook: jit.rs 1,048,417 B < 1,048,576).

## Measurement

- Hermetic `sh422_bp_chain_walk_orders_and_terminates`: builds a leaked 3-frame
  chain, asserts the walker returns the saved-lrs in call order and terminates
  on a loop (non-ascending fp) and on a sub-domain fp.
- Hermetic `sh422_guard_inert_unless_override_and_reads_live_chain`: OFF (even
  at the target pc) inert; ON walks the live CpuState chain; a non-target pc
  stays inert even when ON. Both without a real binary.
- **Real binary** (`runs/capture_sh422_lsm_bt.sh`, SH408 far-reach env +
  JIT_ROUTEB_LSM_BT=1, EXIT 139 standing downstream SIGSEGV after the marker):
  the one-shot chain FIRES and names the do-init caller chain into the lane —
  `[routeb-lsm-bt] SH422 at pool-pop entry 0x101d9a5a0 .. caller chain lrs:
  [0]0x1021db13c <- [1]0x1021daf38 <- [2]0x1021e30bc <- [3]0x1021e2fdc <-
  [4]0x1021e2f34 <- [5]0x1021e2e40 <- [6]0x10217429c <- [7]0x0`. Guest->file
  (minus 0x100000000): pool-pop 0x1d9a5a0 is called by the fn returning to
  `0x21db13c`, then `0x21daf38` <- `0x21e30bc` <- `0x21e2fdc` <- `0x21e2f34` <-
  `0x21e2e40` <- `0x217429c` (chain top). The top frame `0x217429c` sits in the
  same do-init zone as the SH381 DM-ctor 0x2173b3c — the LSM lane is drained from
  the do-init world-build callee chain, one concrete named path (SH341 only ever
  logged the single LR=0x10626b6dc for later, .data-reloc-driven pops).

## Honest status

This is an INSTRUMENT (names the path into the standing wall), not a live DM and
not a persistence-lane fix. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0 under every prior complete substrate run). It is distinct from
the closed/re-tread cones: NOT a store-watch (SH4xx), NOT an LSM map manufacture
(SH396), NOT LSM-ctor lane wiring (SH384/385), NOT an LSM skip (SH349/350/358).
Default-inert: product path byte-identical when `JIT_ROUTEB_LSM_BT` is unset
(env gate read at the pool-pop-entry guard, no guest bytes, no behavior change).
Re-targeted from the SH285 reader 0x1d99e30 to the pool-pop entry 0x101d9a5a0
after MEASURED evidence that the reader is short-circuited (lsm-map seeder NOP's
the LSM init store) while the pool-pop is the genuinely-reached lane terminal.

Files: crates/arm64jit/src/session.rs (+walker/guard/2 hermetics),
crates/arm64jit/src/jit.rs (one dispatch line + prose condensation, addresses
kept).