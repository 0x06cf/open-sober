# Frontier SH419 — R1 content surface spans files + app-CACHE roots

## Why this cycle

STATUS next-forward #1 (the missing-env crossing audit) is VERIFIED COMPLETE:
a full sweep of every `--v2boot` boot-path runbook shows only
`capture_sh416_input_poll.sh` lacks the 3-gate crossing env, and SH417b
deliberately reverted it unchanged (the env does not cleanse that LSM lane). The
two canonical stable-baseline full-boot runbooks (capture_v2boot.sh SH54 +
capture_v2boot_gate.sh SH81) carry the env and SH417c measured them clean of the
hashfix misattribution. So the audit named as the standing next-forward is done.

With the audit closed, the open Route-B-supporting content surface gap is
deleg_8d5648cf's R1 instruction "also mirror under cache-root": the R1 synthetic
CoreScript (recon-v3 R1-first — engine SELF-constructs GuiObjects, zero host
layout) was staged under ONLY the files-dir mirror
(`data/.../files/scripts/CoreScripts/`, SH412/sh351/sh354). The fsmap remaps
guest `/cache` too, and real Android asset resolvers commonly probe the app
cache before the files dir — so a cache-only probe would miss the module once a
live DM drives the loader, and Route-B's R1-first content half would be
incomplete at the exact moment it is reached.

## What landed

`crates/arm64jit/src/session.rs` (off the 1MiB hooks; jit.rs untouched at
1,048,571 B < 1,048,576):

- New `pub fn mirror_r1_cache_root() -> Vec<String>` — copies the two staged R1
  candidates (`AppShell.lua`, `CoreScripts.lua`) from the files-dir mirror into
  the app-`cache` mirror
  `<root>/data/user/0/com.roblox.client/cache/scripts/CoreScripts/<Name>.lua`.
  Pure file staging: honors the fsmap test override, returns `[]` on a no-root
  harness, idempotent, guarded (`src.is_file()` skip), no guest byte touched.
- `drive_content_surface()` now calls it right after `stage_r1_core_scripts()`
  (SH412's G3 content step), so the ordered session-substrate's content surface
  serves BOTH the files root and the cache root a resolver may probe. Latent-but-
  correct exactly like SH412: the mirror is only READ once a live DM drives the
  Lua loader.

New hermetic `sh419_r1_cache_mirror_serves_both_roots` (session.rs): under a
test fs root, stages the files mirror, mirrors into the cache root, and proves
(a) each cache file exists + names a self-constructing `ScreenGui`, and (b) a
guest open of the app-cache path resolves through `fsmap::remap_path` to exactly
that mirror (the serve half, SH354-style).

## Measured (real libroblox.so)

- recon-v3 deliverables re-verified green at this exact HEAD:
  `capture_taskv4_frame.sh` attempt 1 = real task-driven frames
  `present swap Ok(0x1)`, 0 json abort, 0 crash (run-variable frame/node counts;
  both the 24-frame baseline and this run are crash-free real frames — the frame
  path `--startapp` does not touch the session substrate).
- Workspace green: `cargo test --workspace` EXIT 0; arm64jit lib 473 passed/0
  failed incl. the new sh419; `cargo build --example elfjit` OK.

## Honest

NOT a DM (DM-root [0x106a68818]=0, no make_shared, MH_GAME_LOADED false). Route-B
live-DM structural gate UNCHANGED. This is a content-side
BUILD-THE-RUNTIME completion: the R1 self-constructing-CoreScript content half
now spans both roots the resolver may probe, removing the cache-probe miss that
would otherwise blunt Route-B's R1-first screen the instant a live DM arrives.
No re-treads. recon-v3 deliverables unchanged-green.

## Files

- crates/arm64jit/src/session.rs (+mirror_r1_cache_root, +sh419, +drive_content_surface wiring)
- docs/frontier-sh419-r1-cache-root-mirror.md (this file)
- runs/capture_taskv4_frame.sh re-run log /home/hermes-worker/runs/sh60-after-sh419.txt