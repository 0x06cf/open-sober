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
- recon-v3 frame plane RE-VERIFIED green at this HEAD (capture_taskv4_frame.sh: 24 frames, 195 node
  pops, 0 json abort, 0 crash) — the jit.rs/elfjit.rs gate-addr change caused no render regression.
- Repro live run (completing ladder, R1 arms all 5 gates): `runs/capture_sh352_r1_completing_ladder.sh`.

## Addendum — honest region-watch negative (same cycle)
JIT_REGION_WATCH across the post-do-init/governor/ScriptContext/app-shell bands on the completing
ladder (full seed set incl. GOVFLAG+PRELOAD_VALUECELL, with append/pack skips): the governor
0x102e9fa80 and ScriptContext loader 0x101f1d8ac bands show **0 hits**; the run terminates run-variable
in the SH341/SH350 persistence pool-pop family (0x101d9a030 / 0x101d9a528) only when the do-init pipe's
deep app-start continuation is re-driven (i.e. a partial flag set without skip-appstart's benign
loop-completion). This matches SH340/SH308/SH342: the app-shell ctor runs but the session half
(governor -> Lua) stays gated on a live DM. No Route-B advance beyond the SH352-measured completing
ladder (app-events return, R1 stages, gates arm, once-slot "Execute" handle). DM-root 0, MH_* false.

## Addendum 2 — decisive REG_LIVE answer: the completing ladder NEVER registers "App" (SH332/333 closed with measured evidence)
JIT_ROUTEB_REG_LIVE=1 (SH334) on the completing ladder snapshots the service registry at the DM-ctor
name->service lookup (fn 0x2168798) on EVERY count transition. Measured (real so, EXIT 124 clean):
- count 0..11 walks the task-scheduler family (Thread/Spawn/Yield/Close BG+FG, Sleep, Sched, UNKNOWN0);
- the 12th entry is "Execute"; the ctor fast-path (cbnz x0 @0x61e3124) MATCHES it -> once-slot
  [0x106a68408]=0x400000b ("Execute" service handle, no live DM);
- the tier-2 controller-name cell [0x106fe4f78] reads "Runtime0" AT EVERY count (invariant, SH317/318)
  — "App" is NEVER registered on the completing headless ladder.
This closes SH332/333's open "does app-start register 'App'?" question with the strongest possible
headless evidence: even with the do-init once-lambda COMPLETING (once-slot populated) and the full
12-entry registry available, the DM-ctor fast-path returns the "Execute" handle, not a live
DataModel-controller, because "App" is a live-session-ctor-only registration (SH313/316 propped). The
Route-B lever is unchanged and now sharper: the gate is a live-session "App" service registration,
not reachable by any headless seed. DM-root 0, MH_* false.