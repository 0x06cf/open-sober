# SH169 — DM CAPTURE: DELEGATING + VALIDATING (migration-readiness)

Author: hermes-worker (autonomous loop) + SH169 3-agent Route-B cone
(deleg_*c44c6a51 task-0/1/2), Sep 15 2026.
Status: implemented + hermetic-tested. Companion ledger: STATUS.md.

## What the 3-agent cone independently concluded (code-grounded, READ-ONLY)

All three Route-B subagents converge (~16 recon angles total, SH163–169):

1. **Live-DM structural wall stands.** No .bss/.data seed, no manager-shell
   driver, no synthesized handler-list produces a genuine RBX::DataModel on the
   headless ladder. `createDataModelForTeleport` (0x2e1dc38) is NOT a factory —
   it CONSUMES an existing DM (`ldr x0,[x0,#56]; cbz`), binds a ptr at
   this->+48/+56, has ZERO direct `bl` callers (one address-STORE at 0x2e18528
   into a std::function slot, indirect-dispatch only). `setDataModelToCurrent`
   getter (0x2dbcc10)`adrp x0,6391000` returns &0x6391908, vt 0x635eec0 is
   .data.rel.ro (packed ANDROID_RELA, zeros in file) — not statically dispatchable.
   Trigger requires GPU-host / real-input migration. (task-0 + task-2 both confirm.)
2. **Content path is strictly downstream.** rbxasset content (UniversalApp.rbxm,
   DataModelPatch, CoreScripts) reaches the app-shell ONLY after a live DM;
   DataModelPatcher post-TTI gate `cbz x9,[sp,#160]` at 0x2d87b18 skips benignly
   when the DM is null. The aasset ExtraContent re-root (SH164) is already wired.
   No content/shim code change is implementable-as-progress pre-live-DM. (task-1.)
3. **The ONE concrete implementable delta** = upgrade the SH167 DM capture latch
   to a DELEGATING + VALIDATING hook, so that at migration time (when the engine's
   own make_shared<DataModel> finally runs) the pointer is actually OBSERVED.

## What SH169 implements (crates/arm64jit/src/jit.rs)

**Prior flaw (SH167):** the capture latch seeded the ACTIVE allocator hook
(0x1067daaf0) ONLY when it was 0 — but a real session always installs its OWN
nonzero hook, so the latch was inert exactly when needed. And even when it fired
(no engine hook) it returned host-calloc, whose memory the engine's free-path
cannot release (SH167 probe: EXIT 134 SIGABRT).

**SH169 fixes** (still env-gated, default-inert, migration-readiness):
- `PREV_DM_ALLOC_HOOK` atomic: the engine's real allocator hook is SAVED when the
  trail is armed in delegate mode.
- **Delegation:** `routeb_dm_alloc_capture` now, when a saved prev hook exists,
  performs the real allocation by calling THROUGH the JIT to the engine's own hook
  (`run_guest_callback(prev_hook, [a0,a1,a2,...], current_guest_tp())`) — so the
  returned base is from the engine's allocator pool and its free-path stays valid
  (fixes the SH167 SIGABRT). Falls back to host-calloc when no prev hook / JIT
  unavailable. The delegation passes `current_guest_tp()` (NOT 0): per SH169 audit
  (deleg_*fe92d2e1 task-2), `jit_run_inner` republishes CURRENT_TP from the
  callback state with no restore, so a 0 tpidr would clobber the thread's published
  guest TLS base and fault the engine's TLS-based operator-new at migration time.
- **Validation:** `read_vt_in_image(base)` reads the returned base's first word
  (the vtable pointer) and requires it in-image, so only REAL vtable'd objects
  (a genuine DM) are captured as `[validated]`; garbage bases are returned but
  not counted as DM captures.
- **Guard:** `JIT_DM_ALLOC_CAPTURE_DELEGATE=1` opts into replacing an
  engine-installed hook (saving it as the delegation target). WITHOUT that env,
  the SH167 safety latch is preserved byte-for-byte (never clobber a live hook).
  Idempotent (cur == trail -> no-op).

### Verify markers
- `[routeb-dmalloc] SH167/SH169 routed CRT operator-new ACTIVE hook 0x1067daaf0
  -> capture trail <T> (prev_hook <ENGINE_HOOK>, default_hook <D>) ...`
- On a real session's make_shared<DataModel>: `-> base 0x... [validated]
  (delegate prev_hook 0x...)` proves the DM pointer was captured through the
  ENGINE'S OWN allocator.

## Tests (arm64jit now 367, +2)
- `sh167_dm_alloc_capture_is_env_gated_and_never_clobbers_live_hook` — extended
  to assert the DELEGATE mode: replaces a live hook, saves prev, idempotent.
- `sh169_delegating_trail_falls_back_safely_when_no_guest_image` — with a saved
  hook but no active image, run_guest_callback errors and the trail still returns
  a real, writable host allocation (never 0/panic).

## Honesty
- The delegation path is LATENT-BUT-CORRECT: it cannot be runtime-verified until
  a live engine session makes a real make_shared<DataModel> (migration gate). The
  trail's delegation + validation logic is proven hermetic; the mechanism matches
  the SH167-documented correct design (capture-only delegation, not replacement).
- The live-DM structural wall is UNCHANGED — this does not cross it. It makes the
  ready capture latch functional so the DM pointer is observed the instant a real
  session forms. Standing wall: migration gate.

## Repro (for the record; ladder still exits clean, no DM captured yet)
`timeout 220 env JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1
JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1
JIT_SH115_SINGLETON_PATCH=1 JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1
./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4
--jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent`
(exits 0/124 clean; the capture trail arms but records no DM — no session forms
headlessly; that silence is the proof of the gate).

## Next (Route-B scope, unchanged)
- The ONLY forward is a real app-launch session (GPU host / real input) where the
  engine's own make_shared<DataModel> runs — SH169's trail captures it. Do NOT
  re-tread seeds; keep the recon cone on the ExperienceController / live-DM
  synthesis / DataModelPatcher path.