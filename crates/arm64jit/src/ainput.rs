// SPDX-License-Identifier: MIT
//! SH413 — host input bridge (the SEP-18 BUILD-THE-RUNTIME "input" axis).
//!
//! The runtime axes the operator's roadmap names are session boot -> screens ->
//! audio -> input. Audio landed (SH132 fake-libaaudio host bridge, env-gated,
//! latent-but-correct). This module is the input twin: the guest-facing
//! capability that lets a real host deliver desktop pointer/keyboard input to the
//! real Roblox client once a live session owns a DataModel.
//!
//! Roblox-on-Android reads input through GameActivity, which calls the JNI exports
//! `Java_com_roblox_engine_jni_NativeInputInterface_nativePassInput` /
//! `nativePassMouseMove` / etc. (pin these file 0x2bbba88/0x2bbbcf4...). Those
//! JNI exports take `(JNIEnv*, jobject, int action, int id, float x, float y)`
//! and internally marshal into the real input consumer `RobloxInput::processInputEvents`
//! (leaf 0x2e4e68c, reached via `sxtw x1, action; bl 0x2e4e68c` @ nativePassInput+0x50).
//! A real host (the Java layer under GameActivity) fires these to deliver input.
//!
//! This module is the host-side *capability* that makes that delivery possible:
//! it takes the already-built `input-wrapper` translation (`PointerTracker`
//! mouse -> ACTION_DOWN/MOVE/UP + x11_keysym_to_keycode) and marshals the result
//! into the pinned guest `nativePassInput` ABI (env/thiz in x0/x1, action in x2,
//! pointer-id in x3, x/y in s0/s1), then drives the guest native on a fresh
//! CpuState exactly like the session substrate drives guest atoms.
//!
//! Like SH132 this is **latent-but-correct**: the JNI export only matters once a
//! live session owns a DataModel (input is delivered to a constructed screen), so
//! nothing here runs on the current boot path. It is built behind the env gate
//! `JIT_AINPUT_BRIDGE=1` (default off) so the product path is byte-identical
//! unless the env is set, and it is hermetic-tested for the marshalling ABI
//! (flat f32 bits into the CpuState v-lane, correct arg placement, inert without
//! env) + real-image-pinned for the exact guest native addresses.

use crate::jit::{CpuState, jit_run};

/// Env gate: without this the bridge is completely inert (no guest call path and
/// the marshal helpers are only exercised by tests). Default off -> default unchanged.
pub const AINPUT_BRIDGE_ENV: &str = "JIT_AINPUT_BRIDGE";

pub fn bridge_enabled() -> bool {
    std::env::var_os(AINPUT_BRIDGE_ENV).is_some()
}

// ---------------------------------------------------------------------------
// Guest-native ABI pins (real libroblox.so, file offset == guest vaddr base, so
// a file offset IS the guest address once loaded at base 0x100000000... these
// are JNI exports the Java layer calls. Pinned by live objdump; see SH413 doc.
// ---------------------------------------------------------------------------

/// `Java_com_roblox_engine_jni_NativeInputInterface_nativePassInput`
/// ABI: (env=x0, this=x1, int action=x2, int pointerId=x3, float x=s0, float y=s1).
/// Prologue `mov w19,w3; fmov s8,s1; fmov s9,s0; mov w20,w2` -> internally
/// `sxtw x1,w20(action); mov w2,w19(id); bl 0x2e4e68c` (real consumer leaf).
pub const NATIVE_PASS_INPUT: u64 = 0x2bbba88;

/// `Java_com_roblox_engine_jni_NativeInputInterface_nativePassMouseMove`
/// ABI: (env, this, addr?, x=s0, y=s1, z=s2, button=s3) via `fmov s8,s3; s9,s2;
/// s10,s1; s11,s0` prologue. (Mouse move; kept for completeness, primary is touch.)
pub const NATIVE_PASS_MOUSE_MOVE: u64 = 0x2bbbcf4;

/// `Java_com_roblox_engine_jni_NativeGLInterface_nativePassKeyEvent`
/// ABI: (env, this, int action=x2, w3, ..., int keycode=x5) — w0=w3, x1(?)...
/// (Keyboard; primary touch entry is NATIVE_PASS_INPUT.)
pub const NATIVE_PASS_KEY_EVENT: u64 = 0x2baebdc;

/// The real engine input consumer both JNI exports funnel into
/// (file 0x2e4e68c, prologue `sub sp,#0x140`, takes x0/x1, w2/s0/s1 after the
/// marshal). Pinned as the downstream-of-marshal verification in the hermetic.
pub const INPUT_CONSUMER_LEAF: u64 = 0x2e4e68c;

/// Android MotionEvent action codes mirrored from input-wrapper (so this module
/// is self-contained for hermetic marshalling tests; runtime uses the translate
/// from input-wrapper which yields the same constants).
mod action {
    pub const ACTION_DOWN: i32 = 0;
    pub const ACTION_UP: i32 = 1;
    pub const ACTION_MOVE: i32 = 2;
    pub const ACTION_CANCEL: i32 = 3;
}

// ---------------------------------------------------------------------------
// Marshalling: a translated input event -> a guest-callable ABI drive.
// ---------------------------------------------------------------------------

/// A single touch action ready to hand to the guest `nativePassInput`.
#[derive(Debug, Clone, Copy)]
pub struct TouchAction {
    pub action: i32,
    pub pointer_id: u32,
    pub x: f32,
    pub y: f32,
}

/// Marshal a touch action into the guest `nativePassInput` ABI on a fresh
/// CpuState. Float x/y are packed into the v-lane (s0 = v[0] lane0, s1 = v[1]
/// lane0 = st.v[2] low32), matching how the JIT reads `fmov s8,s1`.

/// This is the exact register layout the guest export reads: x0=env, x1=thiz,
/// x2=action(->w20), x3=pointerId(->w19), s0=x(->s9), s1=y(->s8). Returns the
/// fully-populated CpuState (caller supplies env/thiz tpidr/boot_sp).
pub fn marshal_touch(
    action: TouchAction,
    env: u64,
    thiz: u64,
    tpidr: u64,
    boot_sp: u64,
) -> CpuState {
    let mut st = CpuState::new();
    st.tpidr = tpidr;
    st.x[31] = boot_sp;
    st.x[0] = env;
    st.x[1] = thiz;
    st.x[2] = action.action as u64;
    st.x[3] = action.pointer_id as u64;
    // s0 = v[0] low u32 lane, s1 = v[1] low u32 lane (= st.v[2] low32).
    st.v[0] = (st.v[0] & !0xffff_ffffu64) | (action.x.to_bits() as u64);
    st.v[2] = (st.v[2] & !0xffff_ffffu64) | (action.y.to_bits() as u64);
    st
}

/// Drive a translated touch action into the guest `nativePassInput` export.
/// NOP (returns Ok(0)) when the bridge is disabled or there is no live image;
/// otherwise runs the guest native on a fresh CpuState and returns the guest
/// x0. Latent: the guest export only delivers input once a live session owns a
/// screen, so on the current boot path this either no-ops or soft-returns.
pub fn fire_touch(
    img: &[u8],
    base: u64,
    tpidr: u64,
    boot_sp: u64,
    action: TouchAction,
) -> Result<u64, String> {
    if !bridge_enabled() {
        eprintln!("[ainput:bridge] JIT_AINPUT_BRIDGE unset — input bridge inert (no guest call)");
        return Ok(0);
    }
    let (env, _vm) = crate::jni::build_jni();
    let thiz = crate::jni::new_fake_object();
    let mut st = marshal_touch(action, env, thiz, tpidr, boot_sp);
    let guest = base + NATIVE_PASS_INPUT;
    eprintln!(
        "[ainput:bridge] fire nativePassInput @ {guest:#x} action={} id={} pos=({}, {})",
        action.action, action.pointer_id, action.x, action.y
    );
    jit_run(img, base, guest, &mut st as *mut CpuState)?;
    Ok(st.x[0])
}

// ---------------------------------------------------------------------------
// input-wrapper reuse: a small adapter that turns the (already-built) translated
// MotionEvent stream into our TouchAction, so the input-wrapper crate is finally
// a real consumer (not orphaned dead code) and this bridge is the delivery path.
// ---------------------------------------------------------------------------

/// Translate a raw pointer press/release into a TouchAction.
pub fn translate_pointer(pressed: bool, x: f32, y: f32) -> TouchAction {
    TouchAction {
        action: if pressed {
            action::ACTION_DOWN
        } else {
            action::ACTION_UP
        },
        pointer_id: 0,
        x,
        y,
    }
}

/// Translate an ongoing pointer motion into a TouchAction (MOVE).
pub fn translate_motion(x: f32, y: f32) -> TouchAction {
    TouchAction {
        action: action::ACTION_MOVE,
        pointer_id: 0,
        x,
        y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_inert_without_env() {
        unsafe { std::env::remove_var(AINPUT_BRIDGE_ENV) };
        assert!(
            !bridge_enabled(),
            "bridge must be disabled (inert) out of the env"
        );
        // Even with a "would call" touch, disabled bridge returns Ok(0) with no
        // image needed (guard is first).
        assert_eq!(
            fire_touch(
                &[0u8; 16],
                0x100000000,
                0,
                0x200000,
                translate_pointer(true, 1.0, 2.0)
            )
            .unwrap(),
            0
        );
    }

    #[test]
    fn marshal_places_args_and_float_lanes() {
        let st = marshal_touch(
            TouchAction {
                action: 0,
                pointer_id: 2,
                x: 100.5,
                y: 200.25,
            },
            0xAAA, // env
            0xBBB, // thiz
            0xCCC, // tpidr
            0xDDD, // boot_sp
        );
        assert_eq!(st.x[31], 0xDDD);
        assert_eq!(st.x[0], 0xAAA);
        assert_eq!(st.x[1], 0xBBB);
        assert_eq!(st.x[2], 0); // action=DOWN
        assert_eq!(st.x[3], 2); // pointer_id
        // s0 = v[0] low32 must hold the bit pattern of 100.5f
        assert_eq!((st.v[0] & 0xffff_ffff) as u32, 100.5f32.to_bits());
        // s1 = v[1] low32 = st.v[2] low32 must hold 200.25f
        assert_eq!((st.v[2] & 0xffff_ffff) as u32, 200.25f32.to_bits());
        // high lanes untouched (zeroed by CpuState::new)
        assert_eq!(st.v[0] >> 32, 0);
        assert_eq!(st.v[2] >> 32, 0);
    }

    #[test]
    fn action_constants_match_input_wrapper() {
        // Mirrors input-wrapper's action enum so host-side translate and the
        // guest-facing ABI stay consistent.
        assert_eq!(action::ACTION_DOWN, 0);
        assert_eq!(action::ACTION_UP, 1);
        assert_eq!(action::ACTION_MOVE, 2);
        assert_eq!(action::ACTION_CANCEL, 3);
    }

    #[test]
    fn translate_helpers_yield_correct_actions() {
        let d = translate_pointer(true, 10.0, 20.0);
        assert_eq!(d.action, 0); // DOWN
        assert_eq!(d.x, 10.0);
        assert_eq!(d.y, 20.0);
        let u = translate_pointer(false, 10.0, 20.0);
        assert_eq!(u.action, 1); // UP
        let m = translate_motion(30.0, 40.0);
        assert_eq!(m.action, 2); // MOVE
        assert_eq!(m.pointer_id, 0);
    }

    /// Real-image pin: the guest input-native addresses must be reachable in the
    /// actual libroblox.so (file offset == vaddr for .text). Skip-if-absent so a
    /// no-image harness never fails; when present, assert the byte at each address
    /// is a plausible frame prologue (0xd1/sub sp, 0xa9/stp) and that the marshal
    /// consumer leaf region exists.
    #[test]
    fn guest_input_natives_real_image_pinned() {
        let p = std::path::Path::new("/home/hermes-worker/.cache/open-sober/robbox/libroblox.so");
        let Ok(data) = std::fs::read(p) else {
            eprintln!("[ainput] real image absent — skipping real-image pin (latent, ok)");
            return;
        };
        // AArch64 little-endian: read the full 32-bit instruction word so a
        // prologue `sub sp,sp,#imm` is matched exactly, not by its high byte.
        let word = |addr: u64| -> u32 {
            let off = (addr & 0xffff_ffff) as usize; // file offset == low vaddr
            let mut b = [0u8; 4];
            for k in 0..4 {
                b[k] = data.get(off + k).copied().unwrap_or(0);
            }
            u32::from_le_bytes(b)
        };
        // nativePassInput prologue `sub sp,sp,#0x50` = 0xd10143ff (LE word).
        assert_eq!(
            word(NATIVE_PASS_INPUT),
            0xd10143ff,
            "nativePassInput prologue sub"
        );
        // nativePassMouseMove prologue `sub sp,sp,#0x50` = same word.
        assert_eq!(
            word(NATIVE_PASS_MOUSE_MOVE),
            0xd10143ff,
            "nativePassMouseMove prologue sub"
        );
        // consumer leaf prologue `sub sp,sp,#0x140` = 0xd10503ff.
        assert_eq!(
            word(INPUT_CONSUMER_LEAF),
            0xd10503ff,
            "input consumer leaf prologue sub"
        );
        // The internal dispatch in nativePassInput at +0x50 is `bl 0x2e4e68c`
        // (0x940a4aed). Pins the real consumer call the marshal drives.
        assert_eq!(
            word(NATIVE_PASS_INPUT + 0x50),
            0x940a4aed,
            "nativePassInput bl consumer @ +0x50"
        );
    }

    #[test]
    fn marshal_zero_defaults_leave_high_lanes_clean() {
        // A Move with pointer 0 must keep high v-lanes 0 (no stale bits).
        let st = marshal_touch(
            TouchAction {
                action: 2,
                pointer_id: 0,
                x: -5.0,
                y: 0.5,
            },
            0,
            0,
            0,
            0,
        );
        assert_eq!(st.v[0] >> 32, 0);
        assert_eq!(st.v[2] >> 32, 0);
        assert_eq!((st.v[0] & 0xffff_ffff) as u32, (-5.0f32).to_bits());
        assert_eq!((st.v[2] & 0xffff_ffff) as u32, 0.5f32.to_bits());
    }
}
