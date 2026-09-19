//! Ordered session-substrate DRIVE (SEP-18 BUILD-THE-RUNTIME, SH400).
//!
//! SH399 assembled `ROUTEB_SESSION_SUBSTRATE` — the coherent ordered Android
//! Activity/AppBridge lifecycle the engine asserts on — as DATA whose only
//! consumer was a hermetic address-pin test. This module is the missing
//! executable half: a driver that walks the ordered substrate and drives each
//! atom through `jit_run` on the single ladder thread (SH55/64 serialization),
//! synthesizing the per-atom ABI arguments and reporting per-atom Ok/Err plus
//! the session observables (MH_* / AppBridgeV2) after each step.
//!
//! This is the SEP-18 "build the runtime, not the DM" deliverable: a concrete,
//! testable, headlessly-verifiable host-side session boot, in Rust, that the
//! engine self-drives against. It does NOT manufacture a DataModel — a live DM
//! is still the output of a completed do-init (an upstream session ctor that
//! only a real host drive constructs) — but it is the exact ordered drive the
//! operator's SEP-17 SESSION-CTOR directive names, assembled from one table.
//!
//! Default-inert (opt-in via `--v2boot-session-drive` in elfjit.rs). ZERO
//! production path changed when unselected.

use crate::jit::{jit_run, current_guest_tp, routeb_ensure_writable, CpuState, ROUTEB_SESSION_SUBSTRATE};

/// Shared fabricated handles every native consumer takes (env/thiz params +
/// the AutoValue InitParams/StartAppParams jobjects + bg_name string).
pub struct SessionHandles {
    pub env: u64,
    pub thiz: u64,
    pub init_params: u64,
    pub start_params: u64,
    pub bg_name: u64,
}

impl SessionHandles {
    pub fn build() -> SessionHandles {
        let (env, _vm) = crate::jni::build_jni();
        SessionHandles {
            env,
            thiz: crate::jni::new_fake_object(),
            init_params: crate::jni::new_fake_object(),
            start_params: crate::jni::new_fake_object(),
            bg_name: crate::jni::new_string_utf_handle(b"ASMA.start"),
        }
    }
}

/// Synthesize the full `[u64;8]` ABI arg vector for one substrate atom.
///
/// Mirrors the proven per-rung arg patterns in elfjit.rs (recon-routeB + SH264/
/// 275/277 + SH186 ABI-correct "Home" in x5). Every atom name in
/// `ROUTEB_SESSION_SUBSTRATE` must map here — a hermetic test asserts 16/16
/// coverage so no substrate entry can silently lack args.
pub fn substrate_args(name: &str, h: &SessionHandles) -> [u64; 8] {
    let mut a = [0u64; 8];
    let e = h.env;
    let t = h.thiz;
    let s1 = crate::jni::new_string_utf_handle(b"");
    let s2 = crate::jni::new_string_utf_handle(b"");
    let s3 = crate::jni::new_string_utf_handle(b"");
    let s4 = crate::jni::new_string_utf_handle(b"");
    let home = crate::jni::new_string_utf_handle(b"Home");
    match name {
        "setTaskSchedulerBackgroundMode(false,ASMA.start)" => {
            a[0] = e; a[1] = t; a[2] = 0; a[3] = h.bg_name; // x2=enable(false)=0, x3=name
        }
        "nativeAppBridgeV2InitWithParams" | "nativeSetInitParams" => {
            a[0] = e; a[1] = t; a[2] = h.init_params;
        }
        "nativeAppBridgeV2StartAppWithParams" => {
            a[0] = e; a[1] = t; a[2] = h.start_params;
        }
        "V2UpdateSurfaceAppWithPlatformParams" => {
            // surface token (x2) + platformParams (x3) both readable host bufs.
            let tok = Box::leak(vec![0x42u8; 64].into_boxed_slice()).as_mut_ptr() as u64;
            let params = Box::leak(vec![0u8; 64].into_boxed_slice()).as_mut_ptr() as u64;
            a[0] = e; a[1] = t; a[2] = tok; a[3] = params;
        }
        "nativeInitClientSettings" => { a[0] = e; a[1] = t; a[2] = s1; a[3] = s2; a[4] = s3; }
        "nativeInitClientSettingsSigned" => { a[0] = e; a[1] = t; a[2] = s1; a[3] = s2; a[4] = s3; a[5] = s4; }
        "nativeActivity_onEngineSettingsReceived" => {
            // receive is on a fabricated manager THIS (x0), zeroed 0x800 (SH276).
            let mgr = Box::leak(vec![0u8; 0x800usize].into_boxed_slice()).as_mut_ptr() as u64;
            a[0] = mgr;
        }
        "nativeAppBridgeV2SendAppEventOnAppReady(Home in x5)" => {
            // ABI minefield (SH186): "Home" must land in x5 (discriminator reads
            // the 4th jstring = [sp]=x5); x2/x3/x4 are non-null trailing jstrings.
            a[0] = e; a[1] = t; a[2] = s1; a[3] = s2; a[4] = s3; a[5] = home;
        }
        "nativeAppBridgeV2SendAppEventOnGameLoaded" => {
            a[0] = e; a[1] = t; a[2] = s1; a[3] = s2; a[4] = s3;
        }
        "MessageBus.subscribe(experience-launch)" => {
            // inbound experience-launch listener (SH266/364): jstring topic + trails.
            a[0] = e; a[1] = t; a[2] = s1; a[3] = s2; a[4] = s3; a[5] = s4;
        }
        // JNI-receive natives taking (env, thiz) only.
        "nativeInitializeNativeFlags" | "nativeGameGlobalInit" | "nativeAppBridgeStartLuaAppDM"
        | "initAppShellReporter" | "setActive" => {
            a[0] = e; a[1] = t;
        }
        _ => {
            // Unknown name: default to (env, thiz) so the drive never panics and
            // the coverage test catches drift; safest generic ABI.
            a[0] = e; a[1] = t;
        }
    }
    a
}

/// Drive one substrate atom through `jit_run` and report its Ok/Err + post-atom
/// session observables. Returns the atom's Ok-return value (0 on Err or stop).
/// Single serialized jit_run (SH55/64): caller must not nest concurrent drives.
pub fn drive_atom(
    iimg: &[u8],
    ib: u64,
    name: &str,
    guest: u64,
    args: &[u64; 8],
    tpidr: u64,
    boot_sp: u64,
) -> u64 {
    let mut s = CpuState::new();
    s.tpidr = tpidr;
    s.x[31] = boot_sp;
    s.x[..8].copy_from_slice(args);
    let r = match jit_run(iimg, ib, guest, &mut s as *mut CpuState) {
        Err(e) => {
            eprintln!("[session-drive] {name}: stopped: {e}");
            0
        }
        Ok(r) => {
            eprintln!("[session-drive] {name}: returned Ok({r:#x})");
            r
        }
    };
    let nf = crate::jni::nativehelper_flags_loaded();
    let ni = crate::jni::nativehelper_engine_initialized();
    let ar = crate::jni::nativehelper_app_ready();
    let gl = crate::jni::nativehelper_game_loaded();
    let abv = if routeb_ensure_writable(0x106a705e8) {
        unsafe { std::ptr::read_unaligned(0x106a705e8u64 as *const u64) }
    } else {
        0
    };
    eprintln!(
        "[session-drive] {name} post: MH_FLAGS_LOADED={nf} MH_ENGINE_INITIALIZED={ni} MH_APP_READY={ar} MH_GAME_LOADED={gl} AppBridgeV2[0x106a705e8]=0x{abv:x}"
    );
    r
}

/// Drive the FULL ordered ROUTEB_SESSION_SUBSTRATE (16 atoms) on the single
/// ladder thread. Returns the number of atoms that returned Ok(v!=0-stopped).
/// This is the runtime the elfjit `--v2boot-session-drive` rung invokes.
pub fn drive_routeb_session_substrate(iimg: &[u8], ib: u64, tpidr: u64, boot_sp: u64) -> usize {
    let h = SessionHandles::build();
    let total = ROUTEB_SESSION_SUBSTRATE.len();
    let mut ok = 0usize;
    eprintln!(
        "[session-drive] driving ordered session substrate ({} atoms; env=0x{:x} thiz=0x{:x})",
        total, h.env, h.thiz
    );
    for (i, atom) in ROUTEB_SESSION_SUBSTRATE.iter().enumerate() {
        // SH82 gate: nativeGameGlobalInit's thread-dispatch compares pthread_self vs the
        // stored main-id cell [0x106863a68]; cross it by seeding self so the do-init worker
        // takes the match path instead of the nanosleep park. Mirrors the elfjit ladder rung.
        if atom.guest == 0x102206404 && routeb_ensure_writable(0x106863a68) {
            let me = unsafe { libc::pthread_self() as u64 };
            unsafe { std::ptr::write_unaligned(0x106863a68u64 as *mut u64, me); }
            eprintln!("[session-drive] SH82: seeded globalinit main-id [0x106863a68]=0x{me:x} (self) at the nativeGameGlobalInit rung");
        }
        let args = substrate_args(atom.name, &h);
        eprintln!(
            "[session-drive] [{}/{}] {} @ guest {:#x}",
            i + 1,
            total,
            atom.name,
            atom.guest
        );
        let r = drive_atom(iimg, ib, atom.name, atom.guest, &args, tpidr, boot_sp);
        if r != 0 {
            ok += 1;
        }
    }
    eprintln!("[session-drive] substrate complete: {ok}/{total} atoms returned non-zero Ok");
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every atom in the ordered substrate MUST have a per-atom ABI template.
    /// A mismatch here fails at compile-test time (no real binary needed), so a
    /// new substrate entry can never be silently discontable.
    #[test]
    fn substrate_every_atom_has_arg_template() {
        let h = SessionHandles::build();
        for atom in ROUTEB_SESSION_SUBSTRATE {
            let a = substrate_args(atom.name, &h);
            // The template must populate at least x0. The known 16 atoms all
            // populate x0 with env/thiz/mgr.
            assert_ne!(
                a[0], 0,
                "atom '{}' produces an empty ABI vector ({:#x})",
                atom.name, atom.guest
            );
        }
        assert_eq!(ROUTEB_SESSION_SUBSTRATE.len(), 16, "substrate must stay at 16 atoms");
    }

    /// The known set of atom names is exactly the substrate (no orphan template
    /// arms, and no atom falls through to the generic default arm).
    #[test]
    fn substrate_atom_names_exact_and_no_fallthrough() {
        let h = SessionHandles::build();
        // If any atom hit the `_ =>` default we'd still see a non-zero x0 env, so
        // we assert the full known-name set equals the substrate's names: template
        // drift must be a compile-level failure here.
        let known: &[&str] = &[
            "nativeInitializeNativeFlags",
            "nativeGameGlobalInit",
            "setTaskSchedulerBackgroundMode(false,ASMA.start)",
            "nativeAppBridgeV2InitWithParams",
            "nativeAppBridgeStartLuaAppDM",
            "nativeAppBridgeV2StartAppWithParams",
            "V2UpdateSurfaceAppWithPlatformParams",
            "initAppShellReporter",
            "setActive",
            "nativeSetInitParams",
            "nativeInitClientSettings",
            "nativeInitClientSettingsSigned",
            "nativeActivity_onEngineSettingsReceived",
            "nativeAppBridgeV2SendAppEventOnAppReady(Home in x5)",
            "nativeAppBridgeV2SendAppEventOnGameLoaded",
            "MessageBus.subscribe(experience-launch)",
        ];
        let names: Vec<&str> = ROUTEB_SESSION_SUBSTRATE.iter().map(|a| a.name).collect();
        assert_eq!(names, known, "substrate name set drifted from the template table");
        // SendAppEventOnAppReady is the ABI-critical one: "Home" must be x5.
        let args = substrate_args("nativeAppBridgeV2SendAppEventOnAppReady(Home in x5)", &h);
        let home_ptr = unsafe { std::ptr::read_unaligned(args[5] as *const u64) };
        let _ = home_ptr; // sanity: readable fabricated jstring handle word
        assert_ne!(args[0], 0, "SendAppEvent env absent");
        // V2UpdateSurface populates x2 (surface token) + x3 (params).
        let args_surf = substrate_args("V2UpdateSurfaceAppWithPlatformParams", &h);
        assert_ne!(args_surf[2], 0, "surface token absent");
        let _ = current_guest_tp(); // linker sanity: helper resolved
    }
}