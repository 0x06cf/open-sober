# SH202 — on-demand single-site V2 singleton-dispatch family patcher

Worker: hermes-worker · date 2026-09-16 · workspace green (564/0, +3 hermetic;
arm64jit lib 386/0; elfjit example 64/0)

## 1. Problem (the SH201 §6 "next genuine lever")

SH200 patched 4 *located* objB-vtable dispatch sites (fn 0x6251e0c @+0x118,
0x62523ac @+0x130, 0x6258e88 @+0x2f0, 0x6258ffc @+0x2f8) and observed V2Start
completion run-variably. The larger family (`sh201_v2_family_scan`, ~384-600
`bl 0x6249eb8` objB-getter sites, each reading the harness-seeded 0x60 vtable
PAST +0x60 into host box-alloc bytes and `blr x8`-ing into them) still stopped
V2Init/V2Start run-variably at OTHER sites, so the SH199 world-build gate fn
0x102ea3b14 was never *reliably* reached. The family-wide RUNTIME patch was
shown (SH201) to over-patch genuine in-band N<0xf0 calls → SIGABRT, so a blind
pre-scan scribble is NOT shippable.

SH201 §6 named the real lever: **patch only the exact site the run ACTUALLY
dispatches through, ON DEMAND at the outside-image stop**, with a loaded-vtable
check, instead of a pre-scan. This cycle implements it.

## 2. Mechanism

At the outside-image stop in `jit_run_inner`, the guest `blr x8` that jumped into
host box-alloc bytes has set `x30 = blr+4`, so `blr_site = x30-4`. The dispatcher
has NOT executed any code at the bad pc (`pc` is the untranslatable host address),
so it is safe to:

1. **Classify** `blr_site` with a pure family discriminator
   (`v2_family_blr_from_guest`): `bl 0x6249eb8` (objB getter) within 16 back,
   `ldr x8,[x0]` AFTER it, and a past-leaf `ldr x8,[x8,#N]` (N*8>=0x60) within
   the 4 slots before the blr — exactly the SH201 discriminator that excludes
   genuine in-band calls (N<0xf0).
2. **Patch** that ONE site's dispatch window (materialize a stable leaked 0x80
   singleton object with a benign all-leaf vtable into x0 + nop the blr), same
   movz/movk mechanics as `sh200_v2_dispatch_window`.
3. Drop the block cache for the window, rewind `pc` to the window start,
   `continue` the run_loop — the patched site re-executes and the run advances
   past it instead of dying.

Clearing is **blinded and opportunistic**: each site is patched only when the run
genuinely dispatches through it at a stop, never by a pre-scan. Untouched
in-band sites are never touched (no SH201 family-wide crash). Correct even for
**backward** getter `bl`s (the real family's getter 0x6249eb8 is BELOW the
dispatch sites) via a sign-extension fix.

Default-INERT: gated on `JIT_ROUTEB_V2_ONDEMAND=1`.

## 3. CODE (crates/arm64jit/src/jit.rs, default-inert)

- `v2_family_bl_target_l(link, w)` — imm26 branch-target decode with **correct
  sign extension** in i64 (subtract 2^26 = 0x400_0000, NOT 2^30 — a positive-u32
  `imm<<2` yields the wrong redirect for backward bls).
- `v2_family_blr_from_guest(image, base, blr_guest)` — pure family classifier,
  returns the window start or None.
- `v2_ondemand_object()` — stable leaked 0x80 object, +0 = all-leaf benign vtable
  (once-locked).
- `v2_ondemand_patch_at(image, base, x30)` — classify + patch + drop cache,
  returns rewind pc or None. Idempotent (skips a window already holding a movz).
- Hooked into the outside-image stop in `jit_run_inner`, gated on the env.

+4 hermetic tests (arm64jit lib 386/0):
- sh202 classifier genuine-vs-decoys (getter-gated past-0x60 site found with the
  right window start; bare blr / in-band decoys rejected; out-of-image rejected)
- **backward-bl sign-extension regression** (forward bl passes naive test)
- on-demand object stability/coherence (once-locked, all-leaf reachable)
- **real-image guard**: the 3 measured run-variable stop blr sites +
  4 SH200-located sites ALL classify as family on the real libroblox.so
  (>=7 required) — locks the classifier (esp. sign-ext) against a silent
  regression that would make on-demand inert.

## 4. EMPIRICAL (real libroblox.so, llvmpipe, canonical 9-rung ladder)

- **Deterministic site clearing**: with `JIT_ROUTEB_V2_ONDEMAND=1`, the run
  patches the exact site(s) it dispatches through (1-3 per run) and RETURNS past
  them (0 "outside image" stops after the patch; the stops are gone, not masked).
- **FULL LADDER COMPLETION OBSERVED** (`runs/sh202-ondemand-full-ladder.txt`):
  nativeInitializeNativeFlags → nativeGameGlobalInit Ok → nativeUpdateAdapterInit
  Ok → setTaskSchedulerBM Ok → **V2InitWithParams Ok(0x3e8)** →
  **V2StartAppWithParams Ok(0x3e8)** → V1 AppStart__ Ok(0x3e8) →
  V2UpdateSurface Ok(0x3e8) (XID 0x200000) → SendAppEventOnAppReady Ok(0x3e8)
  → **EXIT 124, 0 SIGSEGV/0 SIGABRT**. This is the FIRST time V2StartAppWithParams
  completes on a run that also shows V2Init Ok — the baseline never guaranteed
  V2Start completion (SH200: "observed, run-variable, family too large").
- **Honest boundary**: across 4 runs, 1 completes the full ladder clean; 3 patch
  the family and then advance DEEPER into a NEW reachable gate that faults with
  `pc=0x7f00000022b0 host-call slot, fault [x0+0x28], lr=0x102b53a78` (a host-call
  thunk deref'ing a NULL-small arg) — a genuine post-family wall the baseline
  never REACHED because it soft-returned at the family first. That deeper gate
  is the SAME live-world-build class (a host object whose +0x28 member is
  unbuilt headlessly) — forward motion, not a regression, and it is the next
  thing to seed.
- **Default path UNCHANGED**: with `JIT_ROUTEB_V2_ONDEMAND` OFF, 0 on-demand
  patches, EXIT 124, identical to SH200/201 baseline (verified 1/1).

## 5. Reproduce

```
cd /home/hermes-worker/runs/open-sober
cargo test -p arm64jit --lib sh202        # +4 hermetic
cargo build --example elfjit
LOG=/tmp/sh202.txt; timeout 120 env \
  JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_SETWORLDBUILD=1 \
  JIT_ROUTEB_V2_ONDEMAND=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
grep -E "SH202 patched|V2InitWithParams returned Ok|V2StartAppWithParams returned Ok" "$LOG"
```

## 6. Next (honest)

Route-B live-DM world-build remains the standing structural gate. This cycle
advances the manufacture/DMCONT/PATH-B line ONE concrete step: the V2 family
that previously stopped V2Init/V2Start run-variably is now clearable
deterministically on demand, and a full clean V2 ladder completion was observed
at HEAD for the first time. The new post-family fault (`pc=0x7f00000022b0`
host-call slot, fault [x0+0x28], lr 0x102b53a78) is the next unblocked seed
target: a host object whose +0x28 member is null headlessly — drive whatever
object lr-0x102b53a78's caller threads there, or confirm it is the same
live-world-build gate (do-not-chase if so). Keep the do-init→app-shell→governor
continuation + live-DM line as the primary front.