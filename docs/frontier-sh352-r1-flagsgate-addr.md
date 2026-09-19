# Frontier SH352 — fix the R1 content-path flags-loaded gate address (was dead on a read-only cell) + measure the completing skip-appstart ladder end-to-end

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Production fix + 1 new
hermetic real-image test (`sh352_flags_loaded_latch_addr_corrected`, elfjit example).
Workspace green (elfjit examples 159/0; arm64jit lib 418/0; cargo test --workspace exit 0).
elfjit.rs held under the 1MB pre-commit hook (1,048,523 B < 1,048,576).

## What was wrong (real bug, measured on the real binary)
`stage_r1_core_scripts` (SH351) arms the Route-B loader gates, but the PRIMARY gate —
the flags-loaded latch the whole TaskScheduler/do-init ladder hangs on — was written to
**0x10672739d4** (file vaddr 0x672739d4, a read-only .rodata/.data cell). The TRUE latch is
guest **0x1072739d4** (file vaddr 0x72739d4, verified: gate read at 0x224fa20
`ldrb w8,[x8,#2516]` after `adrp x8,7273000` @0x224fa18; same cell as jit.rs:1201 GATE_FLAGS
and every other latch reader in the tree). Consequences, all measured on the live binary:

- Before (SH351 as committed): the R1 run logged `[r1] gate @0x10672739d4 0x0->0x1` is
  ABSENT; the loader-gates line printed `0x10672739d4:unmapped` — `page_writable_rw` correctly
  refused the read-only cell, so the flags-loaded gate **silently never armed** (all other 4
  gates armed). The content-path synthesis could not fire because its primary gate stayed 0.
- After (SH352 fix): the live ladder logs `[r1] gate @0x1072739d4 0x0->0x1` and the loader-gates
  line is `0->1, 0->1, 80->0, 80->0, 0->0` — all 5 gates now write, none dropped.

## What else this cycle measured (the completing skip-appstart ladder, 3/3 deterministic EXIT 124)
With `--v2boot-skip-appstart` + the SH269 GOVFLAG seed + SH307 preload-valuecell (the full
SEP-17 session-ctor seed set), the ladder COMPLETES end-to-end headlessly for the first time:

- Upload/SendAppEventOnAppReady: previously terminated at the governor NULL-app-DM-controller
  wall 0x102ea0b9c (SH306 baseline, GOVFLAG-off) then the SH270 preload wall 0x102bb803c;
  with GOVFLAG + PRELOAD_VALUECELL it **returns Ok(0x107273d50)** cleanly (0 SIGSEGV on the normal
  RunAppEvent path).
- SendAppEventOnGameLoaded: **returns Ok(0x107273d50)** cleanly.
- R1 content STAGES on the completing ladder: `STAGED AppShell.lua / CoreScripts.lua` under
  `/…/files/scripts/CoreScripts/` (both inferred candidates, 518 B each).
- SH155/SH315 post-ladder probes run and read back real state:
  `once-guard[0x6a68410]=0x0 once-slot[0x106a68408]=0x400000b DM-root[0x106a68818]=0x0
  mark_b(liveDM)=false service-registry-count[0x106fe2f08]=12 app-data-model-count[0x106dca000+0xe88]=0x1`.
  The once-slot 0x400000b is the "Execute" service handle (SH316) — the do-init once-lambda
  completed and stored the matched controller handle, but NOT a live DataModel.

## Honest (do-not-over-claim)
- This is a real correctness fix (the content-path gate now arms), plus the strongest-yet
  measured session drive: the ladder completes, both app-events return, R1 content stages, and the
  once-slot/registry probes read real values (count=12, once-slot=Execute handle).
- It does NOT manufacture a live DataModel. DM-root [0x106a68818] stays 0; MH_* all stay false.
  once-slot=0x400000b is the matched "Execute" service handle, not a DM-controller (SH316, and
  SH313: the ctor fast-path needs the "App" service registered — the task-scheduler family is a
  different tier). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
  forward hook.

## Verify
- `cargo test -p arm64jit --example elfjit sh352` = 1 passed (real-image pins: gate-read adrp
  0x90028128 @0x102_24fa18, ldrb 0x39675108 @0x102_24fa20; asserts the corrected 0x1072739d4 and the
  stale 0x10672739d4 resolve to DIFFERENT image pages).
- `cargo test --workspace` EXIT 0; elfjit examples 159/0; arm64jit lib 418/0.
- Repro live run (completing ladder, R1 arms all 5 gates): `runs/capture_sh352_r1_completing_ladder.sh`.