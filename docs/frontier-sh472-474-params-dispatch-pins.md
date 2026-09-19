# SH472 + SH473 + SH474 — close the session-content fn-table dispatch coverage (json-abort + params + display-fields)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified GREEN
at fresh HEAD FIRST (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames
`present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash, EXIT 124 = stable idle).
Do-init/Route-B baseline re-probed on the real binary too (capture_sh415: substrate 14/16,
once-guard bit0=1, DM-root [0x106a68818]=0x0, MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all
true, AppBridgeV2 vt resolved, 0 crash) — Route-B live-DM gate UNCHANGED. Workspace green
(arm64jit lib 680/0 at every commit; full cargo test --workspace green). Production code
UNCHANGED in SH472-474 — all test-only jni.rs additions (off the 1MiB hooks; jit.rs/elfjit.rs/
session.rs untouched; runtime byte-identical).

## The defect class closed
A systematic cross-check of the recon-named session-content getter/field surface against what
the fn-table dispatch tests actually EXERCISE found a whole class of *production arms with no
dispatch pin*: a getter/field returns a real recon value in `auto_value_string_getter` /
`jni_call_boolean_method` / `jni_call_long_method` / `jni_get_float_field`, but NO test routes
that name through the real JNIEnv `Call*Method`/`Get*Field` slot — so a regression that
collapsed the value to 0/NULL would pass the entire suite. These were the latent holes:

- **SH472** (getLanguage + getDisplayResolution): both have `assert_eq!(len, …)` arms in the
  big params fn-table test, but were NOT in the loop's name array — the arms were dead. Worse,
  when I added them to the array, `getDisplayResolution` FAILED: it's "1280x720" = **8 chars**
  but the pin said 7. The wrong expected length had been latent for the same reason (never
  iterated). The production getter is correct; only the test's hardcoded expectation was wrong
  and unexercised.
- **SH473** (isCpu64Bit/isLowRamDevice/isMouseDevice/isPotato/isTablet/isVrDevice booleans +
  getDeviceTotalMemoryMB=8192 long): production arms with ZERO CallBooleanMethod/CallLongMethod
  assertion. A regressed isCpu64Bit=0 or device TB flipped to 0 would have gone unnoticed.
- **SH474** (scaledDensity=1.0, xdpi/ydpi=96.0 float fields): production arms with only
  `density` asserted in the display field test — the other three DisplayMetrics floats had no
  GetFloatField pin.

## The closure
Each of those names is now asserted through the REAL JNIEnv dispatch the guest uses:
- getLanguage + getDisplayResolution added to the fn-table loop array (CallObjectMethod ->
  GetStringUTFLength), expected length fixed to the true 8.
- All six boolean getters + getDeviceTotalMemoryMB asserted via CallBooleanMethod / CallLongMethod.
- scaledDensity/xdpi/ydpi asserted via the GetFloatField HostJniF32 s0-return bridge.

Every new assertion runs on the same synthetic-ENV host thunk path (`host_call_at`, readable
method-id handles) as the existing coverage — no image, no env, parallel-safe, deterministic,
same build_jni() harness the whole params suite uses.

## Honest status
- recon-v3 deliverable (type4 self-driven frames + json-abort) re-verified GREEN at fresh HEAD —
  the session-content work stays latent-but-correct.
- NOT a DM / NOT a live-DM step (Route-B gate UNCHANGED: DM-root 0, LIVE DM=false). BUILD-THE-
  RUNTIME test-contract completion: the session-content surface the engine reads is now fully
  pinned through the real dispatch, so the json-writer can't leak and the params/display values
  can't silently collapse. No re-treads (the arms existed but were unexercised — distinct from
  any prior pin).
- Files: crates/arm64jit/src/jni.rs. Commits 9680499, ce26ab6, ef69765.