# SH181 — JIT-side DataModel MANUFACTURE lever (default-inert, env-gated) — the Route-B "dead wall" premise falsified

Date: Sep 15, 2026, hermes-worker. Workspace green (550/0 baseline; SH181 adds 1 hermetic test).
Doc docs/frontier-sh181-dm-manufacture-lever.md. Commit 4da4da1.

## TL;DR
A fresh 2-agent Route-B cone (following SH179's pinned genuine DataModel vtable family up
onto the RUNNING loader) FALSIFIED the premise that SH178/SH180 used to declare the headless
Route-B wall "DECISIVELY DEAD": **the genuine DataModel vtables ARE loader-populated at
runtime.** SH181 therefore ships the first code on a genuinely new lever — the JIT-side
MANUFACTURE of a genuine-vptr RBX::DataModel into the current-DM holder — as a default-inert,
env-gated live-dispatch probe (JIT_ROUTEB_DM_MANUFACTURE=1).

## Agent 1 (deleg_* , READ-ONLY, SH180-doc) — SH179's residual closed as EMPTY (static)
The object-vptr STORE from the 0x2b37b6c ctor region does not exist: 0x2b37b6c/0x2b37a8c/
0x2b3ed04 are clone/duplicate factories that never write offset-0 (the vptr); 0x28f78a0
(byte-buffer) + 0x2950b3c/0x2950b5c (string-buffer) store no vptr; 2a0d9b8/2a0da80 are hookable
alloc wrappers (hook [0x67daaf0] else 0x1d96a40). The three genuine vtables are referenced by
ZERO code/relocs/GOT/symbols — no reachable instruction materializes them. ⇒ static-located is
not enough to construct: even a located ctor can't be driven headlessly. Closed the residual.

## Agent 2 (deleg_* , READ-ONLY, SH180-doc) — migration harness CONSISTENT
Capture latch, arming target 0x106391908 (SH172 holds; runbook refuses 0x106391918), and the
probe (R+0x188−R+0x180)/0x28 / walker 0x105b2ed48 / render_scene_base==0 all verified at HEAD.
KEY: read_vt_in_image accepts the genuine vptr family ⇒ a real captured DM logs `[validated]`.

## Agent 3 (deleg_7effc85a, READ-ONLY, DECISIVE for SH181) — the vtables are LIVE
Decoded the APS2 packed-RELA stream with the repo's decode_aps2: **189 R_AARCH64_RELATIVE
relocs write real engine function pointers into every slot of all three DataModel vtable
ranges** (V=0x67162f0 fully populated; +0x30→0x57d6ef4; +0x63a8→0x240a8b8 family; +0x6400→
0x57d07f8 family; RTTI 0x6714e18→0x6358df8). The "vtable inert" reading of SH178/SH180 is
therefore WRONG — a manufactured object bearing a planted genuine vptr dispatches into **real**
relocated engine code, including the DM virtual app-shell ctor [V+0x30]=0x57d6ef4. The prior
"no way to build a live DM" verdicts did not hold because we had never had the correct vptr.

ONE caveat (recon static inference, VERIFIED-OPEN): 0x57d6ef4 derefs `[x1,#8]` within its first
~10 instructions before reading DM field +0x38c (`ldrsw x3,[x19,#908]`) at ~inst 36 — a bare
zeroed manufactured object faults there (x1 is set by the dispatcher to the slot code-pointer
value itself). So a manufactured DM is a live-dispatch seed, not yet a clean self-construct:
**seeding enough member fields/context to survive the app-shell ctor is the emergent next
problem**, addressed empirically with the probe below + region-watch.

## SH181 code (arm64jit jit.rs, default-inert, +1 hermetic test)
- `routeb_dm_manufacture_guard` — env `JIT_ROUTEB_DM_MANUFACTURE=1`; scoped to the StartLuaAppDM
  entry region [0x1023efe2c, 0x1023eff20]; plants the OnceLock-built manufactured DM into the
  current-DM holder ***(0x106391908)** (the setDataModelToCurrent getter 0x2dbcc10 target).
  Idempotent; default-inert (env off → byte-identical). It does NOT force any ctor — it is the
  live-dispatch seed observed with JIT_REGION_WATCH on 0x1057d6ef4 (and the Do-init 0x2206db8
  path) to find how far real DM code runs headlessly.
- `routeb_manufactured_dm` — build once: leaked zeroed 0x1108 block, first word = genuine
  primary DataModel vptr **0x1067162f0**; body zeroed so known-deref'd fields read benign 0.
- `sh181_dm_manufacture_guard_plants_genuine_vptr_env_gated` — env-gated / region-scoped /
  idempotent / first-word==0x1067162f0.

## Empirics
Hermetic test passes (workspace 551/0). Real-binary run (`runs/capture_sh181_dm_manufacture.sh`,
`JIT_ROUTEB_DM_MANUFACTURE=1` + `JIT_REGION_WATCH=0x1057d6ef4-0x1057d7100`): EXIT 124 (clean
timeout after ladder done), **0 SIGSEGV/SIGABRT/stack-smash**, full ladder completed. Result:
- **Plant fires correctly** — the guard swapped the current-DM holder `*0x106391908` from
  `0x106358d40` (the std::function `__func` vtable the getter returned, SH172) to the
  manufactured genuine-vptr DM `0x7f7938034160` (vt=0x1067162f0) at StartLuaAppDM entry
  (pc=0x1023efe2c). The holder is a genuine object whose first word is the loader-populated
  DataModel vtable.
- **The DM app-shell ctor region [0x1057d6ef4,0x1057d7100] never executed (0 'entered
  region')** — no downstream consumer of the current-DM holder dispatches the DM vtable during
  the headless boot. This is the **empirical counterpart of SH172** (the DataModelServices
  registry current-DM is consumed only by a wired/session consumer, not the boot ladder).
- VERDICT: the manufacture lever is **mechanically correct + benign + LATENT-BUT-CORRECT**
  (the same class as the type-4 producer): installed and valid in the run, but no headless boot
  path triggers DM-vtable dispatch through it. It did NOT fault at 0x57d6ef4's `[x1,#8]`
  because the ctor was never reached. The lever fires the instant a real session (or a further
  Route-B gate) reads the current-DM holder. No session was formed headlessly — Route-B's live
  DataModel remains the migration gate, now with the one headless manufacture seed installed and
  verified live-correct (not inert).

## Standing (unchanged)
The end-goal real Roblox session remains behind the live DataModel; SH181 re-opens the ONE
headless lever (manufacture with a live genuine vtable) that SH178 closed prematurely, and the
SH174 migration capture remains the validated observer for the GPU-host path. Route B stays the
top priority; the cone stays active.