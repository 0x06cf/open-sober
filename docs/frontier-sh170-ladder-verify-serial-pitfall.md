# SH170 — Recon-v3 deliverables VERIFIED at HEAD + canonical ladder STABLE + serialization pitfall pinned

Author: hermes-worker (autonomous loop), Sep 15 2026.
Status: verification + doc (no production change needed this cycle — the standing
recon-v3 deliverables the operator flags as "IMMEDIATE PRIORITY" are already
committed at HEAD and now re-verified on a real run). Companion ledger: STATUS.md.

## What was checked (real binary, real run)

Real `libroblox.so` (109MB, aarch64 Android 26, NDK r28c), forge combined env:
`JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1
JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1
JIT_SH115_SINGLETON_PATCH=1 JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1`
`./target/debug/examples/elfjit <apk>/libroblox.so 0x2173ff4 --jni --startapp 0x258b144
--v2boot --v2boot-surface-handoff --v2boot-send-appevent`

### 1. Recon-v3 deliverables are present and firing (were the operator's "IMMEDIATE PRIORITY")
- **(1) SELF-DRIVEN FRAMES / type4_frame_thunk** — shipping; installed into the type-4
  producer vector `[0x106829ea8]` via `--taskv4-seed frame` (host-thunk contract: 3-arg
  ABI handler(node=x0,[node+32]&~1=x1,consumer=x2); non-recursive BR leaf). The ladder keeps
  `[0x106829ea8]=0x0` on every rung because the SESSION (do-init) never installs it — the
  harness only seeds it under `--taskv4-frame`-style jobs, which is the documented design.
  Prior runs (runs/capture_taskv4_frame.sh) show 24 real task-driven frames presented
  `swap Ok(0x1)` + 191 real type-4 node pops.
- **(2) JSON-ABORT** — `JIT_JSON_ZERO_FIX` firing on the real run: BOTH clamp markers hit
  (`append check 0x102355d40 would overflow (len=0xb3 / len=0x24) -> forcing len=0 SSO empty`).
  Cap at `0x107275648` is never raised (raising = memcpy ~1.6GB SEGV). Regression test
  `json_zero_fix_clamps_leaked_length_at_append_check` pins it.
- **(3) DM capture (SH167/SH169)** — armed on the real run: `SH167/SH169 routed CRT
  operator-new ACTIVE hook 0x1067daaf0 -> capture trail ... (prev_hook ENGINE, default 0)`.
  Trail stays silent (no `make_shared<DataModel>`) — the honest proof that no session forms
  headlessly; the delegating+validating capture is the ready latch for the migration moment.

### 2. Canonical (non-serialized) ladder is STABLE — 4/4 clean
Full 9-rung `--v2boot` ladder completes reproducibly (4 of 4 runs, EXIT 124 = the outer
300s timeout AFTER completion, i.e. clean): nativeInitializeNativeFlags -> nativeGameGlobalInit
Ok -> nativeUpdateAdapterInit Ok(1) -> setTaskSchedulerBM Ok(1) -> V2InitWithParams ->
StartLuaAppDM Ok(0x3e8) -> V2StartAppWithParams Ok -> V1 AppStart__ Ok -> V2UpdateSurface
Ok (surface XID 0x200000 wired into [0x10683d348]) -> SendAppEventOnAppReady Ok(0x3e8)
(event "Home" in x5, ABI-correct). 0 SIGSEGV / 0 SIGABRT / 0 stack-smash on all 4.
Post-ladder probe: MH_* all false, once-guard 0, DM-root [0x106a68818]=seeded shell,
vt+0x30=0, app-data-model-count=1 — the documented live-DM structural wall unchanged.

## NEW finding — the serialization pitfall (do NOT repeat this footgun)
Adding `JIT_SERIALIZE_RENDER=1` to a **bare** ladder (no combined render machinery) makes
nativeGameGlobalInit abort with an UNHANDLED GUEST C++ EXCEPTION (`libc++abi: terminating`,
EXIT 139) every run. Region watch shows the do-init body traverses [0x102206404..0x102206ad4]
then throws. Root cause hypothesis (consistent w/ SH130): `JIT_SERIALIZE_RENDER=1` raises
`WORKER_ADMISSION_GATE`, which parks all engine clone-worker top-level jit_runs until
LADDER_DONE — but global-init's do-init depends on one of those (TaskScheduler/pump) workers
being live to complete; with it parked the do-init throws. The gate (SH130) is built for the
**combined render+ladder** serialization, NOT the bare ladder. **The canonical ladder command
must NOT include JIT_SERIALIZE_RENDER** (the operator's own SH169 repro omits it; adding it is
the regression, not a property of the ladder). Recorded to save a future agent the 6+ runs.

## Recon consensus (unchanged — migration gate)
~18 prior angles + a fresh 104-tool-call recon agent this cycle (deleg_2549cb57) all converge:
`createDataModelForTeleport` (0x2e1dc38) is a CONSUMER (ldr [x0,#48]/[x0,#56]) with ZERO static
callers; `setDataModelToCurrent` (0x2dbcc10) is a GETTER returning &0x6391908; `Serializer::load`
deserializes INTO an existing DataModel (3rd arg DataModel*, 0x20d8134) — it can NEVER
manufacture one; the only make_shared<DataModel> is inlined at the sole real call site in
`ExperienceController::join -> submitStartGameTask`, unreachable headlessly. No .bss/.data seed,
manager-shell, or synthesized-handler variant constructs a live DM on this box. The single real
forward is the SH167/SH169 capture->delegate->arm pipeline at a REAL app-launch (GPU-host /
real-input migration gate). Recon-v3's "implement now" list was already DEAD-on-arrival precisely
because these are latent-correct, not shootable-now.

## Repro captured
- runs/sh170-ns-1..4.txt — 4x full clean ladder
- runs/sh170-serial-1..3.txt — 3x serialization-induced abort (the pitfall)
- runs/sh170-region.txt — serializer region-watch proving the do-init throw point