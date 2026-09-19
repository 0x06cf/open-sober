# Frontier SH362 — the do-init MAIN dispatch body (0x258b5d8) NEVER EXECUTES headlessly: SH361 read only the dispatch *target pointer*, SH362 proves the body is never entered — every run faults at the SH285 persistence wall before control reaches it (3/3)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace EXIT 0, 0 fails; arm64jit lib 424->425 with the new
hermetic). One new READ-ONLY observation guard `routeb_startapp_dispatch_body_guard`
(jit.rs, opt-in `JIT_ROUTEB_DISPATCH_BODY_TRACE=1`, default-inert, fires once per
run, ZERO guest mutation) + one hermetic sh362 + a capture probe. elfjit.rs
unchanged (product path identical to HEAD SH361).

## Why this cycle
STATUS.md's named next-forward candidate #1: "trace whether the measured dispatch
target 0x10258b5d8 (StartAppWithParams+0x494) body can be advanced past the SH285
persistence wall." SH361 (previous cycle) measured that the do-init MAIN-branch
`br x1` @0x2206e24 reads the DM/app-shell dispatch target vt[+48] = 0x10258b5d8 —
i.e. it PROVED the dispatch *pointer*. It did NOT prove the body executes: SH361
dumps the stored pointer value; it never observes the body's own block-entry.

## The decision point (disasm-verified on real libroblox.so)
The dispatch target fn at 0x258b5d8 (guest 0x10258b5d8 =
nativeAppBridgeV2StartAppWithParams+0x494; a real function, `sub sp,#0x170`
prologue + `mov x19,x0` @0x258b5f0) immediately branches on a NEW flag byte:
`adrp x8,6a64000; ldrb w8,[x8,#3488]` @0x258b604 reads **flag [0x106a64da0]**, then
`cbz w8,0x258b640` @0x258b608:
- nonzero -> path A @0x258b60c: bl nativePreloadFlagOverrides (0x2dae640) -> blr
  vt[+144] -> bl 0x2366694 -> bl nativePreloadFlagOverrides -> blr vt[+296];
- zero   -> path B @0x258b640: bl 0x2367270 -> blr vt[+144] -> bl 0x2366694 ->
  bl 0x2367270 -> blr vt[+296].
Both converge @0x258b670 -> bl 0x23c19e0 then the stack-canary check -> `ret`.
So the body is a two-branch init dispatcher; WHICH branch the ladder takes is the
unmeasured question the operator's "trace the body" directive names.

## MEASURED (real libroblox.so, full app-start ladder, SH344-style env, 3/3)
```
[routeb-doinit-dyn] SH361 DM-ctor trace @0x2206db8: container=...
  [container+32]=... (non-NULL) -> reach DM-ctor dispatch:
  [obj]vt=0x10635dde8 vt[+48]=0x10258b5d8 (br x1 @0x2206e24). ...
```
**`[routeb-dispatch-body] SH362 body trace` fires 0/3.** Since the SH362 guard is
wired into the same per-block-entry hook (jit_run_inner) and fires whenever pc ==
0x10258b5d8 is ENTERED as a block, 0 fires = the body fn is NEVER entered
headlessly (a reachability result, not a value one). Same guard-hook pattern as
SH361 (which DOES fire at 0x2206db8), so the wiring is exercised — the miss is
the body, not the hook.
- Every run: EXIT 134, SIGSEGV guestpc=0x101db1b08 (SH285 persistence-lane live-
  object wall), DM-root [0x106a68818]=0, MH_* false.
- The SH285 wall sits BETWEEN the do-init dispatch (0x2206db8, reached) and the
  dispatch body entry (0x258b5d8, never reached). Control diverges into the LSM/
  persistence lane after the dispatch and faults; it never returns to execute the
  StartAppWithParams+0x494 body.

## Conclusion / refinement
STATUS.md next-forward #1 ("can 0x10258b5d8 advance past SH285?") is answered:
NO, and the reason is now pinned tighter than "dies in the region" — the run
faults at SH285 BEFORE the 0x258b5d8 body is ever entered. SH361 closed the
measured-vs-assumed gap on the dispatch *target*; SH362 closes the body-*execution*
gap. The StartAppWithParams+0x494 body (a two-branch preload/direct init) remains
entirely unreachable headlessly, gated behind the SH285 persistence wall. This does
NOT manufacture a DataModel and does NOT change the Route-B live-DM structural gate
(DM-root 0, MH_* false); SH174 capture-latch stays the single forward observer.

## Honest
- SH362 is a measured negative (0/3 reach) + a read-only observation guard. It does
  not advance the session; it re-pins where the session drive actually dies.
- Do NOT re-drive LSM sub-call skips to "reach" 0x258b5d8 (SH358/SH349/SH350 closed
  that whack-a-mole as unbounded); do NOT re-attack the [0x106a64da0] flag with the
  ladder (the body never runs, so the flag is irrelevant until SH285 is crossed).

## Files / verify
- crates/arm64jit/src/jit.rs: +routeb_startapp_dispatch_body_guard (read-only,
  opt-in, once-per-run) + sh362 hermetic (arm64jit lib 424->425).
- runs/capture_sh362_dispatch_body.sh (probe; live captures gitignored).
- Workspace green (cargo test -p arm64jit sh362 ok; full suite baseline green).
- Commit: local `dev` only (operator pushes).