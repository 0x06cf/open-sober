# SH203 — post-family fault = CONFIRMED same live-world-build gate + app-data-model register trace (SH197 NEXT answered)

Worker: hermes-worker · date 2026-09-16 · workspace green (564/0). Two open
recon questions from SH202/§6 and SH197 closed with real-run evidence on the
real libroblox.so under the canonical seeded ladder. No production code change
(the findings warrant no new seed — see verdicts).

## 1. SH202 §6 open question — the post-family fault is the SAME live-world-build gate

SH202 reported that with `JIT_ROUTEB_V2_ONDEMAND=1`, 3/4 runs clear the V2
singleton-dispatch family and advance DEEPER into a NEW fault:
`SIGSEGV guestpc=0x7f00000022b0 fault=[x0+0x28] lr=0x102b53a78`. Its §6 said the
next step is to "drive whatever object lr's caller threads there, or confirm it
is the same live-world-build gate (do-not-chase if so)."

### Deterministic characterization (12 runs, 4 faulting)
Reproduced on the canonical ladder (`--v2boot --v2boot-surface-handoff
--v2boot-send-appevent`, env: JIT_DRIVE_LIFECYCLE + ROUTEB_DM_SEED + HASHFIX +
JSON_ZERO_FIX + SETFIX + SH115_SINGLETON_PATCH + SETWORLDBUILD + V2_ONDEMAND):
- **8/12 runs complete the FULL V2 ladder clean** (nativeInitializeNativeFlags →
  GlobalInit → UpdateAdapterInit → SetTaskSchedulerBM → V2InitWithParams →
  StartLuaAppDM → V2StartAppWithParams → V1 AppStart__ → V2UpdateSurface →
  SendAppEventOnAppReady → EXIT 124, 0 SIGSEGV/0 SIGABRT). SH202's "1/4 clean"
  is now measured at **8/12** — the V2 l/singleton-family stop is effectively
  cleared on the majority floor.
- **4/12 runs fault at the IDENTICAL site, deterministically**:
  - `SIGSEGV tid=… fault=0x28 guestpc=0x7f00000022b0`
  - host-call slot `0x22b0/8 = 1110` = the **host `pthread_mutex_lock` bridge**
  - `lr = 0x102b53a78` = guest return from `bl 62d63b0 <pthread_mutex_lock@plt>`
    at file 0x2b53a74
  - `x0 = 0x28` across ALL faulting runs (tid varies, x0 identical) ⇒ the guest
    reaches a **NULL `this` object with a `pthread_mutex_t` at +0x28**
    (`obj==0`, locking `&obj+0x28` → deref of absolute 0x28).

### Where the NULL-`this` comes from (disassembly)
The enclosing fragment file 0x2b53a64 is a **vtable/computed-dispatch target —
ZERO direct `bl` callers** (only reachable via `blr`/registry dispatch, unlike a
leaf with a fixed call-site). It sits in the `JNICallProtocol_receiveCall` /
app-bridge call-protocol region (symbol `…_JNICallProtocol_receiveCall__@+0x558`
and neighbours). Reachable only AFTER the V2 family is cleared (the baseline soft-
returns before reaching it). The object it locks (`this+0x28` mutex) is an
app-bridge receiver-registry object that a real **Java CookieManager / app-bridge
session** builds — NULL headlessly.

### VERDICT (do-not-chase)
Per SH202 §6's own branch this is **the same live-world-build / Java-session
gate class**: a vtable-dispatched app-bridge receiver with a NULL `this`, 0
direct callers (not a fixed-address seed like SH116's nativeInit lock-owner
helper, which WAS a `.bss` global). It is **forward-motion evidence, not a new
frontier**: the run now advances PAST the V2 singleton family into app-bridge
territory the baseline never reached. Do NOT static-seed it; it fires only when
a real session populates the app-bridge receiver registry (same migration
pattern as type-4 producer / DM-capture latch / resolver map).

## 2. SH197 open NEXT — app-data-model register trace (answered)

SH197 asked: "trace whether 0x102207b50's app-data-model register (0x2208354)
now yields a self-built DM controller vs the intern." Answered on the 1/12
completing run with the three-region watch
(`JIT_REGION_WATCH=0x102207b50-0x102212d00,0x1023eff4c-0x1023f0100,0x102e9fa84-0x102ea4000`):

- The app-shell ctor body executes through its FULL construction body: entry
  (0x102207b50) → 0x102207b88..0x102207f58 (the task-scheduler / ctor cluster) →
  **app-data-model register 0x102208354** → its operator-new(0x18) driver
  **0x1022086c0** → vector-init store **0x1022086e8** into
  `[0x106dca000+0xed8]` → deeper 0x2086d8..0x208710.
- The do-init/base continuation reaches **0x1023f0008/0x1023f0020/0x1023f00f8**
  (nativeAppBridgeStartLuaAppDM body past the pipe `bl 0x2baeeec`), and the
  governor tail runs its full sequence through the known **canary-ret terminal
  0x102ea30dc** (137 region hits total, EXIT 124).
- **BUT the SH155 DM-root markers stay unchanged**: `once-guard=0x1`,
  `DM-root=[0x106a68818]=SH156 seed`, `liveDM-image=false`, `once-slot=0x400000b`
  (the strcmp string-intern — NOT a DM controller), `app-data-model-count
  [0x106dca000+0xe88]=0x1` (stable baseline: the JSON-serialization path writes
  that counter once every run, per elfjit.rs:7336-7338 — it advanced 0→1 in the
  FIRST run and stays 1, it is NOT per-run DM progress).

### VERDICT
0x2208354's "app-data-model" name is a registration/vector-entry builder, not a
DataModel factory: it operator-news a 0x18 entry and stores it into the
app-registry (`[0x106dca000+0xed8]`), independent of the do-init once-slot
(`[0x106a68408]=0x400000b` intern). Route-B's live-DM is **unchanged** — do-init
completes but yields the string-intern, not a DM (SH196 holds). The completion
of the construction BODY confirms the ladder now finishes clean on 8/12, which
is the SH202 milestone — not new Route-B construction.

## 3. Default path unregressed
`V2_ONDEMAND` OFF ⇒ 0 on-demand patches, EXIT 124, 0 SIGSEGV/0 SIGABRT —
identical to the SH200/201/202 baseline (verified 1/1 this cycle). recon-v3
render plane re-verified green (capture_taskv4_frame.sh: **24 real task-driven
frames**, `present #19..#23 swap Ok(0x1)`, 195 node pops, no json abort,
0 SIGSEGV). Workspace `cargo test --workspace` green (564/0).

## 4. Next (honest, standing Route-B wall unchanged)
Route-B live-DM = structural gate (unchanged; ~30 recon angles + this cycle).
The post-family fault is CONFIRMED same-gate → do-not-chase (SH202 §6 answered).
The V2 ladder now completes clean on 8/12 (was 1/4). The one genuine open
thread remains the **R1 synthetic-CoreScript content path** (one synthetic
`ScreenGui` module into filesdir for `rbxasset://scripts/CoreScripts`), which
still needs a live DM to parent the GuiObject — so it is REACHABILITY-blocked,
not a headless seed. Do-not-re-tread: app-data-model register 0x2208354 is a
registry-entry builder (NOT a DM factory); once-slot 0x400000b is the intern,
not a controller.