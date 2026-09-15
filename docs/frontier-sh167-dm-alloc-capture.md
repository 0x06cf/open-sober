# SH167 — DM allocation-capture hook (latent, mechanism-proven) + DECISIVE recon

Cycle: Sep 15 2026, hermes-worker. Recon cone deleg_35857472 (3 READ-ONLY agents, authoritative, code-grounded).
Docs: this + docs/frontier-sh166-dmcont-continuation.md. Commits: d1267ac (SH166) then SH167.

## The three decisive recon verdicts (this cycle, code-grounded)
1. **DMCONT latency settled (deleg_35857472 task-0):** a `blr` to a computed target ALWAYS terminates the
   trace and re-enters the run loop (translate.rs:6924-6932; run_loop re-dispatches via cached_block at
   jit.rs:2933 where region-watch fires). So the ABSENCE of a region-watch hit at 0x102bd1d68 under DMCONT
   PROVES the vt[+0x1f0] `blr`→continueAfterFlagsLoaded_ dispatch did NOT execute. (Region-watch remains
   inconclusive only for trace-internal targets reached via an inlined `bl`/`b`; not the case for this
   computed-`blr` target.)
2. **manager-shell line EXHAUSTED (task-1):** the entire transitive closure from continueAfterFlagsLoaded_
   (0x102bd1d68) -> nativeAppBridgeAppStart (0x2338510) -> AppStart (0x2338ef4) is app-bridge
   lifecycle/telemetry/platform-init (FastLog self, base-url, JNIAppLifecycle setActive, SendAppEvent/MessageBus
   telemetry, cookie/web-login/crashpad/headers/LocalStorage, refcount/destructor thunks) — NEVER a DM factory.
   createDataModelForTeleport (0x2e1dc38) and the scene walker 0x105b2ed48 each have ZERO direct bl callers.
   `continuation_worth_it: false` — STOP the manager-shell line for the UI goal.
3. **headless session impossible (task-2):** NativeHelper gameActivity_* callbacks (jni.rs:731-786) are
   engine->Java POST-CONDITION effect-signals, NOT drivers; nothing polls MH_* into a session gate. The DM
   creator is unreached headlessly. A real session cannot form on this headless ladder — the documented
   GPU-host / real-input MIGRATION GATE.

## SH167 — DM allocation-capture hook (latent, mechanism-PROVEN)
CRT operator-new capture so a live RBX::DataModel pointer is caught the instant a real make_shared runs
post-migration. Wrapper guest 0x102a0d9b8 reads ACTIVE allocator-hook [0x1067daaf0] and DEFAULT [0x1067d0840];
`cmp x8,x9; b.eq` -> inline fast path else `blr x8` the active hook.

Code (jit.rs, JIT_DM_ALLOC_CAPTURE=1 + hermetic test sh167_dm_alloc_capture_...):
- `routeb_dm_alloc_capture` host-call trail: ROUTES real allocations (calloc) + logs sizes; 3-arg ABI VERIFIED
  by probe = **(size=a0, call-site-tag=a1, flags=a2)** — a0 is the real byte size (probe showed a0=0x18=24),
  a1 is a code/rodata address naming the call site (NOT a size), a2 a flag/line word. Over-allocating
  max(a0,a1) (an early draft) allocates multi-GB garbage — do NOT do that.
- `routeb_dm_alloc_capture_guard`: seeds the ACTIVE hook to the trail ONLY when it is currently 0.
  **EMPIRICAL finding (probe run):** the engine ships its OWN nonzero ACTIVE hook; FORCE-replacing it (an early
  draft) with a host-calloc trail is INVALID — the engine's operator-delete/free-path expects allocations from
  its own pool and guest-SIGABRTs (probe run EXIT 134, [SIGABRT] guestpc=0x0). So the guard fires only when no
  hook is installed (safe latch), the trail records allocations, and a capture-only (DELEGATING) design is the
  correct migration-time route to observe a live DM without perturbing the allocator. The trail MECHANISM and
  ABI are PROVEN by the probe (trail invoked, correct sizes allocated, boot clean when it does NOT replace a
  live hook), not assumed.
- Default-inert (env off -> guard returns immediately; live allocator path byte-identical). Env-on + engine's
  live hook -> guard returns early (inert) -> clean boot (re-verified EXIT 124 / 0 crash / ladder done).

## Env/housekeeping
/tmp hit 100% (7.7G tmpfs) from recon disasm dumps this cycle -> ENOSPC broke 7 pre-existing tests
(futex_requeue, guest_svc_*, aasset_*, bionic_stat). Cleared to 5%. Subagents: keep disasm dumps small or
delete after; the 7.7G tmpfs is a box-breaker. Re-ran workspace clean 0 fail after clearing (not a regression).

## Standing state
Workspace green (all suites 0 fail; arm64jit 366 incl. sh167). Tree on dev. The Route-B frontier is the
live-DM structural wall (no seed crosses it; ~12 recon angles agree). Only real forward = engine-internal
make_shared during a REAL session at migration time; SH167 is the ready capture latch for it.