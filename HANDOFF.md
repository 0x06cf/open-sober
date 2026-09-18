# Open Sober — Agent Handoff
## SH336 (Sep 19, 2026, hermes-worker): pinned the real V1 nativeAppBridgeAppStart__ ABI
## (0x102338510) — the SEP-17-named lifecycle primitive reachable only via the --v2boot V1
## fallback (skipped by --v2boot-skip-appstart), never independently driven. Reconfirmed at HEAD
## that all standing closures reproduce. Route-B live-DM gate UNCHANGED.

Hermetic `sh336_v1_appstart_abi_pinned` (elfjit.rs, real-image, 1 passed) byte-pins V1
AppStart__'s ABI: prologue `sub sp,#0x1a0` (0x102338510=0xd10683ff), `add x29,sp,#0x150`,
`mov x19,x7` (6th arg), version-gate adrp 0x6a64000/ldr [0x10683d350] (0x338550/554),
x0=env (mov x21,x0 @0x33859c), first jstring marshal `bl 0x21e1fec` (0x3385a4=0x97faa692,
SH186 identity shim). ABI: x0=env, x1=thiz, x2..x7 = 5 fabricatable jstring handles + jboolean Z
(like the existing --startapp-v1 fallback ~elfjit.rs:7026 but as a SEP-17 rung slot).
Disassembled the service-registry writer family: the controller-table write sites
(0x10243cb64, 0x1061f0e40) are in the app-data-model/game-init + socket family, NOT a seedable
"App"-insert; consistent with SH313/317/335 (registry session-constructed only).

Re-verified at newest HEAD (no regression since SH335):
- recon-v3 SELF-DRIVED FRAMES: 24 task frames, present #19..#23 swap Ok(0x1), 197 pops,
  0 json abort, EXIT 124 (runs/capture_taskv4_frame.sh).
- SH307 A/B: arm-B SendAppEventOnAppReady Ok, 0 SIGSEGV, EXIT 124; do-init/app-shell-ctor/
  StartAppWithParams pipe executes real code (app-data-model count 0->1).
- SH335 reglive closure: registry count 0..12 task-scheduler-only at the DM-ctor lookup,
  "App" never an entry, once-slot 0->0x400000b ('Execute'), DM-root 0. EXIT 124.

Workspace green (elfjit examples 154->155/0; arm64jit lib 410/0; ws 0 fail). elfjit.rs = 1048566 B
< 1MB hook (prose condensed from earlier SH blocks, facts/addresses preserved).
HONEST: no DM (DM-root [0x106a68818]=0, MH_* false); Route-B live-DM structural gate UNCHANGED;
SH336 is a lock-the-ABI + re-verify cycle, not a session advance. Driving V1 AppStart__ is NOT
attempted (walks the same live-object wall --v2boot-skip-appstart bypasses). SH174 capture-latch
stays the single forward hook. NEXT (unchanged): a real Activity/AppBridge session that registers
the "App" service — the SESSION-CTOR primary lever. +probe/runs unchanged; +frontier doc
sh336-v1-appstart-abi-pinned.md.