# SH196 — do-init `__call_once` self-latches headlessly: the engine's own
# GlobalInit do-init lambda RUNS to completion on this JIT (once-guard 0->1),
# but its body is a strcmp string-intern GetOrCreate (0x2173b3c) returning a
# small status/hash (0x400000b) — NOT a live DataModel.

## Date / context
Sep 16, 2026 (hermes-worker). Route-B line after SH195 (scene-attach ABI dead-end).
Per the operator's "MIGRATION IS NOT A STOPPING-POINT" directive: the do-init /
live-DM construction is the unexhausted lever, so this cycle drove StartLuaAppDM
into the real do-init completion and OBSERVED the outcome live-in-process rather
than via post-run re-construction.

## Method
Ran the full --v2boot ladder (SH191 recipe) with `timeout 130`. The StartLuaAppDM
rung no longer returns (its `jit_run` parks forever in the engine main-loop idle
nanosleep poll, lr=0x10284d134 — the same park the ROUTE-B_UNBLOCK doc describes
for gameGlobalInit). Because the rung never returns, the existing post-rung SH155
probe never fires. So this cycle added a **default-inert in-run observable**
`[elfjit:dmcells]` (gated `JIT_THREADS=1 + JIT_DMCELLS=1`) to the JIT_THREADS
sampler thread: polled once per tick, page-guarded reads of the four do-init cells.
Live result (reproducible, EXIT 124 = timeout-after-completion, 0 crash):

```
[elfjit:dmcells] once-guard[0x106a68410]=0x1 once-slot[0x106a68408]=0x400000b
                  DM-root[0x106a68818]=0x7f4a30030680 (SH156 seed) flags-latch=1
[elfjit:stats]    compiles 1656 -> 5576 then FLAT (world-build expanded then settled)
```

## Findings
1. **The do-init's own `__call_once` now RUNS headlessly and self-latches.**
   once-guard[0x106a68410].bit0 = 1 (was 0 pre-rung; GATE-FIX leaves it clear so
   `bl 0x284ce54 != 0x284cf5c` at file 0x2206d18/0x2206d7c runs the lambda, which
   self-latches via std::call_once's stlrb on completion). This is the first
   empirical confirmation that the once-lambda COMPLETES on this JIT — the "never
   completes" premise (recon-routeB-globaltinit-unblock step 2) is now measured
   as completing-but-not-producing-a-DM. **Independently confirmed without our
   seeds applied:** an early-tick capture (sampler fired before the v2boot guard
   seeds ran) showed once-guard=0x1, DM-root=0x0, holder=0x106358d40 (raw
   unseeded) — the engine's own lambda ran and latched from the normal ladder,
   not from our seed chain.
2. **It produces a string/app-registry intern, NOT a DataModel.** The lambda body
   (file 0x2206d24..0x2206d74) computes TWO string addresses and calls
   `bl 0x2173b3c`, the strcmp-based RTApp app-registry GetOrCreate (prologue
   `bl strcmp`, hash-bucket walk; SH178 identified it). **Fresh precise string
   decode (the adrp+add+sub chain cancels to the straight rodata addrs):
   x0=file 0x2d34ab = "App", x1=file 0x3d1ba8 = "Execute".** So the once-lambda
   interns the RTApp registry handle for "App"+"Execute" (the Roblox RTApp
   app-registry entry), and its small return (measured 0x400000b) is stored at
   [0x6a68000+#1032] = [0x106a68408] (`str x0,[x23,#1032]` at file 0x2206d74).
   The once-slot is therefore the RTApp app-registry intern value — NOT a DM
   controller. No live DM is constructed by the do-init.**
3. **The live DM stays behind the SH156 seed.** DM-root[0x106a68818] still holds our
   fabricated 0x10-byte dispatch object (object[0]=vt 0x10635cce0), not an
   engine-built DM. The world-build DID expand (compiles 1656 -> 5576, ~3900 new
   JIT blocks through real engine init including the PlayerGui/ScreenGui
   construction registers SH189/190d/e), then settled flat at idle — so the route
   genuinely advances real construction code, but ends in the engine main-loop
   idle park with no live DM owned by the once-slot or DM-root.

## Consequence / honest standing
The do-init completion path is now OBSERVED, not assumed: it runs the once-lambda
to bit0-latch and expands real construction, but the lambda's payload is a
string-intern, and no live DM materializes in either the once-slot or DM-root at
idle. Combined with SH195 (scene-attach x1-bounce), SH194 (resolver map
world-build-gated), SH193 (registry lazy-ctor), and the ~30 prior live-DM recon
angles, the evidence is now: **headless do-init completion produces a strcmp
string-intern + real-but-DM-less world-build expansion + healthy idle park — not
a self-constructed session, because the actual DM factory (make_shared at the
live ExperienceController/app-launch path) is not reached by any do-init cell
write.** This is the closest the operator's "dynamic do-init trace" could get: the
trace is now visibly captured in-run and confirms completion-without-DM.

## Do-not-re-tread (additions)
- Do NOT write the once-slot [0x106a68408] to a fabricated "DM controller" to try
  to force the match-path `ldr x0,[x19,#32]` forward: x19 would then be a real
  object but the do-init's own completion already returns a THIRD value (the
  intern) into the once-slot, and the match path consumes [x19+32] (the DM-root
  +0x20 field, i.e. the SH156 seed harness already controls). Forcing the once-slot
  to a controller says nothing about a live DM and adds a fourth writer to an
  engine-owned cell.
- Do NOT re-attempt "the once-lambda builds the DM" — measured to be a strcmp
  intern.

## What IS shipped
- `[elfjit:dmcells]` observable (elfjit.rs, default-inert under JIT_THREADS=1 +
  JIT_DMCELLS=1): one line per sampler tick reading once-guard / once-slot /
  DM-root / flags-latch with page-guarded reads (never crashes an unmapped boot).
  Gives future runs the do-init completion state in-run without external /proc
  access. Workspace green (558/0); the observable fires only under its env.