# SH423 — the durable persistence contract proven at the fsmap layer

Date: Sep 2026 (hermes-worker). Single-agent (cone suppressed).

## Problem

Objective 2b ("remembers sign-in") rests on the fsmap remap layer: a guest
datastore / shared-prefs / cookie path under an Android mount root must land as
a REAL, durable host file that survives a restart. But `crates/arm64jit/src/
fsmap.rs` — the module that owns that contract — had **zero** tests of its own
logic. It was only exercised indirectly, by jit.rs hermetics (sh351/sh354)
that harness remap for R1 CoreScript STAGING and the serve-LOOKUP; nothing
pinned:

1. the 5-root longest-prefix remap precedence (how `/storage/emulated` is
   handled before the shorter `/storage`),
2. the pass-through rules (relative / null / unmapped absolute `/proc`,
   `/system`),
3. — most importantly — the DURABLE round-trip: a value written through a
   remapped guest path is a real on-disk file that survives the in-memory
   override being cleared (a simulated restart) and is readable back through a
   fresh, independent remap. That is the literal "remembers sign-in" artifact.

## Deliverable

A small hermetic test module appended to fsmap.rs (module-local
`FSMAP_ROOT_LOCK` to serialize the two tests against each other, mirroring
jit.rs's `FS_ROOT_LOCK` since both mutate the process-global fsmap override
cell):

- `sh423_durable_datastore_write_survives_remap_restart`: sets the fsmap root
  to a temp dir, remaps `/data/user/0/com.roblox.client/databases/rbx-session.db`
  (proving the mapped path preserves the guest dirs and stays under the root),
  ensures the deep parent chain (an O_CREAT open never ENOENTs via
  `ensure_parents`), writes a remembered-session value to the host path, proves
  it is a real on-disk file (metadata present, correct size), then CLEARS the
  in-memory override to `None` (simulated restart), re-arms the same on-disk
  root, re-remaps through a fresh independent `remap_path`, and reads the
  EXACT value back.
- `sh423_remap_precedence_and_passthrough`: pins the longest-prefix precedence
  (`/storage/emulated` → `storage/emulated`, `/storage/foo` → `storage/foo`),
  the pass-through of unmapped absolute roots (`/proc`, `/system`) and of
  relative paths + null, and the fully-inactive state (override cleared to
  None) where even `/data` passes through unchanged.

## Measurement

- Hermetic: 2 passed / 0 failed, stable across 3 consecutive runs (the
  FSMAP_ROOT_LOCK removed a genuine first-run parallel-test race where the two
  tests clobbered each other's override).
- Full workspace: `cargo test --workspace` green (arm64jit lib 482 passed / 0
  failed, up from 480; full workspace 0 failed). `cargo build --example elfjit`
  OK (exit 0, 0 errors).
- recon-v3 immediate-priority deliverables re-verified green at this HEAD first
  (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames `present swap
  Ok(0x1)`, 197 node pops, 0 json abort, 0 crash).

## Honest status

NOT a DM (DM-root [0x106a68818]=0, MH_GAME_LOADED false — re-confirmed by the
SH415 do-init probe under the complete substrate: once-guard bit0=1, once-lambda
let to run, DM-root still 0). Route-B live-DM structural gate UNCHANGED. This
closes a genuine BUILD-THE-RUNTIME COVERAGE gap: the persistence contract the
client's datastore lands on is now pinned end-to-end (remap → on-disk file →
survive restart → read-back), all headlessly. It is NOT a re-tread: it is the
first test of fsmap's own logic (distinct from the R1-staging tests in jit.rs;
not an LSM/DM/store-watch seam). No production path / guest byte / JIT-hook
default changed (fsmap.rs is pure host-side path remapping, off-hook);
`libc`/env/test-only additions only.

Files: crates/arm64jit/src/fsmap.rs (+2 hermetics).