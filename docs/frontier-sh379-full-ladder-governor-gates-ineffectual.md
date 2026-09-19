# Frontier SH379 — the GOVFLAG+PRELOAD+PACK_SKIP governor-crossing gates are INEFFECTUAL on the FULL --v2boot ladder: even with all three armed, governor/DM-creator/setDataModelToCurrent/ScriptContext get 0 hits and the run drains into the same closed LSM pool-pop lane 0x101d9a528 (never-run intersection closed negative)

Session: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One new probe
runs/capture_sh379_full_ladder_govgates.sh + this doc. No production path edited (all existing
default-inert guards). Workspace green (cargo test --workspace EXIT 0, 616/0).

## The genuinely-new intersection measured this cycle

SH376/377/378 proved GOVFLAG + PRELOAD_VALUECELL + PACK_SKIP cross the governor NULL-DM deref and
preload-valuecell walls on the --v2boot-skip-appstart send-appevent env (SendAppEventOnAppReady
returns Ok). The operator's SESSION-CTOR lever targets the FULL ladder (app-start DRIVEN), which
SH344 showed reaches the governor tail (22 pcs) + the DM-creator/initEngine_ band on the SH343 env.
But NOBODY had combined the skip-appstart-only governor gates with the FULL ladder. This cycle runs
that never-run intersection with region-watch on the governor, DM-creator, setDataModelToCurrent,
app-shell ctor, and ScriptContext bands.

Result (real libroblox.so, full --v2boot ladder + GOVFLAG + PRELOAD + PACK_SKIP + append-skip):

- app-shell/do-init world-build band 0x102207b50..0x102209000 RUNS DEEP (registrar pcs
  0x102208xxx — the same SH340-measured 77-block construction band), i.e. do-init world-build
  happens again under the gates.
- **governor 0 hits, DM-creator (0x102bd1xxx) 0 hits, setDataModelToCurrent (0x102dbcc) 0 hits,
  ScriptContext (0x101f1d8ac) 0 hits.**
- Terminal: `guestpc=0x101d9a528 fault=0x0` (EXIT 134 SIGABRT after SIGSEGV) — the SAME closed LSM
  pool-pop write site the skip-appstart env drains into. MH_FLAGS_LOADED/APP_READY false,
  AppBridgeV2[0x106a705e8]=0x0.

## Interpretation (closes the never-run intersection, refines the map)

The governor/preload/pack gates are INEFFECTUAL on the full ladder: the full (app-start-driven)
path drains into the persistence LSM lane (0x101d9a528) BEFORE control reaches the governor, so
arming the governor gates changes nothing here — they only mattered on the send-appevent path that
drives SendAppEventOnAppReady directly (SH376-378). This refines SH376's "corrected terminal
sequence" to be path-specific: the skip-appstart env crosses governor+preload+pack and re-enters
persistence; the full ladder hits persistence first, gates moot. Both converge on the same closed
lane; Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false, governor/session 0-hit).

## Do-not-re-tread (unchanged)

- Do NOT re-drive LSM sub-call skips (SH349/350/358/373/375/377/378).
- Do NOT re-attack EC reader/0x258b5d8/window-attach/ALooper (SH356/362/367/365).
- Do NOT expect the governor gates to change the full-ladder terminal (this cycle measured they don't).

## Files

- runs/capture_sh379_full_ladder_govgates.sh (probe), log /home/hermes-worker/runs/sh379-full-ladder-govgates.txt.
- No Rust edited. Commit: local `dev` only (operator pushes).
