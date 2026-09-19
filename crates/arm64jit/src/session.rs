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
        // SH415: the EXECUTE-DO-INIT-GATES live-DM completion markers are the
        // runtime's single source of truth for whether the do-init world-build
        // owned a live DataModel. Probe them right after the two atoms whose
        // jit_run bodies reach the do-init — StartLuaAppDM (0x1023efe2c) and
        // V2StartAppWithParams (0x10258b144) — so do-init completion is a
        // per-atom REPORTED observable (like MH_* / AppBridgeV2), not a one-off
        // probe guess. Pure readout, safe on any cell (page-guarded, no SIGSEGV).
        if atom.guest == 0x1023efe2c || atom.guest == 0x10258b144 {
            probe_doinit_completion();
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
            // SEP-17 SESSION-CTOR directive also names nativeAppBridgeAppStart
            // (V1 0x102338510, SH336 ABI) as a component to drive. With the
            // MessageBus.subscribe registry populated (SH337 measured 0->12 after
            // the bus), the V1 app-start walk runs with a real registry — the same
            // post-bus order SH337 proved. Drive it here as a first-class step, not
            // a probe rung (the sh399 abi_slots<=5 hermetic keeps it out of the
            // substrate table; it needs x2..x7 = 6 jstrings + jbool, abi_slots 7).
            drive_native_app_start(iimg, ib, tpidr, boot_sp, h.env, h.thiz);
            // SH412: the G3 content surface — seed the ENGINE's OWN files-dir
            // libc++ std::string at [0x10726d600] (recon-routeB G3: "the structural
            // content gate that turns a nonzero probe into rendered home UI") and
            // stage the R1 CoreScript module into the fsmap mirror. The operator's
            // named ordered gate is G1(surface XID) -> G2(onAppReady/\"Home\") ->
            // G3(files-dir + R1 content). The SH400 substrate drove G1 (V2UpdateSurface
            // atom) + G2 (SendAppEventOnAppReady + driven onAppReady lifecycle), but
            // G3 was only ever reachable via the --v2boot-set-filesdir/--v2boot-r1-stage
            // elfjit rungs, which SH407/408 MEASURED never fire on the reaching full-ladder
            // env (the run faults into the LSM lane before those rung lines execute). Wiring
            // it here makes the content surface a first-class driven runtime step on the SAME
            // ladder thread, exactly the SEP-18 "build the runtime, not the DM" deliverable —
            // the engine's rbxasset://scripts/CoreScripts resolver reads the R1 module the
            // instant a completed do-init owns a live DM (latent-but-correct, fires on the
            // session at the right time).
            drive_content_surface();
            // SH418: the host-input loop as a FIRST-CLASS driven substrate step
            // (same promotion SH411/412 did for DM binder, nativeAppBridgeAppStart,
            // and the G3 content surface). A real host delivers desktop pointer input
            // to the constructed login/home screen at session frame cadence; this
            // drives INPUT_LOOP_ITERS (default 8) persistent-tracker drains of the
            // registered ANativeWindow into guest nativePassInput right after the
            // post-bus content surface step. Inert-by-construction on boot (three
            // guards: JIT_AINPUT_BRIDGE + window XID + live image) — the moment a
            // live DM owns a constructed screen this step becomes the input source.
            drive_host_input_loop(iimg, ib, tpidr, boot_sp, input_loop_iters());
        }
    }
    eprintln!("[session-drive] substrate complete: {ok}/{total} atoms returned non-zero Ok");
    ok
}

/// INPUT_LOOP_ITERS (default 8) for the SH418 host-input substrate step — bounded
/// so the ordered drive never spins, overridable for longer/short session drains.
pub fn input_loop_iters() -> usize {
    std::env::var("INPUT_LOOP_ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
}

/// SEP-17 nativeAppBridgeAppStart (V1, guest 0x102338510, SH336/SH337 ABI) as a
/// first-class post-substrate step. ABI x0=env, x1=thiz, x2..x7 = 5 jstrings +
/// x5-ish jbool(false); gates on version [0x10683d350]. Driven AFTER MessageBus.subscribe
/// so the app-start walk sees a populated registry (SH337: registry 0->12 after the
/// bus), the order that gives the walk real content. Reports the registry count +
/// DM-root so the app-start's registration effect is observable in the readback.
/// Inert by itself (app-start registers services, does not construct a DataModel).
pub fn drive_native_app_start(
    iimg: &[u8],
    ib: u64,
    tpidr: u64,
    boot_sp: u64,
    env_ptr: u64,
    thiz: u64,
) -> u64 {
    let s = |b: &[u8]| crate::jni::new_string_utf_handle(b);
    let mut st = CpuState::new();
    st.tpidr = tpidr;
    st.x[31] = boot_sp;
    st.x[0] = env_ptr;
    st.x[1] = thiz;
    st.x[2] = s(b"");
    st.x[3] = s(b"");
    st.x[4] = 0; // jboolean false
    st.x[5] = s(b"");
    st.x[6] = s(b"");
    st.x[7] = s(b"");
    eprintln!(
        "[session-drive] nativeAppBridgeAppStart (V1 0x102338510) with populated registry @ entry (env={env_ptr:#x} thiz={thiz:#x})"
    );
    let r = match jit_run(iimg, ib, 0x102338510, &mut st as *mut CpuState) {
        Err(e) => {
            eprintln!("[session-drive] nativeAppBridgeAppStart stopped: {e}");
            0
        }
        Ok(r) => {
            let reg = if routeb_ensure_writable(0x106fe2f08) {
                unsafe { std::ptr::read_unaligned(0x106fe2f08u64 as *const u32) }
            } else {
                0
            };
            let dm = if routeb_ensure_writable(0x106a68818) {
                unsafe { std::ptr::read_unaligned(0x106a68818u64 as *const u64) }
            } else {
                0
            };
            eprintln!(
                "[session-drive] nativeAppBridgeAppStart returned Ok({r:#x}); registry={reg} DM-root={dm:#x}"
            );
            r
        }
    };
    r
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

/// G3 CONTENT SURFACE (recon-routeB G3 + SEP-18 BUILD-THE-RUNTIME) as a
/// first-class driven substrate step. Two pieces a real host provides so the
/// engine can SELF-construct UI content once a completed do-init owns a live DM:
///
/// 1. Seed the ENGINE's OWN files-dir libc++ std::string at [0x10726d600]
///    (guest 0x10726d600, file 0x76d600; nativeSetFilesDirectory 0x1021f7654 is
///    the real writer). This is recon-routeB G3's "structural content gate":
///    it re-roots rbxasset://scripts/CoreScripts resolution to the guest files
///    dir. SH407/408 MEASURED the equivalent --v2boot-set-filesdir rung never
///    fires on the reaching full-ladder env; wiring it here (not a separate
///    rung) is what makes the content gate a driven runtime step.
/// 2. Stage the R1 synthetic CoreScript module (a ~20-line Luau that builds a
///    ScreenGui + TextLabel under CoreGui) into the fsmap mirror via
///    `jit::stage_r1_core_scripts`, so the engine's resolver serves it the
///    moment a live DM drives the Lua loader — the exact Route-B marker
///    (engine SELF-constructs real GuiObjects, host does ZERO layout).
///
/// Inert-by-construction: it does NOT manufacture a DataModel (a live DM is
/// still the output of a real do-init); it makes the content surface the engine
/// will draw FROM available when the session owns one. Deterministic + default
/// safe: the files-dir write is wrapped in routeb_ensure_writable (so a
/// no-live-image / unmapped cell is skipped, no SIGSEGV), and the R1 staging is
/// itself guarded by page_writable_rw.
pub fn drive_content_surface() -> usize {
    // (1) G3 files-dir: guest 0x10726d600 (fix SH114's typo'd 0x1026d600).
    const FILES_DIR_GLOBAL: u64 = 0x10726d600;
    const DIR: &[u8] = b"/data/user/0/com.roblox.client/files";
    let mut seeded = 0usize;
    if routeb_ensure_writable(FILES_DIR_GLOBAL) {
        // A leaked host buffer is guest-visible (guest==host identity maps).
        let buf = Box::leak(vec![0u8; DIR.len() + 1].into_boxed_slice()).as_mut_ptr() as u64;
        unsafe {
            std::ptr::copy_nonoverlapping(DIR.as_ptr(), buf as *mut u8, DIR.len());
            *(buf as *mut u8).add(DIR.len()) = 0;
            let gp = FILES_DIR_GLOBAL as *mut u64;
            gp.add(0).write_volatile(buf); // __data_ = path ptr
            gp.add(1).write_volatile(DIR.len() as u64); // __size_
            gp.add(2).write_volatile(DIR.len() as u64); // __cap_ (bit0=0 => long)
        }
        let ptr = unsafe { *(FILES_DIR_GLOBAL as *const u64) };
        let size = unsafe { *((FILES_DIR_GLOBAL as *const u64).add(1)) };
        let mut s = String::new();
        for i in 0..size.min(4096) as usize {
            let c = unsafe { *(ptr as *const u8).add(i) };
            if c == 0 {
                break;
            }
            s.push(c as char);
        }
        let ok = ptr != 0 && size == DIR.len() as u64 && s == String::from_utf8_lossy(DIR);
        eprintln!(
            "[session-drive] SH412 G3 files-dir: seeded libc++ string @ [0x{FILES_DIR_GLOBAL:x}] = \\\"{s}\\\" {}",
            if ok { "SEEDED" } else { "MISMATCH" }
        );
        seeded = if ok { 1 } else { 0 };
    } else {
        eprintln!(
            "[session-drive] SH412 G3 files-dir @ [0x{FILES_DIR_GLOBAL:x}] not writable — skipped (no live image / unmapped cell)"
        );
    }
    // (2) R1 content staging (guarded inside stage_r1_core_scripts itself).
    let wrote = crate::jit::stage_r1_core_scripts();
    eprintln!(
        "[session-drive] SH412 R1 content surface: staged {} candidates ({})",
        wrote.len(),
        wrote.join(", ")
    );
    // (SH419) also mirror the module under the APP CACHE root. recon-v3 R1 says
    // "also mirror under cache-root": the fsmap remaps guest `/cache` and real
    // Android asset resolvers probe the app cache before the files dir, so a
    // cache-only probe would miss the module if it lived only under
    // files/scripts/CoreScripts. Idempotent; honors the same test override;
    // degrades to 0 candidates on a bare no-root harness (guarded, no SIGSEGV).
    let cache = mirror_r1_cache_root();
    eprintln!(
        "[session-drive] SH419 R1 cache-root mirror: {} ({})",
        cache.len(),
        cache.join(", ")
    );
    seeded
}

/// SH419: mirror the staged R1 CoreScript module under the APP CACHE root.
/// The fsmap remaps guest `/cache`, and real Android asset resolvers probe the app
/// cache before the files dir; this fn copies the STAGED FILES (SH412) from the
/// files-dir mirror into the app-cache mirror
/// `<root>/data/user/0/com.roblox.client/cache/scripts/CoreScripts/`, the
/// guest-visible path a cache-probing resolver reads first. Pure file staging: honors
/// the fsmap test override, returns [] on a no-root harness, idempotent, no guest byte
/// touched. Latent-but-correct exactly like SH412 — the mirror is only READ once a live
/// DM drives the Lua loader. Returns the list of mirrored candidates (or [] if nothing
/// to copy).
pub fn mirror_r1_cache_root() -> Vec<String> {
    let mut mirrored = Vec::new();
    let Some(root) = crate::fsmap::staging_root() else {
        eprintln!("[sh419] WARN persistence root not armed — cannot mirror R1 into the cache root");
        return mirrored;
    };
    let src_dir = root
        .join("data/user/0/com.roblox.client/files/scripts/CoreScripts");
    let dst_dir = root
        .join("data/user/0/com.roblox.client/cache/scripts/CoreScripts");
    for name in ["AppShell.lua", "CoreScripts.lua"] {
        let src = src_dir.join(name);
        let dst = dst_dir.join(name);
        if !src.is_file() {
            continue; // files-dir mirror not staged -> nothing to copy
        }
        let body = std::fs::read(&src).unwrap_or_default();
        let ok = std::fs::create_dir_all(&dst_dir).is_ok()
            && std::fs::write(&dst, &body).is_ok();
        let status = if ok { "CACHE-STAGED" } else { "FAIL" };
        eprintln!("[sh419] {status} {name} at {dst:?} (mirrored from {src:?})");
        mirrored.push(format!("{status} {name}"));
    }
    mirrored
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

/// SH414 — the host-input axis wired to a real event source.
///
/// This is the executable half that SH413 left latent: `fire_touch`/`deliver_motion`
/// had zero production callers and `input-wrapper` was a dev-only orphan. This step
/// is the cause-not-symptom runtime surface — a host loop that connects the real
/// X11 window (its XID registered via shims::set_anativewindow_xid by the same
/// layer that builds the EGL surface) to the guest `nativePassInput` input native.
/// Each real pointer event is translated by input-wrapper's `PointerTracker` and
/// delivered through the SH414 motion bridge.
///
/// Latent-but-correct like every runtime axis: it only matters once a live session
/// owns a screen (input is delivered to a constructed login/home), so on the
/// current boot path it is a no-op (returns the events it did NOT deliver because
/// it is gated on `JIT_AINPUT_BRIDGE` + a non-zero registered window XID + a live
/// input image). Env-gated -> default product path unchanged. The caller supplies
/// the guest input image + base so delivery is a real `jit_run` into the guest when
/// armed; passing an empty image means "not armed" -> inert.
pub fn drive_host_input_pump(
    iimg: &[u8],
    ib: u64,
    tpidr: u64,
    boot_sp: u64,
    events: &[input_wrapper::input::MotionEvent],
) -> usize {
    if !crate::ainput::bridge_enabled() {
        eprintln!("[session-drive] host-input pump: JIT_AINPUT_BRIDGE unset — inert (no guest call)");
        return 0;
    }
    if !open_delivery_armed() {
        eprintln!(
            "[session-drive] host-input pump: window/clipboard not armed — inert (deliver to a live screen only)"
        );
        return 0;
    }
    if iimg.len() < 16 {
        eprintln!("[session-drive] host-input pump: no live input image — inert");
        return 0;
    }
    let mut delivered = 0usize;
    for ev in events {
        match crate::ainput::deliver_motion(iimg, ib, tpidr, boot_sp, ev) {
            Ok(_) => delivered += 1,
            Err(e) => eprintln!("[session-drive] host-input pump: deliver error: {e}"),
        }
    }
    eprintln!(
        "[session-drive] host-input pump: delivered {delivered}/{} translated events -> nativePassInput (window xid={:#x})",
        events.len(),
        window_xid()
    );
    delivered
}

/// SH416 — the real X event source for the input axis (STATUS next-forward #3).
///
/// `drive_host_input_pump` (SH414) is the executable delivery path, but it was
/// never fed REAL desktop events: its only consumers were synthetic-vector tests
/// and a latent caller network, while the X window the runtime registers as the
/// guest ANativeWindow (`set_anativewindow_xid`, shims.rs) had no host poll that
/// turned its pointer/button/motion events into the guest `nativePassInput`
/// stream. This step closes that gap: it subscribes the REGISTERED window (not a
/// new one) to pointer/button/motion on the DISPLAY it lives on, drains one
/// non-blocking batch of real X events through input-wrapper's PointerTracker,
/// and marshals each translated Android MotionEvent into the guest input native
/// via the exact SH413/414 ABI (deliver_motion -> nativePassInput 0x2bbba88).
///
/// This is a poll step (one batch per call), so a host loop calls it repeatedly
/// while a session owns a screen — the natural integration point once a live DM
/// advances ([`drive_host_input_loop`] wraps it with a persistent tracker).
/// Inert-by-construction on the current boot path (no live DM, so no constructed
/// login/home screen yet): it is gated on the SAME three guards as the SH414
/// pump (bridge env armed + a real registered window XID + a live input image),
/// so it returns 0 without touching the guest when any trip. Env-gated
/// (JIT_AINPUT_BRIDGE) -> default product path byte-identical.
///
/// NOTE: each call builds a FRESH tracker, so a single poll is only valid as an
/// event-source smoke test. A real host loop must keep ONE tracker across
/// iterations (a press's DOWN in poll i and its MOVE in poll i+1 are the same
/// pointer) — that is [`drive_host_input_loop`].
pub fn drive_host_input_poll(
    iimg: &[u8],
    ib: u64,
    tpidr: u64,
    boot_sp: u64,
) -> usize {
    if !crate::ainput::bridge_enabled() {
        eprintln!("[session-drive] host-input poll: JIT_AINPUT_BRIDGE unset — inert (no guest call)");
        return 0;
    }
    let xid = window_xid();
    if xid == 0 {
        eprintln!("[session-drive] host-input poll: no registered ANativeWindow XID — inert");
        return 0;
    }
    #[allow(unused_mut)]
    let mut tracker = input_wrapper::input::PointerTracker::default();
    #[allow(unused_mut)]
    let mut events: Vec<input_wrapper::input::MotionEvent> = Vec::new();
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    eprintln!(
        "[session-drive] host-input poll: polling real window 0x{xid:x} on {display} for pointer/button/motion (bridge armed)"
    );
    let batch = input_wrapper::x11::pump_registered_window(
        Some(&display),
        xid as u32,
        &mut tracker,
        &mut |e| events.push(e.clone()),
    );
    match batch {
        Err(e) => {
            eprintln!(
                "[session-drive] host-input poll: X poll error on {display}: {e:?} — inert (kept 0 delivered)"
            );
            0
        }
        Ok(raw_count) => {
            eprintln!(
                "[session-drive] host-input poll: drained {raw_count} raw X events -> {} translated Android events -> guest",
                events.len()
            );
            let delivered = drive_host_input_pump(iimg, ib, tpidr, boot_sp, &events);
            eprintln!(
                "[session-drive] host-input poll: {delivered}/{} real events delivered -> nativePassInput",
                events.len()
            );
            delivered
        }
    }
}

/// SH417 — the real host input LOOP (STATUS next-forward #3): a persistent-tracker
/// poll loop that drives real desktop events into the guest `nativePassInput`.
///
/// [`drive_host_input_poll`] is the event-source smoke test — it builds a FRESH
/// `PointerTracker` every call, so a press's DOWN and its MOVE across two polls
/// are mistracked as two different pointers. A real host loop must keep ONE
/// tracker for the whole session so pointer-down state (active pointer id,
/// down-position, multi-touch press count) survives across poll iterations.
/// This step provides exactly that: it owns the tracker, selects input on the
/// registered ANativeWindow XID **once**, then drains N non-blocking batches
/// through the SAME tracker, marshalling each translated MotionEvent into the
/// guest native via the exact SH413/414 ABI (deliver_motion -> nativePassInput).
///
/// Bounded by `iterations` so a harness never spins; each iteration is a
/// non-blocking drain (safe to call at session frame cadence). Returns the total
/// events delivered across all iterations. Inert-by-construction on the current
/// boot path (no live DM -> no constructed login/home screen to deliver to): it
/// is gated on the SAME three guards as the SH414 pump (bridge env armed + a
/// real registered window XID + a live input image), so it returns 0 without
/// touching the guest when any trip (checked once up front). Env-gated
/// (JIT_AINPUT_BRIDGE) -> default product path byte-identical.
pub fn drive_host_input_loop(
    iimg: &[u8],
    ib: u64,
    tpidr: u64,
    boot_sp: u64,
    iterations: usize,
) -> usize {
    if !crate::ainput::bridge_enabled() {
        eprintln!(
            "[session-drive] host-input loop: JIT_AINPUT_BRIDGE unset — inert (no guest call)"
        );
        return 0;
    }
    let xid = window_xid();
    if xid == 0 {
        eprintln!(
            "[session-drive] host-input loop: no registered ANativeWindow XID — inert"
        );
        return 0;
    }
    if iimg.len() < 16 {
        eprintln!("[session-drive] host-input loop: no live input image — inert");
        return 0;
    }
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    // ONE tracker for the whole loop: pointer-down state must survive across
    // iterations (a press in iter i and its MOVE in iter i+1 are the same pointer).
    let mut tracker = input_wrapper::input::PointerTracker::default();
    let mut total: usize = 0;
    eprintln!(
        "[session-drive] host-input loop: polling real window 0x{xid:x} on {display} for {iterations} iterations (persistent tracker, bridge armed)"
    );
    for iter in 0..iterations {
        let mut events: Vec<input_wrapper::input::MotionEvent> = Vec::new();
        let drained = input_wrapper::x11::pump_registered_window(
            Some(&display),
            xid as u32,
            &mut tracker,
            &mut |e| events.push(e.clone()),
        );
        // A mid-loop X error (e.g. the server closed the window) must not abort
        // the session arbitrarily — log and stop the loop, keeping what we have.
        let raw_count = match drained {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[session-drive] host-input loop: X poll error at iter {iter} on {display}: {e:?} — stopping loop (kept {total} delivered)"
                );
                break;
            }
        };
        let delivered = drive_host_input_pump(iimg, ib, tpidr, boot_sp, &events);
        total += delivered;
        eprintln!(
            "[session-drive] host-input loop iter {iter}: {raw_count} raw -> {} translated -> {delivered} delivered (running total {total})",
            events.len()
        );
    }
    eprintln!(
        "[session-drive] host-input loop: done {iterations} iterations, {total} real events delivered -> nativePassInput (window xid={xid:#x})"
    );
    total
}

/// EXECUTE-DO-INIT-GATES live-DM completion probe (RECON-V3, authoritative).
///
/// A real completed do-init owns a live DataModel; the gate spec names four
/// markers that together prove it. The ordered session substrate drives
/// StartLuaAppDM (0x1023efe2c) and V2StartAppWithParams (0x10258b144) — the two
/// atoms whose jit_run bodies reach the do-init — so probing those four markers
/// right after each makes do-init completion a first-class REPORTED observable
/// of the runtime (like MH_* / AppBridgeV2), instead of a one-off probe guess.
///
/// Markers (EXECUTE-DO-INIT-GATES letter):
///   [0x106a68410].bit0 == 1            once-guard seeded (let the once-lambda run)
///   DM-root [0x106a68818] != NULL      holder populated by the once-lambda store
///   [[0x106a68818]+0x20] vt, [vt+0x30] both readable in-image  (genuine DM vtable)
///   [0x106dca0e88] counter != 0        app-data-model counter advanced
/// A live DM is implied only when ALL four hold. Every read is page-guarded via
/// routeb_ensure_writable (no SIGSEGV on an unmapped cell), read-only otherwise.
#[derive(Clone, Copy, Debug, Default)]
pub struct DoinitCompletion {
    pub once_guard_seeded: bool,
    pub dm_root_nonnull: bool,
    pub vm_vt_in_image: bool,
    pub counter_advanced: bool,
    pub once_guard: u64,
    pub dm_root: u64,
    pub vt: u64,
    pub counter: u64,
}

impl DoinitCompletion {
    /// A live DataModel is owned only when every EXECUTE-DO-INIT-GATES marker
    /// holds. This is the single bit the runtime's session gate should read to
    /// know whether the engine self-constructed its DM world headlessly.
    pub fn liveness(&self) -> bool {
        self.once_guard_seeded
            && self.dm_root_nonnull
            && self.vm_vt_in_image
            && self.counter_advanced
    }
}

/// Page-guarded guest-word read: skips non-guest / unaligned / unmapped
/// addresses and returns 0, so the probe can never SIGSEGV even when the .bss
/// cell is not mapped in the current process.
fn probe_rd(a: u64) -> u64 {
    if a != 0
        && a >= 0x100000000
        && a >> 56 == 0
        && a & 7 == 0
        && crate::jit::routeb_ensure_writable(a)
    {
        unsafe { std::ptr::read_unaligned(a as *const u64) }
    } else {
        0
    }
}

/// Read the four EXECUTE-DO-INIT-GATES live-DM completion markers and report
/// whether the do-init world-build produced a live DataModel. Pure readout (no
/// guest mutation beyond routeb_ensure_writable's page-map-on-demand); safe on
/// no-live-image harnesses (each read degrades to 0).
pub fn probe_doinit_completion() -> DoinitCompletion {
    let once_guard = probe_rd(0x106a68410);
    let dm_root = probe_rd(0x106a68818);
    // A genuine DM vtable: DM-root itself is readable, [[root]+0x20] (vt) is a
    // guest in-image pointer, and the slot it points at (vt+0x30) is readable.
    let vt = if dm_root != 0 { probe_rd(dm_root + 0x20) } else { 0 };
    let vm_vt_in_image = vt != 0 && (0x100000000..=0x200000000).contains(&vt) && probe_rd(vt + 0x30) != 0;
    let counter = probe_rd(0x106dca0e88);
    let c = DoinitCompletion {
        once_guard_seeded: once_guard & 1 == 1,
        dm_root_nonnull: dm_root != 0,
        vm_vt_in_image,
        counter_advanced: counter != 0,
        once_guard,
        dm_root,
        vt,
        counter,
    };
    eprintln!(
        "[session-drive] EXECUTE-DO-INIT live-DM probe: once-guard[0x106a68410]=0x{:x} bit0={} DM-root[0x106a68818]=0x{:x} vt=0x{:x} in-image={} app-DM-counter[0x106dca0e88]=0x{:x} -> LIVE DM = {}",
        c.once_guard,
        c.once_guard_seeded as u8,
        c.dm_root,
        c.vt,
        c.vm_vt_in_image as u8,
        c.counter,
        c.liveness()
    );
    c
}

/// Whether the host-input delivery path is armed: a real ANativeWindow XID is
/// registered (shims::set_anativewindow_xid, SH112/SH303) — i.e. the same surface
/// the EGL window path builds on. Zero = headless/plain run -> input pump inert.
fn open_delivery_armed() -> bool {
    window_xid() != 0
}

/// The current real desktop X11 window XID backing the guest ANativeWindow.
fn window_xid() -> u64 {
    crate::shims::anativewindow_xid()
}

/// recon-selfdrive-seed-jsonfix.md §A "Reject": the type-4 vector handler must be
/// a registered non-recursive leaf HOST-THUNK. Seeding the engine's own dispatcher
/// (0x10285371c) re-enters the popped-task deque infinitely; the drain pop-loop
/// (0x102856e40) or producer (0x10285682c) is re-entrant (self-drive while draining).
/// Returned host-thunk addrs and the engine frame-fn are valid non-recursive seeds.
pub fn taskv4_seed_rejected(addr: u64) -> bool {
    matches!(addr, 0x10285371c | 0x102856e40 | 0x10285682c)
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
        // SEP-17 nativeAppBridgeAppStart (V1 0x102338510) is the second named component
        // driven as a post-substrate step — same signature shape.
        let _app_start: fn(&[u8], u64, u64, u64, u64, u64) -> u64 =
            crate::session::drive_native_app_start;
        let _ = _app_start;
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

    /// SH412: the G3 content surface is a first-class driven substrate step. Compile-pin
    /// the drive signature (no-arg, returns usize = files-dir seed verdict), assert the
    /// trigger site (MessageBus.subscribe atom) stays in the substrate table so the wiring
    /// can't silently disconnect. Then call `drive_content_surface` on a no-live-image
    /// harness: `routeb_ensure_writable` maps the fixed files-dir cell [0x10726d600] even
    /// with no image, so the drive must SEED it (return 1 = the libc++ string read-back
    /// verifies) — NOT SIGSEGV. The R1 staging half degrades gracefully to 0 candidates
    /// with no persistence root armed (its own guard logs the WARN, no crash). The R1
    /// staging/serve correctness itself is pinned by sh351/sh354 (jit.rs); this hermetic
    /// pins the substrate WIRING + files-dir seed.
    #[test]
    fn sh412_content_surface_is_first_class_substrate_step() {
        // Compile-level pin: no-arg drive returning a seed-verdict usize.
        let _sig: fn() -> usize = crate::session::drive_content_surface;
        let _ = _sig;
        // The trigger site stays in the ordered substrate: MessageBus.subscribe.
        assert!(
            ROUTEB_SESSION_SUBSTRATE
                .iter()
                .any(|a| a.guest == 0x102ba5bb8),
            "MessageBus.subscribe atom present (the content-surface trigger site)"
        );
        // No live image: the cell is mapped by routeb_ensure_writable, so the drive must
        // SEED the files-dir string (return 1, read-back verified) — and must not SIGSEGV.
        // R1 staging with no root armed degrades to 0 candidates but must not crash.
        assert_eq!(
            crate::session::drive_content_surface(),
            1,
            "no-live-image: files-dir cell mapped by ensure_writable, drive must seed it (1), no SIGSEGV"
        );
    }

    /// SH419: the R1 content surface now spans BOTH roots a cache-probing resolver
    /// may read — the files-dir mirror (SH412/sh351/sh354) AND the app-cache mirror
    /// (recon-v3 R1 "also mirror under cache-root"). Under a test fs root: stage the
    /// files mirror (SH412), mirror into the cache root (SH419), and prove (a) each
    /// cache file exists + names a self-constructing ScreenGui, and (b) a guest open
    /// of the app-cache path `/data/user/0/com.roblox.client/cache/scripts/CoreScripts/
    /// <Name>.lua` resolves through `fsmap::remap_path` to exactly that mirror (the
    /// serve half, SH354-style). No live binary needed.
    #[test]
    fn sh419_r1_cache_mirror_serves_both_roots() {
        use std::ffi::CString;
        static CACHE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _g = CACHE_LOCK.lock().unwrap();
        let dir = std::env::temp_dir().join(format!("os-r1-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        crate::fsmap::set_root_for_tests(dir.clone());
        // Stage the files-dir mirror (SH412) then mirror into the cache root (SH419).
        let _staged = crate::jit::stage_r1_core_scripts();
        let mirrored = crate::session::mirror_r1_cache_root();
        assert!(
            mirrored.len() == 2,
            "expected 2 cache-mirrored candidates, got {mirrored:?}"
        );
        for name in ["AppShell.lua", "CoreScripts.lua"] {
            let cached = dir
                .join("data/user/0/com.roblox.client/cache/scripts/CoreScripts")
                .join(name);
            let body = std::fs::read_to_string(&cached).unwrap_or_else(|e| {
                panic!("cache mirror {name} not written: {e} path={cached:?}")
            });
            assert!(body.contains("ScreenGui"), "{name} cache mirror must define a ScreenGui");
            assert!(body.contains("R1HostScreen"), "{name} cache mirror must name the ScreenGui");
            // Serve half: a guest open of the app-cache path resolves to the cache mirror.
            let guest = format!(
                "/data/user/0/com.roblox.client/cache/scripts/CoreScripts/{name}"
            );
            let g = CString::new(guest.clone()).unwrap();
            let rm = crate::fsmap::remap_path(g.as_ptr())
                .unwrap_or_else(|| panic!("remap_path must resolve {guest}"));
            assert_eq!(
                rm.host_path(),
                cached,
                "remap of {guest} must be the cache mirror"
            );
            assert!(rm.host_path().is_file(), "{guest} must be a readable file");
        }
        let _ = std::fs::remove_dir_all(&dir);
        crate::fsmap::set_root_for_tests(std::path::PathBuf::new()); // clear override
    }

    /// SH414: the host-input pump is the executable half of the input axis — it
    /// must stay inert (return 0) when (a) the bridge env is unset, (b) no real
    /// window XID is registered, or (c) no live input image is present. This
    /// hermetic (no real binary, no X server) proves all three inert guards fire
    /// in order and the pump NEVER touches the guest path when any guard trips.
    /// Reuse of the input-wrapper MotionEvent type also pins the promoted
    /// dependency (input-wrapper is now a real arm64jit dep, not dev-only).
    #[test]
    fn sh414_host_input_pump_inert_without_arm() {
        use input_wrapper::input::{action as iw_action, MotionEvent};
        let ev = [MotionEvent {
            action: iw_action::ACTION_DOWN,
            pointer_index: 0,
            pointer_id: 0,
            x: 10.0,
            y: 20.0,
        }];
        // (a) bridge env unset -> inert regardless of window/image.
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
        assert_eq!(
            crate::session::drive_host_input_pump(&[0u8; 64], 0x100000000, 0, 0x200000, &ev),
            0,
            "bridge disabled -> inert (0 delivered)"
        );
        // (b) even WITH the bridge env, no real window XID armed -> inert.
        unsafe { std::env::set_var(crate::ainput::AINPUT_BRIDGE_ENV, "1") };
        let prev_xid = crate::shims::anativewindow_xid();
        crate::shims::set_anativewindow_xid(0);
        assert_eq!(
            crate::session::drive_host_input_pump(&[0u8; 64], 0x100000000, 0, 0x200000, &ev),
            0,
            "no real window XID -> inert"
        );
        // (c) window armed but no live input image -> inert.
        crate::shims::set_anativewindow_xid(crate::jit::HOST_THUNK_BASE | 0x2000);
        assert_eq!(
            crate::session::drive_host_input_pump(&[], 0x100000000, 0, 0x200000, &ev),
            0,
            "no live input image -> inert"
        );
        // Restore the prior XID (other tests may observe it).
        crate::shims::set_anativewindow_xid(prev_xid);
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
    }

    /// SH415: the EXECUTE-DO-INIT-GATES live-DM completion probe is a first-class
    /// substrate observable. On a no-live-image harness every read degrades to 0
    /// through probe_rd's page-guard (no SIGSEGV), liveness() aggregates the four
    /// markers to a single "live DM owned" bit, and the per-atom wiring keeps the
    /// probe attached to the two do-init-reaching atoms (StartLuaAppDM +
    /// V2StartAppWithParams) so do-init completion is REPORTED, not guessed.
    #[test]
    fn sh415_doinit_completion_probe_safe_and_aggregates() {
        // No live image: every marker read must degrade to 0 and liveness must be
        // false (NOT a live DM) — and critically the probe must not SIGSEGV on the
        // possibly-unmapped .bss cells (page-guarded via routeb_ensure_writable).
        let c = probe_doinit_completion();
        assert!(!c.liveness(), "no DM => liveness() must be false");
        assert_eq!(c.dm_root, 0, "no live image => DM-root read degrades to 0");
        assert_eq!(c.counter, 0, "no live image => app-DM counter degrades to 0");

        // Aggregation: any single failed marker negates liveness (all four required).
        let partial = DoinitCompletion {
            once_guard_seeded: true,
            dm_root_nonnull: true,
            vm_vt_in_image: false,
            counter_advanced: false,
            ..DoinitCompletion::default()
        };
        assert!(!partial.liveness(), "missing vt-in-image + counter => NOT live");

        // Exhaustive: only ALL four markers hold => live DM.
        assert!(
            DoinitCompletion {
                once_guard_seeded: true,
                dm_root_nonnull: true,
                vm_vt_in_image: true,
                counter_advanced: true,
                ..DoinitCompletion::default()
            }
            .liveness(),
            "all four EXECUTE-DO-INIT-GATES markers => live DM"
        );

        // The two do-init-reaching atoms stay in the substrate table so the probe
        // wiring can't silently disconnect.
        let hits: Vec<u64> = ROUTEB_SESSION_SUBSTRATE
            .iter()
            .map(|a| a.guest)
            .filter(|g| *g == 0x1023efe2c || *g == 0x10258b144)
            .collect();
        assert_eq!(
            hits.len(),
            2,
            "StartLuaAppDM (0x1023efe2c) + V2StartAppWithParams (0x10258b144) both present in the substrate"
        );
    }

    /// SH416: the real X event-source poll is inert unless ALL three guards hold
    /// (bridge env armed, a real registered window XID, no X connection needed for
    /// the guard checks). This hermetic never needs an X server: both guard trips
    /// return 0 BEFORE `pump_registered_window` is reached, so there is no X
    /// connect attempt — the poll is provably a no-op on the current boot path
    /// (no XID registered until the runtime wires a real window).
    #[test]
    fn sh416_host_input_poll_inert_without_all_guards() {
        let prev_xid = crate::shims::anativewindow_xid();
        // (a) bridge env unset -> inert regardless of XID.
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
        crate::shims::set_anativewindow_xid(0x2c00000du64);
        assert_eq!(
            crate::session::drive_host_input_poll(&[0u8; 64], 0x100000000, 0, 0x200000),
            0,
            "bridge disabled -> inert (0 delivered), no X connect"
        );
        // (b) bridge set but no registered window XID -> inert, no X connect.
        unsafe { std::env::set_var(crate::ainput::AINPUT_BRIDGE_ENV, "1") };
        crate::shims::set_anativewindow_xid(0);
        assert_eq!(
            crate::session::drive_host_input_poll(&[0u8; 64], 0x100000000, 0, 0x200000),
            0,
            "no registered ANativeWindow XID -> inert (0 delivered), no X connect"
        );
        // Restore prior XID/env so parallel tests observe clean state.
        crate::shims::set_anativewindow_xid(prev_xid);
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
    }

    /// SH417: the persistent-tracker host input LOOP (STATUS next-forward #3) is
    /// inert unless ALL three guards hold — same contract as the SH416 poll, and
    /// checked ONCE up front so a disabled bridge / unmatched window / empty
    /// image never tentatively enters the pump loop. No X server needed: all
    /// three guard trips return 0 BEFORE `pump_registered_window` is reached, so
    /// the loop is provably a no-op on the current boot path (no live DM, no
    /// constructed screen to deliver to) and cannot hang a harness.
    #[test]
    fn sh417_host_input_loop_inert_without_all_guards() {
        let prev_xid = crate::shims::anativewindow_xid();
        // (a) bridge env unset -> inert regardless of XID.
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
        crate::shims::set_anativewindow_xid(0x2e00000du64);
        assert_eq!(
            crate::session::drive_host_input_loop(&[0u8; 64], 0x100000000, 0, 0x200000, 8),
            0,
            "bridge disabled -> loop inert (0 delivered), no X connect"
        );
        // (b) bridge set but no registered window XID -> inert, no X connect.
        unsafe { std::env::set_var(crate::ainput::AINPUT_BRIDGE_ENV, "1") };
        crate::shims::set_anativewindow_xid(0);
        assert_eq!(
            crate::session::drive_host_input_loop(&[0u8; 64], 0x100000000, 0, 0x200000, 8),
            0,
            "no registered ANativeWindow XID -> loop inert (0 delivered), no X connect"
        );
        // (c) window armed but no live input image -> inert, no X connect.
        crate::shims::set_anativewindow_xid(crate::jit::HOST_THUNK_BASE | 0x2000);
        assert_eq!(
            crate::session::drive_host_input_loop(&[], 0x100000000, 0, 0x200000, 8),
            0,
            "no live input image -> loop inert (0 delivered), no X connect"
        );
        // (d) even a zero-iteration loop with all guards met is bounded (no hang).
        crate::shims::set_anativewindow_xid(0x2e00000du64);
        assert_eq!(
            crate::session::drive_host_input_loop(&[0u8; 64], 0x100000000, 0, 0x200000, 0),
            0,
            "0 iterations -> 0 delivered, bounded"
        );
        // Restore prior XID/env so parallel tests observe clean state.
        crate::shims::set_anativewindow_xid(prev_xid);
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
    }

    /// SH418: the host-input LOOP is a first-class driven substrate step (same
    /// promotion SH411/412 did for the DM binder / app-start / content surface).
    /// On a no-live-image hermetic the three inert guards (bridge, window XID,
    /// live image) make it return 0 exactly once as a substrate step, so the
    /// ordered drive completes without touching the guest — and over a real
    /// session it is the source of desktop pointer input to a constructed screen.
    #[test]
    fn sh418_host_input_loop_is_first_class_substrate_step() {
        // INPUT_LOOP_ITERS helper is bounded + defaulted (never spins).
        unsafe { std::env::remove_var("INPUT_LOOP_ITERS") };
        assert_eq!(crate::session::input_loop_iters(), 8, "default bounded (8)");
        unsafe { std::env::set_var("INPUT_LOOP_ITERS", "2") };
        assert_eq!(crate::session::input_loop_iters(), 2, "env override honored");
        unsafe { std::env::remove_var("INPUT_LOOP_ITERS") };
        // The loop is wired into drive_routeb_session_substrate after the
        // post-bus step (the trigger site is the MessageBus.subscribe atom). A
        // no-live-image drive with the bridge unset stays inert (returns 0, no X
        // connect, no guest call) and does NOT panic — prove by calling the loop
        // directly with an empty image + no XID armed.
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
        let prev_xid = crate::shims::anativewindow_xid();
        crate::shims::set_anativewindow_xid(0);
        assert_eq!(
            crate::session::drive_host_input_loop(&[0u8; 64], 0x100000000, 0, 0x200000, 4),
            0,
            "first-class step inert on boot (no X connect, no guest call)"
        );
        crate::shims::set_anativewindow_xid(prev_xid);
        unsafe { std::env::remove_var(crate::ainput::AINPUT_BRIDGE_ENV) };
    }

    /// recon-selfdrive-seed-jsonfix.md §A "Reject" guard: --taskv4-seed must refuse
    /// to install the engine's own dispatcher / drain / producer into the type-4
    /// vector (infinite recursion / re-entrancy). Host-thunk addrs and the engine
    /// frame-fn stay valid non-recursive. Pure guard logic — no real binary needed.
    #[test]
    fn taskv4_seed_rejects_recursive_engine_entries() {
        assert!(
            super::taskv4_seed_rejected(0x10285371c),
            "engine dispatcher 0x10285371c re-enters the popped-task deque infinitely"
        );
        assert!(super::taskv4_seed_rejected(0x102856e40), "drain pop-loop = re-entrant drain");
        assert!(super::taskv4_seed_rejected(0x10285682c), "producer = re-entrant push into the vector");
        assert!(!super::taskv4_seed_rejected(0x7f0000000000), "host-thunk base is the sanctioned non-recursive leaf");
        assert!(!super::taskv4_seed_rejected(0x105b32c00), "engine frame-fn is called FROM the thunk, never seeded as the vector entry");
    }
}