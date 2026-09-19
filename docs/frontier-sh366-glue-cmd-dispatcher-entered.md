# Frontier SH366 — ENTERED the guest app-command DISPATCHER process_cmd (0x102bcd6e4): APP_CMD_INIT_WINDOW case body EXECUTED headlessly for the first time (marker [inner+9]==1), the SESSION-CTOR window/GL-surface precondition the operator names for initEngine_ — a genuine first-entry (SH39b/SH365 only ever watched the loop / measured dead-drain)

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). One new bounded,
cause-level drive `drive_glue_process_cmd` (arm64jit/src/jit.rs, opt-in rung
`--v2boot-glue-cmd` at the TOP of the ladder so the measurement survives the
run-variable persistence-lane walls) + one new real-image hermetic
`sh366_glue_process_cmd_abi_and_init_window_case_pinned` + capture
`runs/capture_sh366_glue_cmd.sh`. elfjit.rs held <1MB (condensed SH-prose
comments to make room). Workspace green (cargo test --workspace EXIT 0, 430/0 —
arm64jit lib 429 passed after sh366).

## Why this cycle
The operator's SESSION-CTOR lever names the window/GL-surface `APP_CMD_INIT_WINDOW`
as the precondition behind initEngine_'s "*** Engine settings is null" hard-assert.
SH39b region-watched the android_app glue LOOP (guest 0x102bcd5d0) at **0 hits** and
declared the looper lever inert; SH365 MEASURED the app-command FIFO is never drained
(addfd=0, pollonce=0, posted=3). But BOTH only ever *observed* the path — neither ever
*entered* it, because the glue main loop is an INFINITE `ALooper_pollOnce` loop that
cannot be jit_run to completion. The loop dispatches to a BOUNDED function,
`process_cmd(app, cmd)` at guest **0x102bcd6e4** (w1 = the APP_CMD value), which CAN be
entered. Nobody had ever driven that dispatcher. That was the gap.

## The drive
- Disasm (real libroblox.so): process_cmd entry `sub w8,w1,#1; cmp w8,#0x13; b.hi`
  -> w1 is the APP_CMD. `ldr x20,[x0]` (= [app]); INIT_WINDOW(11) -> x8=10 ->
  jump-table offset 0x17 -> case **0x2bcd78c**. Case version-gates on [0x10683d8b0]
  (keep 0 = straight to body), then body: `ldr x0,[x20,#64]; strb w8,#1,[x20,#9];
  bl 0x2bd29a0` (window-attach). 0x2bd29a0 reads [this+0x268] once-guard, skips on 0.
- Fabrication: app=[inner], [inner+64]=win (zeroed window obj) -> the INIT_WINDOW case
  sets [inner+9]=1 (the strb) then calls the real window-attach path, which benign-
  returns on the zeroed once-guard.
- All 10 byte-pins + the jump-table index-10->0x2bcd78c mapping verified against the
  real binary in the hermetic.

## MEASURED (real libroblox.so, completing ladder + --v2boot-glue-cmd, EXIT 124 stable)
```
[elfjit:glue-cmd] driving process_cmd @ guest 0x102bcd6e4 (app=0x7f25d8008c20 [app]=0x7f25d802e6a0 [inner+64]=win 0x7f25d800c990 cmd=11 INIT_WINDOW; version-gate [0x10683d8b0]=0)
[elfjit:glue-cmd] process_cmd returned Ok(0x7f25d800c990)
[elfjit:glue-cmd] INIT_WINDOW case body marker [inner+9]=1 EXECUTED (engine window-attach path entered headlessly)
```
- 0 SIGSEGV/ABRT. EXIT 124 (reached stable idle).
- Context unchanged: once-guard=0x1, once-slot[0x106a68408]=0x400000b, DM-root
  [0x106a68818]=0x0, MH_* false, service-registry-count=12, app-data-model-count=0x1.
  The complementary SH365 readback still shows the ALooper drain dead (addfd=0) — that
  loop is still not reached — but the DISPATCHER that loop targets IS now entered.

## Interpretation (honest)
- This is the FIRST headless entry into the engine's own app-command dispatcher and the
  FIRST execution of its APP_CMD_INIT_WINDOW case body — the exact window/GL-surface
  SESSION-CTOR precondition was never tested before because the only way to reach it
  (the infinite glue loop) looked unenterable. Driving the bounded process_cmd leaf is
  the correct way to deliver INIT_WINDOW.
- The engine responds: it runs the INIT_WINDOW case, marks the app state byte, and
  calls its real window-attach helper (which benign-returns on the fabricated zeroed
  object's once-guard). It does NOT yet attach a real window (the win obj is a zeroed
  host buffer, not a wired EGL surface) and does NOT construct a DataModel — the Route-B
  live-DM structural gate is UNCHANGED. This is the window-precondition *entry*, not its
  completion.
- Next forward (same line): instead of a zeroed window obj, hand the engine a REAL wired
  ANativeWindow (the X11 XID 0x200000 via anativewindow_fromsurface, SH112/SH365) in
  [inner+64] so the window-attach helper 0x2bd29a0 takes its real GL-surface path — then
  chain process_cmd to the do-init ladder (the operator's SESSION-CTOR "drive until the
  upstream ctor RUNS" directive). Do NOT re-attach a zeroed obj expecting the DM to move.

## Do-not-re-tread
- Do NOT re-attempt entering the infinite ALooper glue LOOP (0x102bcd5d0) — it never
  returns; the bounded process_cmd leaf is the correct entry (SH39b/SH364/365).
- Do NOT treat this marker as a live DM — DM-root stays 0 (SH184/185 four-stacked closure
  still binds the session ctor).

## Verify
- `cargo test --workspace` green (cargo test -p arm64jit sh366_glue ... ok; workspace
  EXIT 0, 430 passed/0 failed).
- Real-binary run: runs/capture_sh366_glue_cmd.sh -> confirm=YES (enter + marker + 0 crash).
- Files: arm64jit/src/jit.rs (+drive_glue_process_cmd, +sh366 hermetic); elfjit.rs
  (opt-in rung at ladder top + condensed SH-prose comments, held <1MB);
  runs/capture_sh366_glue_cmd.sh. Commit: local dev only (operator pushes).