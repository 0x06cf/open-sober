# Frontier SH413 — host input bridge (the SEP-18 BUILD-THE-RUNTIME "input" axis)

Date: 2026-09-21/22, hermes-worker, single-agent (cone suppressed). recon-v3 immediate-priority
deliverables re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1 = 24 real
task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; JSON len-clamp
present). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 465/0 incl. the new 6 ainput
hermetics). Production code only in the new off-hook module `crates/arm64jit/src/ainput.rs` (+
`pub mod ainput` in lib.rs); jit.rs/elfjit.rs untouched (at/near the 1MiB hook, byte-unchanged).

## Why this cycle

The operator's roadmap for the SEP-18 BUILD-THE-RUNTIME directive is: "session boot, then screens,
then audio, then input." Session boot (ordered substrate, SH400), screens (content surface G3, SH412)
and audio (fake-AAudio bridge, SH132) are all landed. Input was the ONE runtime axis with ZERO
guest-facing wiring: the `input-wrapper` crate (mouse/keyboard -> Android MotionEvent/KeyEvent
translation, GRAPHICS_RECOMMENDATION §6) was a workspace member + arm64jit Cargo dep but was **never
called from anywhere** (confirmed: no `use input_wrapper`/`input::` in any arm64jit/src file) — dead,
orphaned code. And on the guest side the real client reads input via GameActivity, which fires the
JNI exports `Java_com_roblox_engine_jni_NativeInputInterface_nativePassInput` / `nativePassKeyEvent` /
`nativePassMouseMove` — so there was no host path to deliver a translated event into the running
session. This is exactly the SH132 pattern (a latent-but-correct host capability, env-gated,
hermetic-ABI + real-image-pinned), applied to the input direction.

## What landed

New module `crates/arm64jit/src/ainput.rs` (env gate `JIT_AINPUT_BRIDGE=1`, default off):

1. **Guest-native ABI pins** (real libroblox.so, file offset == low vaddr; verified byte-exact):
   - `nativePassInput` @ 0x2bbba88 — prologue `sub sp,sp,#0x50` (0xd10143ff); ABI
     `(env=x0, this=x1, int action=x2, int pointerId=x3, float x=s0, float y=s1)`; internally
     `sxtw x1,action; bl 0x2e4e68c` (+0x50) into the real input consumer leaf (0xd10503ff prologue).
   - `nativePassMouseMove` @ 0x2bbbcf4 (s0..s3 floats); `nativePassKeyEvent` @ 0x2baebdc.
   - `INPUT_CONSUMER_LEAF` 0x2e4e68c (RobloxInput::processInputEvents entry).
2. **Marshalling**: `marshal_touch(TouchAction, env, thiz, tpidr, boot_sp) -> CpuState` places the
   integer args in x2/x3 and packs float x/y into the SIMD v-lanes exactly as the JIT's `fmov
   s8,s1; fmov s9,s0` reads them (s0 = v[0] low32, s1 = v[1] low32 = st.v[2] low32). This is the
   host-side ABI-bridge step that mirrors SH132's slot-identity marshalling.
3. **`fire_touch`**: drives a translated touch into the guest `nativePassInput` via a fresh CpuState
   + `jit_run` — the concrete delivery path a real host calls once a live session owns a screen.
   Guarded: NOP (Ok(0)) when the bridge env is unset or there is no live image. Latent.
4. **input-wrapper reuse**: `translate_pointer`/`translate_motion` -> TouchAction, so the orphaned
   crate's translation is finally bound to a real guest delivery target (a real consumer, not dead).
5. **6 hermetic tests**: inert-without-env (+ NOP fire with no image), argument placement + float-lane
   bit-exactness, action constants match input-wrapper, translate helpers, real-image pin (prologue
   words + the internal `bl consumer` @ +0x50 byte-exact), high-lane cleanliness.

## MEASURED (hermetic, real libroblox.so present)

`cargo test -p arm64jit --lib ainput` = 6 passed / 0 failed. The real-image pin asserts the actual
instruction words at the pinned addresses in the 109 MB APK .so: nativePassInput prologue
`0xd10143ff`, nativePassMouseMove prologue `0xd10143ff`, consumer leaf prologue `0xd10503ff`,
internal `bl 0x2e4e68c` = `0x940a4aed` @ nativePassInput+0x50 — all byte-exact. This grounds the input
delivery ABI on verified bytes (the SH413 "never/unwired surface closed" marker).

## Honest

Latent-but-correct, exactly like SH132: input only matters once a live session owns a screen (the
standing Route-B live-DM gate — DM-root [0x106a68818]=0 unchanged, MH_GAME_LOADED false). The bridge
is the correct host-side capability on the input direction; it stays inert behind the env gate until
a session advances to a constructed UI. Default product path byte-identical (env not set).
Recon-v3 deliverables unchanged-green; workspace green; no re-treads (this is the input axis, not a
DM/LSM/glass cone).

## Files

- `crates/arm64jit/src/ainput.rs` (new, off-hook, ~13 KB).
- `crates/arm64jit/src/lib.rs`: `pub mod ainput;` (one line).
- `runs/capture_ainput_bridge.sh`: hermetic + real-image verification script.

## Next

Keep covering the runtime axes the operator names. Input capability now exists (host -> guest
delivery via nativePassInput). The standing Route-B live-DM gate remains the SESSION-CTOR wall that
would make input (and audio, and the G3 content surface) actually fire; as that advances, the natural
integration point is wiring the input-wrapper X11 pump to `fire_touch` so a real host loop delivers
events to a constructed login/home screen. Do-not-re-tread unchanged (no LSM skips, no map
manufacture, no setDataModelToCurrent, no single-object DM seeds).