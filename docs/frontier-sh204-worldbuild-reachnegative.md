# SH204 — world-build gate fn 0x102ea3b14 = measured REACHABILITY NEGATIVE on the completing V2 path (SETWORLDBUILD seed stays latent, AppBridge-line closure)

Worker: hermes-worker · date 2026-09-16 · workspace green (564/0, unregressed).

Closes the genuinely-new open question posed by the SH202/203 state change:
SH199's `routeb_worldbuild_gate_seed` was measured LATENT at SH199 because the
V2Init rung always stopped at the run-variable host-pointer flake (SH198)
BEFORE reaching its gate block 0x102368100. SH202/203 made the **full V2 ladder
complete clean ~8/12** — a state the SH199 measurement never ran under. So the
open question became: **on the now-completing V2 path, does the run cross the
SH199 world-build gate 0x102368100 and execute the deep world-build body fn
0x102ea3b14?**

## Answer: NO — measured reachability negative on BOTH clean and fault paths

Method: region-watch calibrated to prove fire-ability, then the negative.

### Calibration (region-watch proves it fires on executing blocks)
`JIT_REGION_WATCH=0x106251e94-0x106251eb0,0x102365c54-0x102365c70` on a clean
run fired on BOTH known-executing blocks (0x106251e94 = SH200 V2-dispatch patch
site, 0x102365c54 = V2InitWithParams driver entry). The watch mechanism is live;
a zero-hit on a watched region means the region truly did not execute.

### The negative (3/3 clean runs)
`JIT_REGION_WATCH=0x102368100-0x102368140,0x102ea3b14-0x102ea3c50` over the
canonical completing ladder (JIT_DRIVE_LIFECYCLE + DM_SEED + HASHFIX +
JSON_ZERO_FIX + SETFIX + SH115_SINGLETON_PATCH + SETWORLDBUILD + V2_ONDEMAND):
**3/3 clean (EXIT 124, ladder completes through SendAppEventOnAppReady Ok) all
show gate_block_hits=0 and deepbody_hits=0.** The world-build gate block
0x102368100 is NOT a block on the completing path — V2Init completes via the
SH200 singleton-materialization soft-return and never executes the natural
V2Init body that would fall through to 0x102368100. The SETWORLDBUILD seed
[0x106a70568] therefore never fires with any effect on a completing run.

### Fault paths (3/3) also never reach it
Runs 2/3 fault at the exact pre-existing SH203 post-family gate (guestpc
0x7f00000022b0 = pthread_mutex_lock bridge, fault=0x28, same NULL-`this`+0x28
receiver) and run 5 at the SH198/SH55 host-pointer flake (0x102b9e950) — all
known classes, none reaching 0x102368100. Confirms SH199's original note ("the
V2Init rung stops FIRST at the host-pointer flake BEFORE reaching the gate
block") is now refined: the completing path does not reach it either.

## Deep body 0x102ea3b14 → AppBridge lifecycle closure (already closed by SH174)

Fresh disassembly of fn 0x102ea3b14 (nativePreloadFlagOverrides+0x2d6b54):
```
ldr x0,[x0,#688]; cbnz x0,skip      ; gate on [this+688]
mov x0,#0x18; bl operator-new (0x1d96768)
stp x9,x8,[sp,#8]                   ; build 0x18 wrapper from [x20+40]/[x20+48]
bl ctor 0x2eacccc                    ; thin copy-ctor
...
bl nativeAppBridgeAppStart 0x2365960 ; the terminal action
ldr x0,[x20]; <stack-canary check>; ret
```
Its one real action is `bl nativeAppBridgeAppStart 0x2365960` — the AppBridge
app-start lifecycle closure that SH174 already measured (via V1 AppStart__
0x102338510 driving the AppStart governor 0x2338ef4) as **"an app-bridge
lifecycle/telemetry/platform-init closure that is NEVER a DM factory (ZERO
GuiObjects/no session node). Reached the headless ceiling; MOVE ON, do not seed
the V1/AppBridge line further."** So even a hypothetical host-drive of
0x102ea3b14 (SH182/SH187 ctor-driver style) would route its terminal call into
already-closed AppBridge territory — NOT Route-B DM construction. Do not build it.

## VERDICT
- SH199's world-build seed is now measured, not assumed: **inert-by-reachability
  on both the completing path AND the residual fault paths** under the SH202/203
  state. It remains correct-and-harmless (idempotent, preserves a live value)
  and fire-if-a-real-session-ever-executes 0x102368100 — but it is NOT an
  unblock. Do-not-re-tread: no seed change gets the deep body 0x102ea3b14 to run
  headlessly, because the gate block is never a translated-entry block in any
  reachable headless path.
- The deep body's terminal (`nativeAppBridgeAppStart`) is the already-closed
  AppBridge lifecycle line (SH174) — host-driving it is not Route-B progress.
- Standing wall UNCHANGED: Route-B live-DM world-build = structural gate
  (~30+ recon angles + this measurement). Wrote this doc to a fresh file so the
  latent-SETWORLDBUILD expectation is not re-chased by a future session that
  sees "V2 completes now" and assumes the world-build gate became reachable.

## Traps (recorded for future agents)
- **Region-watch zero-hit IS meaningful** — but ONLY after calibration on a
  known-executing block in the same run (the watch fires on every translated
  block entry; SH197's multi-range fix is live). Did not skip this.
- World-build gate block 0x102368100 lives in the NATURAL V2Init body, never the
  SH200-patched soft-return — the SH200 materialization is WHY the completing
  path bypasses it. The two fixes are mutually exclusive by design.
- Do NOT host-drive 0x102ea3b14 to "advance DMCONT" — its terminal
  `nativeAppBridgeAppStart` is the SH174-AppBridge closed line, not a DM/Route-B
  body.