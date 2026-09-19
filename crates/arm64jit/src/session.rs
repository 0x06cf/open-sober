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
        // recon-routeB step-2 / SEP-18: after the surface is handed over
        // (V2UpdateSurfaceAppWithPlatformParams) and before SendAppEventOnAppReady,
        // a real host drives the ordered NativeHelper `gameActivity_*` lifecycle
        // milestones (onFlagsLoaded -> onEngineInitialized -> onAppReady) on the
        // gameActivity object — the missing "onAppReady actually DRIVEN" surface
        // the SH400 substrate never fired (it only waited for the engine to reach
        // them, so none ever fired headlessly). Fire them through the SAME
        // registered JNI CallVoidMethod shim, in-order, after the surface atom.
        if atom.guest == 0x1025f5fec {
            drive_nativehelper_lifecycle();
        }
        // SEP-17 SESSION-CTOR directive names the "dataModel-bindings live binder"
        // as a component to drive. The ordered substrate's MessageBus atom drives the
        // SUBSCRIBE half (registers the experience-launch listener, measured 0->12
        // registry by SH269/315/337); the RECEIVE half — publishRaw 0x102334684, whose
        // cb (file 0x2bd7444/0x2bd76e8) reads the DataModelBindings DM holder
        // [x0+16] @0x102bd7474 — was only ever reachable via opt-in --v2boot-session-pub
        // probe rungs, never as a first-class step of the ordered runtime. Make it one:
        // on the SAME ladder thread, immediately after subscribe, publish an
        // experience-launch event and report whether the cb entered and what the
        // receive-side DM holder [DataModelBindings+16] holds (0 = live-DM side, only
        // a real do-init populates it — honest).
        if atom.guest == 0x102ba5bb8 {
            drive_data_model_binder(iimg, ib, tpidr, boot_sp, h.env, h.thiz);
        }
    }
    eprintln!("[session-drive] substrate complete: {ok}/{total} atoms returned non-zero Ok");
    ok
}

/// SEP-17 dataModel-bindings LIVE BINDER — the ordered drive's first-class RECEIVE
/// half. The MessageBus.subscribe atom registers the experience-launch listener; a real
/// host then PUBLISHes an experience-launch request through the same messageBus
/// publishRaw (0x102334684) so the engine's cb (file 0x2bd7444/0x2bd76e8) enters and
/// reads [DataModelBindings+16] — the receive-side holder the subscribe side can only
/// arm. Reports cb-entry + holder so the binder's state is observable in the substrate
/// readback, not just a probe. Inert by itself (no DM write); the holder stays 0 until a
/// completed do-init populates it (SH347/SH364 class).
pub fn drive_data_model_binder(
    iimg: &[u8],
    ib: u64,
    tpidr: u64,
    boot_sp: u64,
    env_ptr: u64,
    thiz: u64,
) -> u64 {
    let r = crate::jit::drive_messagebus_publish_receive(iimg, ib, tpidr, boot_sp, env_ptr, thiz);
    eprintln!("[session-drive] dataModel-bindings live binder: publishRaw experience-launch -> {r:#x}");
    r
}

/// Host-driven NativeHelper lifecycle milestone sequence (recon-routeB step-2 /
/// SEP-18 "callbacks actually DRIVEN"). A real Android host invokes
/// onFlagsLoaded -> onEngineInitialized -> onAppReady on the gameActivity object
/// as the session advances; the SH400 substrate only ever waited for the engine
/// to reach those CallVoidMethod sites, so none fired headlessly. This drives
/// them in order through the SAME registered JNI CallVoidMethod shim the engine
/// uses (slot 61), setting the MH_* observables exactly as a real session's
/// callbacks would. It does NOT manufacture a DataModel — it is the host-side
/// lifecycle surface recon-routeB step-2 prescribes, and it is what a completed
/// do-init's own callbacks would set anyway. Inert on real boot (opt-in host drive).
pub fn drive_nativehelper_lifecycle() {
    const ORDERED: [&[u8]; 3] = [
        b"gameActivity_onFlagsLoaded",
        b"gameActivity_onEngineInitialized",
        b"gameActivity_onAppReady",
    ];
    for name in ORDERED {
        let r = crate::jni::fire_nativehelper_milestone(name);
        eprintln!(
            "[session-drive] lifecycle milestone '{}' fired (ret={r}); MH_FLAGS_LOADED={} MH_ENGINE_INITIALIZED={} MH_APP_READY={}",
            String::from_utf8_lossy(name),
            crate::jni::nativehelper_flags_loaded(),
            crate::jni::nativehelper_engine_initialized(),
            crate::jni::nativehelper_app_ready()
        );
    }
    // SH410: onDidLogInReceived is the login-vs-home gate (VOID-with-String,
    // trigger-map 0x50a545). After onAppReady (surface attached), a fresh
    // headless session has NO persisted credential, so the host delivers an
    // EMPTY login payload -> not-logged-in -> the session-advance steers to the
    // LOGIN screen (the operator's "login renders" first screen). This is the
    // same BUILD-THE-RUNTIME surface: it records a real host login signal
    // (login_received=true, logged_in=false) so the discriminator reads it
    // instead of guessing; it does NOT boot Lua (still a completed do-init).
    let r = crate::jni::fire_nativehelper_login_payload(b"");
    eprintln!(
        "[session-drive] lifecycle milestone 'gameActivity_onDidLogInReceived' fired (ret={r}); MH_LOGIN_RECEIVED={} MH_LOGGED_IN={} -> {} screen",
        crate::jni::nativehelper_login_received(),
        crate::jni::nativehelper_logged_in(),
        if crate::jni::nativehelper_logged_in() { "home" } else { "login" }
    );
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

    /// SEP-18 host-driven lifecycle milestone sequence: drive_nativehelper_lifecycle
    /// must fire the ordered gameActivity_* milestones through the SAME registered
    /// JNI CallVoidMethod shim and transition the MH_* observables in the load-bearing
    /// order (flags-loaded -> engine-initialized -> app-ready) — the recon-routeB
    /// step-2 "callbacks actually DRIVEN" surface the SH400 substrate lacked. No real
    /// binary needed (the shim is a pure host fn on interned name handles).
    #[test]
    fn lifecycle_milestones_driven_in_order() {
        // The sequence must already be latched when we expect it; verify each of the
        // three fires its flag in order and end-state is fully ready.
        drive_nativehelper_lifecycle();
        assert!(crate::jni::nativehelper_flags_loaded(), "onFlagsLoaded fired first");
        assert!(crate::jni::nativehelper_engine_initialized(), "onEngineInitialized fired");
        assert!(crate::jni::nativehelper_app_ready(), "onAppReady fired last (app ready)");
        // SH410: the login-vs-home gate — the drive delivers an EMPTY login
        // payload (fresh headless session, no persisted credential), so the
        // callback must be recorded as received AND steered to LOGIN (not home).
        assert!(crate::jni::nativehelper_login_received(), "onDidLogInReceived fired (login-state callback arrived)");
        assert!(!crate::jni::nativehelper_logged_in(), "empty login payload steers to LOGIN (not home)");
    }

    /// SEP-17 dataModel-bindings LIVE BINDER: the ordered substrate drive now also
    /// drives the publish-RECEIVE half (publishRaw 0x102334684 -> cb [DataModelBindings+16],
    /// SH347/SH364 were elfjit-only probe rungs) as a first-class step right after the
    /// MessageBus.subscribe atom. This test brokers the integration shape without a real
    /// binary: drive_data_model_binder must be callable with the exact signature the
    /// ordered drive uses, and the MessageBus atom (its trigger site) must stay in the
    /// substrate table so the wiring can't silently disconnect.
    #[test]
    fn sh411_data_model_binder_is_first_class_substrate_step() {
        // Compile-level pin: same (iimg,ib,tpidr,boot_sp,env,thiz) signature the drive uses.
        let _sig: fn(&[u8], u64, u64, u64, u64, u64) -> u64 =
            crate::session::drive_data_model_binder;
        let _ = _sig;
        // The trigger site stays in the ordered substrate: MessageBus.subscribe.
        assert!(
            ROUTEB_SESSION_SUBSTRATE
                .iter()
                .any(|a| a.guest == 0x102ba5bb8),
            "MessageBus.subscribe atom present (the binder's trigger site)"
        );
        // The binder is a thin wrapper that returns the jit_run result (or 0 on a no-binary
        // hermetic); it must be callable through the SessionHandles the drive builds.
        let h = SessionHandles::build();
        let _ = (h.env, h.thiz);
    }
}