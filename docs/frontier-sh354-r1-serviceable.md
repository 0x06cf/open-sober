# Frontier SH354 — close the R1 content-half art with two new measurements: (a) even with the SH352-corrected flags-loaded gate armed (5/5), a completing ladder runs JIT_ASSET_TRACE with 0 hits — the staged CoreScript is staged-but-DORMANT until a live DM drives the loader; (b) new hermetic test proves the content half is SERVICEABLE end-to-end (a guest open of the exact files-dir CoreScript path resolves through fsmap::remap_path to the staged mirror and is readable)

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). One new hermetic test
(`sh354_r1_core_script_is_serviceable_through_remap`, arm64jit lib 418->419). No
production path edited (default-inert). Workspace green. Route-B live-DM structural gate
UNCHANGED (DM-root 0, MH_* false); SH174 capture-latch stays the single forward hook.

## (a) Measured: R1 content is staged + gate-armed but DORMANT (0 asset-trace hits)
Reran the completing skip-appstart ladder (runs/capture_sh352_r1_completing_ladder.sh env,
COMPLETING_RETRY_MAX=3) with JIT_ASSET_TRACE=1. Two of three attempts reached the full
completion marker (probe=1, crash=0, EXIT 124); the third died early (run-variable SH350-closed
LSM lane). On BOTH clean completions:

- `[r1] gate @0x1072739d4 0x0->0x1` (the SH352-corrected primary flags-loaded latch ARMS).
- clean `[r1] loader gates` line: `0->1, 0->1, 80->0, 80->0, 0->0` (all 5 write, none dropped).
- `[r1] STAGED AppShell.lua / CoreScripts.lua` under the fsmap mirror (both 518 B).
- `[asset-trace]` lines: **0** on both clean completions — the staged CoreScript is never
  opened by the loader even though the primary gate is now correctly armed.

This is genuinely new information (my prior memory note "asset-trace 0 fired" predates the
SH352 0x1072739d4 gate-address fix). Interpretation: arming the loader GATES does not by itself
cause the loader to run. The ScriptContext/CoreScripts loader (0x101f1d8ac) stays 0-hit on the
completing ladder because it is driven only by a live DataModel session (the standing SH340/344c
session-half wall). The R1 content half is correctly staged + armed + now proven serviceable
below, but remains DORMANT (fires the moment do-init owns a live DM). No Route-B advance; the
live-DM structural gate is unchanged.

## (b) New hermetic: the content half is SERVICEABLE end-to-end, not just staged
`sh354_r1_core_script_is_serviceable_through_remap` (arm64jit lib, hermetic, no live image):
stages the R1 module under a temp root override, then opens the EXACT guest files-dir CoreScript
path the engine resolver would use (`/data/user/0/com.roblox.client/files/scripts/CoreScripts/
{AppShell,CoreScripts}.lua`) through `fsmap::remap_path`, and asserts:
1. remap_path RESOLVES the guest path (not None — the syscall shim passes it through unchanged otherwise),
2. the resolved host path equals the STAGED mirror under staging_root (remap == stage, no drift),
3. the file at that resolved path is READABLE and contains the self-constructing ScreenGui body.

This closes the one untested link in the R1 content path: SH351 tested the STAGING write only.
Nothing pinned the SERVE lookup. Now staged AND resolvable AND readable are proven — a guest
open (`openat` via jit.rs:4429 mappath) of the CoreScript path returns the real self-constructing
module. Only the live-DM session half remains between the armed content path and real
engine-authored GuiObjects.

## Honest
- No DataModel manufactured; DM-root [0x106a68818]=0, MH_* false. Route-B live-DM structural
  gate UNCHANGED.
- The asset-trace dormancy measurement and the serve-path test together make the R1 content half
  as complete as it can be WITHOUT a live DM: staged + all gates armed + provably serviceable.
  The remaining gate is purely the session half (do-init owning a live DataModel drives the
  loader).
- SH174 capture-latch stays the single forward observer.

## Verify
- `cargo test -p arm64jit --lib sh354` = 1 passed.
- `cargo test --workspace` EXIT 0 (arm64jit lib 419/0; elfjit examples unchanged).
- Repro (asset-trace dormancy): the completing-ladder env above + JIT_ASSET_TRACE=1, grep
  `[asset-trace]` = 0 on a clean (probe=1 crash=0) completion. Log /tmp/r1_asset_trace_hit.txt.
- Commit: local `dev` only (operator pushes).