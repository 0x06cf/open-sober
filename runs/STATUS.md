# Open-Sober run status (hermes-worker)

Updated 2026-09-20, this cycle: SH372 (convergence proof) then SH373 — MEASURED SH285
CROSSOVER from the SH371 reaching-env: the standing SH285 persistence leaf (guestpc
0x101db1b08) is deterministically crossed (5/5) via SH349's append sub-call skip +
the M+0x48/appname seeds that make the DM-creator continuation run deep; terminal
advances to 0x101d9a708 in the SAME measured-closed unconstructed-live-object family.
recon-v3 deliverables re-verified green. Workspace green (615/0).

## Current state

- `dev` HEAD: SH373 (jit.rs sh372 convergence hermetic, arm64jit lib 434/0 + capture
  capture_sh373_cont_appendskip.sh + frontier-sh373 doc).
- Workspace green (cargo test --workspace EXIT 0, 615 passed/0 failed).
- recon-v3 deliverables green (24 task-driven frames swap Ok(0x1), 0 json abort, 0 crash).
- Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0, MH_* all false.

## What advanced this session

- SH372: settings-state & DM-continuation init paths CONVERGE on the identical SH285 leaf
  (path-independent), answering SH371's "same object or different?" with a fresh register dump.
- SH373: first deterministic CROSS of the SH285 wall from the reaching env (sh285=0, 5/5).
  Explanation of SH358's "0 continuation hits": that run lacked the M+0x48/appname seeds.
  The cross lands at 0x101d9a708 (same family); append+pack combo parks at the pool-pop
  write-site 0x101d9a528. Persistence lane remains measured-returned (whack-a-mole UNBOUNDED).
- No production path edited (measurement + probes + docs only).

## Honest status

- Route-B live-DM structural gate UNCHANGED. SH174 capture-latch stays the single forward
  hook. SH373 closes the "is SH285 itself the invariant?" loophole (it is crossable into the
  already-known closed family). Only a REAL LocalStorageManager/session ctor gets past.

## Next-forward candidates

1. (PRIMARY, Route-B) The SESSION half remains THE wall: do-init must own a live DataModel
   (SH184/185). Both measured dead-ends from the now-reached continuation are measured-closed
   (SH285 live-object wall + F+0x18 controller floor) — do NOT re-drive LSM sub-call skips.
2. R1 content half staged+armed+serviceable (SH351/352/354); latent until a live DM drives the
   loader.
3. Do NOT re-drive LSM sub-call skips (SH349/350/358/373); do NOT re-arm window-attach
   once-guard (SH367); do NOT re-enter ALooper loop (SH365).