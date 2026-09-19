# Frontier SH365 — MEASURED dead-letter: the host app-command FIFO is never drained by the guest android_app glue loop (addfd=0, pollonce=0, posted=3, 3/3), pinning the SESSION-CTOR "window/GL-surface APP_CMD_INIT_WINDOW" precondition as an undelivered lifecycle event on the completing ladder

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Always-on drain
counters added to the ALooper shims (shims.rs) + a 1.5s-grace readback in the
elfjit app-command feed + one hermetic (`app_command_drain_stats_count_shim_entries_and_posted`,
arm64jit lib 427) + capture `runs/capture_sh365_alooper_drain.sh`. No production
path edited (shims counters are 4 cheap atomics; elfjit readback is inside the
existing JIT_DRIVE_LIFECYCLE block). Workspace green (cargo test --workspace
EXIT 0, 608/0 — was 607).

## Why this cycle
SH264/276 drove the lifecycle NATIVES directly (`initAppShellReporter`,
`JNIAppLifecycleNativeAdapter_setActive`, `nativeInitClientSettings`/
`nativeInitClientSettingsSigned`, `nativeActivity_onEngineSettingsReceived`) and each
now executes headlessly, returning Ok. But SH264 *suspected* the android_app glue
main loop (guest 0x102bcd5d0) "busy-spins rather than dispatch APP_CMD_START/RESUME/
INIT_WINDOW" — an UNVERIFIED claim. The SESSION-CTOR directive names the window/
GL-surface `APP_CMD_INIT_WINDOW` as the precondition behind initEngine_'s
"*** Engine settings is null" hard-assert, and the app-command FIFO
(`shims.rs::app_cmd_queue` + `post_app_command`, fed under JIT_DRIVE_LIFECYCLE)
was built precisely to carry that. Nobody had MEASURED whether the guest ever
consumes it. That is the gap SH365 closes.

## What landed
- `shims.rs`: always-on atomics `ALOOPER_ADD_FD`, `ALOOPER_POLL_ONCE`,
  `APP_CMD_POSTED`, bumped respectively in `alooper_addfd`, `alooper_pollonce`,
  `post_app_command`; `pub fn app_command_drain_stats() -> [u64;3]` =
  [addfd, pollonce, posted]. Zero-cost unless contended.
- `elfjit.rs` (inside the existing JIT_DRIVE_LIFECYCLE app-command feed thread):
  after the 3 posts, sleep 1.5s (a grace window for the glue loop), then print
  the drain stats + a human verdict (DEAD-LETTER vs drained).
- Hermetic test pins (a) each shim bumps only its own counter, (b) post bumps
  only `posted`, (c) the dead-letter predicate reachable + false once a shim fires.

## MEASURED (real libroblox.so, completing --v2boot-skip-appstart ladder, 3/3 deterministic)
```
[elfjit:appcmd] posting APP_CMD_0 (1)      ; START
[elfjit:appcmd] posting APP_CMD_1 (2)      ; RESUME
[elfjit:appcmd] posting APP_CMD_2 (11)     ; INIT_WINDOW
[elfjit:appcmd] SH365 drain-stats: addfd=0 pollonce=0 posted=3 -> DEAD-LETTER (guest glue loop never consumed APP_CMD)
```
- 3/3 clean completing runs: `posted=3`, `addfd=0`, `pollonce=0`. The guest
  NEVER enters the ALooper lifecycle shims even though the host queued all three
  commands and ran a full N-second completing ladder (EXIT 124 = reached idle).
- `anativewindow_fromsurface` fires exactly once (`wired real X11 window XID=0x200000`)
  — so the ANativeWindow/surface EXISTS in the harness, but the lifecycle event
  that would hand it to the engine (`APP_CMD_INIT_WINDOW` consumed by the glue
  loop) is never delivered. The window is wired; the INIT_WINDOW callback is not.
- Context unchanged: once-guard=0x1, once-slot[0x106a68408]=0x400000b,
  DM-root[0x106a68818]=0x0, MH_* false, service-registry-count=12,
  app-data-model-count=0x1. Route-B live-DM structural gate UNCHANGED.

## Interpretation (honest)
- This is a cause-level refinement, MEASURED where SH264 inferred. It does NOT
  manufacture a DataModel and does NOT change the Route-B live-DM gate.
- It does convert the SESSION-CTOR's "needs window/GL-surface APP_CMD_INIT_WINDOW"
  precondition from a static assumption into a measured undelivered event: the
  glue main loop that would consume the app-command pipeline never executes its
  ALooper poll headlessly. So either (a) the glue loop is never entered at all in
  this harness (the natives are driven directly instead of through it), or (b) it
  is entered but calls a different looper entry than the shim resolves. Both are
  the "the upstream session ctor never runs" family; the direct-native drive
  (SH264/276) remains the productive line because the natives DO complete.
- The drain stats give a cheap, always-available health probe: if a future drive
  ever brings the glue loop alive, `addfd`/`pollonce` will go nonzero and the
  SH365 readback flips from DEAD-LETTER to drained — an objective trigger for the
  NEXT forward on this line.

## Files / verify
- crates/arm64jit/src/shims.rs: +app_command_drain_stats + 3 counters + bumps.
- crates/arm64jit/examples/elfjit.rs: SH365 readback in the app-command feed.
- crates/arm64jit/src/jit.rs: +1 hermetic (`app_command_drain_stats_...`).
- runs/capture_sh365_alooper_drain.sh (confirmed-green completing-ladder artifact).
- Workspace green (cargo test --workspace EXIT 0, 608/0). Commit: local `dev` only.