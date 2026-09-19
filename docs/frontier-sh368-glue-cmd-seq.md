# Frontier SH368 — bounded app-command SEQUENCE drive on the guarded SH366 entry (cmds {6,8,11}), session-state readback; full dispatcher jump-table + window-attach contract pinned

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). New opt-in rung
`--v2boot-glue-cmd-seq` -> `drive_glue_process_cmd_seq` (jit.rs) + real-image hermetic
`sh368_glue_cmd_seq_jump_table_and_safe_cases_pinned` (arm64jit lib 431) + capture
`runs/capture_sh368_glue_seq.sh`. elfjit.rs product path unchanged (rung opt-in, before the
crash-prone lifecycle natives). Workspace green (cargo test --workspace EXIT 0; arm64jit 431/0).

## Why
SH366 entered the engine's REAL app-command dispatcher `process_cmd` (0x102bcd6e4) headlessly
for the FIRST time and delivered a single APP_CMD (cmd 11 = INIT_WINDOW), marker `[inner+9]=1`.
The operator's SESSION-CTOR directive is to "drive the engine's REAL Activity-session init state
machine" — a real Activity consumes a QUEUE of APP_CMD values, not one. SH368 extends the
bounded, confirmed-green SH366 entry to a command SEQUENCE over a SHARED fabricated app/inner/win
so command state accumulates the way a real command queue does, and reads back the session
observables after EACH command. SH367 measured that arming the window-attach once-guard
fabricates a surface obj and faults; SH368 does NOT arm it — the once-guard stays OFF, so this
is a safe observability advance on the SESSION-CTOR entry, not a return to the SH367 re-arm.

## The sequence (verified-safe subset, version-gate [0x10683d8b0]=0)
process_cmd (0x2bcd6e4, `sub w8,w1,#1; cmp w8,#0x13` = cmd in [1..20]) dispatches via a 16-bit
jump table at 0x69408a (base 0x2bcd730). Both non-INIT_WINDOW commands in the sequence are
disasm-verified cycle-safe at version-gate 0:
- cmd 6  (case 0x2bcd7e4) : reads [0x683d8b0] version gate, `b.lo 0x2bcdbf0` (byte0<6 -> epilogue).
- cmd 8  (case 0x2bcd864) : same gate read, `b.lo 0x2bcd8a8` -> writes only glue state bytes
  (strb wzr,[x20,#8] / [0x683d8d0] / [x20,#0x20]) then returns.
- cmd 11 (case 0x2bcd78c) : INIT_WINDOW — `ldr x0,[x20,#0x40]` ([inner+64]=win), `strb w8,#1,
  [x20,#9]` (marker), `bl 0x2bd29a0` (window-attach; benign with once-guard OFF). SH366-proven.

## The hermetic (real libroblox.so)
`sh368` pins: the dispatcher prologue + jump-table address words (adrp/add x9,0x69408a; adr
x10,0x2bcd730); the 16-bit rel table entries for cmd1/6/8/11 -> their exact case targets; the
sanity that ALL 20 rel offsets land in-dispatcher [0x2bcd730,0x2bcde00); the cmd6/cmd8
version-gate short-circuit bodies; the dispatcher epilogue; and the window-attach contract
(once-guard +0x268, [inner+64]=win, bl 0x2bd29a0). Verified words were dumped from the binary
(not guessed) — this gives the next SESSION-CTOR iteration pinned reference addresses for the
real command queue and the window object it must construct.

## MEASURED (real libroblox.so, runs/capture_sh368_glue_seq.sh — confirm:1 on attempt 1)
```
[elfjit:glue-seq] SH368 command sequence {6,8,11} over shared app=0x7fb7bc000fc0 win=0x7fb7bc0013d0; pre AppBridgeV2=0x0 surfaceXID=0x200000; version-gate [0x10683d8b0]=0; once-guard OFF
[elfjit:glue-seq] cmd 6: process_cmd returned Ok(0x0)   [marker=0 onceGuard=0 AppBridgeV2=0 surfaceXID=0x200000]
[elfjit:glue-seq] cmd 8: process_cmd returned Ok(0x0)   [marker=0 onceGuard=0 AppBridgeV2=0 surfaceXID=0x200000]
[elfjit:glue-seq] cmd 11: process_cmd returned Ok(0x7fb7bc0013d0)   [marker=1 onceGuard=0 AppBridgeV2=0 surfaceXID=0x200000]
[elfjit:glue-seq] SH368 done: AppBridgeV2 selftransition 0x0->0; surface XID 0x200000->2097152
```
EXIT 124, 0 SIGSEGV/ABRT, cmd-11 INIT_WINDOW marker `[inner+9]=1` fires, once-guard stays OFF —
the SH366/367 confirmed-green clean entry PRESERVED with the sequence armed (attempt 1). The
sequence drives all three commands cleanly through the engine's own dispatcher. The session
observables confirm the operator's SESSION-CTOR reading precisely: the real app-command
dispatcher alone does NOT self-transition AppBridgeV2 ([0x106a705e8] 0x0->0) or the surface XID
([0x10683d348] stays 0x200000, the already-wired X11 XID); those move only when a live
session/do-init builds the DM world (DM-root [0x106a68818]=0, MH_* false unchanged).

## Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false expected). The sequence
drive is observability on the confirmed-green SESSION-CTOR entry — it does not arm the (SH367
faulting) window-attach once-guard, does not create a real EGL surface, and does not cross the
live-DM structural gate. It advances the "drive the engine's real command queue" half of the
SESSION-CTOR directive with pinned, re-verifiable addresses and a bounded live readback.

## Do-not-re-tread
- Do NOT arm the window-attach once-guard with a FABRICATED surface obj (SH367 measured fault;
  the real surface is a genuine SESSION-CTOR object, not a value seed).
- Do NOT re-enter the infinite ALooper glue LOOP (bounded process_cmd is the entry; SH365 dead-drain).
- Do NOT re-attack the persistence lane with LSM sub-call skips (SH349/350/358 unbounded).

## Verify
- `cargo test -p arm64jit --lib sh368` = 1 passed (real-book pins).
- `cargo build --workspace` EXIT 0; `cargo test --workspace` green (arm64jit lib 431).
- Real-binary confirmed-green sequence: runs/capture_sh368_glue_seq.sh -> confirm:1 (cmd11 marker
  + SH368 done + EXIT 124). Repro live log: gitignored runs/sh368-glue-seq.txt.

## Files
arm64jit/src/jit.rs (+drive_glue_process_cmd_seq, +sh368 hermetic); examples/elfjit.rs
(+--v2boot-glue-cmd-seq rung); runs/capture_sh368_glue_seq.sh; docs/frontier-sh368-glue-cmd-seq.md;
HANDOFF.md; STATUS.md. Commit: local dev only (operator pushes).