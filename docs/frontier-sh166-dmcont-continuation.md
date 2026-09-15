# SH166 — DMCONT continuation: empirical floor + corrected disassembly of the engine-init chain

Cycle: Sep 15 2026, hermes-worker. Doc only (trace + recon); repro `runs/capture_sh166_dmcont_chain.sh`, log `runs/sh166-dmcont-chain.txt`.

## What was run
DMCONT ladder (JIT_ROUTEB_DMFORCE=1 + JIT_ROUTEB_DMCONT=1 + the SH165-fwd/DM_SEED/SETFIX/SH115 stack)
with JIT_REGION_WATCH covering the whole engine-init chain `0x102bd1a30-0x102bd9058`
(fnB -> 0x102bd8ce8 -> continueAfterFlagsLoaded_ 0x102bd1d68 -> the vt[+0x1f0] dispatch site).

## Empirical result (real libroblox.so, llvmpipe)
- SH164 governor-tail shell fired @0x102e9fcc4 (vt[+0x30]=engine-init fnB 0x102bd1b98).
- Region-watch entered **fnB @0x102bd1b98** then **0x102bd8ce8** (the fnB bl target), once each.
- Region-watch did **NOT** log 0x102bd1d68 (continueAfterFlagsLoaded_) NOR 0x102bd8dac (sub_2bd8dac,
  the vt[+0x1f0] dispatcher).
- Ladder still completes clean: SendAppEventOnAppReady returned Ok(0x3e8), "ladder done", 0 SIGSEGV/SIGABRT, EXIT 124.
- MH_FLAGS_LOADED/MH_APP_READY stay false; DM-root[0x106a68818] holds a zero-vt obj; app-data-model count=1.

## Corrected disassembly facts (new, supersede the stale recon premise)
The prior recon (and the SH165-fwd doc) claimed the pipeline is "latent because the vt[+0xf8] network
feature-flag fetch never completes synchronously, so the +0x1f0 completion callback never fires."
**That premise is WRONG against direct disassembly of 0x102bd8ce8** (guest 0x102bd8ce8, file 0x2bd8ce8):

```
2bd8d14 bl 2174c04            ; getter -> tail-calls 0x624e6c0 -> manager M (via [sp+16])
2bd8d1c ldr x1,[x19,#24]      ; x1 = arg+0x18 C-string
2bd8d24 ldr x8,[x8,#248]      ; vt[+0xf8]
2bd8d2c blr x8                ; <-- dispatch +0xf8 (first vtable call)
2bd8d34 ldr x8,[x8,#264]      ; vt[+0x108]
2bd8d50 blr x8                ; <-- dispatch +0x108 (second vtable call)
2bd8d60 bl 2bd8dac            ; <-- sub_2bd8dac (UNCONDITIONAL)
... in sub_2bd8dac:
2bd8e18 ldr x8,[x0]           ; x0 = manager M
2bd8e20 ldr x8,[x8,#496]      ; vt[+0x1f0]
2bd8e28 blr x8                ; <-- dispatch +0x1f0 = continueAfterFlagsLoaded_ (0x102bd1d68)
```

The three vtable dispatches (`+0xf8`, `+0x108`, `+0x1f0`) happen **unconditionally, serially** — there is
NO branch gating `+0x1f0` on `+0xf8`'s return. Making `+0xf8` "complete synchronously" is therefore NOT
the lever the prior recon assumed.

**Getter chain (0x2174c04) corrected:** on a non-NULL holder it calls the manager's own `vt[+0x30]`
(`blr x8` #248... no: #48) with (x0=M, x1=out+8, x2=0x10006), then **tail-calls** `b 0x624e6c0` (NOT a
ret). 0x624e6c0 reads `[out+8]=M`, dispatches the manager's **vt[+0x720]** (`ldr x8,[x8,#1824]; blr x8`)
— which is 0 on the fabricated M, so it benign-soft-returns — then rets back into 0x102bd8ce8 at
0x102bd8d18. So x20 (the manager) comes back through `[sp+16]` and the `+0xf8` chain proceeds.

## Honest reading
- Whether continueAfterFlagsLoaded_ truly executes under DMCONT is **not decidable from region-watch alone**:
  it only logs fresh run-loop block entries, and `blr`-to-computed-target / internal-return points inside a
  single traced chain can run without firing it. Static analysis says 0x102bd1d68 SHOULD be dispatched
  unconditionally; the probe cannot confirm or refute that it runs.
- This matters little for the goal: the DECISIVE recon (deleg_7e5b7101 task-1) established that
  continueAfterFlagsLoaded_->nativeAppBridgeAppStart(0x2338ef4) is a **logging/flag warmer** — it sets
  base-url + JNIAppLifecycle setActive and produces ZERO GuiObjects/scene nodes. The scene walker
  0x105b2ed48 has no direct bl caller.
- Standing wall unchanged: real self-constructed login/home still requires a live RBX::DataModel + Luau VM +
  ScriptContext + CoreScript/content stack. ~10 recon angles (SH163/164/164c/165/165-fwd + 3-subagent cone
  this cycle) agree NO static/manager-shell seed produces a live DM; the only source direction is
  engine-internal synthesis during a real app-launch session the headless ladder cannot yet form.

## Next (re-batched cone)
Sharper questions for the next 3-wide Route-B cone, armed with these corrected facts:
(a) Given +0xf8/+0x108/+0x1f0 are dispatched unconditionally, why does region-watch never see 0x102bd1d68?
   Pin by instrumenting a RETURN-POINT probe (0x102bd8d18 / 0x102bd8d30) to detect trace-vs-diverge, AND
   check whether the block at 0x102bd1d68 is even translated (cached_block called) in a DMCONT run.
(b) Is there ANY real entry reachable from the fabricated-manager shell that runs a bona fide engine
   boot (nativeAppBridgeAppStart + beyond) rather than the logging warmer — or is the manager-shell line
   of work exhausted? If exhausted, state it plainly.
(c) Re-derive on the ONLY sanctioned forward (deleg_7e5b7101/task-2): what is the minimal harness change
   to let a real engine session form so its IT'S-OWN make_shared allocates a live DM (allocation-capture), and
   is there any replayable real-session bite in the current --v2boot ladder, or is a GPU-host/real-input the
   only route to a session (which would move this to the migration path)?