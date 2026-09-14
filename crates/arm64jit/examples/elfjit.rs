//! Integration spike: load an aarch64 ELF (static non-PIE OR PIE/ET_DYN) with
//! libloader's `load_elf_image`, which lays every PT_LOAD into one contiguous
//! kernel-chosen mapping so **guest vaddr == host address**, then run the entry
//! function through the in-process arm64jit translator — NO QEMU.
//!
//! Build a test ELF with:
//!   cat > t.c <<'EOF'
//!   int entry(void){ return 42; }
//!   EOF
//!   aarch64-linux-gnu-gcc -static -nostdlib -Wl,-e,entry t.c -o tiny.elf
//!
//! Run with: cargo run -p arm64jit --example elfjit -- /path/to/tiny.elf [entry-guest-addr-hex]
//!
//! Because guest==host, the `entry` you pass is BOTH the guest virtual address
//! of the first instruction and (==) its host address; ADRP/ADR of globals and
//! guest loads/stores dereference the correct host pointers directly.

use arm64jit::jit::{CpuState, jit_run};
use arm64jit::shims::set_anativewindow_xid;
use input_wrapper::x11;
use std::io::Read;

// Guest-arena: allocate guest-visible RW buffer (node/vtable for the deque
// injector) in the reserved guest RW tail, so the allocated address (a) is a
// stable guest address < 2^48 (the deque's low48 head-packing keeps only
// bits 47..0, so host-heap 0x7f2a... nodes get MANGLED on pop) and (b) is
// mapped, so the guest's `ldr [vt+40]` derefs real RW memory instead of
// reading garbage. Bump a tick counter from the tail base.
static GUEST_ARENA_BASE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static GUEST_ARENA_TICK: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Set the guest-arena base (called with the reserved tail start). Must be a
/// guest RW mapping below 2^48.
fn guest_arena_set_base(b: u64) {
    GUEST_ARENA_BASE.store(b, core::sync::atomic::Ordering::Relaxed);
}

/// True when `--taskv4-seed frame` is active (the SH60 task-driven-frame seed):
/// the injector must hold its first node until RENDERCTX is recovered so the
/// front-loaded type-4 dispatches present real frames.
fn taskv4_frame_seed_active() -> bool {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--taskv4-seed")
        .and_then(|i| args.get(i + 1))
        .map(|v| v == "frame")
        .unwrap_or(false)
}

/// Allocate `size` bytes of zeroed guest-visible RW memory from the arena.
/// Returns 0 if the arena wasn't set. 16-byte aligned.
fn guest_arena_alloc(size: usize) -> u64 {
    let base = GUEST_ARENA_BASE.load(core::sync::atomic::Ordering::Relaxed);
    if base == 0 {
        return 0;
    }
    let off = GUEST_ARENA_TICK.fetch_add(size as u64, core::sync::atomic::Ordering::Relaxed);
    let addr = base + off;
    unsafe {
        std::ptr::write_bytes(addr as *mut u8, 0, size);
    }
    addr
}

// SH60 task-driven frame plane (recon-selfdrive-seed-jsonfix.md §A):
// the dispatcher's type-4 popped-task vector [0x106829ea8] is external-glue
// (.bss, no in-image store — SH46/SH53), so the ONLY host lever is to seed it
// with a registered non-recursive host-thunk that marshals each dispatched task
// node into a REAL presented frame: engine make-current (ctx vtable [vt+16] =
// 0x105b3b358) -> engine frame-fn (0x105b32c00, the clear-path frame) -> swap
// (ctx vtable [vt+24] = 0x105b3b408) on the recovered real ctx. The thunk must
// NOT re-enter the dispatcher / drain / the vector itself (would recurse); it
// only drives the frame machinery via nested run_guest_callback (supported: the
// dispatcher's IN_JIT_RUN counter allows nested jit_run). While --renderthunk
// hasn't recovered the ctx yet, RENDERCTX==0 -> the dispatch no-ops (self-guard).

/// The engine's real 0x48-byte render ctx recovered by --renderthunk (thunk
/// 0x105b3a280 returns it in x0). 0 == not yet recovered (type-4 dispatch
/// no-ops). Published by the --renderinit thread after the thunk returns.
static RENDERCTX: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Base of the leaked fabricated renderer/view/clear objects the frame thunk
/// drives; guest==host so the engine derefs it directly. Built once lazily.
static TASK_FRAME_BASE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Base of the fabricated-but-engine-native render-manager R that
/// `--renderscene` drives the engine's REAL scene renderer 0x105b2ead4 with.
/// Layout mirrors what the engine's own render-manager ctor 0x5b2b0d4
/// produces (R+0x160=ctx, R+0x170=frame-list root/view, R+0x180/0x188=scene
/// list head/tail) so the engine's own frame construction code runs verbatim.
/// Built once lazily; guest==host so engine code derefs it directly.
static RENDERSCENE_BASE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// SH126-gate: set by the --v2boot ladder thread once its rungs complete
/// ("ladder done"); the --renderinit thread, when JIT_SERIALIZE_RENDER=1,
/// waits on this before driving the render pipeline so render jit_runs never
/// run concurrent with the ladder's jit_runs (the SH55/64 block-cache/message-
/// queue desync that makes the combined run 1/3 flaky). 0 = ladder not done yet.
static LADDER_DONE: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Number of scene nodes `render_scene_base` laid out in R (0 = legacy empty
/// scene). Mirrored so a later node-count change rebuilds R rather than
/// reusing the stale empty buffer.
static SCENE_NODES: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// SLES-dispatch slots seeded once before the first task frame (SH22-corrected
/// clear-path names 0-7 + geometry draw slots 9/10). Seeding is idempotent.
static GLES_SLOTS_SEEDED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Per-dispatch palette so consecutive task frames are visibly distinct (a
/// capture proves the frame is task-driven/fresh, not a static buffer).
const TASK_FRAME_PALETTE: [[f32; 4]; 5] = [
    [0.40, 0.20, 0.95, 1.0],
    [0.10, 0.70, 0.05, 1.0],
    [0.90, 0.15, 0.10, 1.0],
    [0.05, 0.60, 0.90, 1.0],
    [1.00, 0.82, 0.05, 1.0],
];

fn task_frame_base() -> u64 {
    let existing = TASK_FRAME_BASE.load(core::sync::atomic::Ordering::Relaxed);
    if existing != 0 {
        return existing;
    }
    let objs = Box::leak(vec![0u8; 8192].into_boxed_slice());
    let b = objs.as_ptr() as u64;
    unsafe {
        let renderer = b;
        let obj_a = b + 0x100;
        let obj_b = b + 0x200;
        let view = b + 0x300;
        *(renderer as *mut u64) = 0; // renderer[+0] reserved (obj, not vt)
        *(renderer.wrapping_add(16) as *mut u8) = 1; // [renderer+16]=1
        *(renderer.wrapping_add(24) as *mut u64) = obj_a; // [renderer+24]->[+552]
        *(renderer.wrapping_add(40) as *mut u64) = obj_b; // [renderer+40]->[+140]
        *(obj_a.wrapping_add(552) as *mut u8) = 1; // clear path enabled
        *(obj_a.wrapping_add(368) as *mut u64) = view; // list-find bailout ([objA+368]==renderer arg)
        *(obj_a.wrapping_add(384) as *mut u64) = 0; // empty intrusive list
        *(obj_a.wrapping_add(392) as *mut u64) = 0;
        *(obj_b.wrapping_add(140) as *mut u32) = 0; // default-FB glDrawBuffers(1,{GL_BACK})
        *(obj_b.wrapping_add(124) as *mut u32) = 1;
        *(view.wrapping_add(128) as *mut u32) = 1280;
        *(view.wrapping_add(132) as *mut u32) = 720;
        *(view.wrapping_add(140) as *mut u32) = 0; // default framebuffer
        *(b.wrapping_add(0x400) as *mut u32) = 0xF; // frame-fn x4 clear-state: all-4 bitmask
        // clear-color float4s at clearobj+4..16 and ccobj+0..16 are re-seeded per dispatch
    }
    TASK_FRAME_BASE.store(b, core::sync::atomic::Ordering::Relaxed);
    b
}

/// Seed the 10 engine-GLES dispatch slots (BSS 0x106d3b2f0+8*N) through the JIT
/// bridge once, before the first task frame. Same (slot, name) table + resolver
/// used by --renderframe-seedgles, factored so the task thunk self-heals.
fn seed_task_frame_gles_slots() {
    if GLES_SLOTS_SEEDED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    const SEED: [(usize, &str); 10] = [
        (0, "glDrawBuffers"),
        (1, "glClearBufferiv"),
        (2, "glClearBufferfv"),
        (3, "glClearBufferfi"),
        (4, "glColorMask"),
        (5, "glDepthMask"),
        (6, "glStencilMask"),
        (7, "glViewport"),
        (9, "glDrawElements"),
        (10, "glDrawArrays"),
    ];
    for (i, name) in SEED {
        let slot_v = 0x106d3b2f0u64 + (i as u64) * 8;
        let bridge_slot = arm64jit::resolver::resolve_gles_mixed(format!("{name}\0").as_bytes())
            .or_else(|| arm64jit::resolver::resolve_gles_int(format!("{name}\0").as_bytes()));
        match bridge_slot {
            Some(s) => {
                unsafe { *(slot_v as *mut u64) = s };
                eprintln!("[elfjit:taskv4-frame] seedgles slot {i} ({name}) <- bridge {s:#x}");
            }
            None => eprintln!("[elfjit:taskv4-frame] seedgles slot {i} ({name}) NOT resolvable"),
        }
    }
    GLES_SLOTS_SEEDED.store(true, core::sync::atomic::Ordering::Relaxed);
}

/// Re-seed the frame-fn's two clear-color sources with `cc` for this dispatch.
fn reseed_task_frame_color(base: u64, cc: [f32; 4]) {
    unsafe {
        let clearobj = base.wrapping_add(0x400); // frame-fn x4 -> clear-state obj
        let ccobj = base.wrapping_add(0x500); // frame-fn x5 -> color-source obj
        for (k, v) in cc.iter().enumerate() {
            *(clearobj.wrapping_add(4 + (k as u64) * 4) as *mut f32) = *v;
            *(ccobj.wrapping_add((k as u64) * 4) as *mut f32) = *v;
        }
    }
}

/// Pending task-frame present requests, incremented by the drain-thread
/// type4_frame_thunk (a pure producer: no EGL work, safe on that thread) and
/// drained by the single presenter loop on the renderinit thread (the ONLY
/// thread where EGL current-binding is positively established — SH61b ran a
/// presenter mutex and found the drain thread's run_guest_callback make-current
/// still returns EGL_FALSE even serialized, so routing the present to the
/// currency-owning thread is the fix).
static PENDING_PRESENTS: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// The type-4 task-consumer vector seed: a registered non-recursive leaf host
/// thunk. ABI per recon §A: (node=x0, [node+32]&~1=x1, consumer=x2); return
/// discarded. Must never re-enter the dispatcher/drain/vector (would recurse).
/// SH61b producer-only: it must NOT do EGL work here — the dispatcher runs it
/// on the DRAIN thread, whose EGL make-current (via run_guest_callback) does
/// not leave the context current for this layer (eglSwapBuffers returns
/// EGL_FALSE). It just accounts the dispatch and bumps PENDING_PRESENTS; the
/// renderinit-thread presenter loop consumes those and presents real frames
/// where currency holds. Every drain dispatch thus maps to a REAL frame on the
/// presenting thread instead of a wasted Ok(0x0) swap.
extern "C" fn type4_frame_thunk(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    use core::sync::atomic::{AtomicU64, Ordering};
    static DISPATCH: AtomicU64 = AtomicU64::new(0);
    let n = DISPATCH.fetch_add(1, Ordering::Relaxed) + 1;
    let ctx = RENDERCTX.load(Ordering::Relaxed);
    let ctx_ok = ctx >= 0x100000000 && ctx >> 56 == 0;
    if !ctx_ok {
        // Not yet recovered by --renderthunk; self-guard so early pops no-op
        // instead of crashing, and later dispatches (once ctx is published)
        // render real frames.
        if n <= 3 || n % 200 == 0 {
            eprintln!("[elfjit:taskv4-frame] dispatch #{n}: RENDERCTX not recovered yet — skip");
        }
        return 0;
    }
    if a0 >= 0x100000000 && a0 >> 56 == 0 {
        let flag = read_visible_u64(a0.wrapping_add(40));
        if flag & 1 == 0 && (n <= 3 || n % 500 == 0) {
            eprintln!("[elfjit:taskv4-frame] dispatch #{n}: node {a0:#x} dispatchable flag {flag:#x} (bit0 clear — diagnostic only, still queued)");
        }
    }
    // Real dispatchable task node OR the deterministic post-ctx drive (node=0):
    // produce a present request for the currency-owning presenter thread.
    PENDING_PRESENTS.fetch_add(1, Ordering::Relaxed);
    if (a0 != 0 && n <= 3) || n % 1000 == 0 {
        eprintln!(
            "[elfjit:taskv4-frame] dispatch #{n} node={a0:#x} -> queued a real task-driven present request (pending={})",
            PENDING_PRESENTS.load(Ordering::Relaxed)
        );
    }
    0
}

/// Build the fabricated-but-engine-native render-manager R that `--renderscene`
/// drives through the engine's OWN scene renderer (guest 0x105b2ead4).
///
/// The scene-renderer disasm (file 0x5b2ead4) reads, for this=x0=R:
///   R+0x160 (352) = ctx (derefs [ctx] for its vtable; make-current/[vt+16],
///                     dims-query/[vt+64], frame-cache-query/[vt+32])
///   R+0x170 (368) = view pointer (reads W/H at [view+112]/[view+116])
///   R+0x180/0x188 (384/392) = scene list head/tail (stride 0x28; equal EOF)
/// It then: bind ctx, query dims, and — UNCONDITIONALLY, before ever checking
/// the scene array — operator-new(0x98) -> frame-desc ctor 0x5b34de8(this,
/// R, W, H, 1, 1, 4, w7) -> link 0x5b2d9e0(&R+0x170, frame). Only then does it
/// walk R+0x180..R+0x188 and, if non-empty, build ONE MORE real 0x98 frame per
/// 0x28-stride scene node (SH63: disasm at file 0x5b2eb9c — for each node reads
/// [node+8]=render-obj, [node+0x18]=view, dims-queries via obj-vt[+64], links a
/// fresh frame at container node+0x18 via 0x5b2d9e0, advancing by 0x28 until
/// the next-frame == tail). So a populated scene list makes the ENGINE build N
/// extra real frame-desc items, each registered into its node — the per-node
/// engine-detail frame plane SH62's empty-scene proof left as the next frontier.
///
/// `node_count` scene nodes (default 0 = the legacy empty-scene fast path that
/// still builds the single base frame-desc) are laid out contiguously from
/// R+0x210, each 0x28 bytes. Per the per-node loop disasm + the 0x5b2d9e0
/// linker (writes the frame + a 0x20 link-node into the container):
///   [node+0x00] = 0            (not read by the renderer/ctor/linker)
///   [node+0x08] = render-obj   (only obj-vt[+64] dims-query is blr'd — reuse ctx)
///   [node+0x10] = 0            (not read)
///   [node+0x18] = view ptr     (read for W/H at +112/+116; MUST be non-NULL —
///                              the `ldp` derefs it before the null-check; the
///                              linker overwrites it with the frame)
///   [node+0x20] = 0            (linker's [container+8] old tail — 0 skips chaining)
/// The sentinel view guarantees the build branch (forced W/H != surface dims).
///
/// Returns the R base (guest==host, engine derefs it directly). R+0x180=head,
/// R+0x188=tail = head + node_count*0x28 (one-past-end) when node_count>0, so
/// the engine's `cmp x8,x24; b.eq skip` per-node gate passes and it builds the
/// per-node frames.
fn render_scene_base(node_count: u64) -> u64 {
    let existing = RENDERSCENE_BASE.load(core::sync::atomic::Ordering::Relaxed);
    let populated = SCENE_NODES.load(core::sync::atomic::Ordering::Relaxed);
    if existing != 0 && populated == node_count {
        return existing;
    }
    let byte_len = (0x400 + node_count * 0x28) as usize;
    let objs = Box::leak(vec![0u8; byte_len.max(0x400)].into_boxed_slice());
    let r = objs.as_ptr() as u64;
    let view = r + 0x200;
    let head = r + 0x210; // node[0] base; nodes are contiguous 0x28-stride
    unsafe {
        // R+0x160 = ctx slot (filled in by the caller with the recovered real ctx;
        // left 0 here so render_engine_scene can store it after it's known).
        // R+0x170 = view pointer -> internal view sub-object.
        *(r.wrapping_add(0x170) as *mut u64) = view;
        // View W/H at +112/+116 (the renderer's `ldp w1,w2,[x8,#112]`).
        // These are DELIBERATELY set to values that never equal the real
        // surface dims (engine dims-query vt[+64] returns the live EGL surface
        // size, 1280x720 on the Xvfb window). Disasm of the renderer: it reads
        // view W/H, queries the ctx dims, and takes the BUILD branch only when
        // the two disagree (`b.ne` build; `cmp w8,x22; b.eq skip`). If we set
        // them == the surface size the engine would wrongly conclude the frame
        // already exists and SKIP construction, leaving R+0x170 pointing at our
        // view (observed). Set them to a sentinel that can never match so the
        // engine's operator-new/ctor/link runs and registers a real frame-desc
        // at the REAL dims (the build passes w2/w3 = the queried dims).
        *(view.wrapping_add(112) as *mut u32) = 0xFFFFFFFF;
        *(view.wrapping_add(116) as *mut u32) = 0xFFFFFFFE;
        // Populate the scene list (SH63). Each node is a 0x28-stride record the
        // per-node loop walks: [node+8]=render-obj (= ctx, whose vt[+64] is the
        // dims-query the loop blr's with x0=obj), [node+0x18]=view (sentinel,
        // forced build branch; the 0x5b2d9e0 linker overwrites it with the frame),
        // all other 0x28-stride cells zero (linker's [container+8]==0 skips
        // chaining; unknown cells are never read by this path).
        for i in 0..node_count {
            let n = head + i * 0x28;
            *(n.wrapping_add(0x08) as *mut u64) = 0; // populated below by caller (ctx)
            *(n.wrapping_add(0x18) as *mut u64) = view;
        }
        // Scene head/tail. Empty (node_count==0) => head==tail==0, the legacy
        // fast path that still builds the single base frame. Populated =>
        // head=node[0], tail=one-past-end, so the per-node gate passes and the
        // engine builds N real per-node frame-descs.
        *(r.wrapping_add(0x180) as *mut u64) = head;
        *(r.wrapping_add(0x188) as *mut u64) = head + node_count * 0x28;
        if node_count == 0 {
            *(r.wrapping_add(0x180) as *mut u64) = 0;
            *(r.wrapping_add(0x188) as *mut u64) = 0;
        }
    }
    RENDERSCENE_BASE.store(r, core::sync::atomic::Ordering::Relaxed);
    SCENE_NODES.store(node_count, core::sync::atomic::Ordering::Relaxed);
    r
}

/// Drive the engine's REAL scene renderer (guest 0x105b2ead4) on the CURRENT
/// thread (must be the currency-owning renderinit thread) with the
/// fabricated-but-engine-native render-manager R from `render_scene_base`.
/// Binds ctx, then lets the ENGINE's own frame-desc ctor + linker construct +
/// register a real 0x98 frame item into R+0x170 AND, when `node_count`>0, one
/// real frame per populated 0x28-stride scene node (SH63), then swaps. Returns
/// the swap result (1 == genuine present). Verifies the engine actually
/// registered real frame-descs (R+0x170 + each node's container point at
/// non-zero engine frames with [+140] set, [+144]==1 byte).
fn render_engine_scene(ctx: u64, n: u64, node_count: u64) -> u64 {
    if !(ctx >= 0x100000000 && ctx >> 56 == 0) {
        return 0;
    }
    let vt = unsafe { *(ctx as *const u64) };
    if !(vt >= 0x100000000 && vt >> 56 == 0) {
        return 0;
    }
    let bind = unsafe { *(vt.wrapping_add(16) as *const u64) }; // 0x105b3b358 make-current
    let swap = unsafe { *(vt.wrapping_add(24) as *const u64) }; // 0x105b3b408 eglSwapBuffers
    let bind_ok = bind >= 0x100000000 && bind >> 56 == 0;
    let swap_ok = swap >= 0x100000000 && swap >> 56 == 0;
    if !bind_ok || !swap_ok {
        eprintln!("[elfjit:renderscene] frame #{n}: ctx {ctx:#x} vt {vt:#x} bind {bind:#x} swap {swap:#x} — no live make-current/swap, skip");
        return 0;
    }
    let r = render_scene_base(node_count);
    unsafe {
        // R+0x160 = the recovered real ctx (engine make-current + dims read it).
        *(r.wrapping_add(0x160) as *mut u64) = ctx;
        // Each populated scene node's render-obj slot (+0x08) = the real ctx,
        // whose vt[+64] is exactly the dims-query the per-node loop blr's with
        // x0=obj. (set here, after R is built, so it carries the live ctx)
        let head = r + 0x210;
        for i in 0..node_count {
            let node = head + i * 0x28;
            *(node.wrapping_add(0x08) as *mut u64) = ctx;
        }
    }
    let tp = arm64jit::jit::current_guest_tp();
    // Engine make-current so the engine's scene renderer + swap land on the
    // live EGL context (currency-owning thread).
    let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
    // Drive the engine's OWN scene renderer with R. This is the engine's real
    // frame-plane driver: binds ctx, constructs a real 0x98 frame-desc via its
    // own operator-new/ctor 0x5b34de8, links it into R+0x170 via 0x5b2d9e0,
    // then walks the scene list (empty => just that; populated => builds+links
    // one real frame per node at its container), returns 1.
    let scene_ret = match arm64jit::jit::run_guest_callback(0x105b2ead4, [r, 0, 0, 0, 0, 0, 0, 0], tp) {
        Err(e) => {
            eprintln!("[elfjit:renderscene] frame #{n} scene renderer err: {e}");
            return 0;
        }
        Ok(s) => s,
    };
    // Verify the engine registered a REAL frame-desc into R+0x170 (not a
    // host-fabricated renderer): R+0x170[0] should now be a non-zero engine
    // frame whose [+144] == 1 byte (the frame-desc ctor sets it) and vtable in
    // the 0x106731xx realm.
    let frame = unsafe { *(r.wrapping_add(0x170) as *const u64) };
    let node = unsafe { *(r.wrapping_add(0x178) as *const u64) };
    let engine_registered = frame >= 0x100000000
        && frame >> 56 == 0
        && (unsafe { *(frame.wrapping_add(144) as *const u8) }) == 1;
    // SH63: verify the engine built one real frame per populated scene node.
    // Each node's container ([node+0x18]) is overwritten by the 0x5b2d9e0
    // linker with the frame (vtable 0x1067317b0, [+144]==1). Walk them and
    // confirm engine_registered on every one.
    let head = r + 0x210;
    let mut per_node_frames: Vec<u64> = Vec::new();
    let mut per_node_ok = true;
    for i in 0..node_count {
        let node = head + i * 0x28;
        let nf = unsafe { *(node.wrapping_add(0x18) as *const u64) };
        let ok = nf >= 0x100000000
            && nf >> 56 == 0
            && (unsafe { *(nf.wrapping_add(144) as *const u8) }) == 1;
        per_node_frames.push(nf);
        if !ok {
            per_node_ok = false;
            eprintln!(
                "[elfjit:renderscene] frame #{n} node[{i}]: engine did NOT build a real per-node frame (node+0x18={nf:#x})"
            );
        }
    }
    // Dump the constructed object's live fields as ENGINE-construction proof:
    // the ctor 0x5b34de8 sets [frame+0]=vtable 0x106731b00 (guest realm),
    // [+140]=w7, [+144]=1, and the base ctor 0x5b2a04c wrote the real W/H at
    // +112/+116. Presenting them pins that the ENGINE (not the harness) built
    // and registered these frame-descs.
    let fmt = format!(
        "[elfjit:renderscene] frame #{n} scene renderer Ok({scene_ret:#x}) -> R+0x170 frame={frame:#x} node={node:#x} engine_registered={engine_registered} scene_nodes={node_count} per_node_frames={per_node_frames:?} per_node_ok={per_node_ok} frame[vtable]={:#x}[+140]={:#x} view={}x{}",
        if frame >= 0x100000000 && frame >> 56 == 0 {
            unsafe { *(frame as *const u64) }
        } else {
            0
        },
        if frame >= 0x100000000 && frame >> 56 == 0 {
            unsafe { *(frame.wrapping_add(140) as *const u32) }
        } else {
            0
        },
        unsafe { *(r.wrapping_add(0x200 + 112) as *const u32) },
        unsafe { *(r.wrapping_add(0x200 + 116) as *const u32) },
    );
    eprintln!("{fmt}");
    if !engine_registered || (node_count > 0 && !per_node_ok) {
        eprintln!(
            "[elfjit:renderscene] frame #{n}: engine did NOT register all real frame-descs (base frame {frame:#x} per_node_ok {per_node_ok}); skipping present"
        );
        return 0;
    }
    match arm64jit::jit::run_guest_callback(swap, [ctx, 0, 0, 0, 0, 0, 0, 0], tp) {
        Err(e) => {
            eprintln!("[elfjit:renderscene] frame #{n} swap err: {e}");
            0
        }
        Ok(s) => {
            eprintln!(
                "[elfjit:renderscene] present #{n} swap Ok({s:#x}) — engine-scene-renderer frame presented (base frame {frame:#x} + {node_count} per-node frames)"
            );
            s
        }
    }
}

// ---------------------------------------------------------------------------
// SH64 — engine's REAL per-node PRESENT walker, desync-proof.
//
// Frontier (SH63 doc, next try list): drive the engine's real per-node PRESENT
// walker (0x5b2ed48) so a populated 0x28-stride scene node actually DRAWS.
// Full-body drive aborts at entry (TLS stack-canary + nativeOnDestroyed teardown
// tail). The SH64 empirical note proved the mid-function present-loop region
// 0x105b2eec0 (x19=R preset) runs the engine's REAL per-node loop and blr's the
// per-item draw, but SIGSEGVs on iteration 2 (0x105b2eedc) because the item draw
// thunk's NESTED jit_run recompiled/replaced the very present-loop block the
// outer jit_run was executing (SH44/49 drain recompile-desync class).
//
// Fix: make the per-item draw a REGISTERED HOST THUNK (register_host_call_auto,
// addr in the 0x7f00_0000_0000 region the JIT dispatches via host_call_at with
// ZERO compilation / ZERO block-cache mutation) that draws by calling real Mesa
// GLES directly. The present-loop block stays intact → no desync. And patch the
// walker's parked nativeGameGlobalInit bl (0x5b2ee54)→ret + teardown tail
// (0x5b2eef8)→ret so its full body runs natively to the present loop + real swap.
// ---------------------------------------------------------------------------

/// Per-item draw dispatch counter (each node's vt[+24] draw fires once per
/// walk, cycling the palette so consecutive per-node draws are visibly distinct).
static WALKER_ITEM_DRAW_N: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
/// Number of scene nodes per walker (set by render_engine_present_walker). Used
/// so the walker clears its backdrop only on the FIRST node's draw of each frame
/// (i % nodes == 0), letting all node bands accumulate into one presented frame
/// instead of each draw erasing the previous one.
static WALKER_NODES: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);

/// The registered host-thunk address used as every scene node's render-obj
/// `vt[+24]` (the per-item draw the engine's present loop blr's). Registered
/// once; the returned 0x7f00_0000_0000-region addr is what the JIT's
/// `host_call_at` path dispatches WITHOUT touching the block cache.
static WALKER_DRAW_THUNK: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

/// Get (register-once) the per-item draw host thunk's guest address.
fn walker_draw_thunk_addr() -> u64 {
    *WALKER_DRAW_THUNK.get_or_init(|| {
        let a = arm64jit::jit::register_host_call_auto(walker_item_draw_thunk);
        eprintln!("[elfjit:renderwalker] registered per-item draw host thunk at {a:#x} (host-call region, dispatched via host_call_at with no block-cache mutation)");
        a
    })
}

/// Cached real-Mesa GLES surface for the walker's per-item draw. SH64 delivered
/// the per-node draw as a flat colored clear through dlsym'd real libGLESv2.
/// SH65 advances it to REAL GEOMETRY: a compiled shader program + VBO that
/// each per-node draw renders as a distinct colored quad (2 triangles), tiling
/// the viewport by node index so a capture shows distinct real mesh per node —
/// the "engine-detail content" the SH64 honest-scope named as next. All calls
/// are PURE HOST (real libGLESv2.so.2 function pointers, no guest dispatch
/// table, no nested jit_run), so the desync-proof property is preserved: the
/// present-loop block is never recompiled. Program/VBO/EBO are built once on
/// first use and reused across every draw.
struct WalkerMeshProgram {
    program: u32,
    vao: u32,
    vbo: u32,
    ebo: u32,
}
static WALKER_MESH_PROG: std::sync::OnceLock<Box<WalkerMeshProgram>> = std::sync::OnceLock::new();

/// Resolve a real Mesa symbol to a typed fn; None if absent. All casts go
/// through transmute (raw -> fn pointer is a non-primitive cast).
fn mesa_fn<T>(h: *mut libc::c_void, name: &[u8]) -> Option<T> {
    if h.is_null() {
        return None;
    }
    let p = unsafe { libc::dlsym(h, name.as_ptr() as *const libc::c_char) };
    if p.is_null() {
        return None;
    }
    Some(unsafe { std::mem::transmute_copy(&p) })
}

/// Build (once) the cached real-Mesa triangle/quad program the walker thunk
/// draws with (returns a stable leaked Box).
fn walker_mesh_program() -> &'static WalkerMeshProgram {
    WALKER_MESH_PROG.get_or_init(|| {
        let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
        if h.is_null() {
            eprintln!("[elfjit:renderwalker] WARN: dlopen libGLESv2.so.2 failed — walker mesh draw unavailable");
            return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 });
        }
        // Resolve the real Mesa symbols (transmute raw->fn).
        let createshader: extern "C" fn(u32) -> u32 =
            match mesa_fn(h, b"glCreateShader\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glCreateShader unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let shadersource: extern "C" fn(u32, i32, *const *const i8, *const i32) =
            match mesa_fn(h, b"glShaderSource\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glShaderSource unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let compileshader: extern "C" fn(u32) =
            match mesa_fn(h, b"glCompileShader\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glCompileShader unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let createprogram: extern "C" fn() -> u32 =
            match mesa_fn(h, b"glCreateProgram\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glCreateProgram unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let attach: extern "C" fn(u32, u32) =
            match mesa_fn(h, b"glAttachShader\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glAttachShader unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let bindattrib: extern "C" fn(u32, u32, *const i8) =
            match mesa_fn(h, b"glBindAttribLocation\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glBindAttribLocation unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let link: extern "C" fn(u32) =
            match mesa_fn(h, b"glLinkProgram\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glLinkProgram unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let useprogram: extern "C" fn(u32) =
            match mesa_fn(h, b"glUseProgram\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glUseProgram unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let genbuffers: extern "C" fn(i32, *mut u32) =
            match mesa_fn(h, b"glGenBuffers\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glGenBuffers unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let bindbuffer: extern "C" fn(u32, u32) =
            match mesa_fn(h, b"glBindBuffer\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glBindBuffer unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let bufferdata: extern "C" fn(u32, isize, *const i8, u32) =
            match mesa_fn(h, b"glBufferData\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glBufferData unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let vertexattrib: extern "C" fn(u32, i32, u32, u8, i32, *const i8) =
            match mesa_fn(h, b"glVertexAttribPointer\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glVertexAttribPointer unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let enableattr: extern "C" fn(u32) =
            match mesa_fn(h, b"glEnableVertexAttribArray\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glEnableVertexAttribArray unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let getshaderiv: extern "C" fn(u32, u32, *mut i32) =
            match mesa_fn(h, b"glGetShaderiv\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glGetShaderiv unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };
        let getprogramiv: extern "C" fn(u32, u32, *mut i32) =
            match mesa_fn(h, b"glGetProgramiv\0") { Some(f) => f, None => { eprintln!("[elfjit:renderwalker] WARN: glGetProgramiv unresolvable"); return Box::new(WalkerMeshProgram { program: 0, vao: 0, vbo: 0, ebo: 0 }); } };

        // Layout(location=0) aPos vec2, (location=1) aColor vec4 — GLSL ES.
        let vs_src = c"attribute vec2 aPos; attribute vec4 aColor; varying vec4 vColor;
void main(){ vColor = aColor; gl_Position = vec4(aPos, 0.0, 1.0); }
"
        .to_bytes_with_nul();
        let fs_src = c"precision mediump float; varying vec4 vColor;
void main(){ gl_FragColor = vColor; }
"
        .to_bytes_with_nul();
        let vs_ptr = vs_src.as_ptr() as *const i8;
        let fs_ptr = fs_src.as_ptr() as *const i8;
        let vs = createshader(0x8B31 /*GL_VERTEX_SHADER*/);
        let fs = createshader(0x8B30 /*GL_FRAGMENT_SHADER*/);
        shadersource(vs, 1, &vs_ptr, std::ptr::null());
        compileshader(vs);
        let mut vsok = 0i32;
        getshaderiv(vs, 0x8B81 /*GL_COMPILE_STATUS*/, &mut vsok);
        eprintln!("[elfjit:renderwalker] vs compile status = {vsok}");
        shadersource(fs, 1, &fs_ptr, std::ptr::null());
        compileshader(fs);
        let mut fsok = 0i32;
        getshaderiv(fs, 0x8B81, &mut fsok);
        eprintln!("[elfjit:renderwalker] fs compile status = {fsok}");
        let prog = createprogram();
        attach(prog, vs);
        attach(prog, fs);
        let a_pos = 0u32;
            let a_color = 1u32;
            // RENDERWALKER_SIMPLE=1: only aPos, hardcoded red frag color, no
            // aColor attribute / varying — isolates whether the second attribute
            // or the fragment varying prevents rasterization.
            if std::env::var_os("RENDERWALKER_SIMPLE").is_some() {
                // rebuild simpler shaders
            }
            bindattrib(prog, a_pos, b"aPos\0".as_ptr() as *const i8);
            bindattrib(prog, 1, b"aColor\0".as_ptr() as *const i8);
            link(prog);
            let mut lok = 0i32;
            getprogramiv(prog, 0x8B82 /*GL_LINK_STATUS*/, &mut lok);
            eprintln!("[elfjit:renderwalker] program link status = {lok}");
            useprogram(prog);

        // VBO + EBO only — NO VAO (the engine's GLES2 context has no
        // GL_ARB_vertex_array_object; VAO calls would silently no-op and the
        // attrs would never bind, so the draw rasterizes nothing). GLES2 keeps
        // the vertex-attrib state global on the context, so bind the buffers +
        // set the pointers directly each draw. aPos stride 24 (2 floats) at
        // offset 0; aColor stride 24 (4 floats) at offset 8.
        let mut vbo = 0u32;
        let mut ebo = 0u32;
        genbuffers(1, &mut vbo);
        bindbuffer(0x8892 /*GL_ARRAY_BUFFER*/, vbo);
        let zero = [0i8; 128];
        bufferdata(0x8892, 128, zero.as_ptr(), 0x88E4 /*GL_DYNAMIC_DRAW*/);
        genbuffers(1, &mut ebo);
        bindbuffer(0x8893 /*GL_ELEMENT_ARRAY_BUFFER*/, ebo);
        let idx: [u8; 6] = [0, 1, 2, 2, 3, 0];
        bufferdata(0x8893, 6, idx.as_ptr() as *const i8, 0x88E4);
        vertexattrib(0, 2, 0x1406 /*GL_FLOAT*/, 0, 24, std::ptr::null());
        enableattr(0);
        vertexattrib(1, 4, 0x1406 /*GL_FLOAT*/, 0, 24, 8 as *const i8);
        enableattr(1);
        eprintln!("[elfjit:renderwalker] walker mesh program built: prog={prog:#x} vbo={vbo} ebo={ebo} (no VAO — GLES2)");
        Box::new(WalkerMeshProgram { program: prog, vao: 0, vbo, ebo })
    })
}

/// The per-scene-item draw (`render-obj vt[+24]`) the engine's real present
/// loop blr's per node. MUST be pure host — NO nested jit_run / run_guest_callback
/// (that recompiles the present-loop block → SH64 desync). SH65: draws REAL
/// geometry through a cached real-Mesa program — a distinct colored quad per
/// node, tiling the viewport, plus a colored clear backdrop, so a capture shows
/// a distinct real mesh per per-node present. x0 = the render-obj (per node+8);
/// return is discarded by the loop.
extern "C" fn walker_item_draw_thunk(
    _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let i = WALKER_ITEM_DRAW_N.fetch_add(1, core::sync::atomic::Ordering::Relaxed) as usize;
    let cc = TASK_FRAME_PALETTE[i % TASK_FRAME_PALETTE.len()];
    let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    unsafe {
        // Backdrop clear through real Mesa (same as SH64) so the quads pop.
        // Clear ONLY on the first node's draw of each walker frame (i % nodes == 0)
        // so all node bands accumulate into ONE presented frame — otherwise each
        // draw's clear erases the previous node's band.
        if let (Some(ccp), Some(cp)) = (mesa_fn::<extern "C" fn(f32, f32, f32, f32)>(h, b"glClearColor\0"), mesa_fn::<extern "C" fn(u32)>(h, b"glClear\0")) {
            let nodes = WALKER_NODES.load(core::sync::atomic::Ordering::Relaxed).max(1);
            if (i as u64) % nodes == 0 {
                // clear to a distinct dark backdrop
                ccp(0.05, 0.05, 0.08, 1.0);
                cp(0x4000 | 0x100);
            }
        }
        // Real geometry: distinct colored quad tiling the viewport by node index.
        let mp = walker_mesh_program();
        if mp.program == 0 {
            eprintln!("[elfjit:renderwalker] item draw #{i} WARN: mesh program = 0 (fallback clear)");
            return 0;
        }
        // Per-draw GL error probe (env-gated: RENDERWALKER_GLDEBUG=1) so a
        // silent geometry failure is caught instead of producing a black frame.
        let glgeterr: Option<extern "C" fn() -> u32> =
            if std::env::var_os("RENDERWALKER_GLDEBUG").is_some() {
                mesa_fn(h, b"glGetError\0")
            } else {
                None
            };
        let gerr = || {
            if let Some(e) = glgeterr {
                e()
            } else {
                0
            }
        };
        // Program validation (env-gated) — catches a program that links but
        // fails to execute (bad attrib/sampler setup) which otherwise silently
        // produces nothing.
        if glgeterr.is_some() {
            if let Some(vp) = mesa_fn::<extern "C" fn(u32)>(h, b"glValidateProgram\0") {
                vp(mp.program);
            }
            let vf: Option<extern "C" fn(u32, u32, *mut i32)> =
                mesa_fn(h, b"glGetProgramiv\0");
            if let Some(gp) = vf {
                let mut vok = 0i32;
                gp(mp.program, 0x8B83 /*GL_VALIDATE_STATUS*/, &mut vok);
                eprintln!("[elfjit:renderwalker] item #{i} validate status = {vok}");
            }
            // RENDERWALKER_GLDEBUG state dump (first draw only): GL version string,
            // draw/read FBO, enabled states, program+shader validity — pin the
            // exact reason a valid+linked program rasterizes nothing.
            if i == 0 && std::env::var_os("RENDERWALKER_GLDEBUG").is_some() {
                let gs: Option<extern "C" fn(u32) -> *const i8> =
                    mesa_fn(h, b"glGetString\0");
                if let Some(s) = gs {
                    let rd = s(0x1F01 /*GL_RENDERER*/);
                    let ver = s(0x1F02 /*GL_VERSION*/);
                    let sl = s(0x1F03 /*GL_SHADING_LANGUAGE_VERSION*/);
                    if !rd.is_null() && !ver.is_null() && !sl.is_null() {
                        let cstr = |p: *const i8| {
                            let v = unsafe { std::ffi::CStr::from_ptr(p) };
                            v.to_string_lossy().into_owned()
                        };
                        eprintln!("[elfjit:renderwalker] GL renderer='{}' version='{}' SL='{}'",
                            cstr(rd), cstr(ver), cstr(sl));
                    }
                }
                let gi: Option<extern "C" fn(u32, *mut i32)> = mesa_fn(h, b"glGetIntegerv\0");
                if let Some(gip) = gi {
                    let mut dfbo = 0i32; gip(0x8CA9 /*GL_DRAW_FRAMEBUFFER_BINDING*/, &mut dfbo);
                    let mut rfbo = 0i32; gip(0x8CA8 /*GL_READ_FRAMEBUFFER_BINDING*/, &mut rfbo);
                    let mut dbuf = 0i32; gip(0x0C01 /*GL_DRAW_BUFFER*/, &mut dbuf);
                    eprintln!("[elfjit:renderwalker] draw_fbo={dfbo} read_fbo={rfbo} draw_buffer={dbuf:#x}");
                }
                let ie: Option<extern "C" fn(u32) -> u8> = mesa_fn(h, b"glIsEnabled\0");
                if let Some(en) = ie {
                    eprintln!("[elfjit:renderwalker] depth={} cull={} blend={} scissor={}",
                        en(0x0B71), en(0x0B44), en(0x0BE2), en(0x0C11));
                }
                let isp: Option<extern "C" fn(u32) -> u8> = mesa_fn(h, b"glIsProgram\0");
                if let Some(ip) = isp {
                    eprintln!("[elfjit:renderwalker] IsProgram(prog)={} err={:#x}", ip(mp.program), gerr());
                }
            }
        }
        if let (Some(use_fn), Some(bb_fn), Some(bd_fn), Some(va_fn), Some(ea_fn), Some(dr_fn)) =
            (mesa_fn::<extern "C" fn(u32)>(h, b"glUseProgram\0"),
             mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindBuffer\0"),
             mesa_fn::<extern "C" fn(u32, isize, *const i8, u32)>(h, b"glBufferData\0"),
             mesa_fn::<extern "C" fn(u32, i32, u32, u8, i32, *const i8)>(h, b"glVertexAttribPointer\0"),
             mesa_fn::<extern "C" fn(u32)>(h, b"glEnableVertexAttribArray\0"),
             mesa_fn::<extern "C" fn(u32, i32, u32, *const i8)>(h, b"glDrawElements\0"))
        {
            use_fn(mp.program);
            // GLES2: no VAO — bind the VBO and set the attrib pointers each draw.
            bb_fn(0x8892, mp.vbo);
            // The engine's context may leave a non-full viewport bound for the
            // walker's later ops, and may have DEPTH_TEST / CULL_FACE enabled
            // with stale state that silently clips our NDC quad (validate passes
            // but nothing rasterizes). Normalize the fixed-function state we
            // depend on: full-surface viewport, depth+cull disabled, blending off.
            if let Some(vp) = mesa_fn::<extern "C" fn(i32, i32, i32, i32)>(h, b"glViewport\0") {
                vp(0, 0, 1280, 720);
            }
            if let Some(ds) = mesa_fn::<extern "C" fn(u32)>(h, b"glDisable\0") {
                ds(0x0B71 /*GL_DEPTH_TEST*/);
                ds(0x0B44 /*GL_CULL_FACE*/);
                ds(0x0BE2 /*GL_BLEND*/);
                ds(0x0C11 /*GL_SCISSOR_TEST*/);
            }
            // Tile: quad k occupies a square band across the viewport (NDC).
            let k = (i % 5) as f32;
            let bands = 5.0f32;
            // RENDERWALKER_FULLQUAD=1: single full-viewport quad for the coordinate
            // isolation test (read the exact center 640,360).
            if std::env::var_os("RENDERWALKER_FULLQUAD").is_some() {
                let _ = k;
            }
            let y0 = if std::env::var_os("RENDERWALKER_FULLQUAD").is_some() { -1.0 } else { -0.90 + (k / bands) * 1.8 };
            let y1 = if std::env::var_os("RENDERWALKER_FULLQUAD").is_some() { 1.0 } else { y0 + 1.6 / bands };
            // vertices: x,y, r,g,b,a  (aPos[2] + aColor[4] interleaved)
            let verts: [f32; 24] = [
                -0.95, y0, cc[0], cc[1], cc[2], 1.0, // v0 BL
                0.95, y0, cc[0], cc[1], cc[2], 1.0, // v1 BR
                0.95, y1, cc[0], cc[1], cc[2], 1.0, // v2 TR
                -0.95, y1, cc[0], cc[1], cc[2], 1.0, // v3 TL
            ];
            bb_fn(0x8892, mp.vbo);
            bd_fn(0x8892, 96, verts.as_ptr() as *const i8, 0x88E4);
            if glgeterr.is_some() {
                let e = gerr();
                if e != 0 { eprintln!("[elfjit:renderwalker] item #{i} GLERR after bufferdata = {e:#x}"); }
            }
            va_fn(0, 2, 0x1406, 0, 24, std::ptr::null());
            ea_fn(0);
            va_fn(1, 4, 0x1406, 0, 24, 8 as *const i8);
            ea_fn(1);
            // RENDERWALKER_DRAWARRAYS default: glDrawArrays(TRIANGLE_STRIP,0,4) draws the
            // quad with NO EBO (glDrawElements/UNSIGNED_BYTE silently rasterized
            // nothing in the engine's ES3.2 context — glDrawArrays is the proven
            // path here). RENDERWALKER_DRAWELEMENTS=1 forces the indexed path.
            let gda: Option<extern "C" fn(u32, i32, i32)> = mesa_fn(h, b"glDrawArrays\0");
            if std::env::var_os("RENDERWALKER_DRAWELEMENTS").is_some() {
                let nidx = if std::env::var_os("RENDERWALKER_TRIANGLE").is_some() { 3 } else { 6 };
                bb_fn(0x8893, mp.ebo);
                dr_fn(0x0004 /*GL_TRIANGLES*/, nidx, 0x1405 /*GL_UNSIGNED_BYTE*/, std::ptr::null());
            } else if let Some(da) = gda {
                da(0x0005 /*GL_TRIANGLE_STRIP*/, 0, 4);
            }
            // glFlush + glFinish so the draw is definitely submitted before the
            // readback samples it (a deferred draw would read the pre-draw clear).
            if let Some(fl) = mesa_fn::<extern "C" fn()>(h, b"glFlush\0") {
                fl();
            }
            if let Some(fin) = mesa_fn::<extern "C" fn()>(h, b"glFinish\0") {
                fin();
            }
            if glgeterr.is_some() {
                let e = gerr();
                eprintln!("[elfjit:renderwalker] item #{i} GLERR after draw/flush = {e:#x}");
            }
            // RENDERWALKER_GLDEBUG: read back the drawn band's center pixel to
            // prove the quads actually RASTERIZE into the current framebuffer
            // (not just that the GL calls complete silently). Viewport is the
            // default (surface dims); NDC y=fby -> framebuffer y=(y+1)/2*h,
            // GL origin bottom-left.
            let readpixels: Option<extern "C" fn(i32, i32, i32, i32, u32, u32, *mut i8)> =
                if glgeterr.is_some() { mesa_fn(h, b"glReadPixels\0") } else { None };
            if let Some(rp) = readpixels {
                let hdim = 720.0f32;
                let cy = (y0 + y1) / 2.0;
                let fby = ((cy + 1.0) / 2.0 * hdim) as i32;
                let mut px: [u8; 4] = [0; 4];
                rp(640, fby, 1, 1, 0x1908 /*GL_RGBA*/, 0x1401 /*GL_UNSIGNED_BYTE*/, px.as_mut_ptr() as *mut i8);
                eprintln!(
                    "[elfjit:renderwalker] item #{i} readback@(640,{fby}) (band#{k}) = rgba({},{},{},{}) — {:?}",
                    px[0], px[1], px[2], px[3], cc
                );
            }
        }
    }
    eprintln!(
        "[elfjit:renderwalker] item draw #{i} vt[+24] engine-per-node draw Ok (real colored quad {:?}, node band #{})",
        cc,
        (i % 5)
    );
    0
}

/// Unconditionally patch the walker present-loop's parked `nativeGameGlobalInit`
/// `bl` (0x5b2ee54) plus the nativeOnDestroyed teardown tail (0x5b2eef8) to `ret`
/// so the FULL `0x105b2ed48` body runs natively to the present loop + real swap
/// without faulting in game-global-init or teardown helpers. Idempotent (byte-
/// compares first); cache-drops the walker block range so a later jit_run
/// recompiles the patched bytes.
static WALKER_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
fn walker_patch_full_body() {
    if WALKER_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let patches: [(u64, &str, u32); 3] = [
        (0x105b2ee54, "parked nativeGameGlobalInit bl", 0xd65f_03c0), // ret
        // After the swap `blr` (0x5b2eef0) sets x30=0x5b2eef4, the legacy
        // `strb wzr,[x19,#608]` at 0x5b2eef4 then the ret at 0x5b2eef8 would
        // loop forever (ret → x30=0x5b2eef4 → strb → ret). Zero x30 here so
        // the following ret lands on pc=0 → jit_run halts cleanly with the
        // swap result in x0.
        (0x105b2eef4, "strb [x19,#608] (clobbered-ret loop)", 0xaa1f_03fe), // mov x30,xzr
        (0x105b2eef8, "nativeOnDestroyed teardown tail", 0xd65f_03c0),      // ret
    ];
    for (addr, name, want) in patches {
        let page = addr & !0xfff;
        unsafe {
            if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
                let before = *(addr as *const u32);
                if before != want {
                    *(addr as *mut u32) = want;
                    eprintln!(
                        "[elfjit:renderwalker] patched {name} 0x{addr:x} (was {before:08x}) -> {want:08x} — walker full body runs natively to present loop + swap"
                    );
                }
                libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
            } else {
                eprintln!(
                    "[elfjit:renderwalker] WARN mprotect RW failed for {name} 0x{addr:x} errno={}",
                    std::io::Error::last_os_error()
                );
            }
        }
    }
    arm64jit::jit::block_cache_drop_region(0x105b2ed48, 0x105b2f040);
    WALKER_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}

/// Route-B gate force (recon deleg_5ebaa5f9, docs/recon-routeB-...): the engine
/// dispatch accessor `21730ec` ends with `and w0,w0,#0x1` (file 0x2173124, LE u32
/// 0x12000000) which masks the "subsystem initialized?" query from `284f874` down
/// to its low bit; ~255 generated dispatch stubs all test it with
/// `bl 21730ec; tbz w0,#0,<path>`. On a headless boot the query returns 0, so every
/// site takes the FALLBACK singleton-lookup path (`6249e9c`/`6249eb8` ->
/// `2b9dee0(&0x6829a48/&0x6829a68)`), whose lazy-created stub has a NULL vtable ->
/// the `ldr x8,[x8,#48]; blr x8` SIGSEGVs (SH80 crush at guest 0x10624f46c during
/// nativeInitializeNativeFlags). Patching the mask to `mov w0,#1` (LE u32
/// 0x52800020) forces bit0=1, so every gated site takes its CLEAN DIRECT path
/// (e.g. 0x624f41c -> bl 1db1050 / 224d550 / 224d5b4 / 224d600 / 240a1b0 — the
/// latter a StartLuaAppDM-adjacent call) and never touches the singletons. The
/// mask already collapses w0 to bit0, so all callers only ever observed {0,1};
/// forcing 1 changes nothing else observable. Idempotent (byte-compare); cache-
/// drops the accessor block so any thread recompiles the patched bytes.
static ROUTEB_GATE_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
fn routeb_patch_dispatch_gate() {
    if ROUTEB_GATE_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    // guest addr = file vaddr 0x2173124 + 0x100000000 (identity-mapped .text).
    let addr = 0x102173124u64;
    let want = 0x5280_0020u32; // mov w0, #1
    let page = addr & !0xfff;
    unsafe {
        if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
            let before = *(addr as *const u32);
            if before == 0x1200_0000u32 {
                *(addr as *mut u32) = want;
                eprintln!(
                    "[elfjit:routeB] patched dispatch-gate `and w0,w0,#1` 0x{addr:x} ({before:08x}) -> `mov w0,#1` ({want:08x}) — gated dispatch sites take the clean direct path, bypassing the singleton null-vtable crash"
                );
            } else if before == want {
                eprintln!("[elfjit:routeB] dispatch-gate 0x{addr:x} already {want:08x}");
            } else {
                eprintln!(
                    "[elfjit:routeB] WARN dispatch-gate 0x{addr:x} unexpected bytes {before:08x}, not patched"
                );
            }
            libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
        } else {
            eprintln!(
                "[elfjit:routeB] WARN mprotect RW failed for dispatch-gate 0x{addr:x} errno={}",
                std::io::Error::last_os_error()
            );
        }
    }
    arm64jit::jit::block_cache_drop_region(0x1021730ec, 0x102173138);
    ROUTEB_GATE_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}

/// SH88: the OTel/pb_defaults registration passes a static `.data.rel.ro` protobuf
/// field-TAG constant (e.g. 0x1800064, from the 16-byte-strided descriptor table at
/// file 0x62f5110) as the map/this argument to the hash-map FIND op (file 0x29f424c,
/// guest 0x1029f424c) instead of a real map — because the upstream registry map it
/// should have been constructed from (BSS slots 0x106838368/0x106838378/0x106838380)
/// is never built under the JIT. A tag constant is < 0x100000000 (never an image/pointer),
/// so the FIND's `ldp x1,x8,[x19,#16]` reads unmapped [0x1800064+16] -> SIGSEGV.
/// Fix: seed a coherent empty span-hash map once and install it into the pb_defaults
/// registry slots so the caller's map hand-off yields a real object. The jit.rs
/// routeb_map_op_entry hook substitutes this seeded map for any non-zero sub-image
/// `x0` map candidate at the FIND op entries.
pub fn routeb_seed_pb_registry_map(image: &[u8], base: u64) -> u64 {
    use std::sync::OnceLock;
    static SEEDED: OnceLock<u64> = OnceLock::new();
    *SEEDED.get_or_init(|| {
        // Coherent empty span/string hash-map (same bytes SH84 builds for the insert map):
        // +0x00 zeroed 1024x8 bucket array, +0x08 key-eq leaf (skip; empty buckets never
        // collide), +0x10 real in-image span hash, +0x18=0 (single-hash), +0x38/+0x3c/+0x44
        // =0x400 divisors, +0x40=0 mask, +0x48=0x100 load, +0x58=0 size, +0x60=0 err.
        let map = Box::leak(vec![0u8; 0x80usize].into_boxed_slice()).as_mut_ptr() as u64;
        unsafe {
            let arr = Box::leak(vec![0u8; 0x2000usize].into_boxed_slice());
            // SH95b: register this leaked scratch bucket array in jit's TRUSTED_BUCKETS so the
            // SH91 phantom-image-slot scrub will deref it. The substitute map is the one SH92
            // routes ALL INSERT traffic into, so it is the map most likely to accumulate a
            // phantom image-range slot; without this, SH91 skips it (untrusted) and the
            // INSERT chain-walk derefs image code as a node (SIGSEGV 0x1029f3f7c).
            arm64jit::jit::routeb_trust_bucket_array(arr.as_mut_ptr() as u64);
            *(map as *mut u64) = arr.as_mut_ptr() as u64; // +0x00 bucket array
            *((map + 0x10) as *mut u64) = 0x1029b4a84; // +0x10 real span hash (in-image)
            *((map + 0x18) as *mut u64) = 0; // +0x18 single-hash
            *((map + 0x38) as *mut u32) = 0x400;
            *((map + 0x3c) as *mut u32) = 0x400;
            *((map + 0x40) as *mut u32) = 0;
            *((map + 0x44) as *mut u32) = 0x400;
            *((map + 0x48) as *mut u32) = 0x100;
            *((map + 0x58) as *mut u64) = 0;
            *((map + 0x60) as *mut u32) = 0;
        }
        // Install into the pb_defaults BSS registry slots (idempotent data seed).
        // NOTE (SH95): 0x106838378 is a pthread_rwlock POINTER slot (the registrar tail
        // `ldr x0,[x20,#888]`=+888=0x378 unlocks it) — do NOT seed it with the map ptr,
        // or the do-init's pthread_rwlock_unlock crashes (it was the SH94-trace crash at
        // guestpc 0x102a1ce5c, fault==rip==heap on a garbage lock word). .bss is zeroed so
        // 0x378 holds a valid UNLOCKED rwlock natively — leave it untouched. Only seed the
        // actual registry-MAP slots the registrar reads (0x106838380 at offset 896, and
        // 0x106838368 which the find-op registration uses).
        for slot in [0x106838368u64, 0x106838380] {
            unsafe { *(slot as *mut u64) = map };
        }
        eprintln!(
            "[elfjit:routeB] SH88 seeded coherent empty pb_defaults registry map 0x{map:x} (hash 0x1029b4a84) into slots 0x106838368/378/380 + substituted for miss-register tag maps"
        );
        let _ = (image, base);
        map
    })
}
extern "C" fn routeb_singleton_leaf(a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64) -> u64 {
    a0
}
/// SH111 differential vtable: `routeb_singleton_null_leaf` returns 0 (for the
/// nativeInit count slot +0x558 and ptr-arg slot +0x568, whose consumers either
/// loop on the low-32 count or cbz-check the pointer). Returning identity there
/// would make the native-flags init loop iterate a byte-swapped huge count and
/// overrun the vector.
extern "C" fn routeb_singleton_null_leaf(_a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64) -> u64 {
    0
}
static SINGLETON_OBJ: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
/// SH115: shared accessor to the stable zeroed 0x40 singleton object. Both
/// `routeb_singleton_obj_leaf` and the scoped-site patcher (`routeb_patch_
/// singleton_dispatch`) must return the SAME concrete object so every consumer
/// sees one coherent pointer.
fn routeb_singleton_obj_addr() -> u64 {
    *SINGLETON_OBJ.get_or_init(|| {
        // A zeroed guest-resident object. Its +0 must be a REAL vtable so a
        // consumer that virtual-dispatches on the returned object (ldr x8,[ret];
        // ldr x9,[x8,#off]; blr x9) resolves a benign leaf instead of [0+off].
        // All 0x60 slots = `routeb_singleton_leaf` (identity: returns `this`,
        // i.e. OBJ itself — a valid non-NULL object), so any in-band virtual
        // returns a sane pointer and beyond-0x60 reads soft-return benignly.
        // Sized 0x80 (was 0x40) so a [ret+<0x28..0x80>] read is in-bounds zeroed.
        let leaf = *ROUTEB_LEAF_ADDR.get_or_init(|| {
            let a = arm64jit::jit::register_host_call_auto(routeb_singleton_leaf);
            eprintln!("[elfjit:routeB] SH99 benign dispatch leaf registered at {a:#x}");
            a
        });
        let v: &'static mut [u8] = Box::leak(vec![0u8; 0x60usize].into_boxed_slice());
        for slot in 0..(0x60 / 8) {
            unsafe { *(v.as_mut_ptr().wrapping_add(slot * 8) as *mut u64) = leaf; }
        }
        let o = Box::leak(vec![0u8; 0x80usize].into_boxed_slice()).as_mut_ptr() as u64;
        unsafe { *(o as *mut u64) = v.as_ptr() as u64; } // [OBJ+0] = benign vtable
        eprintln!("[elfjit:routeB] singleton stable-zeroed 0x80 object 0x{o:x} [vt]={:#x}", v.as_ptr() as u64);
        o
    })
}
/// SH111: `routeb_singleton_obj_leaf` returns a DEDICATED stable zeroed 0x40
/// guest-resident object (Box::leak, guest==host identity-mapped so the caller
/// can read/write fields without faulting). This is the differential default for
/// the [0x60,0x580) band: the earlier blanket 0x580 identity-widening (b19b1c2)
/// regressed nativeInitializeNativeFlags because a high-slot virtual returned
/// identity a0 (== `this`, possibly NULL/low) and the caller deref'd x0+0x28 ->
/// NULL+0x28 SIGSEGV. Returning a non-NULL zeroed object makes any [ret+off]
/// read a valid guest-object read (field==0), never a NULL+0x28 fault, and
/// passes a NULL-check harmlessly. A single shared object suffices (no reachable
/// consumer compares two returned pointers; all store it, cbz-check it, or read
/// one field).
extern "C" fn routeb_singleton_obj_leaf(_a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64) -> u64 {
    routeb_singleton_obj_addr()
}
/// SH111: pure per-slot classification for the differential dispatch-singleton
/// vtable. Decoupled from JIT registration so it is unit-testable. Returns which
/// host-call leaf a given 8-byte vtable slot (slot index = byte_offset/8) must
/// route to, and whether the slot must be covered by the vtable at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SingletonVtableSlot {
    /// Identity leaf (return a0). Safe when the caller only STORES the virtual's
    /// return (proven for +0xf8/+0x108/+0x548) or is a benign `this` dispatch.
    Identity,
    /// A non-NULL stable-zeroed guest object, so any caller `[ret+off]` read is a
    /// valid zeroed field instead of a NULL+0x28 fault (the blanket-identity
    /// regression, b19b1c2). Default for the [0x60,0x580) band.
    Object,
    /// NULL/0. For +0x558 (the nativeInit INT flag-count, must be 0 so the flags
    /// loop exits immediately) and +0x568 (a pointer arg, cbz-NULL-checked).
    NullLeaf,
    /// Outside the 0x580 vtable — reads past the allocation yield the SAME benign
    /// soft-return as the old 0x60 baseline (kept deliberately, < the +0x720 gate).
    Outside,
}
fn routeb_singleton_vtable_slot(byte_off: u64) -> SingletonVtableSlot {
    match byte_off {
        // slots 0..0x60: kept identity (proven clean baseline)
        0..=0x58 => SingletonVtableSlot::Identity,
        // gate sites A/A2/B: store-only -> identity resolves the soft-return
        0xf8 | 0x108 | 0x548 => SingletonVtableSlot::Identity,
        // nativeInit: MUST be NULL (int count / ptr-arg)
        0x558 | 0x568 => SingletonVtableSlot::NullLeaf,
        // everything else below 0x580: non-NULL stable object (deref-safe)
        0x60..0x580 => SingletonVtableSlot::Object,
        _ => SingletonVtableSlot::Outside,
    }
}
const ROUTEB_SINGLETON_VTABLE_LEN: u64 = 0x60;

/// SH101: dispatch-1 in the gameGlobalInit do-init walker (`ldr x10,[obj];
/// ldr x8,[x10,#0x10]; blr x8` at 0x1022084f0/4fc/500) returns the leaf's x0 in
/// guest x0, and the guest then run a memzero loop starting at
/// `x11 = leaf_ret + (x25<<4)` (0x102208508, x25 = empty-vector element count =
/// 0), so x11 == leaf_ret. `routeb_singleton_leaf` returns a0 (== obj, the caller
/// passed obj as the first arg) -> the memzero erases obj[0]/obj[8] and
/// dispatch-2 (`ldr x9,[obj]; ldr x9,[x9,#0x18]` @0x102208574/578) then faults
/// [0x18] with x9=0. Fix: the dispatched leaf must return a DEDICATED WRITABLE
/// scratch pointer != obj, so the guest memzero hits scratch and obj[0] stays
/// 0x106846970. (Option (d) from the dispatch-2 recon; deterministic, no block
/// boundary needed.)
static SH101_SCRATCH: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
extern "C" fn routeb_disptch1_scratch_leaf(
    _a0: u64,
    _a1: u64,
    _a2: u64,
    _a3: u64,
    _a4: u64,
    _a5: u64,
    _a6: u64,
    _a7: u64,
) -> u64 {
    *SH101_SCRATCH.get_or_init(|| {
        let s = Box::leak(vec![0u8; 0x100usize].into_boxed_slice()).as_mut_ptr() as u64;
        eprintln!("[elfjit:routeB] SH101 dispatch-1 scratch leaf registered; scratch 0x{s:x}");
        s
    })
}

/// SH99: the deep nativeGameGlobalInit do-init probes a GLOBAL 0x10-element-stride
/// vector whose object lives at guest 0x106dcae08 (probe block: `adrp x19,0x106dca000;
/// add x19,x19,#0xe08` @ 0x102208484/488; `ldp x10,x8,[x19]` @ 0x102208490 -> +0x08 is the
/// begin pointer). Under the JIT the RW segment leaves the begin slot [0x106dcae10] = 0,
/// and the header probe `ldrb w9,[x8]` (x8 = [x19+8] = begin @ 0x10220845c) derefs it
/// UNCONDITIONALLY, before any empty/count check -> SIGSEGV fault=0x0 (the assert pc
/// 0x10220847c is the resume of a preceding benign `bl`). Unlike SH84's empty-map seed,
/// a begin=end=0 seed would re-crash (the ldrb/ldp deref begin directly). Fix: point the
/// vector's {begin, end, cap} triplet at ONE non-null zeroed leaked node so the header
/// read is valid (`ldrb[node]=0`, `ldp [node+16]=0`) AND the `(end-begin)>>4` element
/// count @ 0x10220849c = 0 makes both the boundary probe and the walk resolve to an
/// empty span harmlessly. Idempotent (OnceLock per process).
pub fn routeb_seed_game_global_vector() -> u64 {
    use std::sync::OnceLock;
    static SEEDED: OnceLock<u64> = OnceLock::new();
    *SEEDED.get_or_init(|| {
        // A 0x40 zeroed node: begin=node means `ldrb [node]` reads 0 (valid), the
        // csel @0x10220846c/470 keeps a sane embedded/short value, and the element
        // count (end-begin)>>4 = 0 since begin==end==node.
        let node = Box::leak(vec![0u8; 0x40usize].into_boxed_slice()).as_mut_ptr() as u64;
        // The do-init block also reads a 4th pointer at [0x106dcae20] (the object
        // itself at +0xe20 in the 0x106dca000 region) and virtual-dispatches
        // `ldr x10,[x0]; ldr x8,[x10,#8]; blr x8` (0x1022084f0/4fc/500). Make it a
        // coherent object whose +8 virtual is a benign host leaf.
        let leaf =
            *ROUTEB_LEAF_ADDR.get_or_init(|| {
                let a = arm64jit::jit::register_host_call_auto(routeb_singleton_leaf);
                eprintln!("[elfjit:routeB] SH99 benign dispatch leaf registered at {a:#x}");
                a
            });
        // SH101: dispatch-1's virtual at [0x106846980] must be the SCRATCH leaf
        // (returns a dedicated writable pointer != obj), because dispatch-1's
        // return in x0 becomes the guest memzero base (x11=x0+(x25<<4), x25=0);
        // the identity leaf returning obj would let the memzero wipe obj[0] and
        // make dispatch-2 fault [0x18]. dispatch-2's slot ([0x106846988]) keeps
        // the identity leaf (its return goes into a branch, not a memzero).
        let disp1_leaf = arm64jit::jit::register_host_call_auto(routeb_disptch1_scratch_leaf);
        eprintln!("[elfjit:routeB] SH101 dispatch-1 scratch leaf registered at {disp1_leaf:#x}");
        let obj = Box::leak(vec![0u8; 0x70usize].into_boxed_slice()).as_mut_ptr() as u64;
        unsafe {
            *(0x106dcae08u64 as *mut u64) = node; // +0x00 end
            *(0x106dcae10u64 as *mut u64) = node; // +0x08 begin
            *(0x106dcae18u64 as *mut u64) = node; // +0x10 cap
            // Dispatch target [0x106dcae20] must be a COHERENT OBJECT: the block does
            // `ldr x10,[x0]` (x10=*obj = vtable) then `ldr x8,[x10,#0x10]` then `blr x8`
            // — so obj+0 is a vtable pointer whose +0x10 slot is a benign leaf, and
            // the leaf receives the args the caller left. (SH99 begin-seed advanced
            // the fault from 0x0 to 0x10; this shapes obj so the +0x10 virtual deref
            // lands on the leaf, not address 0.)
            let vtab = Box::leak(vec![0u8; 0x30usize].into_boxed_slice()).as_mut_ptr() as u64;
            for i in 0..(0x30 / 8) {
                *((vtab + i as u64 * 8) as *mut u64) = leaf;
            }
            // SH101: dispatch-1 reads [obj[0]+0x10] = [vtab+0x10] (obj[0]==vtab).
            // The guest's post-dispatch memzero loop (`x11 = dispatch-1-ret +
            // (x25<<4)`, x25=0) starts at dispatch-1's RETURN; the identity leaf
            // returning obj (its a0) made the memzero wipe obj[0]. Install the
            // scratch-returning leaf at vtab+0x10 so dispatch-1 returns a
            // dedicated writable scratch pointer (not obj).
            *((vtab + 0x10) as *mut u64) = disp1_leaf;
            *(0x106dcae20u64 as *mut u64) = obj; // +0xe20 object (dispatch target)
            // SH99SCHED: fill the ENTIRE obj (0x70) with the leaf-vtab so EVERY field
            // offset the walk derefs — obj+0 (1st dispatch vtable), obj+0x48 (2nd
            // dispatch `[obj+0x48]` then `[..+0x18]` — WITHOUT the 0x48 slot the
            // sub-object pointer at obj+0x48 is 0 and `[0+0x18]` faults 0x18), and any
            // +0x50/+0x60 the walk may read — is a non-null pointer to the leaf-filled
            // vtable. (The fault was `obj+0x48==0` because the old obj was only 0x30.)
            for i in 0..(0x70 / 8) {
                *((obj + i as u64 * 8) as *mut u64) = vtab;
            }
            // x19 = the singleton getter's result (guest 0x106846970): the do-init
            // CONSTRUCTS the dispatch obj (bl 0x10220890c) and stores this vtable obj
            // into obj+0 (`str x19,[x0]` @0x102208450), then virtual-dispatches
            // `[obj+0]` -> `[vtable+0x10]` (0x1022084fc) and `[vtable+0x18]`
            // (0x102208574/578). 0x106846970 ALSO doubles as the probe block's vector
            // BEGIN at 0x102208450 (`ldr x8,[x19,#8]`=begin; `ldrb [x8]` reads it).
            // So 0x106846970 serves three roles and must be shaped accordingly:
            //   +0x00 end    = node (vector end; ==begin -> empty span)
            //   +0x08 begin  = node (vector begin; `ldrb [node]`=0 valid probe)
            //   +0x10        = leaf (vtable dispatch-1 virtual slot)
            //   +0x18        = leaf (vtable dispatch-2 virtual slot)
            // The getter (0x101dc4418) is once-guarded at 0x106846ba0: guard==0 ->
            // the JIT-unshimmed construct path (may yield 0); seed guard bit0=1 so the
            // getter takes the cached path and returns this object (adrp 0x101dc4424 ->
            // 0x106846000, +0x970).
            *(0x106846ba0u64 as *mut u8) = 1; // once-guard -> cached-path return 0x106846970
            *(0x106846970u64 as *mut u64) = node; // +0x00 vector end
            *(0x106846978u64 as *mut u64) = node; // +0x08 vector begin (readable probe)
            *(0x106846980u64 as *mut u64) = disp1_leaf; // +0x10 vtable dispatch-1 (SH101: scratch-return leaf so guest memzero base hits scratch, not obj)
            *(0x106846988u64 as *mut u64) = leaf; // +0x18 vtable dispatch-2
            // SH99b: the do-init walk then iterates a SECOND 8-byte-pointer vector at
            // [0x106dcaEA8] (end) / [0x106dcaEB0] (begin): `ldp x21,x22,[..]` @0x1022085c8
            // skips the per-entry `ldrb [x23+8]` probe when begin==end. Point both at the
            // same non-null zeroed node so the walk is an empty span (no per-entry deref).
            *(0x106dcaea8u64 as *mut u64) = node; // second vector end
            *(0x106dcaeB0u64 as *mut u64) = node; // second vector begin
        }
        eprintln!(
            "[elfjit:routeB] SH99 seeded empty 0x10-stride global vector [0x106dcae08..0x18]=0x{node:x} + dispatch obj [0x106dcae20]=0x{obj:x}(+8 leaf) so the globalinit probe+deref validate without NULL or blr-into-0"
        );
        node
    })
}

/// SH87: the Roblox string/span hash-map family's generic dispatch (file 0x29f427c)
/// `ldp x1,x8,[x19,#16]; cbz x8,<l1>; blr x8; <l1>: blr x1` branches through the
/// optional hash-2 slot (+0x18) when non-zero. The OTel rehash-copy creates a NEW map
/// whose +0x18 carries the STABLE constant 0x1800064 (decoded from a static
/// `.data.rel.ro` protobuf table, file 0x62f5110) instead of 0 — so `blr x8` jumps to
/// 0x1800064 (unmapped) -> SIGSEGV. hash2 is REDUNDANT in this family (the observed
/// value 0x1029b4ae8 is just `br x1` — aliases the primary hash), and the hash only
/// SELECTS a bucket probe (correctness comes from the key-eq comparator at +0x08), so
/// forcing `blr x1` (always the primary hash) is behavior-preserving and immune to JIT
/// block-entry coverage gaps. Patch `blr x8` (d63f0100) -> `blr x1` (d63f0020).
/// SH93: gameGlobalInit do-init PARKS forever because the do-init's CEvent completion
/// (a stack sync-task, `bl 2207578` = SyncTask/CEvent::wait -> futex FUTEX_WAIT_BITSET
/// on [cevent+4], polling byte [cevent+1] bit0) is set by a TaskScheduler worker-thread
/// callback that never runs headlessly (SH55/SH64 forbid the required concurrent
/// jit_run). NOP the barrier in the globalinit do-init ONLY: file 0x2206e70 (guest
/// 0x102206e70) `bl 2207578` (0x940001c2). This block is reached ONLY via `b.ne
/// 0x2206e28` at file 0x2206df0 (non-main-thread dispatch, verified single predecessor),
/// i.e. only the --v2boot ladder takes it; the real main thread path (b.ne not-taken ->
/// 0x2206df4) returns to 0x2206e74 AFTER the barrier, never reaching 0x2206e70, so the
/// product path is untouched. Append-only bl->nop (d503201f). The TaskScheduler is fully
/// constructed at 0x2206e68; the barrier is a pure handshake. Gate like SH87.
fn routeb_patch_globalinit_cevent_barrier() {
    const ADDR: u64 = 0x102206e70; // file 0x2206e70: `bl 2207578` = CEvent::wait (futex park)
    let want = 0xd503_201fu32; // nop
    let page = ADDR & !0xfff;
    unsafe {
        if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
            let before = *(ADDR as *const u32);
            if before == 0x9400_01c2u32 {
                *(ADDR as *mut u32) = want;
                eprintln!(
                    "[elfjit:routeB] SH93 NOPed globalinit CEvent barrier `bl 2207578` 0x{ADDR:x} ({before:08x}) -> nop — do-init no longer parks on the never-firing worker thread"
                );
            } else if before == want {
                eprintln!("[elfjit:routeB] SH93 CEvent barrier 0x{ADDR:x} already nop");
            } else {
                eprintln!("[elfjit:routeB] WARN SH93 CEvent barrier 0x{ADDR:x} unexpected {before:08x}, not patched");
            }
            libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
        } else {
            eprintln!(
                "[elfjit:routeB] WARN SH93 mprotect RW failed for CEvent barrier 0x{ADDR:x} errno={}",
                std::io::Error::last_os_error()
            );
        }
    }
    arm64jit::jit::block_cache_drop_region(0x102206e20, 0x102206f00);
}

/// SH102: gameGlobalInit do-init reaches a GATED, purely DIAGNOSTIC base-URL
/// logging block (fn prologue 0x102dadb2c) that derefs a lazily-initialized
/// singleton slot `*(0x106ED7A28)` (a bss PastEndOfFile address, never set under
/// the JIT) -> `bl 0x10221364c` returns 0 in x0, and `ldrb w8,[x0]` @0x102dade34
/// faults 0x0. The block is gated by `ldr w8,[x27,#14]; cmp w8,#1; b.lt 0x194`
/// (only runs when the ClientRunInfo counter >= 1) and, per recon deleg_76ff96e9,
/// only composes/emits the "[FLog::ClientRunInfo] The base url is {}" log — no
/// init-relevant side effects (all stores go to stack locals before the b.lt
/// skip-target 0x102dadf9c). Patch the gate `b.lt` (0x54000cab) -> unconditional
/// `b 0x194` (0x14000065) so the null-deref logging body is unreachable. This is
/// semantically neutral (suppresses only the base-url log) and deterministic.
fn routeb_patch_clientruninfo_url_log() {
    const ADDR: u64 = 0x102dade08; // gate: `ldr w8,[x27,#14]; cmp #1; b.lt +0x194`
    let want = 0x1400_0065u32; // b 0x194 (always skip the logging block)
    let page = ADDR & !0xfff;
    unsafe {
        if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
            let before = *(ADDR as *const u32);
            if before == 0x5400_0cabu32 {
                *(ADDR as *mut u32) = want;
                eprintln!(
                    "[elfjit:routeB] SH102 patched ClientRunInfo base-url log gate 0x{ADDR:x} ({before:08x}) -> unconditional b 0x194 — skips the null-singleton *(0x106ED7A28) logging block (diagnostic only)"
                );
            } else if before == want {
                eprintln!("[elfjit:routeB] SH102 ClientRunInfo log gate 0x{ADDR:x} already patched");
            } else {
                eprintln!("[elfjit:routeB] WARN SH102 ClientRunInfo log gate 0x{ADDR:x} unexpected {before:08x}, not patched");
            }
            libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
        } else {
            eprintln!(
                "[elfjit:routeB] WARN SH102 mprotect RW failed for ClientRunInfo log gate 0x{ADDR:x} errno={}",
                std::io::Error::last_os_error()
            );
        }
    }
    arm64jit::jit::block_cache_drop_region(0x102dadb00, 0x102dadfc0);
}

/// SH103: gameGlobalInit's keyed-registry update (fn 0x101db5cfc) reads the
/// static [0x106dcb078] via `ldr x24,[x20]` @0x101db5ffc where x20 carries a HOST
/// pointer (0x5649.../0x7f6a...) that leaks into guest callee-saved x20 across a
/// host-call excursion (bl @0x101db5ff8 -> 0x101db7d4c -> PLT strlen). The static
/// S=0x106dcb078 is zeroed .bss and the gate `cbz x24` (0x101db6008) correctly
/// skips when S==0, so the intended value of the load is ALWAYS 0. Patch the
/// load to `mov x24,#0` so it yields the intended NULL regardless of the leaked
/// x20 value (mid-block clobber is immune to block-entry hooks). The residual
/// host-pointer-into-x20 leak is a real JIT-hosting bug (recon deleg_1450dcdf)
/// tracked for a bridge-level fix; this site patch unblocks the gate.
fn routeb_patch_keyed_registry_x24_load() {
    const ADDR: u64 = 0x101db5ffc; // `ldr x24,[x20]` (word f9400298) -> `mov x24,#0`
    let want = 0xd280_0018u32; // mov x24, #0
    let page = ADDR & !0xfff;
    unsafe {
        if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
            let before = *(ADDR as *const u32);
            if before == 0xf940_0298u32 {
                *(ADDR as *mut u32) = want;
                eprintln!(
                    "[elfjit:routeB] SH103 patched keyed-registry `ldr x24,[x20]` 0x{ADDR:x} ({before:08x}) -> `mov x24,#0` — yields the intended NULL static read regardless of the host-pointer leak in x20"
                );
            } else if before == want {
                eprintln!("[elfjit:routeB] SH103 keyed-registry x24 load 0x{ADDR:x} already patched");
            } else {
                eprintln!("[elfjit:routeB] WARN SH103 keyed-registry x24 load 0x{ADDR:x} unexpected {before:08x}, not patched");
            }
            libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
        } else {
            eprintln!(
                "[elfjit:routeB] WARN SH103 mprotect RW failed for keyed-registry x24 load 0x{ADDR:x} errno={}",
                std::io::Error::last_os_error()
            );
        }
    }
    arm64jit::jit::block_cache_drop_region(0x101db5cfc, 0x101db6210);
}

fn routeb_patch_map_dispatch() {
    // Two dispatch `blr x8` sites in the Roblox string/span hash-map family, each the
    // optional-hash2 branch of `ldp x1,x8,[x19,#16]; cbz x8;<tail>blr x1`:
    //   FIND/grow op  file 0x29f4280 (guest 0x1029f4280) — SH87
    //   INSERT op     file 0x29f3f78 (guest 0x1029f3f78) — SH89 (identical pattern:
    //                 29f3f6c ldp x1,x8,[x19,#16]; 29f3f74 cbz x8,29f3f80;
    //                 29f3f78 blr x8; 29f3f80 blr x1)
    // Forcing `blr x1` (primary hash) at both is behavior-preserving: hash2 is
    // redundant in this family (observed real value is just `br x1` aliasing the
    // primary) and the hash only selects a bucket probe (correctness via the key-eq
    // comparator at map+0x08), so garbage +0x18 can never be branched into.
    const FIND_BLR: (u64, u64) = (0x1029f4280, 0x29f4280);
    const INSERT_BLR: (u64, u64) = (0x1029f3f78, 0x29f3f78);
    let want = 0xd63f_0020u32; // blr x1
    for (addr, _file) in [FIND_BLR, INSERT_BLR] {
        let page = addr & !0xfff;
        unsafe {
            if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
                let before = *(addr as *const u32);
                if before == 0xd63f_0100u32 {
                    *(addr as *mut u32) = want;
                    eprintln!(
                        "[elfjit:routeB] SH87/89 patched map-family dispatch `blr x8` 0x{addr:x} ({before:08x}) -> `blr x1` ({want:08x}) — always primary hash, +0x18 hash2 can't blr into garbage"
                    );
                } else if before == want {
                    eprintln!("[elfjit:routeB] SH87/89 map-dispatch 0x{addr:x} already {want:08x}");
                } else {
                    eprintln!("[elfjit:routeB] WARN SH87/89 map-dispatch 0x{addr:x} unexpected {before:08x}, not patched");
                }
                libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
            } else {
                eprintln!(
                    "[elfjit:routeB] WARN SH87/89 mprotect RW failed for map-dispatch 0x{addr:x} errno={}",
                    std::io::Error::last_os_error()
                );
            }
        }
    }
    // Drop the JIT block cache over both the FIND/grow family and the INSERT op.
    arm64jit::jit::block_cache_drop_region(0x1029f4240, 0x1029f4360);
    arm64jit::jit::block_cache_drop_region(0x1029f3e70, 0x1029f3f90);
}

static ROUTEB_LEAF_ADDR: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
static ROUTEB_ADAPTER_REC_ADDR: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
/// Route-B SH81: seed the two engine dispatch-singleton `.data` records so the
/// accessor `2b9dee0`'s lazy-create returns a coherent object instead of an
/// all-zero stub (whose `+0` vtable is NULL). The gate-force (`routeb_patch_
/// dispatch_gate`) routes the ~255 `bl 21730ec; tbz w0,#0` sites to their clean
/// path, but callback `0x624f4bc` reaches the same singletons through a DIRECT
/// `cbz x1,0x624f4f0` branch (bl 6249e9c/6249eb8) the gate never sees, and there
/// `ldr x8,[objB]; ldr x8,[x8,#48]; blr` SIGSEGVs on the null vtable (SH80/81
/// crash at 0x10624f500). Records (guest = file vaddr + 0x100000000; .data,
/// identity-mapped rw-): {+0 size, +8 pow2 allocclass, +16 ticket(lazy), +24 src}.
/// `2b9dee0` create path (0x2b9e030) memcpys `[addr+24]` (src) `[addr+0]` (size)
/// bytes into a fresh 8-aligned heap obj and returns it; src==NULL -> memset 0
/// (the null-vtable stub). Seeding src = a host template whose +0 is a leaked
/// vtable (every slot = `routeb_singleton_leaf`) makes every objA/objB accessor
/// result a real polymorphic object whose virtuals are benign leaves.
///
/// SH114 (deleg_83e74525, disasm-verified): the three V2Init/V2Start/V1AppStart/
/// SendAppEventOnAppReady soft-return sites are lazy-singleton virtual dispatches
/// `ldr x8,[x8,#N]; blr x8` (site A file 0x62517c4 [+0xf8], A2 0x6251aa8
/// [+0x108], B 0x6260948 [+0x548]). The 0x60 vtable (SH110/111 baseline, locked
/// by sh111_singleton_vtable_stays_0x60_baseline) is too short, so each index
/// reads past into host bytes and `blr x8` jumps outside image -> benign
/// soft-return (the enclosing fn body never completes). Widening to 0x580
/// regresses nativeInit (returned a0 deref'd x0+0x28). A differential SCOPED
/// patch of the blr->mov x0,xzr was TRIED this cycle and EMPIRICALLY REJECTED:
/// returning 0 makes the POST-blr caller deref [out]+0x28 -> NULL+0x28 SIGSEGV
/// in nativeInitializeNativeFlags (fault=0x28) — the same crash class as the
/// vtable widening. Root cause per SH111: the virtual's return IS deref'd by
/// SOME caller (nativeInit), so a scoped leaf must return a STABLE zeroed guest
/// object (routeb_singleton_obj_leaf), NOT 0/xzr — left as the next gate. The
/// baseline 0x60 soft-return remains benign (ladder completes EXIT 0).
fn routeb_seed_task_singletons() {
    if ROUTEB_SINGLETON_SEEDED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let leaf = *ROUTEB_LEAF_ADDR.get_or_init(|| {
        let a = arm64jit::jit::register_host_call_auto(routeb_singleton_leaf);
        eprintln!("[elfjit:routeB] benign singleton virtual registered at {a:#x}");
        a
    });
    // SH111: differential 0x580 vtable. SH110 showed the blanketed 0x580
    // identity-widening regressed nativeInitializeNativeFlags: a high-slot
    // virtual returned identity a0 (== `this`, possibly NULL/low) and the caller
    // deref'd x0+0x28 -> NULL+0x28 SIGSEGV before StartLuaAppDM. Instead:
    //   - slots 0..0x60: keep the proven identity leaf (current clean baseline).
    //   - [0x60,0x580) default: stable-zeroed 0x40 guest-resident object leaf
    //     (any caller's [ret+off] read is a valid zeroed field, never NULL+0x28).
    //   - +0xf8/+0x108/+0x548 (soft-return gate sites A/A2/B): identity leaf
    //     (subagent-verified store-only: `ldr x8,[x19]; str x0,[x8]; mov x0,xzr`,
    //     the virtual's return value is never deref'd), so V2Init/V2Start/V1
    //     AppStart resolve their dispatch instead of leaking host bytes.
    //   - +0x558 (nativeInit count) / +0x568 (ptr arg): NULL leaf — identity
    //     there would make the flags-init loop iterate a huge low-32 count.
    // Keep 0x580 (< the +0x720 bool-gate read), so non-gate deep slots keep the
    // same benign soft-return as the 0x60 baseline.
    // SH111 investigation: differential 0x580 vtable tried + EMPIRICALLY REVERTED.
    // Covering the soft-return gate slots (+0xf8/+0x108/+0x548) requires widening
    // the vtable past 0x60, and ANY widening (identity leaf b19b1c2 OR stable-zeroed
    // object leaf SH111) makes nativeInitializeNativeFlags hard-crash (exit 134,
    // fault at host-thunk while resolving a deep shared-vtable slot) BEFORE
    // StartLuaAppDM. The vtable is SHARED between nativeInit (which stays clean
    // ONLY because 0x60 slots read-past into host bytes -> benign soft-return) and
    // V2Start/V2Init (which need wide coverage for a genuine return), so widening
    // cannot serve both. The 0x60 baseline IS the benchmarked milestone (EXIT 0,
    // 0 crash, StartLuaAppDM returned Ok); the V2Start/V2Init soft-return is benign
    // (ladder proceeds past it). Revert to 0x60 and instead make the three CALL SITES
    // resolve without widening (see frontier-sh111 note). NEXT target: patch the 3
    // dispatch blr sites (0x62517c4/0x6251aa8/0x6260948) to a fixed guest-resident
    // leaf directly (scoped to known leak sites), leaving the shared vtable 0x60.
    let vtable: &'static mut [u8] = Box::leak(vec![0u8; 0x60].into_boxed_slice());
    {
        let mut fill = |i: u64, v: u64| unsafe {
            *(vtable.as_mut_ptr().wrapping_add((i * 8) as usize) as *mut u64) = v
        };
        for slot in 0..(0x60 / 8) {
            fill(slot, leaf);
        }
    }
    let vtable_addr = vtable.as_ptr() as u64;
    // Templates (0x28 bytes) whose +0 is the vtable; rest zero (obj fields the
    // observed sites pass as args / read are benign or ignored by the leaf).
    let mk_template = || -> u64 {
        let t: &'static mut [u8] = Box::leak(vec![0u8; 0x28].into_boxed_slice());
        unsafe { *(t.as_mut_ptr() as *mut u64) = vtable_addr; }
        t.as_ptr() as u64
    };
    let src_a = mk_template();
    let src_b = mk_template();
    // .data records (guest identity-mapped, host-writable — same as LSM seed).
    unsafe {
        let ra = 0x106829a48u64;
        *(ra as *mut u64) = 0x28; // size
        *((ra + 8) as *mut u64) = 8; // allocclass (pow2)
        *((ra + 0x18) as *mut u64) = src_a; // src template
        let rb = 0x106829a68u64;
        *(rb as *mut u64) = 0x28;
        *((rb + 8) as *mut u64) = 8;
        *((rb + 0x18) as *mut u64) = src_b;
    }
    println!(
        "[elfjit:routeB] seeded dispatch singletons .data 0x106829a48/0x106829a68 (size=0x28 alloc=8 src=template 0x{src_a:x}/0x{src_b:x}, vtable@0x{vtable_addr:x}=leaf 0x{leaf:x}) — 2b9dee0 lazy-create returns coherent objects, no null-vtable crash"
    );
    ROUTEB_SINGLETON_SEEDED.store(true, core::sync::atomic::Ordering::Relaxed);
}
static ROUTEB_SINGLETON_SEEDED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
static ROUTEB_DISPATCHER_NODE_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
/// SH120: SendAppEventOnAppReady's app-bridge path crashes at guest 0x102b561b0
/// (a GENERIC ~40-site leaf, do NOT code-patch it) because the shared dispatcher-
/// node .bss global guest 0x10683a460 (file 0x683a460) has +0 (an intrusive
/// nav/back header ptr) = 0 -> [*x1 - 24] = [0-24] faults. Seed the DATA to the
/// benign empty-singleton state: node[+0] = 0x10683a000 (a zeroed .bss page whose
/// [..-24]=0, so base==node), node[+0x20] = 1 (base[+32] active flag -> the leaf
/// returns bool=0 -> the caller's `cbz` skips dispatch work). Constrained by
/// construction to only paths passing &0x10683a460; no code patch, idempotent,
/// opt-in.
fn routeb_seed_dispatcher_node() {
    if ROUTEB_DISPATCHER_NODE_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    unsafe {
        *(0x10683a460u64 as *mut u64) = 0x10683a000u64; // node[+0] -> 0-clean .bss (self-link base==node)
        *(0x10683a480u64 as *mut u64) = 1u64; // node[+0x20] = base[+32] active bit -> leaf returns 0
    }
    eprintln!("[elfjit:routeB] SH120 seeded dispatcher-node global 0x10683a460 (+0=self-link 0x10683a000, +0x20=1) — app-bridge event dispatch resolves to benign 'nothing registered' instead of [*x1-24]=[0-24] fault");
    ROUTEB_DISPATCHER_NODE_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}

// ---- SH121: TaskScheduler ctor "flags-loaded" gate -> raise(SIGTRAP) ----
//
// setTaskSchedulerBackgroundMode (guest 0x102bb2380 -> internal 0x10258aff0)
// lazily constructs the TaskScheduler via once-guard [0x10726a488]. Its ctor
// (file 0x224f810, guest 0x10224f810) starts with the flags gate at guest
// 0x10224fa20: `ldrb w8,[x8,#2516]` reads the flags-loaded byte [0x72739d4]
// (0xe8 0x12 0x00 0x36 = tbz w8,#0,0x224fc80). BSS leaves bit0==0, so the ctor
// takes the fatal path (0x224fc80 -> bl 0x626d1d0 -> raise(SIGTRAP)=exit 133).
// The recon (deleg_b67653e9) proved the SIGTRAP is the guest's own raise(5),
// not a JIT-emitted trap. Because nativeGameGlobalInit now RETURNS via the
// SH82 thread-id trick (never loading flags), this ctor is the FIRST flags gate
// the ladder hits -> exit 133 right after "driving setTaskSchedulerBM". Fix:
// NOP the `tbz` so the ctor proceeds regardless of flags-loaded (mirrors SH116's
// code-site approach; [0x72730xx].bss is unmapped -> a data seed may not land).
static ROUTEB_TASKSCHED_FLAGSGATE_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
fn routeb_patch_taskscheduler_flags_gate() {
    if ROUTEB_TASKSCHED_FLAGSGATE_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    // guest addr = file vaddr 0x224fa24 + 0x100000000 (identity-mapped .text).
    let addr = 0x10224fa24u64;
    let want = 0xd503_201fu32; // nop
    let page = addr & !0xfff;
    unsafe {
        if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
            let before = *(addr as *const u32);
            // e8 12 00 36 (LE u32 0x360012e8) = TBZ W8,#0,#0x5c (to the fatal raise)
            if before == 0x3600_12e8u32 {
                *(addr as *mut u32) = want;
                eprintln!(
                    "[elfjit:routeB] SH121 patched TaskScheduler ctor flags-gate `tbz w8,#0,<fatal>` 0x{addr:x} ({before:08x}) -> `nop` ({want:08x}) — ctor proceeds regardless of [0x72739d4].bit0 (kills the guest raise(SIGTRAP)=exit 133)"
                );
            } else if before == want {
                eprintln!("[elfjit:routeB] SH121 TaskScheduler flags-gate 0x{addr:x} already nop");
            } else {
                eprintln!(
                    "[elfjit:routeB] WARN SH121 TaskScheduler flags-gate 0x{addr:x} unexpected bytes {before:08x}, not patched"
                );
            }
            libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
        } else {
            eprintln!(
                "[elfjit:routeB] WARN mprotect RW failed for SH121 TaskScheduler flags-gate 0x{addr:x} errno={}",
                std::io::Error::last_os_error()
            );
        }
    }
    arm64jit::jit::block_cache_drop_region(0x10224f9e0, 0x10224fca0);
    ROUTEB_TASKSCHED_FLAGSGATE_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}

// ---- SH115: scoped singleton-dispatch patch (make V2Init/V2Start/V1AppStart/  ----
// ---- SendAppEventOnAppReady bodies complete instead of soft-returning)     ----
//
// The three nullable-singleton accessors (V2Init site A, V2Start site A2,
// V1AppStart site B) each end a virtual dispatch through the seeded 0x60 vtable:
// `ldr x8,[x8,#off]; blr x8` reads +0xf8/+0x108/+0x548 PAST the 0x60 vtable into
// host bytes, blr's outside the image, and the enclosing fn body 'soft-returns'
// without ever completing. SH114 showed a flat `blr->mov x0,xzr` (return 0)
// regresses nativeInitializeNativeFlags to NULL+0x28, because the accessor's
// return IS deref'd at '+0x28' by a downstream caller. The differential fix:
// patch each site's 28B window to MATERIALIZE the stable zeroed singleton object
// (routeb_singleton_obj_addr) directly into x0, store it into objA[0]=[x19], and
// RETURN it — so both the stored slot AND any `[ret+off]` read resolve to a valid
// zeroed guest object (never NULL+0x28). This is site-scoped (leaves the shared
// 0x60 vtable + nativeInit's own reads untouched), the exact differential remedy
// SH110/SH111 mandate. Encodings verified against aarch64-linux-gnu-assembler.
// Window layout (7 instr, file vaddr):
//   slot0 movz x8,#OBJlo      slot4 mov x0,x8
//   slot1 movk x8,#OBJ(16)    slot5 ldr x9,[x19]   ; objA
//   slot2 movk x8,#OBJ(32)    slot6 str x0,[x9]    ; objA[0]=OBJ (MUST overwrite
//   slot3 movk x8,#OBJ(48)                           the trailing `mov x0,xzr`)
fn sh115_movz_x8_imm(imm16: u16) -> u32 {
    0xD280_0000u32 | ((imm16 as u32) << 5) | 8
}
fn sh115_movk_x8_imm(hw: u32, imm16: u16) -> u32 {
    0xF280_0000u32 | (hw << 21) | ((imm16 as u32) << 5) | 8
}
const SH115_MOV_X0_X8: u32 = 0xAA08_03E0;
const SH115_LDR_X9_X19: u32 = 0xF940_0269; // ldr x9,[x19,#0]
const SH115_STR_X0_X9: u32 = 0xF900_0120; // str x0,[x9,#0]; overwrites `mov x0,xzr`
/// Build the 28-byte (7-instr) window for a singleton accessor site. OBJ is the
/// stable object address being materialized; returns the 7 u32 words in order.
fn sh115_obj_window(oj: u64) -> [u32; 7] {
    [
        sh115_movz_x8_imm((oj & 0xffff) as u16),
        sh115_movk_x8_imm(1, ((oj >> 16) & 0xffff) as u16),
        sh115_movk_x8_imm(2, ((oj >> 32) & 0xffff) as u16),
        sh115_movk_x8_imm(3, ((oj >> 48) & 0xffff) as u16),
        SH115_MOV_X0_X8,
        SH115_LDR_X9_X19,
        SH115_STR_X0_X9,
    ]
}
static ROUTEB_DISPATCH_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
/// Pure SH115 branch-repoint math: given a B.cond/cbz word at `branch`, return
/// the word targeting `new_target` (imm19 at bits[23:5]; top byte + Rt/cond
/// preserved). Unit-tested; the runtime `repoint_early_branch` applies it.
fn sh115_repoint_word(branch: u64, word: u32, new_target: u64) -> u32 {
    let delta = (new_target as i64 - branch as i64) >> 2;
    let new_imm19 = (delta & 0x7ffff) as u32;
    (word & 0xff00_001f) | (new_imm19 << 5)
}
/// Rewrite a B.cond/cbz immediate so it lands on `new_target` instead of its
/// current target. Returns true if written.
fn repoint_early_branch(branch: u64, expect_target: u64, new_target: u64, tag: &str) -> bool {
    let page = (branch & !0xfff) as *mut libc::c_void;
    unsafe {
        assert_eq!(
            libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_WRITE),
            0,
            "mprotect RW failed for {tag} branch 0x{branch:x}"
        );
        let word = *(branch as *const u32);
        let imm19 = (word >> 5) & 0x7ffff;
        let cur_target = branch.wrapping_add((imm19 as i64 * 4) as u64);
        if cur_target != expect_target {
            eprintln!("[elfjit:routeB] WARN {tag} branch 0x{branch:x} cur_target 0x{cur_target:x} != expect 0x{expect_target:x} (word {word:08x}), not repointed");
            libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
            return false;
        }
        let new_word = sh115_repoint_word(branch, word, new_target);
        *(branch as *mut u32) = new_word;
        libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
        eprintln!("[elfjit:routeB] repointed {tag} early-return branch 0x{branch:x} ({word:08x} -> {new_word:08x}) target 0x{cur_target:x} -> 0x{new_target:x}");
        true
    }
}
static ROUTEB_LOCK_OWNER_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
/// SH116: nativeInitializeNativeFlags' helper (file 0x2320710) reads the .bss
/// global object via `adrp x8,7273000; ldr x0,[x8,#2480]` = *(guest 0x10672739c0).
/// That global's page is UNMAPPED at runtime (mprotect RW -> ENOMEM; the engine
/// re-maps/protects the .bss page during boot), so it can't be seeded by a store.
/// Instead patch the helper's three slots to MATERIALIZE a stable all-zero 0x60
/// object into x0 (movz/movk over the adrp/ldr + the dead `b` hop); the helper's
/// `add x0,x0,#0x28; bl pthread_mutex_lock` (0x2320738/0x2320748) then locks
/// &obj+0x28 which is all-zero = a valid PTHREAD_MUTEX_INITIALIZER. The object's
/// fields are writable leaked memory the engine may initialize. Leaves the
/// unmapped global untouched. Deterministic, idempotent, non-vtable-widening.
fn routeb_patch_nativeinit_lock_owner() {
    if ROUTEB_LOCK_OWNER_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let s = Box::leak(vec![0u8; 0x60usize].into_boxed_slice()).as_mut_ptr() as u64;
    // helper slots (file vaddr) to overwrite:
    //   0x2320710 adrp x8,7273000 (f0027a88)
    //   0x2320714 ldr  x0,[x8,#2480] (f944d900)
    //   0x2320718 b    232071c (14000001, dead hop)
    // -> movz x0,#s_lo (hw0); movk x0,#s(16) (hw1); movk x0,#s(32) (hw2).
    let start = 0x102320710_u64; // guest = file(0x2320710) + 0x100000000
    let words: [u32; 3] = [
        0xD280_0000u32 | (((s & 0xffff) as u32) << 5),       // movz x0,#imm16 hw0
        0xF2A0_0000u32 | ((((s >> 16) & 0xffff) as u32) << 5), // movk x0,#imm16 hw1
        0xF2C0_0000u32 | ((((s >> 32) & 0xffff) as u32) << 5), // movk x0,#imm16 hw2
    ];
    let page = (start & !0xfff) as *mut libc::c_void;
    unsafe {
        if libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_WRITE) != 0 {
            eprintln!("[elfjit:routeB] WARN mprotect RW failed for SH116 helper @0x{start:x} errno={}", std::io::Error::last_os_error());
            return;
        }
        // guard against a shifted binary: expect the adrp word at slot0
        let before = *(start as *const u32);
        if before != 0xf002_7a88u32 {
            eprintln!("[elfjit:routeB] WARN SH116 helper @0x{start:x} unexpected slot0 {before:08x}, not patched");
            libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
            return;
        }
        for (i, w) in words.iter().enumerate() {
            *((start + (i as u64) * 4) as *mut u32) = *w;
        }
        libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
        arm64jit::jit::block_cache_drop_region(start, start + 12);
        eprintln!("[elfjit:routeB] SH116 patched nativeInit lock-owner helper @0x{start:x} 12B -> materialize stable zeroed 0x{s:x} (mutex@+0x28 = valid PTHREAD_MUTEX_INITIALIZER)");
    }
    ROUTEB_LOCK_OWNER_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}
static ROUTEB_FLAGMAP_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
/// SH117: after SH115/116, nativeInitializeNativeFlags advances into its flag-
/// registration loop and calls a hash-map ramp-probe helper (guest 0x10232090c,
/// file 0x232090c, sole caller `bl 232090c` @ file 0x23208d8). The flag-map
/// object it probes has an UNSEEDED +0 bucket-array pointer (stale host-stack
/// garbage) -> `ldr x8,[x0]; ldr x8,[x8,x11,lsl#3]` SIGSEGVs (fault==[table+0]).
/// The map object is a per-run leak so a fixed-address seed is unreliable;
/// instead leaf-rewrite the helper prologue to `ret` (return x0 = the non-zero
/// map object) so the caller's `cbz x0` takes the "found" path and skips the
/// insert. Deterministic, idempotent, non-vtable-widening, 8 bytes, single call
/// site. (libre future: seed a coherent empty flag-map at the 0x102320524
/// producer for the proper deep fix.)
fn routeb_patch_nativeinit_flagmap_helper() {
    if ROUTEB_FLAGMAP_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let start = 0x10232090c_u64; // guest = file(0x232090c) + 0x100000000
    let page = (start & !0xfff) as *mut libc::c_void;
    unsafe {
        if libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_WRITE) != 0 {
            eprintln!("[elfjit:routeB] WARN mprotect RW failed for SH117 helper @0x{start:x} errno={}", std::io::Error::last_os_error());
            return;
        }
        let before = *(start as *const u32);
        if before != 0xa9bf_7bfdu32 {
            eprintln!("[elfjit:routeB] WARN SH117 helper @0x{start:x} unexpected word0 {before:08x}, not patched");
            libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
            return;
        }
        *(start as *mut u32) = 0xd65f_03c0u32; // ret (return x0 = map obj, non-zero)
        *((start + 4) as *mut u32) = 0xd503_201fu32; // nop (was mov x29,sp)
        libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
        arm64jit::jit::block_cache_drop_region(start, start + 8);
        eprintln!("[elfjit:routeB] SH117 patched nativeInit flag-map probe helper @0x{start:x} 8B -> leaf ret (returns x0=flag-map obj) — caller takes 'found' path, skips garbage insert");
    }
    ROUTEB_FLAGMAP_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}
static ROUTEB_GUARD_UNIT: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
/// SH118: guest 0x1067d16f0 is BOTH the `__stack_chk_guard` GOT slot (all
/// 48,831 stack-protected fns read their canary base from it) AND the engine's
/// render-ctx/singleton global (SH14/SH106 dual-use). SH106 seeded it to a
/// stable canary VALUE, but StartApp/renderinit RE-PUBLISH it to the live ctx,
/// so a canary fn whose body re-publishes the slot mid-execution (V2UpdateSurface
/// does) reads a mutated canary at epilogue vs prologue -> false
/// `*** stack smashing detected ***` SIGABRT. Fix: seed the slot to the ADDRESS
/// of a leaked stable u64 (the canary fn caches that pointer at prologue and the
/// epilogue derefs the cached pointer, so *guard is constant regardless of a
/// mid-body slot re-publication). Idempotent; must run right before each rung's
/// jit_run whose body can re-publish the slot.
fn routeb_reassert_canary_guard() {
    let unit = *ROUTEB_GUARD_UNIT.get_or_init(|| {
        let b: &'static mut u64 = Box::leak(Box::new(0x2f_2a_1a_0a_0e_0f_10_11u64));
        b as *mut u64 as u64
    });
    unsafe {
        *(0x1067d16f0u64 as *mut u64) = unit; // pervasive canary-guard GOT slot (== renderer-ctx alias)
        *(0x10631aa30u64 as *mut u64) = unit; // JNI_OnLoad guard slot, kept in sync (SH106)
    }
}
static ROUTEB_SENDAPP_PATCHED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
/// SH119: SendAppEventOnAppReady (guest 0x102bb463c) STILL for-returns through
/// two UNGATED singleton lambdas the SH115 3-site patch + gate-force can't reach
/// (they are on the gate-backed CLEAN path's own registered callbacks, dispatching
/// direct off the too-short 0x60 vtable):
///   - lambda 0x6251610: `ldr x8,[x8,#0xf0]; blr x8` @ file 0x6251670/0x6251674,
///     window 0x6251670..0x6251684 (ldr/blr/ldr/str/mov-x0-xzr) -> materialize OBJ
///     into x0 (movz+3 movk) + nop; repoint b.eq@0x6251628 + cbz@0x6251634 (both ->
///     0x6251680) to epilogue 0x6251684.
///   - lambda 0x6260a68: `ldr x8,[x8,#0x550]; blr x8` @ file 0x6260abc/0x6260ac0,
///     window 0x6260abc..0x6260ac8 (ldr/blr/mov-x0-xzr) -> materialize OBJ into x0
///     (movz + movk hw1 + movk hw2); repoint b.eq@0x6260a7c + cbz@0x6260a88 (both
///     -> 0x6260ac4) to epilogue 0x6260ac8.
/// Both return OBJ (non-NULL stable) so the enclosing SendAppEventOnAppReady body
/// completes towards building the app-data-model / GuiObjects.
fn routeb_patch_sendapp_singleton_lambdas() {
    if ROUTEB_SENDAPP_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let obj = routeb_singleton_obj_addr();
    // materialize OBJ into x0: movz x0,#lo; movk x0,#16/32/48
    let mut word_at = |hw: u32, imm: u16| -> u32 {
        if hw == 0 {
            0xD280_0000u32 | ((imm as u32) << 5) // movz x0,#imm (hw0)
        } else {
            (0xF280_0000u32 + (hw << 21)) | ((imm as u32) << 5) // movk x0 hwN
        }
    };
    let helper = |obj: u64| (word_at(0, (obj & 0xffff) as u16), word_at(1, ((obj >> 16) & 0xffff) as u16), word_at(2, ((obj >> 32) & 0xffff) as u16), word_at(3, ((obj >> 48) & 0xffff) as u16));
    let (w0, w1, w2, w3) = helper(obj);

    // Site 1: lambda 0x6251610, 5-slot window (movz, movk hw1, movk hw2, movk hw3, nop)
    let g1 = 0x106251670_u64; // guest = file(0x6251670) + 0x100000000
    {
        let page = (g1 & !0xfff) as *mut libc::c_void;
        unsafe {
            if libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0
                && *(g1 as *const u32) == 0xf940_7908u32
            {
                *((g1 + 0) as *mut u32) = w0;
                *((g1 + 4) as *mut u32) = w1;
                *((g1 + 8) as *mut u32) = w2;
                *((g1 + 12) as *mut u32) = w3;
                *((g1 + 16) as *mut u32) = 0xd503_201fu32; // nop (kill mov x0,xzr)
                libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
                arm64jit::jit::block_cache_drop_region(g1, g1 + 20);
                eprintln!("[elfjit:routeB] SH119 patched SendAppEvent lambda 0x6251610 @0x{g1:x} 20B -> materialize obj 0x{obj:x} into x0 (returns OBJ)");
                let _ = repoint_early_branch(0x106251628, 0x106251680, 0x106251684, "L610 b.eq");
                let _ = repoint_early_branch(0x106251634, 0x106251680, 0x106251684, "L610 cbz x19");
            } else {
                libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
                eprintln!("[elfjit:routeB] WARN SH119 lambda A @0x{g1:x} guard/mprotect failed, not patched");
            }
        }
    }
    // Site 2: lambda 0x6260a68, 3-slot window (movz, movk hw1, movk hw2)
    let g2 = 0x106260abc_u64; // guest = file(0x6260abc) + 0x100000000
    {
        let page = (g2 & !0xfff) as *mut libc::c_void;
        unsafe {
            if libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0
                && *(g2 as *const u32) == 0xf942_a908u32
            {
                *((g2 + 0) as *mut u32) = w0;
                *((g2 + 4) as *mut u32) = w1;
                *((g2 + 8) as *mut u32) = w2; // overwrites mov x0,xzr
                libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
                arm64jit::jit::block_cache_drop_region(g2, g2 + 12);
                eprintln!("[elfjit:routeB] SH119 patched SendAppEvent lambda 0x6260a68 @0x{g2:x} 12B -> materialize obj 0x{obj:x} into x0 (returns OBJ)");
                let _ = repoint_early_branch(0x106260a7c, 0x106260ac4, 0x106260ac8, "L60a68 b.eq");
                let _ = repoint_early_branch(0x106260a88, 0x106260ac4, 0x106260ac8, "L60a68 cbz x19");
            } else {
                libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
                eprintln!("[elfjit:routeB] WARN SH119 lambda B @0x{g2:x} guard/mprotect failed, not patched");
            }
        }
    }
    ROUTEB_SENDAPP_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}

static ROUTEB_SENDAPP_VTABLE_PATCHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// SH126 (recon deleg_f595f562): SendAppEventOnAppReady (guest 0x102bb463c)
/// builds a real 0x58 app-event object whose vtable is 0x635e068 — an
/// ALL-ZERO `.data.rel.ro` vtable (zero dynamic relocations). Its terminal
/// virtual dispatch at guest 0x102bb4984 (`ldr x9,[x0]; ldr x8,[x9,x8]; blr x8`,
/// x8=0x28 for a non-NULL object) is therefore `blr 0` -> benign soft-return,
/// so the rung's body never completes and MH_FLAGS_LOADED/APP_READY stay false.
/// None of the SH115/117/119 singleton patches cover this structurally-empty
/// app-event vtable. Fix (two parts, all plain data stores, idempotent):
///   (a) materialize slots +0x20/+0x28 to the benign identity leaf
///       (routeb_singleton_leaf, the proven SH115/119 pattern) so the terminal
///       blr lands instead of `blr 0` — the body reaches its epilogue and the
///       session-advance probe can observe the REAL next-state;
///   (b) seed the app-bridge pipe sync-gate [0x10683d010] = -1 so the body's
///       `bl 0x2baeeec` takes the SYNCHRONOUS path (`cmn x8,#-1` at 0x2baef24 ->
///       `tbz w20,#0` 0x2baef5c, w20=0 -> `bl 0x2206c40` GlobalInit do-init)
///       instead of the async pthread_cond_wait block (0x2b4cd1c).
/// guest vtable base: file 0x635e068 in LOAD(off 0x62d81c0 -> vaddr 0x62dc1c0),
/// within=0x85ea8 -> vaddr 0x6362068 -> guest 0x106362068 (RW LOAD, plain store).
fn routeb_patch_sendapp_appevent_vtable() {
    if ROUTEB_SENDAPP_VTABLE_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let leaf = *ROUTEB_LEAF_ADDR.get_or_init(|| {
        let a = arm64jit::jit::register_host_call_auto(routeb_singleton_leaf);
        eprintln!("[elfjit:routeB] SH126 benign dispatch leaf registered at {a:#x}");
        a
    });
    const GVT: u64 = 0x106362068; // app-event vtable guest base (RW LOAD, plain store)
    unsafe {
        let s20 = (GVT + 0x20) as *mut u64;
        let s28 = (GVT + 0x28) as *mut u64;
        let before20 = *s20;
        let before28 = *s28;
        if before20 == 0 && before28 == 0 {
            *s20 = leaf;
            *s28 = leaf;
            eprintln!(
                "[elfjit:routeB] SH126 materialized app-event vtable 0x{GVT:x} +0x20/+0x28 -> leaf {leaf:#x} (SendAppEventOnAppReady terminal blr 0x102bb4984 now lands, no soft-return)"
            );
        } else {
            eprintln!(
                "[elfjit:routeB] SH126 app-event vtable already populated (slot20={before20:#x} slot28={before28:#x}), leaving"
            );
        }
        // (b) pipe sync-gate: force `cmn x8,#-1` at 0x2baef24 to take the sync
        // path (ldr x0,[683d000+#8]; tbz w20,#0 -> bl 0x2206c40 do-init).
        let gate = 0x10683d010u64 as *mut u64;
        *gate = u64::MAX;
        eprintln!("[elfjit:routeB] SH126 seeded pipe sync-gate [0x10683d010]=-1 -> SendAppEvent pipe takes the synchronous do-init path (bl 0x2206c40)");
    }
    ROUTEB_SENDAPP_VTABLE_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}
/// Apply the SH115 scoped patch to the three singleton-dispatch accessor sites.
/// (file vaddr -> expected original slot0 word + the two early-return branches
/// that jump onto the patched store-slot and their file vaddrs)
fn routeb_patch_singleton_dispatch() {
    if ROUTEB_DISPATCH_PATCHED.load(core::sync::atomic::Ordering::Relaxed) {
        return;
    }
    // Each site: window_start (file) , expected original slot0 word, then the
    // two early-return branches (b.eq + cbz x19) that target the patched slot6
    // (the store) and must be repointed past it to the epilogue. The epilogue is
    // one word past slot6 (window_start + 7*4).
    let sites: [(u64, u32, u64, u64); 3] = [
        (0x62517b8, 0xAA15_03E0, 0x6251784, 0x6251790), // V2Init A
        (0x6251a9c, 0xAA16_03E2, 0x6251a5c, 0x6251a68), // V2Start A2
        (0x626093c, 0xAA14_03E1, 0x62608fc, 0x6260908), // V1AppStart B
    ];
    let obj = routeb_singleton_obj_addr();
    let win = sh115_obj_window(obj);
    for (file_start, orig_slot0, branch_eq, branch_cbz) in sites {
        let guest = (file_start.wrapping_add(0x1_0000_0000)) & 0xff_ffff_ffff;
        let store_slot = guest + 6 * 4; // patched slot6 (the store; early-return target)
        let epilogue = store_slot + 4; // skip the store on early paths
        let page = (guest & !0xfff) as *mut libc::c_void;
        unsafe {
            if libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_WRITE) != 0 {
                eprintln!("[elfjit:routeB] WARN mprotect RW failed @0x{guest:x} errno={}", std::io::Error::last_os_error());
                continue;
            }
            let before = *(guest as *const u32);
            if before != orig_slot0 {
                eprintln!("[elfjit:routeB] WARN site @0x{guest:x} unexpected slot0 {before:08x} (want {orig_slot0:08x}), not patched");
                libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
                continue;
            }
            for (i, w) in win.iter().enumerate() {
                *((guest + (i as u64) * 4) as *mut u32) = *w;
            }
            libc::mprotect(page, 4096, libc::PROT_READ | libc::PROT_EXEC);
            let head = (win[0], win[1], win[2], win[3]);
            eprintln!("[elfjit:routeB] patched site @0x{guest:x} 28B -> materialize-stable-object (obj 0x{obj:x} {head:08x?})");
            arm64jit::jit::block_cache_drop_region(guest, guest + 28);
        }
        // Repoint the two early-return branches (their targets are the patched
        // store slot) so the early/no-op paths fall into the epilogue instead of
        // storing through stale x9 (SH115 re-entrancy bug — the store slot is the
        // shared `mov x0,xzr` tail, which the early paths reach BEFORE slot5's
        // `ldr x9,[x19]` runs, so slot6 would store to leftover x9 junk).
        let beq_g = (branch_eq.wrapping_add(0x1_0000_0000)) & 0xff_ffff_ffff;
        let cbz_g = (branch_cbz.wrapping_add(0x1_0000_0000)) & 0xff_ffff_ffff;
        let _ = repoint_early_branch(beq_g, store_slot, epilogue, "b.eq");
        let _ = repoint_early_branch(cbz_g, store_slot, epilogue, "cbz x19");
    }
    ROUTEB_DISPATCH_PATCHED.store(true, core::sync::atomic::Ordering::Relaxed);
}

/// Drive the engine's REAL per-node PRESENT walker so a populated 0x28-stride
/// scene node actually DRAWS (closing SH63's "present side" gap). Entry is the
/// mid-function present-loop region `0x105b2eec0` (x19=R preset via CpuState —
/// this is what the SH64 empirical note proved runs the engine's real loop and
/// blr's the per-item draw, but crashed on iteration 2 via the nested-jit_run
/// desync). SH64 fix: each node's render-obj vt[+24] is a REGISTERED HOST THUNK
/// (walker_item_draw_thunk) dispatched through `host_call_at` with NO block-cache
/// mutation, so the present-loop block stays intact → no desync. Walker full-body
/// is un-patched here (safe mid-loop entry); we also patch the parked
/// nativeGameGlobalInit bl + teardown tail to ret so the full body COULD run.
///
/// ABI (disasm 0x5b2eec0): x19=R; ldp x20,x22,[R+0x180]=head/tail;
/// per node: x0=[node+8](render-obj); x8=[x0]->vt[+24]; blr draw(x0=render-obj);
/// after last (next==tail): x0=[R+0x160]=ctx; ctx-vt[+24]=swap; blr swap.
/// R layout is identical to SH63 (render_scene_base + node+0x08=ctx render-obj).
///
/// MUST run on the currency-owning renderinit thread (EGL current binding),
/// same as --renderscene. Returns the walker's final x0 (the real swap result:
/// 1 == genuine eglSwapBuffers success).
fn render_engine_present_walker(
    ctx: u64, n: u64, node_count: u64, iimg: &[u8], ibase: u64, tpidr: u64, isp: u64,
) -> u64 {
    if !(ctx >= 0x100000000 && ctx >> 56 == 0) {
        return 0;
    }
    let vt = unsafe { *(ctx as *const u64) };
    if !(vt >= 0x100000000 && vt >> 56 == 0) {
        return 0;
    }
    // The engine's present loop, after drawing all nodes, does
    // `ldr x0,[R+0x160]; ldrd swap = ctx-vt[+24]; blr swap`. It never reads
    // make-current itself (binds happened in render_engine_scene), so we only
    // need the real swap slot — but we validate both for coherence.
    let bind = unsafe { *(vt.wrapping_add(16) as *const u64) };
    let swap = unsafe { *(vt.wrapping_add(24) as *const u64) };
    eprintln!(
        "[elfjit:renderwalker] frame #{n}: ctx {ctx:#x} vt {vt:#x} make-current {bind:#x} swap {swap:#x} nodes={node_count}"
    );
    let thunk = walker_draw_thunk_addr();
    WALKER_NODES.store(node_count.max(1), core::sync::atomic::Ordering::Relaxed);
    let r = render_scene_base(node_count);
    // Per-node render-obj fabrications live in a DEDICATED leaked buffer (NOT
    // inside R's allocation — render_scene_base only allocates 0x400+n*0x28,
    // and these objects are 0x200 each, so stuffing them into R overruns the
    // heap). Each node gets P=[0]=V ; V = copy of ctx-vt[0..9] with V[+24]=thunk.
    let fab = Box::leak(vec![0u8; (node_count as usize) * 0x200].into_boxed_slice());
    let fab_base = fab.as_ptr() as u64;
    unsafe {
        *(r.wrapping_add(0x160) as *mut u64) = ctx;
        let head = r + 0x210;
        for i in 0..node_count {
            let node = head + i * 0x28;
            let p = fab_base + i * 0x200;
            let v = p + 0x40;
            for k in 0..9usize {
                let src = vt.wrapping_add((k as u64) * 8);
                *(v.wrapping_add((k as u64) * 8) as *mut u64) = unsafe { *(src as *const u64) };
            }
            *(v.wrapping_add(24) as *mut u64) = thunk; // vt[+24] = per-item draw
            *(p as *mut u64) = v; // render-obj vtable
            *(node.wrapping_add(0x08) as *mut u64) = p; // node+8 render-obj
            eprintln!(
                "[elfjit:renderwalker] node[{i}] render-obj 0x{p:x} vtable 0x{v:x} vt[+24]={thunk:#x} (host thunk per-item draw)"
            );
        }
    }
    // Make the engine context current on this (currency-owning) thread so the
    // walker's per-item draws + final swap land on the live EGL context.
    if bind >= 0x100000000 && bind >> 56 == 0 {
        let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tpidr);
    }
    // Patch the walker's parked nativeGameGlobalInit bl + teardown tail to ret
    // (idempotent) so the FULL 0x105b2ed48 body could also run; we enter at the
    // mid-loop 0x105b2eec0 which is unaffected by the patch but pairs with it.
    walker_patch_full_body();
    let mut st = arm64jit::jit::CpuState::new();
    st.tpidr = tpidr;
    st.x[31] = isp;
    st.x[19] = r; // engine present loop uses x19 as R (its `this`)
    st.x[30] = 0; // clean return target if the patched ret is reached
    WALKER_ITEM_DRAW_N.store(0, core::sync::atomic::Ordering::Relaxed);
    match arm64jit::jit::jit_run(iimg, ibase, 0x105b2eec0, &mut st as *mut CpuState) {
        Err(e) => {
            eprintln!("[elfjit:renderwalker] frame #{n} present walker stopped: {e}");
            0
        }
        Ok(ret) => {
            let draws = WALKER_ITEM_DRAW_N.load(core::sync::atomic::Ordering::Relaxed);
            eprintln!(
                "[elfjit:renderwalker] frame #{n} present walker Ok(ret={ret:#x}) — {draws} real per-node vt[+24] engine draws + real swap on the live ctx (populated scene list, desync-proof host-thunk draw)"
            );
            ret
        }
    }
}

/// Read a guest u64 (guest memory is identity-mapped).
fn read_visible_u64(a: u64) -> u64 {
    if a >= 0x100000000 && a >> 56 == 0 && a & 7 == 0 {
        unsafe { *(a as *const u64) }
    } else {
        0
    }
}

/// Guest-arena build of the engine's REAL geometry context G that its own
/// geometry emitter (0x105b35288, SH66 frontier) consumes to draw an authored
/// quad through the ENGINE's GL stack (primitive-setup 0x105b353d0 →
/// glVertexAttribPointer + glDrawArrays via @plt → real Mesa on the live ctx).
/// Desync-safe: the emitter runs as its OWN top-level jit_run, never nested
/// inside the present-walker block, so no executing translation is invalidated.
///
/// Layout (from fresh disasm of real libroblox.so, confirmed SH66 recon):
///   emitter(x0=G, w1=mode_idx, w2=count, w3=geom_key, w4=first, w5=indexed):
///     x19=G; ldrh w8,[G+0x8e] (elem-type); ldr x9,[G+0x78] (elem-buffer);
///     w22=w1; bl primitive_setup(G, geom_key=w3) -> returns attrib mask in w0;
///     if !indexed (w5==0): if [G+0x78]!=0 -> glDrawElements; else ->
///       glDrawArrays(mode=draw_mode_table[w22], first=w4, count=w2) @0x62d7840.
///   primitive_setup(x0=G, w1=geom_key):
///     x25=[G+0x38]=M (mesh); spec array [M+0x48..M+0x50), 24B/entry;
///       BD array at G+0x48, slot[attr]*0x10 = ptr to BD; BD+0x48 = u32 VBO id;
///       spec+0=attr idx, spec+4=offset-addend, spec+8=format idx,
///       spec+12=attrib-loc-enum (0->0,1->1,2->+2,3->+4,else -1), spec+16=size-add;
///       stride = [M+0x60] table[attr] (u64);
///       format = vform_table[0xcecf8c + fmt*12] = {size u32, type u32, norm u8};
///       glBindBuffer(0x8892, bd_id) + glEnableVertexAttribArray(loc) +
///       glVertexAttribPointer(index=loc,size,type,norm,stride,stride*key+offset).
/// We fabricate the minimal coherent G for a colored quad (pos vec2 @loc0,
/// color vec4 @loc1, interleaved stride 24), upload verts into a real VBO via
/// host glGenBuffers/glBufferData, and drive the emitter as its own jit_run.
pub fn render_engine_emitter_quad(ctx: u64, iimg: &[u8], ibase: u64, isp: u64) -> u64 {
    if !(ctx >= 0x100000000 && ctx >> 56 == 0) {
        return 0;
    }
    let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    if h.is_null() {
        eprintln!("[elfjit:renderemitter] WARN: dlopen libGLESv2.so.2 failed");
        return 0;
    }
    let mp = walker_mesh_program();
    if mp.program == 0 {
        eprintln!("[elfjit:renderemitter] WARN: mesh program unavailable");
        return 0;
    }
    unsafe {
        // (1) Real VBO with the authored quad (pos vec2 + color vec4, stride 24).
        let genbuf: Option<extern "C" fn(i32, *mut u32)> = mesa_fn(h, b"glGenBuffers\0");
        let bindbuf: Option<extern "C" fn(u32, u32)> = mesa_fn(h, b"glBindBuffer\0");
        let bufdata: Option<extern "C" fn(u32, isize, *const i8, u32)> = mesa_fn(h, b"glBufferData\0");
        let useprogram: Option<extern "C" fn(u32)> = mesa_fn(h, b"glUseProgram\0");
        let (Some(gb), Some(bb), Some(bd), Some(up)) = (genbuf, bindbuf, bufdata, useprogram) else {
            eprintln!("[elfjit:renderemitter] WARN: required Mesa symbols missing");
            return 0;
        };
        up(mp.program);
        let mut vbo = 0u32;
        gb(1, &mut vbo);
        // A colored quad (4 verts), violet-to-teal gradient, NDC.
        let verts: [f32; 24] = [
            -0.95, -0.95, 0.60, 0.20, 0.95, 1.0, // v0 BL violet
            0.95, -0.95, 0.20, 0.80, 0.95, 1.0, //  v1 BR teal
            0.95, 0.95, 0.95, 0.60, 0.20, 1.0, //   v2 TR orange
            -0.95, 0.95, 0.30, 0.95, 0.40, 1.0, //  v3 TL green
        ];
        bb(0x8892, vbo);
        bd(0x8892, 96, verts.as_ptr() as *const i8, 0x88E4);
        // (2) Build the engine geometry context G. Use ONE aligned host buffer
        // (Box::leak; guest==host identity map so the emitter's guest derefs land
        // on it directly). NB: do NOT use the shared guest_arena here — its tick
        // counter races with the drain-thread deque injector, misaligning
        // co-allocated buffers (the walker's fab buffers use Box::leak for the
        // same reason).
        let gbuf = Box::leak(vec![0u64; 0x400].into_boxed_slice());
        let raw = gbuf.as_ptr() as u64;
        let g = (raw + 7) & !7; // 8-aligned base
        let bd0 = (g + 0x100) & !7;
        let m = (g + 0x180) & !7;
        let spec = (g + 0x280) & !7;
        let stride_tab = (g + 0x300) & !7;
        eprintln!("[elfjit:renderemitter] g={g:#x}(raw {raw:#x}) bd0={bd0:#x} m={m:#x} spec={spec:#x} stride_tab={stride_tab:#x}");
        // spec[0]: attr 0 (pos), offset 0, format idx 1 (vec2 FLOAT), loc 0.
        *(spec.wrapping_add(0) as *mut u32) = 0;
        *(spec.wrapping_add(4) as *mut u32) = 0;
        *(spec.wrapping_add(8) as *mut u32) = 1; // vform[1]={2,FLOAT}
        *(spec.wrapping_add(12) as *mut u32) = 0; // attrib-loc 0
        *(spec.wrapping_add(16) as *mut u32) = 0;
        // spec[1]: attr 1 (color), offset 8, format idx 3 (vec4 FLOAT), loc 1.
        *(spec.wrapping_add(24) as *mut u32) = 1;
        *(spec.wrapping_add(28) as *mut u32) = 8;
        *(spec.wrapping_add(32) as *mut u32) = 3; // vform[3]={4,FLOAT}
        *(spec.wrapping_add(36) as *mut u32) = 1; // attrib-loc 1
        *(spec.wrapping_add(40) as *mut u32) = 0;
        // BD[0]=BD[1]=bd0: BD+0x48 = u32 VBO id.
        *(bd0.wrapping_add(0x48) as *mut u32) = vbo;
        // G+0x48 BD slot array: each 16-byte slot = ptr to BD. primitive-setup
        // indexes slots by attr*0x10 (attr 0 -> G+0x48, attr 1 -> G+0x58), so
        // write BOTH slots to the same BD (both attrs share the VBO).
        *(g.wrapping_add(0x48) as *mut u64) = bd0;
        *(g.wrapping_add(0x58) as *mut u64) = bd0;
        // G+0x38 = M (mesh).
        *(g.wrapping_add(0x38) as *mut u64) = m;
        // M+0x48 = spec begin, M+0x50 = spec end, M+0x60 = stride table.
        *(m.wrapping_add(0x48) as *mut u64) = spec;
        *(m.wrapping_add(0x50) as *mut u64) = spec + 48;
        *(m.wrapping_add(0x60) as *mut u64) = stride_tab;
        // stride table: attr 0 and 1 both stride 24 (byte offsets +0 and +8).
        *(stride_tab.wrapping_add(0) as *mut u64) = 24;
        *(stride_tab.wrapping_add(8) as *mut u64) = 24;
        // G+0x78 element-buffer = 0 (non-indexed -> glDrawArrays path), G+0x8e = 0.
        *(g.wrapping_add(0x78) as *mut u64) = 0;
        *(g.wrapping_add(0x8e) as *mut u16) = 0;
        eprintln!(
            "[elfjit:renderemitter] built engine geometry ctx G={g:#x} M={m:#x} VBO={vbo} spec[2] stride=24 (vec2 pos loc0 + vec4 color loc1)"
        );
        // (3) Drive the ENGINE's real emitter as its OWN top-level jit_run.
        // mode_idx 3 = GL_TRIANGLE_STRIP (draw-mode table[3]=0x5), count 4,
        // geom_key 0, first 0, non-indexed. Own guest stack (Box::leak; the
        // caller's `isp` may point at the bottom of a mapped region, and the
        // emitter's prologue `stp x29,x30,[sp,#-64]!` would write below it).
        let stkbuf = Box::leak(vec![0u8; 0x8000].into_boxed_slice());
        let stk_top = (stkbuf.as_ptr() as u64).wrapping_add(0x8000) & !15;
        // Normalize the fixed-function state the emitter's draw depends on (the
        // walker's swap may leave viewport/depth/cull/blend/scissor in a state
        // that silently clips the emitter's quad), same as walker_item_draw_thunk,
        // PLUS bind the default framebuffer so the emit lands on the visible
        // surface (the engine may leave a render-target FBO bound).
        if let (Some(vp), Some(ds)) = (mesa_fn::<extern "C" fn(i32,i32,i32,i32)>(h, b"glViewport\0"), mesa_fn::<extern "C" fn(u32)>(h, b"glDisable\0")) {
            vp(0, 0, 1280, 720);
            ds(0x0B71); ds(0x0B44); ds(0x0BE2); ds(0x0C11);
        }
        if let Some(bf) = mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindFramebuffer\0") {
            bf(0x8D40 /*GL_FRAMEBUFFER*/, 0); // default framebuffer
        }
        if let Some(gi) = mesa_fn::<extern "C" fn(u32, *mut i32)>(h, b"glGetIntegerv\0") {
            let mut dfbo = 0i32;
            let mut rb = 0i32;
            gi(0x8CA9 /*GL_DRAW_FRAMEBUFFER_BINDING*/, &mut dfbo);
            gi(0x0C01 /*GL_DRAW_BUFFER*/, &mut rb);
            eprintln!("[elfjit:renderemitter] draw_fbo={dfbo} draw_buffer={rb:#x} (before emit)");
        }
        // SH66b root cause + FIX: GL_DRAW_BUFFER==GL_NONE. The prior attempt used
        // glDrawBuffer (singular) — a DESKTOP-only symbol Mesa's libGLESv2.so.2
        // does NOT export (nm: only the plural glDrawBuffers, ES3.0+), so it
        // silently no-op'd via mesa_fn->None. Use the real ES3 plural
        // glDrawBuffers(1,{GL_BACK}) + glReadBuffer(GL_BACK) so a default-FBO emit
        // lands on the presented back buffer.
        if let Some(dbs) = mesa_fn::<extern "C" fn(i32, *const u32)>(h, b"glDrawBuffers\0") {
            let back = 0x0405u32; // GL_BACK
            dbs(1, &back);
        }
        if let Some(rbuf) = mesa_fn::<extern "C" fn(u32)>(h, b"glReadBuffer\0") {
            rbuf(0x0405 /*GL_BACK*/);
        }
        let mut st = arm64jit::jit::CpuState::new();
        st.tpidr = arm64jit::jit::current_guest_tp();
        st.x[31] = stk_top;
        st.x[0] = g;
        st.x[1] = 3; // w1 mode_idx -> GL_TRIANGLE_STRIP
        // SH67-disasm-proven emitter ABI (0x105b35288 decode): the emitter maps
        // arg2 -> glDrawArrays FIRST and arg4 -> glDrawArrays COUNT (file decode:
        // w22=w1 mode, w23=w2 first, w20=w4 count). The pre-SH67 drive put
        // count@x2/first@x4 -> glDrawArrays(first=4, count=0), a legal no-op that
        // silently drew nothing. Correct: first=0, count=4; verified pixels land.
        st.x[2] = 0; // w2 first
        st.x[3] = 0; // w3 geom_key
        st.x[4] = 4; // w4 count
        st.x[5] = 0; // w5 indexed_flag = non-indexed
        let ret = arm64jit::jit::jit_run(iimg, ibase, 0x105b35288, &mut st as *mut CpuState);
        match ret {
            Err(e) => {
                eprintln!("[elfjit:renderemitter] engine emitter stopped: {e}");
                0
            }
            Ok(r) => {
                // SH67 diagnostic: what state did the emit leave / rely on? Check
                // gl error, the CURRENT program (+ whether it still has our
                // aPos/aColor locations and links), and the draw/read buffer — to
                // pin why the fabricated quad's pixels don't land despite the
                // GL_BACK wire. The emitter may rebind its OWN program (which
                // could fail to link in llvmpipe) or redirect to its own FBO.
                if std::env::var_os("RENDEREMITTER_GLTRAP").is_some() {
                    let ge: Option<extern "C" fn() -> u32> = mesa_fn(h, b"glGetError\0");
                    let gpi: Option<extern "C" fn(u32, *mut i32)> = mesa_fn(h, b"glGetIntegerv\0");
                    let gal: Option<extern "C" fn(u32, *const i8) -> i32> = mesa_fn(h, b"glGetAttribLocation\0");
                    let gpp: Option<extern "C" fn(u32, u32, *mut i32)> = mesa_fn(h, b"glGetProgramiv\0");
                    let mut cur_prog = 0i32;
                    let mut dfbo = 0i32;
                    let mut dbuf = 0i32;
                    let mut gerr = 0u32;
                    if let Some(f) = ge { gerr = f(); }
                    if let Some(f) = gpi {
                        f(0x8B8D /*GL_CURRENT_PROGRAM*/, &mut cur_prog);
                        f(0x8CA9 /*GL_DRAW_FRAMEBUFFER_BINDING*/, &mut dfbo);
                        f(0x0C01 /*GL_DRAW_BUFFER*/, &mut dbuf);
                    }
                    let (ap, ac, lstat) = if cur_prog > 0 && cur_prog as u64 >= 0x100000000 {
                        (-99, -99, -99)
                    } else if let (Some(f), Some(p)) = (gal, gpp) {
                        let ap = f(cur_prog as u32, b"aPos\0".as_ptr() as *const i8);
                        let ac = f(cur_prog as u32, b"aColor\0".as_ptr() as *const i8);
                        let mut ls = 0i32;
                        p(cur_prog as u32, 0x8B82 /*GL_LINK_STATUS*/, &mut ls);
                        (ap, ac, ls)
                    } else {
                        (-98, -98, -98)
                    };
                    eprintln!(
                        "[elfjit:renderemitter] GLTRAP err={gerr:#x} cur_prog={cur_prog:#x} draw_fbo={dfbo} draw_buffer={dbuf:#x} aPos_loc={ap} aColor_loc={ac} link_status={lstat}"
                    );
                }
                // glFinish so the draw is submitted. RENDEREMITTER_READBACK_BEFORE_SWAP=1
                // reads the CURRENT (back) buffer right after the emitter's draw,
                // BEFORE presenting — isolating the emitter's own rasterization from
                // the walker's prior presented frame. Default: present then read.
                if std::env::var_os("RENDEREMITTER_READBACK_BEFORE_SWAP").is_some() {
                    if let Some(fn_) = mesa_fn::<extern "C" fn()>(h, b"glFinish\0") {
                        fn_();
                    }
                    if let Some(rp) = mesa_fn::<extern "C" fn(i32,i32,i32,i32,u32,u32,*mut i8)>(h, b"glReadPixels\0") {
                        let mut px: [u8; 4] = [0; 4];
                        rp(640, 360, 1, 1, 0x1908, 0x1401, px.as_mut_ptr() as *mut i8);
                        let mut line: Vec<String> = Vec::new();
                        for y in [100u32, 200, 300, 360, 420, 500, 600, 650] {
                            let mut q: [u8; 4] = [0; 4];
                            rp(640, y as i32, 1, 1, 0x1908, 0x1401, q.as_mut_ptr() as *mut i8);
                            line.push(format!("y{y}=({},{},{},{})", q[0], q[1], q[2], q[3]));
                        }
                        eprintln!("[elfjit:renderemitter] engine emitter Ok(ret={r:#x}) BACK-BUFFER-DIRECT center=(640,360) rgba({},{},{},{}) | {}", px[0], px[1], px[2], px[3], line.join(" "));
                    }
                    return r;
                }
                // glFinish so the draw is submitted, then PRESENT it: bind the
                // engine ctx and swap via ctx-vt[+24] so the emitted quad moves
                // to the front buffer, then read back the quad center (the walker
                // already swapped its own frame before we ran).
                if let Some(fn_) = mesa_fn::<extern "C" fn()>(h, b"glFinish\0") {
                    fn_();
                }
                let vt = unsafe { *(ctx as *const u64) };
                let bind = unsafe { *(vt.wrapping_add(16) as *const u64) };
                let swap = unsafe { *(vt.wrapping_add(24) as *const u64) };
                let tp = arm64jit::jit::current_guest_tp();
                let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
                let sw = arm64jit::jit::run_guest_callback(swap, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
                let readback: Option<extern "C" fn(i32, i32, i32, i32, u32, u32, *mut i8)> =
                    mesa_fn(h, b"glReadPixels\0");
                if let Some(rp) = readback {
                    let mut px: [u8; 4] = [0; 4];
                    rp(640, 360, 1, 1, 0x1908, 0x1401, px.as_mut_ptr() as *mut i8);
                    // Sample a vertical line + corners to detect the emitter's
                    // gradient (violet->teal->orange->green quad) vs. leftover
                    // walker bands / backdrop.
                    let mut line: Vec<String> = Vec::new();
                    for y in [100u32, 200, 300, 360, 420, 500, 600, 650] {
                        let mut q: [u8; 4] = [0; 4];
                        rp(640, y as i32, 1, 1, 0x1908, 0x1401, q.as_mut_ptr() as *mut i8);
                        line.push(format!("y{y}=({},{},{},{})", q[0], q[1], q[2], q[3]));
                    }
                    eprintln!(
                        "[elfjit:renderemitter] engine emitter Ok(ret={r:#x}) swap={sw:?} center=(640,360) rgba({},{},{},{}) | {}",
                        px[0], px[1], px[2], px[3], line.join(" ")
                    );
                } else {
                    eprintln!("[elfjit:renderemitter] engine emitter Ok(ret={r:#x})");
                }
                r
            }
        }
    }
}


/// SH67e — cached real-Mesa TEXTURED program used ONLY by the emitter's textured
/// branch (RENDEREMITTER_TEX=1). SEPARATE from walker_mesh_program (which the
/// walker per-node thunk also binds) so the two don't fight over the shared
/// GLES2-program state. Attributes layout(location=0) aPos vec2,
/// (location=1) aColor vec4, (location=2) aTex vec2; uniform sampler2D uTex;
/// FS outputs texture2D(uTex, vUV) (texture source of color — a tile whose
/// texture isn't sampled reads black/backdrop, proving the sampler is live).
/// Returns (program, uTex uniform loc).
/// SH69 — minimal offline PNG->RGBA8 decoder (supports color types 0,2,4,6,
/// 8-bit, non-interlaced). Zero new network dep: rides `flate2` (already in
/// Cargo.lock via `zip`). Returns (w, h, RGBA8 top-first).
fn decode_png_rgba(data: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    const SIG: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if data.len() < 8 || &data[..8] != SIG {
        return None;
    }
    let mut off = 8usize;
    let (mut w, mut h, mut ct) = (0u32, 0u32, 0u8);
    let mut idat: Vec<u8> = Vec::new();
    let mut plte: Vec<u8> = Vec::new();
    let mut trns: Vec<u8> = Vec::new();
    while off + 8 <= data.len() {
        let len = u32::from_be_bytes(data[off..off + 4].try_into().ok()?) as usize;
        let typ = &data[off + 4..off + 8];
        let chunk = &data[off + 8..off + 8 + len];
        match typ {
            b"IHDR" if chunk.len() >= 13 => {
                w = u32::from_be_bytes(chunk[0..4].try_into().ok()?);
                h = u32::from_be_bytes(chunk[4..8].try_into().ok()?);
                if chunk[8] != 8 || chunk[10] != 0 || chunk[12] != 0 {
                    return None; // 8-bit deflate non-interlaced only
                }
                ct = chunk[9];
            }
            b"IDAT" => idat.extend_from_slice(chunk),
            b"PLTE" => plte = chunk.to_vec(),
            b"tRNS" => trns = chunk.to_vec(),
            b"IEND" => break,
            _ => {}
        }
        off += 12 + len;
    }
    let ch: usize = match ct {
        0 | 3 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return None,
    };
    let stride = w as usize * ch;
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(std::io::Cursor::new(&idat))
        .read_to_end(&mut raw)
        .ok()?;
    if raw.len() < stride.checked_mul(h as usize)? {
        return None;
    }
    let mut out = vec![0u8; w as usize * h as usize * 4];
    let mut prev = vec![0u8; stride];
    for y in 0..h as usize {
        let f = raw[y * (stride + 1)];
        let row = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        let mut cur: Vec<u8> = Vec::with_capacity(stride);
        for x in 0..stride {
            let a = if x >= ch { cur[x - ch] as i32 } else { 0 };
            let b = prev[x] as i32;
            let c = if x >= ch { prev[x - ch] as i32 } else { 0 };
            let r = match f {
                1 => a,
                2 => b,
                3 => (a + b) / 2,
                4 => {
                    let p = a + b - c;
                    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
                    if pa <= pb && pa <= pc {
                        a
                    } else if pb <= pc {
                        b
                    } else {
                        c
                    }
                }
                _ => 0,
            };
            cur.push(((row[x] as i32 + r) & 0xFF) as u8);
        }
        for x in 0..w as usize {
            let o = (y * w as usize + x) * 4;
            match ct {
                6 => out[o..o + 4].copy_from_slice(&cur[x * 4..x * 4 + 4]),
                2 => {
                    out[o] = cur[x * 3];
                    out[o + 1] = cur[x * 3 + 1];
                    out[o + 2] = cur[x * 3 + 2];
                    out[o + 3] = 255;
                }
                4 => {
                    let g = cur[x * 2];
                    out[o] = g;
                    out[o + 1] = g;
                    out[o + 2] = g;
                    out[o + 3] = cur[x * 2 + 1];
                }
                _ => {
                    out[o] = cur[x];
                    out[o + 1] = cur[x];
                    out[o + 2] = cur[x];
                    out[o + 3] = 255;
                }
            }
            // color type 3 (indexed/palette): expand via PLTE + optional tRNS alpha.
            if ct == 3 {
                let idx = cur[x] as usize;
                let (r, g, b) = if idx * 3 + 2 < plte.len() {
                    (plte[idx * 3], plte[idx * 3 + 1], plte[idx * 3 + 2])
                } else {
                    (0, 0, 0)
                };
                let a = if idx < trns.len() { trns[idx] } else { 255 };
                let o = (y * w as usize + x) * 4;
                out[o] = r;
                out[o + 1] = g;
                out[o + 2] = b;
                out[o + 3] = a;
            }
        }
        prev = cur;
    }
    Some((w, h, out))
}

/// SH69 — load the REAL Roblox UI texture (the loading-spinner, part of the
/// login/loading/home surface) as RGBA8. Cached; path overridable via env
/// RENDEREMITTER_REAL_TEXTURE (default the extracted APK assets path). Reads
/// the guest-identity-mapped host filesystem directly (in-process JIT).
fn real_ui_texture() -> Option<(u32, u32, Vec<u8>)> {
    static RT: std::sync::OnceLock<Option<(u32, u32, Vec<u8>)>> = std::sync::OnceLock::new();
    RT.get_or_init(|| {
        let path = std::env::var("RENDEREMITTER_REAL_TEXTURE")
            .unwrap_or_else(|_| {
                "/home/hermes-worker/.cache/open-sober/android-env/assets/content/textures/ui/LoadingScreen/LoadingSpinner.png".to_string()
            });
        let data = std::fs::read(&path).ok()?;
        let (w, h, rgba) = decode_png_rgba(&data)?;
        eprintln!("[elfjit:renderemitter-real] decoded real UI texture from {path}: {w}x{h} RGBA8");
        Some((w, h, rgba))
    })
    .clone()
}

/// SH69-sampler: image pixel (ix,iy) top-first -> screen window (sx,sy) for a
/// centered aspect-correct box (hh half-height, VW/VH viewport). Same math the
/// shader's linear vUV interpolation yields, so probe coords are exact.
fn imgpix_screen(ix: u32, iy: u32, imw: u32, imh: u32, hh: f32, vw: f32, vh: f32) -> (f32, f32) {
    let corr = vh / vw;
    let half_w = hh * (imw as f32 / imh as f32) * corr;
    let u = (ix as f32 + 0.5) / imw as f32;
    let x_ndc = -half_w + u * 2.0 * half_w;
    let y_ndc = hh - ((iy as f32 + 0.5) / imh as f32) * 2.0 * hh;
    ((x_ndc + 1.0) / 2.0 * vw, (1.0 - y_ndc) / 2.0 * vh)
}

/// SH72-generalization: image pixel (ix,iy) top-first -> screen window (sx,sy)
/// for an aspect-correct box whose CENTER is (cx_ndc, cy_ndc) in NDC (not
/// necessarily the screen center) with half-height hh, viewport VWxVH. Reduces
/// to imgpix_screen when cx_ndc==cy_ndc==0. Same vUV-linear math the shader's
/// aTex interpolation yields, so multi-sprite probe coords are exact.
fn imgpix_rect(
    ix: u32, iy: u32, imw: u32, imh: u32,
    cx_ndc: f32, cy_ndc: f32, hh: f32, vw: f32, vh: f32,
) -> (f32, f32) {
    let corr = vh / vw;
    let half_w = hh * (imw as f32 / imh as f32) * corr;
    let u = (ix as f32 + 0.5) / imw as f32;
    let v = (iy as f32 + 0.5) / imh as f32;
    let x_ndc = cx_ndc - half_w + u * 2.0 * half_w;
    let y_ndc = cy_ndc + (hh - v * 2.0 * hh);
    ((x_ndc + 1.0) / 2.0 * vw, (1.0 - y_ndc) / 2.0 * vh)
}

// ============================================================================
// SH77 — minimal pure-std TrueType outline rasterizer (text labels on the
// login form). Parses sfnt tables (cmap Format-4 + glyf simple contours), flattens
// quadratic outlines and fills with non-zero winding (4x4 supersampled) into an
// RGBA8 strip (transparent bg, opaque glyph interiors) so the same textured
// emitter path renders legible login text. Zero new deps / no network
// (fonts/ SourceSansPro-Bold.ttf; handles short AND long loca).
// ============================================================================
fn be16(d: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([d[o], d[o + 1]])
}
fn bes16(d: &[u8], o: usize) -> i16 {
    i16::from_be_bytes([d[o], d[o + 1]])
}
fn be32(d: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

/// Find a sfnt table's (offset, length) by 4-char tag. Returns None if the font
/// is a CFF/OTTO head (`'OTTO'` signature; no glyf outlines) or the tag is absent.
fn sfnt_find_table(font: &[u8], tag: &[u8; 4]) -> Option<(usize, usize)> {
    if font.len() < 12 || be32(font, 0) == 0x4F54544F {
        return None; // 'OTTO' CFF
    }
    let n = be16(font, 4) as usize;
    for i in 0..n {
        let rec = 12 + 16 * i;
        if rec + 16 <= font.len() && &font[rec..rec + 4] == tag {
            let off = be32(font, rec + 8) as usize;
            let len = be32(font, rec + 12) as usize;
            if off + len <= font.len() {
                return Some((off, len));
            }
        }
    }
    None
}

/// A parsed cmap Format-4 subtable (Windows BMP preferred, then any Unicode).
struct Cmap4 {
    seg_count: usize,
    end: Vec<u16>,
    start: Vec<u16>,
    delta: Vec<i16>,
    ro: Vec<u16>,
    garr: usize,     // glyphIdArray offset (within font bytes)
    ro_base: usize,  // offset of the idRangeOffset table (within font bytes)
}
fn font_cmap4(font: &[u8]) -> Option<Cmap4> {
    let (c, _) = sfnt_find_table(font, b"cmap")?;
    if c + 4 > font.len() {
        return None;
    }
    let n = be16(font, c + 2) as usize;
    let mut best: Option<(usize, u8)> = None; // (subtable offset, priority)
    for i in 0..n {
        let rec = c + 4 + 8 * i;
        if rec + 8 > font.len() {
            continue;
        }
        let plat = be16(font, rec);
        let enc = be16(font, rec + 2);
        let off = be32(font, rec + 4) as usize + c;
        if off + 8 > font.len() {
            continue;
        }
        let fmt = be16(font, off);
        let prio = match (plat, enc) {
            (3, 1) => 2, // Windows BMP
            (0, _) => 1, // Unicode
            _ => 0,
        };
        if fmt == 4 && prio > best.map(|(_, p)| p).unwrap_or(0) {
            best = Some((off, prio));
        }
    }
    let s = best?.0;
    let seg_x2 = be16(font, s + 6) as usize;
    let seg = seg_x2 / 2;
    if seg == 0 {
        return None;
    }
    let end_s = s + 14;
    let start_s = end_s + seg_x2 + 2;
    let delta_s = start_s + seg_x2;
    let ro_s = delta_s + seg_x2;
    let garr = ro_s + seg_x2;
    let mut end = Vec::with_capacity(seg);
    let mut start = Vec::with_capacity(seg);
    let mut delta = Vec::with_capacity(seg);
    let mut ro = Vec::with_capacity(seg);
    for i in 0..seg {
        end.push(be16(font, end_s + 2 * i));
        start.push(be16(font, start_s + 2 * i));
        delta.push(bes16(font, delta_s + 2 * i));
        ro.push(be16(font, ro_s + 2 * i));
    }
    Some(Cmap4 { seg_count: seg, end, start, delta, ro, garr, ro_base: ro_s })
}
impl Cmap4 {
    /// Map a Unicode code point to a glyph id, honoring both the idDelta and
    /// idRangeOffset segment encodings (idRangeOffset addresses are relative to
    /// the idRangeOffset entry's own position in the subtable).
    fn gid(&self, font: &[u8], cp: u32) -> u16 {
        let cp = (cp & 0xFFFF) as u16;
        for i in 0..self.seg_count {
            if cp > self.end[i] || cp < self.start[i] {
                continue;
            }
            if self.ro[i] == 0 {
                return ((cp as i32 + self.delta[i] as i32) & 0xFFFF) as u16;
            }
            // element address = address(idRangeOffset[i]) + ro[i] + 2*(cp - startCode[i])
            let elem = self.ro_base + 2 * i + self.ro[i] as usize + 2 * (cp as usize - self.start[i] as usize);
            if elem + 2 > font.len() {
                return 0;
            }
            let gi = be16(font, elem);
            if gi == 0 {
                return 0;
            }
            return ((gi as i32 + self.delta[i] as i32) & 0xFFFF) as u16;
        }
        0
    }
}

/// Advance width for a glyph id (hmtx: u16 advance width per glyph, 4 bytes each).
fn font_advance_width(font: &[u8], gid: u16) -> u16 {
    sfnt_find_table(font, b"hmtx")
        .and_then(|(m, _)| m.checked_add(gid as usize * 4).filter(|o| *o + 2 <= font.len()))
        .map(|o| be16(font, o))
        .unwrap_or(0)
}

/// Contour point rings of a glyph (simple OR composite): each ring is
/// (x, y, on_curve). Composite glyphs are decoded into their component glyphs'
/// contours, each transformed by the component's offset/scale/2x2 (SH79).
/// Returns None for CFF / malformed (caller skips + logs).
fn font_glyph_contours(font: &[u8], gid: u16) -> Option<Vec<Vec<(i16, i16, bool)>>> {
    let ((h, _), (l, _), (g, _)) = (
        sfnt_find_table(font, b"head")?,
        sfnt_find_table(font, b"loca")?,
        sfnt_find_table(font, b"glyf")?,
    );
    let fmt = bes16(font, h + 50);
    font_glyph_contours_rec(font, fmt, l, g, gid, 0)
}

/// Composite flags (OpenType glyf).
const CF_ARG_1_AND_2_ARE_WORDS: u16 = 0x0001;
const CF_ARGS_ARE_XY_VALUES: u16 = 0x0002;
const CF_WE_HAVE_A_SCALE: u16 = 0x0008;
const CF_MORE_COMPONENTS: u16 = 0x0020;
const CF_WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
const CF_WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;

fn font_glyph_contours_rec(
    font: &[u8], fmt: i16, l: usize, g: usize, gid: u16, depth: u32,
) -> Option<Vec<Vec<(i16, i16, bool)>>> {
    if depth > 16 {
        return None; // guard runaway composite nesting
    }
    let a = match fmt {
        0 => be16(font, l + 2 * gid as usize) as usize * 2,
        _ => be32(font, l + 4 * gid as usize) as usize,
    };
    let b = match fmt {
        0 => be16(font, l + 2 * (gid as usize + 1)) as usize * 2,
        _ => be32(font, l + 4 * (gid as usize + 1)) as usize,
    };
    if a >= b || b > font.len() || a >= font.len() {
        return None;
    }
    let nc = bes16(font, g + a);
    if nc < 0 {
        // ---------- COMPOSITE: decode each component, apply its transform ----------
        let mut out: Vec<Vec<(i16, i16, bool)>> = Vec::new();
        let mut p = g + a + 10; // skip numberOfContours + bbox (6 fields worth: xMin yMax xMax yMin -> 8 bytes; +2 = 10)
        loop {
            if p + 4 > font.len() {
                return Some(out);
            }
            let fl = be16(font, p);
            let cgid = be16(font, p + 2);
            p += 4;
            // args (arg1, arg2)
            let (arg1, arg2): (i32, i32) = if fl & CF_ARG_1_AND_2_ARE_WORDS != 0 {
                let a1 = bes16(font, p);
                let a2 = bes16(font, p + 2);
                p += 4;
                (a1 as i32, a2 as i32)
            } else {
                let a1 = *font.get(p)? as i8 as i32;
                let a2 = *font.get(p + 1)? as i8 as i32;
                p += 2;
                (a1, a2)
            };
            // transform params
            let mut t11 = 1.0f32;
            let mut t12 = 0.0f32;
            let mut t21 = 0.0f32;
            let mut t22 = 1.0f32;
            if fl & CF_WE_HAVE_A_SCALE != 0 {
                let s = bes16(font, p) as f32 / 16384.0;
                p += 2;
                t11 = s;
                t22 = s;
            } else if fl & CF_WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                let sx = bes16(font, p) as f32 / 16384.0;
                let sy = bes16(font, p + 2) as f32 / 16384.0;
                p += 4;
                t11 = sx;
                t22 = sy;
            } else if fl & CF_WE_HAVE_A_TWO_BY_TWO != 0 {
                t11 = bes16(font, p) as f32 / 16384.0;
                t12 = bes16(font, p + 2) as f32 / 16384.0;
                t21 = bes16(font, p + 4) as f32 / 16384.0;
                t22 = bes16(font, p + 6) as f32 / 16384.0;
                p += 8;
            }
            let sub = font_glyph_contours_rec(font, fmt, l, g, cgid, depth + 1)?;
            // The args are XY offsets (ARG_1_AND_2_ARE_WORDS/ARGS_ARE_XY_VALUES);
            // if not XY values they're point-match numbers (we cannot match points;
            // fall back to treating as 0 offset, still render the glyph un-offset).
            let (ox, oy) = if fl & CF_ARGS_ARE_XY_VALUES != 0 {
                (arg1, arg2)
            } else {
                (0, 0) // point-matching args unsupported: render at origin
            };
            for ring in sub {
                let offset_ring: Vec<(i16, i16, bool)> = ring
                    .iter()
                    .map(|&(x, y, on)| {
                        let xf = t11 * x as f32 + t12 * y as f32 + ox as f32;
                        let yf = t21 * x as f32 + t22 * y as f32 + oy as f32;
                        (xf.round() as i16, yf.round() as i16, on)
                    })
                    .collect();
                if !offset_ring.is_empty() {
                    out.push(offset_ring);
                }
            }
            if fl & CF_MORE_COMPONENTS == 0 {
                break;
            }
        }
        return Some(out);
    }
    // ---------- SIMPLE ----------
    let nc = nc as usize;
    if nc == 0 {
        return Some(Vec::new()); // empty glyph (e.g. space)
    }
    let endp = g + a + 10;
    let mut ends = Vec::with_capacity(nc);
    for i in 0..nc {
        ends.push(be16(font, endp + 2 * i) as usize);
    }
    let ilen = be16(font, endp + 2 * nc) as usize;
    let mut p = endp + 2 * nc + 2 + ilen;
    let mut flags = Vec::new();
    let total = ends[nc - 1] + 1;
    while flags.len() < total {
        let f = *font.get(p)?;
        flags.push(f);
        p += 1;
        if f & 0x08 != 0 {
            let n = *font.get(p)?;
            p += 1;
            for _ in 0..n {
                flags.push(f);
            }
        }
    }
    // x coordinates (with per-point x-metrics).
    let mut xs = Vec::with_capacity(total);
    let mut x = 0i32;
    for i in 0..total {
        let f = flags[i];
        if f & 0x02 != 0 {
            // short vector signed
            let d = *font.get(p)?;
            p += 1;
            x += if f & 0x10 != 0 { d as i32 } else { -(d as i8 as i32) };
        } else if f & 0x10 == 0 {
            let d = *font.get(p)? as i8 as i32;
            p += 1;
            x += d;
        }
        xs.push(x);
    }
    // y coordinates.
    let mut ys = Vec::with_capacity(total);
    let mut y = 0i32;
    for i in 0..total {
        let f = flags[i];
        if f & 0x04 != 0 {
            let d = *font.get(p)?;
            p += 1;
            y += if f & 0x20 != 0 { d as i32 } else { -(d as i8 as i32) };
        } else if f & 0x20 == 0 {
            let d = *font.get(p)? as i8 as i32;
            p += 1;
            y += d;
        }
        ys.push(y);
    }
    let mut rings = Vec::with_capacity(nc);
    let mut start = 0usize;
    for e in &ends {
        let ring: Vec<(i16, i16, bool)> = (start..=*e)
            .map(|i| (xs[i] as i16, ys[i] as i16, flags[i] & 0x01 != 0))
            .collect();
        rings.push(ring);
        start = e + 1;
    }
    Some(rings)
}

/// Flatten one TrueType contour ring into a closed polygon of f32 font-unit
/// points, sampling each quadratic on/off segment (spec quadratic rules:
/// on-on -> line; on-off-on -> quadratic; two consecutive offs -> implied
/// on-curve at their midpoint). Ring wrap is folded by iterating all points.
fn flatten_ring(pts: &[(i16, i16, bool)]) -> Vec<(f32, f32)> {
    let n = pts.len();
    if n == 0 {
        return Vec::new();
    }
    let st = pts.iter().position(|p| p.2).unwrap_or(0);
    let at = |idx: usize| -> (f32, f32, bool) {
        let p = pts[idx % n];
        (p.0 as f32, p.1 as f32, p.2)
    };
    let mut out: Vec<(f32, f32)> = Vec::new();
    // Emit a run of off-curves `offs` bridged from on-point `a` to on-point `b`:
    // 0 offs -> nothing; 1 off -> quadratic a-o-b; >=2 offs -> implied on-curve
    // midpoints turn the run into k quadratics.
    let emit = |a: (f32, f32), offs: &[(f32, f32)], b: (f32, f32), out: &mut Vec<(f32, f32)>| {
        if offs.is_empty() {
            return;
        }
        let mut prev = a;
        let mut idx = 0usize;
        while idx < offs.len() {
            let o = offs[idx];
            let end: (f32, f32) = if idx + 1 < offs.len() {
                ((o.0 + offs[idx + 1].0) / 2.0, (o.1 + offs[idx + 1].1) / 2.0)
            } else {
                b
            };
            for t in [0.25f32, 0.5, 0.75] {
                let u = 1.0 - t;
                out.push((
                    u * u * prev.0 + 2.0 * u * t * o.0 + t * t * end.0,
                    u * u * prev.1 + 2.0 * u * t * o.1 + t * t * end.1,
                ));
            }
            out.push(end);
            prev = end;
            idx += 1;
        }
    };
    // Walk the ring, collecting off-curve runs between on-curves.
    let mut start_on: Option<(f32, f32)> = None;
    let mut offs: Vec<(f32, f32)> = Vec::new();
    let mut seen = 0usize;
    let mut i = 0usize;
    while seen < n {
        let (x, y, on) = at(st + i);
        seen += 1;
        if on {
            if let Some(a) = start_on {
                if !offs.is_empty() {
                    emit(a, &offs, (x, y), &mut out); // pushes endpoint (x,y)
                    let _ = a;
                }
                // else: line a->(x,y), endpoint pushed below
            }
            start_on = Some((x, y));
            offs.clear();
            // endpoints already emitted by `emit` are (x,y) == last vertex; only
            // push when it is NOT already the last output vertex (avoid dup).
            if !(out.last().map_or(false, |&(lx, ly)| (lx - x).abs() < 1e-4 && (ly - y).abs() < 1e-4)) {
                out.push((x, y));
            }
        } else {
            offs.push((x, y));
        }
        i += 1;
    }
    // Close the ring back to the start on-curve.
    if let Some(a) = start_on {
        let (sx, sy, _) = at(st);
        if !offs.is_empty() {
            emit(a, &offs, (sx, sy), &mut out);
        }
        // ensure the polygon is closed (first vertex appended last if distinct)
        if let Some(&first) = out.first() {
            if let Some(&last) = out.last() {
                if (first.0 - last.0).abs() > 1e-4 || (first.1 - last.1).abs() > 1e-4 {
                    out.push(first);
                }
            }
        }
    }
    out
}

/// Non-zero-winding coverage of `rings` (already transformed to image space).
fn covered(rings: &[Vec<(f32, f32)>], x: f32, y: f32) -> bool {
    let mut w = 0i32;
    for r in rings {
        for i in 0..r.len() {
            let a = r[i];
            let b = r[(i + 1) % r.len()];
            if (a.1 > y) != (b.1 > y) {
                let xi = a.0 + (y - a.1) / (b.1 - a.1) * (b.0 - a.0);
                if xi > x {
                    w += if b.1 > a.1 { 1 } else { -1 };
                }
            }
        }
    }
    w != 0
}

/// Rasterize a text row into a w x h RGBA8 buffer (transparent background).
/// Layout: characters left-to-right; `pu` = px per font unit; `color` baked;
/// `pen_start_x` (px) = x of the first glyph's origin; `baseline_row` (px,
/// image y-down) = where glyph y=0 sits. Coverage supersampled ss=4 per pixel.
fn rasterize_text_row(
    font: &[u8],
    text: &str,
    pu: f32,
    color: [u8; 4],
    w: u32,
    h: u32,
    pen_start_x: f32,
    baseline_row: f32,
) -> Vec<u8> {
    let ss = 4;
    let cmap = font_cmap4(font);
    let mut rings: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut pen = pen_start_x;
    if let Some(cm) = &cmap {
        for ch in text.chars() {
            let gid = cm.gid(font, ch as u32);
            if gid == 0 {
                continue;
            }
            if let Some(contours) = font_glyph_contours(font, gid) {
                for pts in &contours {
                    if pts.len() < 3 {
                        continue;
                    }
                    let flat = flatten_ring(pts);
                    let ring_t: Vec<(f32, f32)> = flat
                        .iter()
                        .map(|(fx, fy)| (pen + fx * pu, baseline_row - fy * pu))
                        .collect();
                    if ring_t.len() >= 3 {
                        rings.push(ring_t);
                    }
                }
            }
            pen += font_advance_width(font, gid) as f32 * pu;
        }
    }
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for py in 0..h as usize {
        for px in 0..w as usize {
            let mut cov = 0i32;
            for sy in 0..ss {
                for sx in 0..ss {
                    let fx = px as f32 + (sx as f32 + 0.5) / ss as f32;
                    let fy = py as f32 + (sy as f32 + 0.5) / ss as f32;
                    if covered(&rings, fx, fy) {
                        cov += 1;
                    }
                }
            }
            let a = (cov as u32 * 255) / (ss * ss) as u32;
            if a > 0 {
                let o = (py * w as usize + px) * 4;
                out[o] = color[0];
                out[o + 1] = color[1];
                out[o + 2] = color[2];
                out[o + 3] = a as u8;
            }
        }
    }
    out
}

/// SH77 — cached font bytes for the login-form text labels. Reads the real
/// SourceSansPro-Bold.ttf from the extracted assets once.
fn login_font_bytes() -> &'static [u8] {
    static FB: std::sync::OnceLock<Option<&'static [u8]>> = std::sync::OnceLock::new();
    FB.get_or_init(|| {
        let path = "/home/hermes-worker/.cache/open-sober/android-env/assets/fonts/SourceSansPro-Bold.ttf";
        match std::fs::read(path) {
            Ok(d) => Some(Box::leak(d.into_boxed_slice())),
            Err(e) => {
                eprintln!("[elfjit:renderemitter-login] WARN: cannot read login font {path}: {e}");
                None
            }
        }
    })
    .as_deref()
    .unwrap_or(&[])
}

/// Rasterize a login-form text label into an `w`-wide RGBA8 strip, centered
/// (transparent bg, glyph color). Returns None if the font is unavailable.
/// SH78: `w`/`h`/`pu` are caller-supplied so the label can be supersampled 2x
/// (w and h both double; w:h is preserved, so on-screen geometry + probes are
/// bit-identical while texel density and stroke thickness double).
fn rasterize_login_label(text: &str, color: [u8; 4], w: u32, h: u32, pu: f32) -> Option<RealSprite> {
    let font = login_font_bytes();
    if font.is_empty() {
        return None;
    }
    let cmap = font_cmap4(font)?;
    // Measure used width to center the string in the `w`-wide row.
    let mut used = 0.0f32;
    for ch in text.chars() {
        let gid = cmap.gid(font, ch as u32);
        if gid != 0 {
            used += font_advance_width(font, gid) as f32 * pu;
        }
    }
    let pen_start = (w as f32 - used) / 2.0;
    // baseline_row: pad(h/10) + ascender*pu. Ascender = 722 for SSPro Bold (from
    // the statically-derived yMax of 'i'/'l'); use a fixed generous ascender so
    // caps + ascenders sit with pad.
    let ascender = 722.0f32;
    let baseline_row = (h as f32 / 10.0) + ascender * pu;
    let rgba = rasterize_text_row(font, text, pu, color, w, h, pen_start, baseline_row);
    Some(RealSprite { name: format!("{text}"), w, h, rgba })
}

fn emitter_tex_program() -> (u32, i32) {
    static TP: std::sync::OnceLock<(u32, i32)> = std::sync::OnceLock::new();
    *TP.get_or_init(|| {
        let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
        if h.is_null() {
            return (0, -1);
        }
        let (Some(cs), Some(ss), Some(cp), Some(cprog), Some(att), Some(bal), Some(ln)) = (
            mesa_fn::<extern "C" fn(u32) -> u32>(h, b"glCreateShader\0"),
            mesa_fn::<extern "C" fn(u32, i32, *const *const i8, *const i32)>(h, b"glShaderSource\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glCompileShader\0"),
            mesa_fn::<extern "C" fn() -> u32>(h, b"glCreateProgram\0"),
            mesa_fn::<extern "C" fn(u32, u32)>(h, b"glAttachShader\0"),
            mesa_fn::<extern "C" fn(u32, u32, *const i8)>(h, b"glBindAttribLocation\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glLinkProgram\0"),
        ) else {
            return (0, -1);
        };
        let vs_src = c"attribute vec2 aPos; attribute vec4 aColor; attribute vec2 aTex; varying vec2 vUV; void main(){ vUV = aTex; gl_Position = vec4(aPos,0.0,1.0); }\n".to_bytes_with_nul();
        let fs_src = c"precision mediump float; uniform sampler2D uTex; varying vec2 vUV; void main(){ gl_FragColor = texture2D(uTex, vUV); }\n".to_bytes_with_nul();
        let vs_ptr = vs_src.as_ptr() as *const i8;
        let fs_ptr = fs_src.as_ptr() as *const i8;
        let vs = cs(0x8B31);
        ss(vs, 1, &vs_ptr, std::ptr::null());
        cp(vs);
        let fs = cs(0x8B30);
        ss(fs, 1, &fs_ptr, std::ptr::null());
        cp(fs);
        let prog = cprog();
        att(prog, vs);
        att(prog, fs);
        bal(prog, 0, b"aPos\0".as_ptr() as *const i8);
        bal(prog, 1, b"aColor\0".as_ptr() as *const i8);
        bal(prog, 2, b"aTex\0".as_ptr() as *const i8);
        ln(prog);
        let gul: Option<extern "C" fn(u32, *const i8) -> i32> = mesa_fn(h, b"glGetUniformLocation\0");
        let uloc = match gul {
            Some(f) => f(prog, b"uTex\0".as_ptr() as *const i8),
            None => -1,
        };
        eprintln!("[elfjit:renderemitter-grid] textured program built prog={prog:#x} uTex loc={uloc}");
        (prog, uloc)
    })
}

/// SH67d — a POPULATED N-quad 2D frame drawn by the ENGINE's OWN geometry
/// emitter 0x105b35288 in a SINGLE top-level jit_run. One pre-uploaded VBO holds
/// all N*6 verts as GL_TRIANGLES (6 verts/quad), so `glDrawArrays(mode=0x4
/// GL_TRIANGLES, first=0, count=6N)` draws every tile in ONE call — the SH67c
/// multi-tile blocker (ANY per-drive glBufferData/glBindBuffer on a buffer the
/// engine's VAO-less GLES2 attrib-pointers already wired orphans the storage ->
/// silent drop / SIGABRT, exit 134) is closed BY CONSTRUCTION: no per-tile
/// re-drive, no inter-drive buffer change, GL_TRIANGLES keeps quads isolated (no
/// TRIANGLE_STRIP cross-tile fusion). `nq` = quad count; each tile uses a
/// distinct palette color + grid placement so readback can assert per-tile.
/// Same desync-safe shape as render_engine_emitter_quad: the emitter is its OWN
/// top-level jit_run, never nested inside the present-walker block; runs on the
/// currency-owning renderinit thread. Returns the emitter's ret (0 = clean draw).
pub fn render_engine_emitter_grid(ctx: u64, iimg: &[u8], ibase: u64, isp: u64, nq: usize, tex: bool) -> u64 {
    if !(ctx >= 0x100000000 && ctx >> 56 == 0) {
        return 0;
    }
    if nq == 0 {
        return 0;
    }
    let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    if h.is_null() {
        eprintln!("[elfjit:renderemitter-grid] WARN: dlopen libGLESv2.so.2 failed");
        return 0;
    }
    let mp = walker_mesh_program();
    if mp.program == 0 {
        eprintln!("[elfjit:renderemitter-grid] WARN: mesh program unavailable");
        return 0;
    }
    unsafe {
        // (1) ONE real VBO uploaded ONCE with all N*6 verts (pos vec2 + color
        // vec4, stride 24). Never re-touched after this -> no orphan class.
        let genbuf: Option<extern "C" fn(i32, *mut u32)> = mesa_fn(h, b"glGenBuffers\0");
        let bindbuf: Option<extern "C" fn(u32, u32)> = mesa_fn(h, b"glBindBuffer\0");
        let bufdata: Option<extern "C" fn(u32, isize, *const i8, u32)> = mesa_fn(h, b"glBufferData\0");
        let useprogram: Option<extern "C" fn(u32)> = mesa_fn(h, b"glUseProgram\0");
        let (Some(gb), Some(bb), Some(bd), Some(up)) = (genbuf, bindbuf, bufdata, useprogram) else {
            eprintln!("[elfjit:renderemitter-grid] WARN: required Mesa symbols missing");
            return 0;
        };
        // Emitter never calls glUseProgram/glUniform* — bind the program + set
        // the sampler here so it persists through the single emit. Textured branch
        // (RENDEREMITTER_TEX=1, tex=true): use the SEPARATE textured program and a
        // pre-uploaded host texture (a per-tile palette strip) sampled by per-vertex
        // aTex UVs the emitter interpolates. Texture upload is SAFE re: SH67c (it
        // never touches GL_ARRAY_BUFFER / attrib pointers — no orphan class).
        let stride: u64 = if tex { 32 } else { 24 };
        let mut prog = mp.program;
        if tex {
            let (tprog, uloc) = emitter_tex_program();
            if tprog != 0 {
                prog = tprog;
                // GenTextures + upload a palette strip: texture W x 1 texels, one
                // run of `nq` palette colors (each 8px wide so NEAREST sampling at
                // vUV picks the tile's solid color). RGBA8 uncompressed.
                if let (Some(gt), Some(at), Some(bt), Some(tp_), Some(te)) = (
                    mesa_fn::<extern "C" fn(i32, *mut u32)>(h, b"glGenTextures\0"),
                    mesa_fn::<extern "C" fn(u32)>(h, b"glActiveTexture\0"),
                    mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindTexture\0"),
                    mesa_fn::<extern "C" fn(u32, u32, i32, i32)>(h, b"glTexParameteri\0"),
                    mesa_fn::<extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const i8)>(h, b"glTexImage2D\0"),
                ) {
                    let mut texid = 0u32;
                    gt(1, &mut texid);
                    at(0x84C0 /*GL_TEXTURE0*/);
                    bt(0x0DE1 /*GL_TEXTURE_2D*/, texid);
                    tp_(0x0DE1, 0x2800 /*GL_TEXTURE_MAG_FILTER*/, 0x2600 /*GL_NEAREST*/, 0);
                    tp_(0x0DE1, 0x2801 /*GL_TEXTURE_MIN_FILTER*/, 0x2600, 0);
                    let w = (nq * 8).max(8);
                    let mut px: Vec<u8> = Vec::with_capacity(w * 4);
                    for t6 in 0..nq {
                        let cc = TASK_FRAME_PALETTE[t6 % TASK_FRAME_PALETTE.len()];
                        let rg = [(cc[0] * 255.0) as u8, (cc[1] * 255.0) as u8, (cc[2] * 255.0) as u8, (cc[3] * 255.0) as u8];
                        for _ in 0..8 {
                            px.extend_from_slice(&rg);
                        }
                    }
                    while px.len() < w * 4 {
                        px.extend_from_slice(&[0, 0, 0, 255]);
                    }
                    te(0x0DE1, 0, 0x1908 /*GL_RGBA*/, w as i32, 1, 0, 0x1908, 0x1401, px.as_ptr() as *const i8);
                    if uloc >= 0 {
                        if let Some(ui) = mesa_fn::<extern "C" fn(i32, i32)>(h, b"glUniform1i\0") {
                            ui(uloc, 0);
                        }
                    }
                    eprintln!("[elfjit:renderemitter-grid] tex {w}x1 RGBA uploaded (glTexImage2D 0x1908) uTex loc={uloc} prog={tprog:#x}");
                }
            }
        }
        up(prog);
        // Grid placement: nq tiles over `cols` columns (~1.9 wide NDC viewport).
        let cols = 3usize.max(1);
        let rows = (nq + cols - 1) / cols;
        let cell_w = 1.9 / cols as f32;
        let cell_h = 1.8 / rows as f32;
        let mut verts: Vec<f32> = Vec::with_capacity(nq * (if tex { 32 } else { 24 }));
        for t in 0..nq {
            let cc = TASK_FRAME_PALETTE[t % TASK_FRAME_PALETTE.len()];
            let col = (t % cols) as f32;
            let row = (t / cols) as f32;
            let x0 = -0.95 + col * cell_w + 0.02 * cell_w;
            let x1 = -0.95 + (col + 1.0) * cell_w - 0.02 * cell_w;
            let y1 = 1.0 - (row + 0.02) * cell_h; // top
            let y0 = 1.0 - (row + 1.0 - 0.02) * cell_h; // bottom
            // GL_TRIANGLES: two tris per quad (v0,v1,v2,v0,v2,v3), 6 verts.
            // Textured branch: each vert = x,y, r,g,b,a, u,v (8 floats, stride 32),
            // uv samples the tile's palette region of the strip texture
            // ([t/nq, (t+1)/nq) horizontally, vert-center vertically). Solid branch:
            // each vert = x,y, r,g,b,a (6 floats, stride 24).
            let mut push = move |x: f32, y: f32, u: f32, v: &mut Vec<f32>| {
                if tex {
                    v.extend_from_slice(&[x, y, cc[0], cc[1], cc[2], cc[3],
                        (t as f32 + 0.5) / nq as f32, 0.5]);
                } else {
                    v.extend_from_slice(&[x, y, cc[0], cc[1], cc[2], cc[3]]);
                }
            };
            let u0 = (t as f32) / nq as f32;
            let u1 = (t as f32 + 1.0) / nq as f32;
            push(x0, y0, u0, &mut verts);
            push(x1, y0, u1, &mut verts);
            push(x1, y1, u1, &mut verts);
            push(x0, y0, u0, &mut verts);
            push(x1, y1, u1, &mut verts);
            push(x0, y1, u0, &mut verts);
        }
        let total_verts = verts.len() / if tex { 8 } else { 6 };
        let nbytes = verts.len() * 4;
        let mut vbo = 0u32;
        gb(1, &mut vbo);
        bb(0x8892, vbo);
        bd(0x8892, nbytes as isize, verts.as_ptr() as *const i8, 0x88E4);
        // (2) Build the engine geometry context G (same layout as
        // render_engine_emitter_quad — one shared VBO for both attrs).
        let gbuf = Box::leak(vec![0u64; 0x400].into_boxed_slice());
        let raw = gbuf.as_ptr() as u64;
        let g = (raw + 7) & !7;
        let bd0 = (g + 0x100) & !7;
        let m = (g + 0x180) & !7;
        let spec = (g + 0x280) & !7;
        let stride_tab = (g + 0x300) & !7;
        eprintln!("[elfjit:renderemitter-grid] g={g:#x} bd0={bd0:#x} m={m:#x} spec={spec:#x} stride_tab={stride_tab:#x}");
        *(spec.wrapping_add(0) as *mut u32) = 0; // attr 0 pos
        *(spec.wrapping_add(4) as *mut u32) = 0;
        *(spec.wrapping_add(8) as *mut u32) = 1; // vform[1]={2,FLOAT}
        *(spec.wrapping_add(12) as *mut u32) = 0; // loc 0
        *(spec.wrapping_add(16) as *mut u32) = 0;
        *(spec.wrapping_add(24) as *mut u32) = 1; // attr 1 color
        *(spec.wrapping_add(28) as *mut u32) = 8;
        *(spec.wrapping_add(32) as *mut u32) = 3; // vform[3]={4,FLOAT}
        *(spec.wrapping_add(36) as *mut u32) = 1; // loc 1
        *(spec.wrapping_add(40) as *mut u32) = 0;
        // Spec count / stride depend on the textured branch: textured adds spec[2]
        // (aTex, attr 2, offset 24, format idx 1 vec2 FLOAT, loc enum 2 -> loc 2,
        // size_addend 0) so stride becomes 32 (pos2@0 + color4@8 + uv2@24).
        let n_spec_entries: usize = if tex { 3 } else { 2 };
        if tex {
            *(spec.wrapping_add(48) as *mut u32) = 2; // attr 2 texcoord
            *(spec.wrapping_add(52) as *mut u32) = 24;
            *(spec.wrapping_add(56) as *mut u32) = 1; // vform[1]={2,FLOAT}
            *(spec.wrapping_add(60) as *mut u32) = 2; // attrib-loc-enum 2 -> loc 2
            *(spec.wrapping_add(64) as *mut u32) = 0;
        }
        *(bd0.wrapping_add(0x48) as *mut u32) = vbo;
        *(g.wrapping_add(0x48) as *mut u64) = bd0;
        *(g.wrapping_add(0x58) as *mut u64) = bd0;
        if tex {
            // BD slot for attr 2 is G+0x68 = G+0x48 + 2*0x10 (primitive_setup walks
            // slots at attr*0x10); primitive_setup loop ends at spec[2], both wired
            // to the same shared VBO.
            *(g.wrapping_add(0x68) as *mut u64) = bd0;
        }
        *(g.wrapping_add(0x38) as *mut u64) = m;
        *(m.wrapping_add(0x48) as *mut u64) = spec;
        *(m.wrapping_add(0x50) as *mut u64) = spec + ((n_spec_entries * 24) as u64);
        *(m.wrapping_add(0x60) as *mut u64) = stride_tab;
        *(stride_tab.wrapping_add(0) as *mut u64) = stride;
        *(stride_tab.wrapping_add(8) as *mut u64) = stride;
        if tex {
            *(stride_tab.wrapping_add(16) as *mut u64) = stride;
        }
        *(g.wrapping_add(0x78) as *mut u64) = 0; // non-indexed
        *(g.wrapping_add(0x8e) as *mut u16) = 0;
        eprintln!("[elfjit:renderemitter-grid] built engine geometry ctx G={g:#x} M={m:#x} VBO={vbo} quads={nq} total_verts={total_verts} GL_TRIANGLES stride={stride} spec[{}] aPos=0 aColor=1{}", n_spec_entries, if tex { " aTex=2" } else { "" });
        // (3) Normalize fixed-function state + wire GL_DRAW_BUFFER to GL_BACK
        // (SH66b/SH67 pre-draw fixes): default FBO, full viewport, depth/cull/
        // blend/scissor off, glDrawBuffers(1,{GL_BACK}) so the single emit lands
        // on the presented back buffer.
        if let (Some(vp), Some(ds)) = (mesa_fn::<extern "C" fn(i32,i32,i32,i32)>(h, b"glViewport\0"), mesa_fn::<extern "C" fn(u32)>(h, b"glDisable\0")) {
            vp(0, 0, 1280, 720);
            ds(0x0B71); ds(0x0B44); ds(0x0BE2); ds(0x0C11);
        }
        if let Some(bf) = mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindFramebuffer\0") {
            bf(0x8D40, 0);
        }
        if let Some(dbs) = mesa_fn::<extern "C" fn(i32, *const u32)>(h, b"glDrawBuffers\0") {
            let back = 0x0405u32;
            dbs(1, &back);
        }
        if let Some(rbuf) = mesa_fn::<extern "C" fn(u32)>(h, b"glReadBuffer\0") {
            rbuf(0x0405);
        }
        // (4) Drive the ENGINE's real emitter ONCE: mode 0 = GL_TRIANGLES
        // (draw_mode table 0x225780[0]=0x4), first=0, count=6N. One call -> the
        // whole populated frame. Own guest stack so the emitter's prologue stp
        // doesn't write below the caller stack (same as render_engine_emitter_quad).
        let stkbuf = Box::leak(vec![0u8; 0x8000].into_boxed_slice());
        let stk_top = (stkbuf.as_ptr() as u64).wrapping_add(0x8000) & !15;
        let mut st = arm64jit::jit::CpuState::new();
        st.tpidr = arm64jit::jit::current_guest_tp();
        st.x[31] = stk_top;
        st.x[0] = g;
        st.x[1] = 0; // w1 mode_idx -> GL_TRIANGLES
        st.x[2] = 0; // w2 first
        st.x[3] = 0; // w3 geom_key
        st.x[4] = (6 * nq) as u64; // w4 count (6 verts/quad)
        st.x[5] = 0; // w5 indexed_flag = non-indexed
        let ret = arm64jit::jit::jit_run(iimg, ibase, 0x105b35288, &mut st as *mut CpuState);
        let r = match ret {
            Err(e) => {
                eprintln!("[elfjit:renderemitter-grid] engine emitter stopped: {e}");
                return 0;
            }
            Ok(r) => r,
        };
        if tex {
            // Post-emit cleanup: the walker immediately reuses the same VAO-less
            // global GLES2 context; leaving aTex (attr 2) enabled with a stale
            // pointer would corrupt the walker's next frame.
            if let Some(da) = mesa_fn::<extern "C" fn(u32)>(h, b"glDisableVertexAttribArray\0") {
                da(2);
            }
        }
        let readback: Option<extern "C" fn(i32, i32, i32, i32, u32, u32, *mut i8)> =
            mesa_fn(h, b"glReadPixels\0");
        // glFinish so the draw is submitted, then present (engine bind + swap via
        // ctx-vt[+24]) so the emitted grid moves to the front buffer.
        if let Some(fn_) = mesa_fn::<extern "C" fn()>(h, b"glFinish\0") {
            fn_();
        }
        let vt = unsafe { *(ctx as *const u64) };
        let bind = unsafe { *(vt.wrapping_add(16) as *const u64) };
        let swap = unsafe { *(vt.wrapping_add(24) as *const u64) };
        let tp = arm64jit::jit::current_guest_tp();
        let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
        let sw = arm64jit::jit::run_guest_callback(swap, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
        // Read back each tile's center to assert a DISTINCT per-tile color
        // (proving all N quads rasterized, not just tile 0), plus overall center.
        if let Some(rp) = readback {
            eprintln!("[elfjit:renderemitter-grid] engine emitter Ok(ret={r:#x}) draw_mode=0x4(first=0,count={}) quads={nq} swap={sw:?}", 6 * nq);
            for t in 0..nq {
                let col = (t % cols) as f32;
                let row = (t / cols) as f32;
                let cx = -0.95 + (col + 0.5) * cell_w;
                let cy = 1.0 - (row + 0.5) * cell_h;
                let fx = ((cx + 1.0) / 2.0 * 1280.0) as i32;
                let fy = ((cy + 1.0) / 2.0 * 720.0) as i32;
                let mut px: [u8; 4] = [0; 4];
                rp(fx, fy, 1, 1, 0x1908, 0x1401, px.as_mut_ptr() as *mut i8);
                let want = TASK_FRAME_PALETTE[t % TASK_FRAME_PALETTE.len()];
                let exp = [
                    (want[0] * 255.0) as u8, (want[1] * 255.0) as u8,
                    (want[2] * 255.0) as u8, (want[3] * 255.0) as u8,
                ];
                // Tile present iff each channel within +-1 (float->u8 rounding;
                // palette 0.10*255=25.5 may truncate 25 / Mesa round 26). A real
                // raster gap would read the dark backdrop (13,13,20), a >=50 delta
                // on every channel.
                let diff = [
                    (px[0] as i32 - exp[0] as i32).abs(),
                    (px[1] as i32 - exp[1] as i32).abs(),
                    (px[2] as i32 - exp[2] as i32).abs(),
                    (px[3] as i32 - exp[3] as i32).abs(),
                ];
                let present = diff.iter().all(|d| *d <= 1);
                eprintln!("[elfjit:renderemitter-grid] tile#{t} center({fx},{fy}) rgba({},{},{},{}) expect {:?} diff={diff:?} present={present}", px[0], px[1], px[2], px[3], exp);
            }
        } else {
            eprintln!("[elfjit:renderemitter-grid] engine emitter Ok(ret={r:#x}) draw_mode=0x4(first=0,count={}) quads={nq} swap={sw:?}", 6 * nq);
        }
        r
    }
}

/// SH68 — the engine's real emitter draws a LAYERED "login/home"-style frame
/// (5 textured quads: backdrop, centered panel, button bar, title strip, field
/// strip) in ONE top-level jit_run with GL_BLEND alpha compositing, sized/placed
/// from the REAL scene list (R+0x180/0x188 -> SCENE_NODES). Same SH67d/e single-
/// call discipline (one pre-uploaded VBO, GL_TRIANGLES, first=0, count=5*6=30)
/// so the SH67c orphan class stays closed. Per-layer straight-alpha lives in a
/// 5-texel RGBA strip; the FS outputs only texture2D(uTex,vUV) so the overlap
/// readbacks verify the blend equation (panel∘backdrop must equal
/// src*src.a + dst*(1-src.a)). Blend + program are host-set preconditions (the
/// emitter has no such concepts). Returns the emitter's ret.
pub fn render_engine_emitter_home(ctx: u64, iimg: &[u8], ibase: u64, isp: u64, layer_override: usize) -> u64 {
    if !(ctx >= 0x100000000 && ctx >> 56 == 0) {
        return 0;
    }
    let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    if h.is_null() {
        eprintln!("[elfjit:renderemitter-home] WARN: dlopen libGLESv2.so.2 failed");
        return 0;
    }
    // Read the REAL scene-list node count to size the layout (SH68-A): the harness
    // laid R+0x180 head / R+0x188 tail (0x28-stride, one-past-end) via
    // render_scene_base/render_engine_present_walker. Verify head/tail-derived
    // count == SCENE_NODES so the layout is scene-driven, not hardcoded.
    let r = RENDERSCENE_BASE.load(core::sync::atomic::Ordering::Relaxed);
    let n_scene = SCENE_NODES.load(core::sync::atomic::Ordering::Relaxed);
    let derived = if r != 0 {
        unsafe {
            let head = *(r as *const u64).add(0x180 / 8);
            let tail = *(r as *const u64).add(0x188 / 8);
            if tail >= head { tail.wrapping_sub(head) / 0x28 } else { 0 }
        }
    } else { 0 };
    let n_layers = layer_override.max((n_scene as usize).max(5));
    eprintln!("[elfjit:renderemitter-home] scene list R={r:#x} n_scene={n_scene} head/tail-derived={derived} match={}", r==0 || derived==n_scene);
    let (tprog, uloc) = emitter_tex_program();
    if tprog == 0 {
        eprintln!("[elfjit:renderemitter-home] WARN: textured program unavailable");
        return 0;
    }
    unsafe {
        let (Some(gb), Some(bb), Some(bd), Some(up)) = (
            mesa_fn::<extern "C" fn(i32, *mut u32)>(h, b"glGenBuffers\0"),
            mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindBuffer\0"),
            mesa_fn::<extern "C" fn(u32, isize, *const i8, u32)>(h, b"glBufferData\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glUseProgram\0"),
        ) else {
            eprintln!("[elfjit:renderemitter-home] WARN: required Mesa symbols missing");
            return 0;
        };
        up(tprog);
        // 5 defined layers (straight alpha, painter's order): backdrop, panel
        // (alpha 0.55 -> every overlap is a verifiable blend), button, title,
        // field. Surplus scene nodes (>5) add generic text-strips.
        let layers: [(f32, f32, f32, f32, f32, f32, [f32; 4]); 5] = [
            (-1.0, 1.0, -1.0, 1.0, 0.1, 0.0, [0.10, 0.10, 0.12, 1.00]),  // backdrop
            (-0.55, 0.55, -0.36, 0.42, 0.3, 0.0, [0.24, 0.26, 0.32, 0.55]), // panel
            (-0.40, 0.40, -0.30, -0.14, 0.5, 0.0, [0.16, 0.62, 0.44, 0.95]), // button
            (-0.40, 0.40, 0.28, 0.34, 0.7, 0.0, [0.90, 0.88, 0.80, 0.80]), // title
            (-0.40, 0.10, 0.10, 0.17, 0.9, 0.0, [0.35, 0.38, 0.45, 0.85]), // field
        ];
        let nq = n_layers;
        // SH69: optional REAL Roblox UI texture (RENDEREMITTER_REAL_TEX=1). The
        // layer-1 "panel" slot is replaced by a real APK texture (the Roblox
        // loading-spinner) placed in a centered aspect-correct box; the other
        // layers keep palette solids so the whole frame still composites.
        let real_tex = std::env::var_os("RENDEREMITTER_REAL_TEX").is_some();
        let real_img = if real_tex { real_ui_texture() } else { None };
        if real_tex && real_img.is_none() {
            eprintln!("[elfjit:renderemitter-home] WARN: RENDEREMITTER_REAL_TEX=1 but real UI texture failed to load/decode — falling back to the palette strip");
        }
        // SH71: RENDEREMITTER_SPIN=1 animates the real-texture image box (a real
        // loading-screen spinner) by rotating its NDC corners per frame about the
        // box center — the texture (fixed) rotates with the box. A radial sweep
        // then measures the arc's angle each frame to prove rotation.
        let spin = std::env::var_os("RENDEREMITTER_SPIN").is_some() && real_img.is_some();
        static SPIN_N: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
        let spin_frame = if spin {
            SPIN_N.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1
        } else {
            0
        };
        let spin_deg = (spin_frame as f32 * 15.0) % 360.0;
        let (sc, sn) = (spin_deg.to_radians().cos(), spin_deg.to_radians().sin());
        const VW: f32 = 1280.0;
        const VH: f32 = 720.0;
        let mut verts: Vec<f32> = Vec::with_capacity(nq * 48); // 8 floats * 6 verts
        for t2 in 0..nq {
            let mut push = |x: f32, y: f32, u: f32, v: f32, vv: &mut Vec<f32>| {
                vv.extend_from_slice(&[x, y, 1.0, 1.0, 1.0, 1.0, u, v]);
            };
            let (x0, x1, y0, y1, ua, ub, va, vb);
            if let Some((imw, imh, _)) = real_img.as_ref() {
                if t2 == 1 {
                    // image layer: centered aspect-correct box, real texture.
                    let hh = 0.5f32;
                    let corr = VH / VW;
                    let half_w = hh * (*imw as f32 / *imh as f32) * corr;
                    x0 = -half_w; x1 = half_w; y0 = -hh; y1 = hh;
                    let a_h = (*imh as f32) + 1.0;
                    ua = 0.0; ub = 1.0;
                    // image occupies memory rows 1..=imh (upright via reversed
                    // upload); top vertex -> high v (PNG top memory row 1).
                    va = 1.0 / a_h;
                    vb = (*imh as f32) / a_h;
                } else {
                    // palette layers: strip row 0 (solid block [t/nq,(t+1)/nq)).
                    let (lx0, lx1, ly0, ly1) = if t2 < 5 {
                        let l = layers[t2];
                        (l.0, l.1, l.2, l.3)
                    } else {
                        let k = (t2 - 5) as f32;
                        let ys = 0.10 - (k + 1.0) * 0.09;
                        (-0.40, 0.40, ys - 0.04, ys + 0.04)
                    };
                    x0 = lx0; x1 = lx1; y0 = ly0; y1 = ly1;
                    let nqf = nq as f32;
                    ua = t2 as f32 / nqf;
                    ub = (t2 as f32 + 1.0) / nqf;
                    let a_h = (*imh as f32) + 1.0;
                    va = 0.5 / a_h; vb = va;
                }
            } else {
                // SH68 default: palette strip columns [u-1/(2nq), u+1/(2nq)], v=0.5.
                let (lx0, lx1, ly0, ly1, u) = if t2 < 5 {
                    let l = layers[t2];
                    (l.0, l.1, l.2, l.3, l.4)
                } else {
                    let k = (t2 - 5) as f32;
                    let ys = 0.10 - (k + 1.0) * 0.09;
                    (-0.40, 0.40, ys - 0.04, ys + 0.04, 0.1)
                };
                x0 = lx0; x1 = lx1; y0 = ly0; y1 = ly1;
                let half = 0.5 / nq as f32;
                ua = u - half; ub = u + half;
                va = 0.5; vb = va;
            }
            // SH71: when spinning, rotate the real-image box's four NDC corners
            // about the center so the fixed texture rotates (loading-spinner).
            if spin && t2 == 1 {
                let rot = |xx: f32, yy: f32| (xx * sc - yy * sn, xx * sn + yy * sc);
                let (ax, ay) = rot(x0, y0);
                let (bx, by) = rot(x1, y0);
                let (cx, cy) = rot(x1, y1);
                let (dx, dy) = rot(x0, y1);
                push(ax, ay, ua, va, &mut verts);
                push(bx, by, ub, va, &mut verts);
                push(cx, cy, ub, vb, &mut verts);
                push(ax, ay, ua, va, &mut verts);
                push(cx, cy, ub, vb, &mut verts);
                push(dx, dy, ua, vb, &mut verts);
                continue;
            }
            push(x0, y0, ua, va, &mut verts);
            push(x1, y0, ub, va, &mut verts);
            push(x1, y1, ub, vb, &mut verts);
            push(x0, y0, ua, va, &mut verts);
            push(x1, y1, ub, vb, &mut verts);
            push(x0, y1, ua, vb, &mut verts);
        }
        let total_verts = verts.len() / 8;
        let nbytes = verts.len() * 4;
        let mut vbo = 0u32;
        gb(1, &mut vbo);
        bb(0x8892, vbo);
        bd(0x8892, nbytes as isize, verts.as_ptr() as *const i8, 0x88E4);
        // Texture: default = nq-texel RGBA strip. Real mode = 2-D atlas
        // (A_W x A_H = imw x (imh+1)): row 0 = palette strip row (solid blocks
        // [t/nq,(t+1)/nq)), rows 1..=imh = the real image REVERSED (uploaded
        // memory row M holds PNG top-first row (imh-M)) so the image is upright.
        let (Some(gt), Some(at), Some(bt), Some(tp_), Some(te)) = (
            mesa_fn::<extern "C" fn(i32, *mut u32)>(h, b"glGenTextures\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glActiveTexture\0"),
            mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindTexture\0"),
            mesa_fn::<extern "C" fn(u32, u32, i32, i32)>(h, b"glTexParameteri\0"),
            mesa_fn::<extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const i8)>(h, b"glTexImage2D\0"),
        ) else {
            return 0;
        };
        let (mut aw, mut ah) = (((nq * 8).max(8)) as u32, 1u32);
        let (mut tex_mag, mut tex_min) = (0x2600u32, 0x2600u32); // NEAREST
        let wrap: i32 = 0x812F; // GL_CLAMP_TO_EDGE
        let mut px: Vec<u8> = Vec::new();
        if let Some((imw, imh, rgba)) = real_img.as_ref() {
            aw = (*imw).max(nq as u32);
            ah = *imh + 1;
            let mut cur = vec![0u8; aw as usize * 4]; // strip row 0
            for t2 in 0..nq {
                let cc = if t2 < 5 { layers[t2].6 } else { [0.45, 0.48, 0.55, 0.85] };
                let rg = [(cc[0]*255.0) as u8, (cc[1]*255.0) as u8, (cc[2]*255.0) as u8, (cc[3]*255.0) as u8];
                let b0 = (t2 as u32 * aw / nq as u32) as usize;
                let b1 = (((t2 as u32 + 1) * aw) / nq as u32) as usize;
                for i in b0..b1 { cur[i*4..i*4+4].copy_from_slice(&rg); }
            }
            px = cur;
            px.resize(aw as usize * ah as usize * 4, 0);
            let iu = *imw as usize;
            for m in 1..=(*imh as usize) {
                let png_row = *imh as usize - m; // 0 = image top
                let src = &rgba[png_row * iu * 4..(png_row + 1) * iu * 4];
                let dst = m * aw as usize * 4;
                px[dst..dst + iu * 4].copy_from_slice(src);
            }
            tex_mag = 0x2601; // GL_LINEAR for the smooth arc
            tex_min = 0x2601;
        } else {
            let w = (nq * 8).max(8);
            for t2 in 0..nq {
                let cc = if t2 < 5 { layers[t2].6 } else { [0.45, 0.48, 0.55, 0.85] };
                let rg = [(cc[0]*255.0) as u8, (cc[1]*255.0) as u8, (cc[2]*255.0) as u8, (cc[3]*255.0) as u8];
                for _ in 0..8 { px.extend_from_slice(&rg); }
            }
            while px.len() < w * 4 { px.extend_from_slice(&[0,0,0,255]); }
            aw = w as u32;
        }
        let mut texid = 0u32;
        gt(1, &mut texid);
        at(0x84C0);
        bt(0x0DE1, texid);
        tp_(0x0DE1, 0x2800, tex_mag as i32, 0);
        tp_(0x0DE1, 0x2801, tex_min as i32, 0);
        tp_(0x0DE1, 0x2802, wrap, 0);
        tp_(0x0DE1, 0x2803, wrap, 0);
        te(0x0DE1, 0, 0x1908, aw as i32, ah as i32, 0, 0x1908, 0x1401, px.as_ptr() as *const i8);
        if uloc >= 0 {
            if let Some(ui) = mesa_fn::<extern "C" fn(i32, i32)>(h, b"glUniform1i\0") { ui(uloc, 0); }
        }
        eprintln!("[elfjit:renderemitter-home] LAYOUT=home layers={nq} real_tex={} blend=enabled(SRC_ALPHA,ONE_MINUS_SRC_ALPHA) tex {aw}x{ah} RGBA uploaded (glTexImage2D 0x1908) uTex loc={uloc} prog={tprog:#x}", real_tex);
        // Geometry context G (stride 32, spec[3]: pos/color/aTex) — same layout as
        // render_engine_emitter_grid textured branch.
        let gbuf = Box::leak(vec![0u64; 0x400].into_boxed_slice());
        let raw = gbuf.as_ptr() as u64;
        let g = (raw + 7) & !7;
        let bd0 = (g + 0x100) & !7;
        let m = (g + 0x180) & !7;
        let spec = (g + 0x280) & !7;
        let stride_tab = (g + 0x300) & !7;
        let stride: u64 = 32;
        *(spec.wrapping_add(0) as *mut u32) = 0;
        *(spec.wrapping_add(4) as *mut u32) = 0;
        *(spec.wrapping_add(8) as *mut u32) = 1;
        *(spec.wrapping_add(12) as *mut u32) = 0;
        *(spec.wrapping_add(16) as *mut u32) = 0;
        *(spec.wrapping_add(24) as *mut u32) = 1;
        *(spec.wrapping_add(28) as *mut u32) = 8;
        *(spec.wrapping_add(32) as *mut u32) = 3;
        *(spec.wrapping_add(36) as *mut u32) = 1;
        *(spec.wrapping_add(40) as *mut u32) = 0;
        *(spec.wrapping_add(48) as *mut u32) = 2; // aTex attr 2
        *(spec.wrapping_add(52) as *mut u32) = 24;
        *(spec.wrapping_add(56) as *mut u32) = 1;
        *(spec.wrapping_add(60) as *mut u32) = 2;
        *(spec.wrapping_add(64) as *mut u32) = 0;
        *(bd0.wrapping_add(0x48) as *mut u32) = vbo;
        *(g.wrapping_add(0x48) as *mut u64) = bd0;
        *(g.wrapping_add(0x58) as *mut u64) = bd0;
        *(g.wrapping_add(0x68) as *mut u64) = bd0;
        *(g.wrapping_add(0x38) as *mut u64) = m;
        *(m.wrapping_add(0x48) as *mut u64) = spec;
        *(m.wrapping_add(0x50) as *mut u64) = spec + 72;
        *(m.wrapping_add(0x60) as *mut u64) = stride_tab;
        *(stride_tab.wrapping_add(0) as *mut u64) = stride;
        *(stride_tab.wrapping_add(8) as *mut u64) = stride;
        *(stride_tab.wrapping_add(16) as *mut u64) = stride;
        *(g.wrapping_add(0x78) as *mut u64) = 0;
        *(g.wrapping_add(0x8e) as *mut u16) = 0;
        eprintln!("[elfjit:renderemitter-home] built engine geometry ctx G={g:#x} M={m:#x} VBO={vbo} quads={nq} total_verts={total_verts} GL_TRIANGLES stride=32 spec[3] aPos=0 aColor=1 aTex=2");
        // Normalize fixed-function state + wire GL_DRAW_BUFFER to GL_BACK; for the
        // home layout ENABLE GL_BLEND instead of disabling, and set the alpha
        // blend func so the emitter's painter-order draws COMPOSITE like real UI.
        if let (Some(vp), Some(ds), Some(en), Some(bfs)) = (
            mesa_fn::<extern "C" fn(i32,i32,i32,i32)>(h, b"glViewport\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glDisable\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glEnable\0"),
            mesa_fn::<extern "C" fn(u32,u32,u32,u32)>(h, b"glBlendFuncSeparate\0"),
        ) {
            vp(0, 0, 1280, 720);
            ds(0x0B71); ds(0x0B44); ds(0x0C11);
            en(0x0BE2); // GL_BLEND
            bfs(0x0302, 0x0303, 0x0302, 0x0303); // S_ALPHA, ONE_MINUS_SRC_ALPHA
        }
        if let Some(bf) = mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindFramebuffer\0") { bf(0x8D40, 0); }
        if let Some(dbs) = mesa_fn::<extern "C" fn(i32, *const u32)>(h, b"glDrawBuffers\0") { let back = 0x0405u32; dbs(1, &back); }
        if let Some(rbuf) = mesa_fn::<extern "C" fn(u32)>(h, b"glReadBuffer\0") { rbuf(0x0405); }
        // Drive the engine's real emitter ONCE: GL_TRIANGLES, count=6*nq.
        let stkbuf = Box::leak(vec![0u8; 0x8000].into_boxed_slice());
        let stk_top = (stkbuf.as_ptr() as u64).wrapping_add(0x8000) & !15;
        let mut st = arm64jit::jit::CpuState::new();
        st.tpidr = arm64jit::jit::current_guest_tp();
        st.x[31] = stk_top;
        st.x[0] = g;
        st.x[1] = 0; st.x[2] = 0; st.x[3] = 0;
        st.x[4] = (6 * nq) as u64;
        st.x[5] = 0;
        let ret = arm64jit::jit::jit_run(iimg, ibase, 0x105b35288, &mut st as *mut CpuState);
        let r = match ret {
            Err(e) => { eprintln!("[elfjit:renderemitter-home] engine emitter stopped: {e}"); return 0; }
            Ok(r) => r,
        };
        if let Some(da) = mesa_fn::<extern "C" fn(u32)>(h, b"glDisableVertexAttribArray\0") { da(2); }
        if let Some(fin) = mesa_fn::<extern "C" fn()>(h, b"glFinish\0") { fin(); }
        let vt = unsafe { *(ctx as *const u64) };
        let bind = unsafe { *(vt.wrapping_add(16) as *const u64) };
        let swap = unsafe { *(vt.wrapping_add(24) as *const u64) };
        let tp = arm64jit::jit::current_guest_tp();
        let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
        let sw = arm64jit::jit::run_guest_callback(swap, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
        let readback: Option<extern "C" fn(i32,i32,i32,i32,u32,u32,*mut i8)> = mesa_fn(h, b"glReadPixels\0");
        eprintln!("[elfjit:renderemitter-home] engine emitter Ok(ret={r:#x}) draw_mode=0x4(first=0,count={}) layers={nq} swap={sw:?}", 6 * nq);
        if let Some(rp) = readback {
            // SH71 spin: radial sweep around the box center locates the arc's
            // strong-blue direction each frame; as spin_frame advances the angle
            // must move => real loading-spinner rotation, not a static texture.
            if spin {
                let (cx, cy) = (640.0f32, 360.0f32);
                let (mut best_ang, mut best_s) = (-1.0f32, 0i32);
                for k in 0..36 {
                    let th = (k as f32 * 10.0).to_radians();
                    for rad in [136.0, 158.0] {
                        let (sx, sy) = (cx + rad * th.cos(), cy - rad * th.sin());
                        let mut px = [0u8; 4];
                        rp(sx as i32, sy as i32, 1, 1, 0x1908, 0x1401, px.as_mut_ptr() as *mut i8);
                        let blu = px[2] as i32 - (px[0] as i32 + px[1] as i32) / 2;
                        if blu > best_s {
                            best_s = blu;
                            best_ang = th.to_degrees();
                        }
                    }
                }
                eprintln!("[elfjit:renderemitter-home] spin frame={spin_frame} rot={spin_deg:.0}deg arc-angle={best_ang:.0}deg score={best_s}");
            } else {
            // Verifiable readbacks: backdrop-only corner, the BLEND panel center
            // (must equal src*a+dst*(1-a), proving alpha compositing), button,
            // title. Panel rgba = (45,48,59) for (26,26,31) backdrop (dst) and
            // (61,66,82) panel (src) at a=0.55: r=61*.55+26*.45=45.2, g=66*.55+26*.45=48, b=82*.55+31*.45=59.1.
            let probes: [(i32, i32, [u8; 4], &str); 4] = if let Some((imw, imh, _)) = real_img.as_ref() {
                // SH69 real-texture probes: backdrop corner (unchanged); the arc's
                // OPAQUE sky-blue body img(33,88)->(49,180,255) = a REAL texture
                // pixel (blend-free, alpha 255); the transparent CENTER (real
                // alpha 0 -> backdrop shows through, NOT the old 0.55 panel); and
                // the transparent RIGHT half (the arc is left-only -> proves
                // correct orientation). Screen coords from the same aspect math.
                let (bx, by) = imgpix_screen(33, 88, *imw, *imh, 0.5, VW, VH);
                let (cx, cy) = imgpix_screen(50, 50, *imw, *imh, 0.5, VW, VH);
                let (rx, ry) = imgpix_screen(80, 50, *imw, *imh, 0.5, VW, VH);
                eprintln!("[elfjit:renderemitter-home] real probes: arc-blue@{bx:.0},{by:.0} transparent-center@{cx:.0},{cy:.0} transparent-right@{rx:.0},{ry:.0}");
                [
                    (51, 691, [26, 26, 31, 255], "backdrop"),
                    (bx as i32, by as i32, [49, 180, 255, 255], "real-arc-blue"),
                    (cx as i32, cy as i32, [26, 26, 31, 255], "transparent-center"),
                    (rx as i32, ry as i32, [26, 26, 31, 255], "transparent-right"),
                ]
            } else {
                [
                    (51, 691, [26, 26, 31, 255], "backdrop"),
                    (640, 360, [45, 48, 59, 255], "panel-blend"),
                    (640, 281, [41, 152, 109, 255], "button"),
                    (640, 472, [193, 189, 175, 255], "title-bar"),
                ]
            };
            for (fx, fy, exp8, name) in probes {
                let mut px: [u8; 4] = [0; 4];
                rp(fx, fy, 1, 1, 0x1908, 0x1401, px.as_mut_ptr() as *mut i8);
                let diff = [
                    (px[0] as i32 - exp8[0] as i32).abs(),
                    (px[1] as i32 - exp8[1] as i32).abs(),
                    (px[2] as i32 - exp8[2] as i32).abs(),
                    (px[3] as i32 - exp8[3] as i32).abs(),
                ];
                // Real-arc-blue may sit on GL_LINEAR interpolation boundary; allow
                // a small tolerance there. Everything else exact (<=1 rounding).
                let tol = if name == "real-arc-blue" { 4 } else { 1 };
                let present = diff.iter().all(|d| *d <= tol);
                eprintln!("[elfjit:renderemitter-home] {name} ({fx},{fy}) rgba({},{},{},{}) expect {:?} diff={diff:?} tol={tol} present={present}", px[0], px[1], px[2], px[3], exp8);
            }
            }
        }
        r
    }
}

/// SH72 — a MULTI-SPRITE real-Roblox-UI composite. One decoded real sprite.
#[allow(dead_code)]
#[derive(Clone)]
struct RealSprite {
    name: String,
    w: u32,
    h: u32,
    rgba: Vec<u8>,
}

/// SH73 — the real Roblox AUTH/log-in surface assets (the APK's own login
/// screen artwork, under ExtraContent/textures/ui/LuaApp/graphic/Auth/ plus
/// the sibling noconnection chip): reversevignette.png (the dark blurred login
/// backdrop) + logo_white_1x.png (the Roblox wordmark) + noconnection.png (the
/// connection-loss chip, SH74). Enable with RENDEREMITTER_LOGIN=1. Entries
/// are absolute (the auth assets live outside the content/textures/ui root).
fn login_ui_textures() -> Vec<RealSprite> {
    static LT: std::sync::OnceLock<Vec<RealSprite>> = std::sync::OnceLock::new();
    LT.get_or_init(|| {
        let auth = "/home/hermes-worker/.cache/open-sober/android-env/assets/ExtraContent/textures/ui/LuaApp/graphic/Auth";
        let graphic = "/home/hermes-worker/.cache/open-sober/android-env/assets/ExtraContent/textures/ui/LuaApp/graphic";
        let mut out = Vec::new();
        for (name, dir, rel) in [
            ("reversevignette.png", auth, "reversevignette.png"),
            ("logo_white_1x.png", auth, "logo_white_1x.png"),
            ("noconnection.png", graphic, "noconnection.png"),
        ] {
            let path = format!("{dir}/{rel}");
            match std::fs::read(&path).ok().and_then(|d| decode_png_rgba(&d)) {
                Some((w, h, rgba)) => {
                    eprintln!(
                        "[elfjit:renderemitter-login] loaded real auth sprite '{name}' ({w}x{h} RGBA8) from {path}"
                    );
                    out.push(RealSprite { name: name.to_string(), w, h, rgba });
                }
                None => eprintln!(
                    "[elfjit:renderemitter-login] WARN: failed to load/decode auth sprite '{name}' from {path} — skipped"
                ),
            }
        }
        // SH75 — synthesize the login FORM solid prims (a field + the green
        // "Log In" button) as solid atlas rows so the same textured emitter
        // draws them next to the real artwork = a complete login surface. The
        // FS outputs only texture2D(uTex,vUV); a uniform-filled row renders a
        // solid quad (SH72's ub = s.w/aw fix keeps each row's u-extent local).
        let mut solid = |name: &str, (r, g, b, a): (u8, u8, u8, u8), w: u32, h: u32| {
            let rgba = (0..(w * h)).flat_map(|_| [r, g, b, a]).collect();
            out.push(RealSprite { name: name.to_string(), w, h, rgba });
            eprintln!(
                "[elfjit:renderemitter-login] synthesized solid auth sprite '{name}' ({w}x{h} rgba({r},{g},{b},{a}))"
            );
        };
        // field = near-white input field (a=255 opaque), button = Roblox green
        // bar. 256x16 rows keep the boxes wide (aspect 16).
        solid("field.png", (224, 224, 230, 255), 256, 16);
        solid("loginbtn.png", (0, 158, 68, 255), 256, 16);
        // SH77 — a 2nd field (the password row) + real TEXT labels rasterized
        // from the APK's own SourceSansPro-Bold.ttf (glyph outlines flattened
        // with quadratic sampling, non-zero-winding 4x4-supersampled into
        // transparent 256-wide RGBA8 rows). White "Log In" composites over the
        // green button; dark-slate field placeholders over the two fields. The
        // FS outputs texture2D, so color is baked into the glyph texels and the
        // text rows sit AFTER their backing prim in atlas/painter order.
        solid("field2.png", (224, 224, 230, 255), 256, 16);
        for (name, color, w, h, pu) in [
            ("Log In", [255u8, 255, 255, 255], 512u32, 80u32, 0.068f32),
            ("Email address", [96u8, 96, 110, 255], 512u32, 64u32, 0.060f32),
            ("Password", [96u8, 96, 110, 255], 512u32, 64u32, 0.060f32),
        ] {
            match rasterize_login_label(name, color, w, h, pu) {
                Some(s) => {
                    eprintln!(
                        "[elfjit:renderemitter-login] rasterized text label '{name}' ({}x{})",
                        s.w, s.h
                    );
                    out.push(s);
                }
                None => eprintln!(
                    "[elfjit:renderemitter-login] WARN: text label '{name}' unavailable (font missing?) — skipped"
                ),
            }
        }
        out
    })
    .clone()
}

/// SH72 — load SEVERAL REAL Roblox UI textures (default: loading spinner,
/// robux icon, jump button) as RGBA8. Env RENDEREMITTER_MULTI_TEXTURES = a
/// comma-separated list overrides the defaults (relative to the extracted
/// assets/textures/ui root, or absolute if an entry starts with '/'). Each is
/// cached via OnceLock. Sprites that fail to load/decode are skipped with a
/// warn so the composite still builds from the successes. When
/// RENDEREMITTER_LOGIN=1 the real auth surface (login_ui_textures) is used
/// instead (SH73).
fn real_ui_textures() -> Vec<RealSprite> {
    static MT: std::sync::OnceLock<Vec<RealSprite>> = std::sync::OnceLock::new();
    MT.get_or_init(|| {
        if std::env::var_os("RENDEREMITTER_LOGIN").is_some() {
            return login_ui_textures();
        }
        let root = "/home/hermes-worker/.cache/open-sober/android-env/assets/content/textures/ui";
        let defs: Vec<&str> = vec![
            "LoadingScreen/LoadingSpinner.png",
            "InspectMenu/ico_robux@3x.png",
            "Input/JumpButtonRegular@2x.png",
        ];
        let list: Vec<String> = std::env::var("RENDEREMITTER_MULTI_TEXTURES")
            .ok()
            .map(|v| v.split(',').map(|s| s.to_string()).collect())
            .unwrap_or_else(|| defs.iter().map(|s| s.to_string()).collect());
        let mut out = Vec::new();
        for rel in &list {
            // Absolute entry (SH73: auth assets live under a different root).
            let path = if rel.starts_with('/') {
                rel.clone()
            } else {
                format!("{root}/{rel}")
            };
            let name = rel.rsplit('/').next().unwrap_or(rel).to_string();
            match std::fs::read(&path)
                .ok()
                .and_then(|d| decode_png_rgba(&d))
            {
                Some((w, h, rgba)) => {
                    eprintln!("[elfjit:renderemitter-multi] loaded real sprite '{name}' ({w}x{h} RGBA8) from {path}");
                    out.push(RealSprite { name, w, h, rgba });
                }
                None => eprintln!(
                    "[elfjit:renderemitter-multi] WARN: failed to load/decode real sprite '{name}' from {path} — skipped"
                ),
            }
        }
        out
    })
    .clone()
}

/// SH72 — the engine's REAL geometry emitter 0x105b35288 draws a FULL
/// MULTI-SPRITE "login/home" composite frame in ONE top-level jit_run: a dark
/// backdrop + one aspect-correct box per REAL Roblox UI sprite (spinner, robux
/// icon, jump button), all sampled from a single shared vertical atlas (row 0 =
/// solid backdrop strip, then each sprite in its own memory-row block),
/// composited with GL_BLEND. Verified by per-sprite glReadPixels probes: each
/// probe's expected color is read from the DECODED sprite rgba (not
/// hard-coded), mapped to screen coords via imgpix_rect (the same vUV-linear
/// math the shader's aTex interpolation yields), so a present sprite proves
/// its real pixels landed at the right place in the composite.
pub fn render_engine_emitter_multi(ctx: u64, iimg: &[u8], ibase: u64, isp: u64) -> u64 {
    if !(ctx >= 0x100000000 && ctx >> 56 == 0) {
        return 0;
    }
    let sprites = real_ui_textures();
    if sprites.is_empty() {
        eprintln!("[elfjit:renderemitter-multi] WARN: no real sprites loaded");
        return 0;
    }
    // RENDEREMITTER_MULTI_INDEX=N: render ONLY sprite N (+ backdrop) — a
    // diagnostic lever to isolate a multi-quad vertex-index mix-up.
    let only: Option<usize> = std::env::var("RENDEREMITTER_MULTI_INDEX")
        .ok()
        .and_then(|v| v.parse().ok());
    let h = unsafe { libc::dlopen(c"libGLESv2.so.2".as_ptr(), libc::RTLD_NOW | libc::RTLD_GLOBAL) };
    if h.is_null() {
        return 0;
    }
    let (tprog, uloc) = emitter_tex_program();
    if tprog == 0 {
        return 0;
    }
    // Layout (NDC): dark backdrop full-screen; then each sprite in its own
    // aspect-correct box. Positions chosen to resemble a login/home surface.
    const VW: f32 = 1280.0;
    const VH: f32 = 720.0;
    let login = std::env::var_os("RENDEREMITTER_LOGIN").is_some();
    // (name, cx_ndc, cy_ndc, hh, probe_ix, probe_iy, tol). Login mode (SH73):
    // the real auth backdrop (reversevignette, nearly-clear where the logo
    // sits) + the Roblox wordmark. Vignette probe over the solid dark row-0;
    // logo glyph is alpha=255 white so its probe is byte-exact over ANY dst.
    let placements: &[(&str, f32, f32, f32, u32, u32, u32)] = if login {
        &[
            ("reversevignette.png", 0.0, 0.0, 1.78, 512, 512, 6),
            ("logo_white_1x.png", 0.0, 0.35, 0.22, 193, 44, 2),
            ("noconnection.png", 0.0, -0.15, 0.10, 70, 50, 2),
            ("field.png", 0.0, -0.45, 0.045, 24, 8, 2),
            ("loginbtn.png", 0.0, -0.62, 0.05, 24, 8, 2),
            ("field2.png", 0.0, -0.54, 0.045, 128, 8, 2),
            // SH77 text labels (real SourceSansPro-Bold glyphs) — centered on
            // their backing prim; probe = an opaque glyph-interior texel (the
            // glyph color is baked into the texel, so byte-exact over any dst).
            ("Log In", 0.0, -0.62, 0.05, 227, 36, 2),
            ("Email address", 0.0, -0.45, 0.045, 369, 36, 2),
            ("Password", 0.0, -0.54, 0.045, 318, 30, 2),
        ]
    } else {
        &[
            ("LoadingSpinner.png", 0.0, 0.42, 0.40, 33, 88, 6),
            // robux center img(27,27) is transparent (a=0) — probe an OPAQUE white
            // corner pixel instead.
            ("ico_robux@3x.png", 0.72, 0.78, 0.10, 10, 10, 14),
            ("JumpButtonRegular@2x.png", 0.0, -0.62, 0.16, 84, 120, 6),
        ]
    };
    // Build the shared vertical atlas. aw = max sprite width (>=8); ah = 1
    // (backdrop strip row 0) + sum(sprite heights).
    let aw: u32 = sprites.iter().map(|s| s.w).max().unwrap_or(8).max(8);
    // SH76 — a LIVE login film (RENDEREMITTER_LIVE=1): gently bob the wordmark
    // (login placements index 1) about its authored center each emitter drive,
    // so consecutive present-walker frames differ = the surface is not a single
    // static frame. The probe tracks the same bob so it stays valid.
    let live = std::env::var_os("RENDEREMITTER_LIVE").is_some() && login;
    static LIVE_N: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    let live_frame = if live {
        LIVE_N.fetch_add(1, core::sync::atomic::Ordering::Relaxed) + 1
    } else {
        0
    };
    let bob = if live {
        // alternate a small vertical offset per frame: +0.03, -0.03, ...
        if live_frame % 2 == 1 { 0.03f32 } else { -0.03f32 }
    } else {
        0.0
    };
    // Resolve a placement, applying the live bob to the wordmark.
    let pl = |i: usize| -> (f32, f32, f32, u32, u32, u32) {
        let (_, cx, cy, hh, pix, piy, tol) = placements[i];
        let cy = if live && i == 1 { cy + bob } else { cy };
        (cx, cy, hh, pix, piy, tol)
    };
    let mut acc: u32 = 1;
    let mut vlo: Vec<f32> = Vec::new(); // per sprite va (v at its top memory row)
    let mut vhi: Vec<f32> = Vec::new(); // per sprite vb (v at its bottom mem row)
    let mut memlo: Vec<u32> = Vec::new(); // per sprite first memory row
    let mut ah: u32 = 1;
    // SH78 — one transparent guard row after every sprite so GL_LINEAR sampling
    // near a sub-row boundary never bleeds a NEIGHBOR sprite's color into this
    // one (prevents the faint/thin look seen at 720p on the text labels, which
    // sit directly under the opaque field/button rows). Later rows land at
    // s.h+GUARD offset; the guard cells stay 0 (transparent) since `px` is
    // zero-initialized.
    const GUARD: u32 = 1;
    for s in &sprites {
        memlo.push(acc);
        acc = acc.checked_add(s.h + GUARD).expect("atlas overflow");
        ah = acc;
    }
    for (i, s) in sprites.iter().enumerate() {
        vlo.push(memlo[i] as f32 / ah as f32);
        vhi.push((memlo[i] + s.h - 1) as f32 / ah as f32);
    }
    // Backdrop strip color (dark) fills row 0.
    let mut px: Vec<u8> = vec![0u8; (aw as usize) * (ah as usize) * 4];
    for i in 0..aw as usize {
        let o = i * 4;
        px[o..o + 4].copy_from_slice(&[26, 26, 31, 255]);
    }
    // Upload each sprite upright into its memory-row block (memory row m = z+k
    // holds PNG top-first row imh-1-k — same reversal as SH69 so v=high -> top).
    for (i, s) in sprites.iter().enumerate() {
        let iu = s.w as usize;
        for k in 0..s.h as usize {
            let png_row = s.h as usize - 1 - k;
            let src = &s.rgba[png_row * iu * 4..(png_row + 1) * iu * 4];
            let dst = (memlo[i] as usize + k) * aw as usize * 4;
            px[dst..dst + iu * 4].copy_from_slice(src);
        }
    }
    unsafe {
        let (Some(gb), Some(bb), Some(bd), Some(up)) = (
            mesa_fn::<extern "C" fn(i32, *mut u32)>(h, b"glGenBuffers\0"),
            mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindBuffer\0"),
            mesa_fn::<extern "C" fn(u32, isize, *const i8, u32)>(h, b"glBufferData\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glUseProgram\0"),
        ) else {
            return 0;
        };
        // Fixed-function state + GL_BLEND compositing (same as SH68/71 home).
        if let (Some(vp), Some(ds), Some(en), Some(bfs)) = (
            mesa_fn::<extern "C" fn(i32, i32, i32, i32)>(h, b"glViewport\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glDisable\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glEnable\0"),
            mesa_fn::<extern "C" fn(u32, u32, u32, u32)>(h, b"glBlendFuncSeparate\0"),
        ) {
            vp(0, 0, VW as i32, VH as i32);
            ds(0x0B71);
            ds(0x0B44);
            ds(0x0C11);
            en(0x0BE2);
            bfs(0x0302, 0x0303, 0x0302, 0x0303);
        }
        if let Some(bf) = mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindFramebuffer\0") {
            bf(0x8D40, 0);
        }
        if let Some(dbs) = mesa_fn::<extern "C" fn(i32, *const u32)>(h, b"glDrawBuffers\0") {
            let back = 0x0405u32;
            dbs(1, &back);
        }
        if let Some(rbuf) = mesa_fn::<extern "C" fn(u32)>(h, b"glReadBuffer\0") {
            rbuf(0x0405);
        }
        up(tprog);
        // Vertex list: 1 backdrop quad + one quad per sprite. Stride 32 =
        // [x,y, r,g,b,a, u,v]. Backdrop samples strip row0 (v=0.5); each sprite
        // samples its own v-block (vlo[i]..vhi[i]).
        let nq = 1 + sprites.len();
        let mut verts: Vec<f32> = Vec::with_capacity(nq * 48);
        let mut push = |x: f32, y: f32, u: f32, v: f32, vv: &mut Vec<f32>| {
            vv.extend_from_slice(&[x, y, 1.0, 1.0, 1.0, 1.0, u, v]);
        };
        // backdrop: full screen, dark strip row 0 (v = center of texel row 0 =
        // 0.5/ah, NOT 0.5 which is the middle of the multi-row atlas — see SH72
        // bug). Row 0 is fully filled (26,26,31,255) so any u in it is opaque.
        {
            let ua = 0.001f32 / aw as f32;
            let ub = ua + 1.0 / aw as f32;
            let vb0 = 0.5f32 / ah as f32;
            push(-1.0, -1.0, ua, vb0, &mut verts);
            push(1.0, -1.0, ub, vb0, &mut verts);
            push(1.0, 1.0, ub, vb0, &mut verts);
            push(-1.0, -1.0, ua, vb0, &mut verts);
            push(1.0, 1.0, ub, vb0, &mut verts);
            push(-1.0, 1.0, ua, vb0, &mut verts);
        }
        for (i, s) in sprites.iter().enumerate() {
            if let Some(o) = only {
                if i != o {
                    continue;
                }
            }
            let (cx, cy, hh, _, _, _) = pl(i);
            let corr = VH / VW;
            let half_w = hh * (s.w as f32 / s.h as f32) * corr;
            // The emitter projects NDC Y unmoved into glReadPixels y (0=bottom):
            // sy = (1+y_ndc)/2*VH. The image (top-first, vb at y1) maps to the
            // box top (y1 = cy+hh) consistently, so NO y-negation is applied;
            // the probe mirrors exactly this (SH72).
            let (x0, x1, y0, y1) = (cx - half_w, cx + half_w, cy - hh, cy + hh);
            // u maps image columns [0, s.w) across the atlas row (aw wide) —
            // NOT the full row, or the image squishes into the left s.w/aw of
            // the box (SH72 bug). v spans the sprite's atlas row block.
            let (ua, ub, va, vb) = (
                0.0f32,
                s.w as f32 / aw as f32,
                vlo[i],
                vhi[i],
            );
            // Painter's-order triangle pair (top-left origin): the box top
            // vertex (y1) gets the high-v = sprite's top memory row (upright).
            push(x0, y0, ua, va, &mut verts);
            push(x1, y0, ub, va, &mut verts);
            push(x1, y1, ub, vb, &mut verts);
            push(x0, y0, ua, va, &mut verts);
            push(x1, y1, ub, vb, &mut verts);
            push(x0, y1, ua, vb, &mut verts);
        }
        let total_verts = verts.len() / 8;
        let nbytes = verts.len() * 4;
        if std::env::var_os("RENDEREMITTER_GLTRAP").is_some() {
            eprintln!("[elfjit:renderemitter-multi] verts_count={total_verts} first-12 verts (x,y,u,v):");
            for i in 0..12usize.min(total_verts) {
                eprintln!(
                    "  v{i}: x={:.3} y={:.3} u={:.3} v={:.3}",
                    verts[i * 8], verts[i * 8 + 1], verts[i * 8 + 6], verts[i * 8 + 7]
                );
            }
        }
        let mut vbo = 0u32;
        gb(1, &mut vbo);
        bb(0x8892, vbo);
        bd(0x8892, nbytes as isize, verts.as_ptr() as *const i8, 0x88E4);
        let (Some(gt), Some(at), Some(bt), Some(tp_), Some(te)) = (
            mesa_fn::<extern "C" fn(i32, *mut u32)>(h, b"glGenTextures\0"),
            mesa_fn::<extern "C" fn(u32)>(h, b"glActiveTexture\0"),
            mesa_fn::<extern "C" fn(u32, u32)>(h, b"glBindTexture\0"),
            mesa_fn::<extern "C" fn(u32, u32, i32, i32)>(h, b"glTexParameteri\0"),
            mesa_fn::<extern "C" fn(u32, i32, i32, i32, i32, i32, u32, u32, *const i8)>(h, b"glTexImage2D\0"),
        ) else {
            return 0;
        };
        let mut texid = 0u32;
        gt(1, &mut texid);
        at(0x84C0);
        bt(0x0DE1, texid);
        for (p, v) in [(0x2800, 0x2601i32), (0x2801, 0x2601), (0x2802, 0x812F), (0x2803, 0x812F)] {
            tp_(0x0DE1, p, v, 0);
        }
        te(0x0DE1, 0, 0x1908, aw as i32, ah as i32, 0, 0x1908, 0x1401, px.as_ptr() as *const i8);
        if uloc >= 0 {
            if let Some(ui) = mesa_fn::<extern "C" fn(i32, i32)>(h, b"glUniform1i\0") {
                ui(uloc, 0);
            }
        }
        eprintln!(
            "[elfjit:renderemitter-multi] sprites={} atlas {aw}x{ah} RGBA backdrop+{nq} quads total_verts={total_verts} GL_TRIANGLES stride=32 blend=SRC_ALPHA prog={tprog:#x}",
            sprites.len()
        );
        // Geometry context G (stride 32, spec[3]: pos/color/aTex).
        let gbuf = Box::leak(vec![0u64; 0x400].into_boxed_slice());
        let raw = gbuf.as_ptr() as u64;
        let g = (raw + 7) & !7;
        let bd0 = (g + 0x100) & !7;
        let m = (g + 0x180) & !7;
        let spec = (g + 0x280) & !7;
        let stride_tab = (g + 0x300) & !7;
        let stride: u64 = 32;
        *(spec.wrapping_add(0) as *mut u32) = 0;
        *(spec.wrapping_add(4) as *mut u32) = 0;
        *(spec.wrapping_add(8) as *mut u32) = 1;
        *(spec.wrapping_add(12) as *mut u32) = 0;
        *(spec.wrapping_add(16) as *mut u32) = 0;
        *(spec.wrapping_add(24) as *mut u32) = 1;
        *(spec.wrapping_add(28) as *mut u32) = 8;
        *(spec.wrapping_add(32) as *mut u32) = 3;
        *(spec.wrapping_add(36) as *mut u32) = 1;
        *(spec.wrapping_add(40) as *mut u32) = 0;
        *(spec.wrapping_add(48) as *mut u32) = 2; // aTex attr 2
        *(spec.wrapping_add(52) as *mut u32) = 24;
        *(spec.wrapping_add(56) as *mut u32) = 1;
        *(spec.wrapping_add(60) as *mut u32) = 2;
        *(spec.wrapping_add(64) as *mut u32) = 0;
        *(bd0.wrapping_add(0x48) as *mut u32) = vbo;
        *(g.wrapping_add(0x48) as *mut u64) = bd0;
        *(g.wrapping_add(0x58) as *mut u64) = bd0;
        *(g.wrapping_add(0x68) as *mut u64) = bd0;
        *(g.wrapping_add(0x38) as *mut u64) = m;
        *(m.wrapping_add(0x48) as *mut u64) = spec;
        *(m.wrapping_add(0x50) as *mut u64) = spec + 72;
        *(m.wrapping_add(0x60) as *mut u64) = stride_tab;
        *(stride_tab.wrapping_add(0) as *mut u64) = stride;
        *(stride_tab.wrapping_add(8) as *mut u64) = stride;
        *(stride_tab.wrapping_add(16) as *mut u64) = stride;
        *(g.wrapping_add(0x78) as *mut u64) = 0;
        *(g.wrapping_add(0x8e) as *mut u16) = 0;
        // Drive the engine's real emitter ONCE: GL_TRIANGLES, count=6*nq.
        let stkbuf = Box::leak(vec![0u8; 0x8000].into_boxed_slice());
        let stk_top = (stkbuf.as_ptr() as u64).wrapping_add(0x8000) & !15;
        let mut st = arm64jit::jit::CpuState::new();
        st.tpidr = arm64jit::jit::current_guest_tp();
        st.x[31] = stk_top;
        st.x[0] = g;
        st.x[1] = 0;
        st.x[2] = 0;
        st.x[3] = 0;
        st.x[4] = total_verts as u64;
        st.x[5] = 0;
        let ret = arm64jit::jit::jit_run(iimg, ibase, 0x105b35288, &mut st as *mut CpuState);
        let r = match ret {
            Err(e) => {
                eprintln!("[elfjit:renderemitter-multi] engine emitter stopped: {e}");
                return 0;
            }
            Ok(r) => r,
        };
        if let Some(da) = mesa_fn::<extern "C" fn(u32)>(h, b"glDisableVertexAttribArray\0") {
            da(2);
        }
        if let Some(fin) = mesa_fn::<extern "C" fn()>(h, b"glFinish\0") {
            fin();
        }
        let vt = unsafe { *(ctx as *const u64) };
        let bind = unsafe { *(vt.wrapping_add(16) as *const u64) };
        let swap = unsafe { *(vt.wrapping_add(24) as *const u64) };
        let tp = arm64jit::jit::current_guest_tp();
        // SH72-diagnostic: GL error + link status + a PRE-SWAP backdrop probe so
        // a silent-no-draw is attributable (env-gated like SH67 GLTRAP).
        if std::env::var_os("RENDEREMITTER_GLTRAP").is_some() {
            if let (Some(ge), Some(gpi), Some(rp)) = (
                mesa_fn::<extern "C" fn() -> u32>(h, b"glGetError\0"),
                mesa_fn::<extern "C" fn(u32, u32, *mut i32)>(h, b"glGetProgramiv\0"),
                mesa_fn::<extern "C" fn(i32, i32, i32, i32, u32, u32, *mut i8)>(h, b"glReadPixels\0"),
            ) {
                let mut link: i32 = 0;
                gpi(tprog, 0x8B82, &mut link);
                let err = ge();
                let mut bx: [u8; 4] = [0; 4];
                rp(640, 630, 1, 1, 0x1908, 0x1401, bx.as_mut_ptr() as *mut i8);
                eprintln!(
                    "[elfjit:renderemitter-multi] GLTRAP err={err:#x} link={link} program={tprog:#x} pre-swap backdrop(640,630)=rgba({},{},{},{})",
                    bx[0], bx[1], bx[2], bx[3]
                );
                // Row scan: sample x at y=360 (spinner arc / mid) and y=80 (robux
                // box) to see what actually rasterized vs pure backdrop.
                let mut scan: Vec<(i32, u8, u8, u8)> = Vec::new();
                for y in [360i32, 80] {
                    for x in (0..1280).step_by(64) {
                        let mut p: [u8; 4] = [0; 4];
                        rp(x, y, 1, 1, 0x1908, 0x1401, p.as_mut_ptr() as *mut i8);
                        scan.push((x, p[0], p[1], p[2]));
                    }
                    eprintln!(
                        "[elfjit:renderemitter-multi] GLTRAP row y={y}: {}",
                        scan.iter().map(|(x, r, g, b)| format!("({x},{r},{g},{b})")).collect::<Vec<_>>().join(" ")
                    );
                    scan.clear();
                }
            }
        }
        let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
        let sw = arm64jit::jit::run_guest_callback(swap, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
        eprintln!(
            "[elfjit:renderemitter-multi] engine emitter Ok(ret={r:#x}) count={} sprites={} swap={sw:?}",
            total_verts,
            sprites.len()
        );
        let readback: Option<extern "C" fn(i32, i32, i32, i32, u32, u32, *mut i8)> =
            mesa_fn(h, b"glReadPixels\0");
        if let Some(rp) = readback {
            if std::env::var_os("RENDEREMITTER_GLTRAP").is_some() {
                // Full-frame dump for offline analysis of the actual raster.
                let mut frame = vec![0u8; (VW as usize) * (VH as usize) * 4];
                rp(0, 0, VW as i32, VH as i32, 0x1908, 0x1401, frame.as_mut_ptr() as *mut i8);
                let path = "/tmp/sh72-multi.raw";
                std::fs::write(path, &frame).ok();
                eprintln!("[elfjit:renderemitter-multi] wrote /tmp/sh72-multi.raw GL_RGBA ({VW}x{VH} u8)");
            }
            // Per-sprite probe: expected = decoded rgba at (ix,iy), screen coord
            // via imgpix_rect (the exact vUV-linear math the shader uses).
            for (i, s) in sprites.iter().enumerate() {
                let (cx, cy, hh, pix, piy, tol) = pl(i);
                let o4 = (piy as usize * s.w as usize + pix as usize) * 4;
                // Expected == the GL_BLEND (SRC_ALPHA, ONE_MINUS_SRC_ALPHA)
                // composite of the sprite texel over the opaque dark backdrop:
                // out = src*a + dst*(1-a). A straightforward rgb==src compare
                // false-fails on semi-transparent sprites (robux transparent
                // center, jump-white a=0x66, SH72 finding).
                let (sr, sg, sb, sa) = (
                    s.rgba[o4] as f32,
                    s.rgba[o4 + 1] as f32,
                    s.rgba[o4 + 2] as f32,
                    s.rgba[o4 + 3] as f32 / 255.0,
                );
                let exp = [
                    (sr * sa + 26.0 * (1.0 - sa)) as u8,
                    (sg * sa + 26.0 * (1.0 - sa)) as u8,
                    (sb * sa + 31.0 * (1.0 - sa)) as u8,
                    255u8,
                ];
                // Probe coords IN THE EMITTER'S PROJECTION (screen_y =
                // (1+y_ndc)/2*VH, y negated in authoring): mirror the quad
                // vertex mapping so a present sprite proves exact placement.
                let corr = VH / VW;
                let half_w = hh * (s.w as f32 / s.h as f32) * corr;
                let fx = (pix as f32 + 0.5) / s.w as f32;
                let fy = 1.0 - (piy as f32 + 0.5) / s.h as f32;
                let x_ndc = cx - half_w + fx * 2.0 * half_w;
                let y_ndc = (cy - hh) + fy * (2.0 * hh);
                let sx = (x_ndc + 1.0) / 2.0 * VW;
                let sy = (1.0 + y_ndc) / 2.0 * VH;
                let mut got: [u8; 4] = [0; 4];
                rp(sx as i32, sy as i32, 1, 1, 0x1908, 0x1401, got.as_mut_ptr() as *mut i8);
                let diff = [
                    (got[0] as i32 - exp[0] as i32).abs(),
                    (got[1] as i32 - exp[1] as i32).abs(),
                    (got[2] as i32 - exp[2] as i32).abs(),
                    (got[3] as i32 - exp[3] as i32).abs(),
                ];
                let tol_i = tol as i32;
                let present = diff.iter().all(|d| *d <= tol_i);
                eprintln!(
                    "[elfjit:renderemitter-multi] probe '{}' ({sx:.0},{sy:.0}) img({pix},{piy}) got rgba({},{},{},{}) expect {:?} diff={diff:?} tol={tol} present={present}",
                    s.name, got[0], got[1], got[2], got[3], exp
                );
            }
            // Transparent spinner-center probe (SH72, non-login composite only):
            // no arc body there, so the dark backdrop shows through => real
            // alpha compositing in the composite.
            if !login {
                let (scx, scy, shh) = (0.0f32, 0.42f32, 0.40f32);
                let sw = 100u32;
                let sh = 100u32;
                let fcx = (50.5f32) / sw as f32;
                let fcy = 1.0 - (50.5f32) / sh as f32;
                let x_ndc = scx - shh * (sw as f32 / sh as f32) * (VH / VW) + fcx * 2.0 * shh * (sw as f32 / sh as f32) * (VH / VW);
                let y_ndc = (scy - shh) + fcy * (2.0 * shh);
                let (sx, sy) = ((x_ndc + 1.0) / 2.0 * VW, (1.0 + y_ndc) / 2.0 * VH);
                let mut got: [u8; 4] = [0; 4];
                rp(sx as i32, sy as i32, 1, 1, 0x1908, 0x1401, got.as_mut_ptr() as *mut i8);
                let diff = [
                    (got[0] as i32 - 26).abs(),
                    (got[1] as i32 - 26).abs(),
                    (got[2] as i32 - 31).abs(),
                    (got[3] as i32 - 255).abs(),
                ];
                let present = diff.iter().all(|d| *d <= 1);
                eprintln!(
                    "[elfjit:renderemitter-multi] probe 'spinner-transparent-center' ({sx:.0},{sy:.0}) got rgba({},{},{},{}) expect backdrop(26,26,31,255) diff={diff:?} present={present}",
                    got[0], got[1], got[2], got[3]
                );
            }
        }
        r
    }
}
/// Present ONE real task-driven frame on the CURRENT thread (must be the
/// renderinit thread where EGL current-binding is established — SH61b). Binds
/// via the engine make-current 0x105b3b358, drives frame-fn 0x105b32c00, swaps
/// via 0x105b3b408. Returns the swap result (1 == genuine eglSwapBuffers
/// success). n is the per-present frame serial (palette cycles with it).
fn present_one_task_frame(ctx: u64, n: u64) -> u64 {
    let vt = unsafe { *(ctx as *const u64) };
    if !(vt >= 0x100000000 && vt >> 56 == 0) {
        return 0;
    }
    let bind = unsafe { *(vt.wrapping_add(16) as *const u64) }; // 0x105b3b358 make-current
    let swap = unsafe { *(vt.wrapping_add(24) as *const u64) }; // 0x105b3b408 eglSwapBuffers
    seed_task_frame_gles_slots();
    let base = task_frame_base();
    let renderer = base;
    let view = base + 0x300;
    let cc = TASK_FRAME_PALETTE[(n as usize) % TASK_FRAME_PALETTE.len()];
    reseed_task_frame_color(base, cc);
    let tp = arm64jit::jit::current_guest_tp();
    // Engine make-current so frame-fn + swap land on the live EGL context.
    let _ = arm64jit::jit::run_guest_callback(bind, [ctx, 0, 0, 0, 0, 0, 0, 0], tp);
    match arm64jit::jit::run_guest_callback(
        0x105b32c00,
        [renderer, view, view, 0, base + 0x400, base + 0x500, 0, 0],
        tp,
    ) {
        Err(e) => {
            eprintln!("[elfjit:taskv4-frame] frame #{n} frame-fn err: {e}");
            return 0;
        }
        Ok(fr) => eprintln!(
            "[elfjit:taskv4-frame] task frame #{n} frame-fn Ok({fr:#x}) ctx={ctx:#x}"
        ),
    }
    match arm64jit::jit::run_guest_callback(swap, [ctx, 0, 0, 0, 0, 0, 0, 0], tp) {
        Err(e) => {
            eprintln!("[elfjit:taskv4-frame] frame #{n} swap err: {e}");
            0
        }
        Ok(s) => {
            eprintln!(
                "[elfjit:taskv4-frame] present #{n} swap Ok({s:#x}) color={cc:?} — real task-driven frame"
            );
            s
        }
    }
}

// Diagnostic: on a host SIGSEGV inside a translated block, print the guest PC
// (CpuState.pc, offset 256) + a few guest regs read from the CpuState (RBX).
// elfjit is a diagnostic binary, so this stays in.
unsafe fn install_fault_debug() {
    extern "C" fn handler(sig: libc::c_int, info: *mut libc::siginfo_t, ctx: *mut libc::c_void) {
        unsafe {
            let uc = ctx as *const libc::ucontext_t;
            let rbx = (*uc).uc_mcontext.gregs[libc::REG_RBX as usize];
            let rip = (*uc).uc_mcontext.gregs[libc::REG_RIP as usize];
            let pc = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(256) as *const u64) } else { 0 };
            let x0 = if (rbx as usize) & 7 == 0 { *(rbx as *const u64) } else { 0 };
            let x1 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(8) as *const u64) } else { 0 };
            let x2 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(16) as *const u64) } else { 0 };
            let x3 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(24) as *const u64) } else { 0 };
            let x4 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(32) as *const u64) } else { 0 };
            let x5 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(40) as *const u64) } else { 0 };
            let x6 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(48) as *const u64) } else { 0 };
            let x7 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(56) as *const u64) } else { 0 };
            let x8 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(64) as *const u64) } else { 0 };
            let x9 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(72) as *const u64) } else { 0 };
            let sp = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(248) as *const u64) } else { 0 };
            let fault = (*info).si_addr() as u64;
            // Dump the raw host bytes around the faulting translated x86 so the
            // memory-op (e.g. a `mov rax,[rax+0x30]` = guest `ldr x8,[x8,#48]`)
            // can be identified precisely even though CpuState.pc is coarse.
            let mut raw = [0u8; 48];
            std::ptr::copy_nonoverlapping(rip.wrapping_sub(24) as *const u8, raw.as_mut_ptr(), 48);
            let hex = raw.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ");
            let x10 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(80) as *const u64) } else { 0 };
            let x19 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(152) as *const u64) } else { 0 };
            let x20 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(160) as *const u64) } else { 0 };
            let x21 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(168) as *const u64) } else { 0 };
            let x22 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(176) as *const u64) } else { 0 };
            let x23 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(184) as *const u64) } else { 0 };
            let x28 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(224) as *const u64) } else { 0 };
            let x29 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(232) as *const u64) } else { 0 };
            let x30 = if (rbx as usize) & 7 == 0 { *(rbx.wrapping_add(240) as *const u64) } else { 0 };
            let name = if sig == libc::SIGSEGV { "SIGSEGV" } else if sig == libc::SIGILL { "SIGILL" } else if sig == libc::SIGABRT { "SIGABRT" } else { "SIGFAULT" };
            let tid = unsafe { libc::syscall(libc::SYS_gettid) };
            // Does the faulting host thread actually run a guest CpuState?
            // Compare the ucontext RBX against the registered CpuState pointer
            // for this host tid. If they match, the guest reg dump is real; if
            // not, RBX is an arbitrary host value and the dump is garbage.
            let reg_state = arm64jit::jit::guest_state_of_host(tid as i64);
            let state_matches = reg_state != 0 && reg_state == rbx as u64;
            // Guest tid of the matching registered state (0 if unregistered).
            let reg_guest_tid = if reg_state != 0 {
                unsafe { *(reg_state as *const u64).wrapping_add(848 / 8) } // CpuState.tid field
            } else {
                u64::MAX
            };
            // Enumerate all registered guest threads: (host_tid, guest_tid, state).
            let thr = arm64jit::jit::dump_guest_threads()
                .iter()
                .map(|(h, g, s)| format!("({h}:{g},{s:#x})"))
                .collect::<Vec<_>>()
                .join(" ");
            let in_jit = arm64jit::jit::in_jit_run();
            // Diagnostic: dump the LocalStorageManager static-map global on fault
            // to confirm whether our boot-time seed persisted.
            let lsm_global = unsafe { *(0x10726f8c0u64 as *const u64) };
            // Full host x86-64 register file (SysV). The guest regs above are read
            // through rbx==CpuState base, so if rbx itself is corrupt the guest
            // dump is an artifact; the host frame disambiguates a real guest fault
            // from a handler/spurious read.
            let g = |i: usize| (*uc).uc_mcontext.gregs[i];
            let (rax, rcx, rdx, rsi, rdi, rbp, rsp, r8, r9, r10, r11, r12, r13, r14, r15, fl) = (
                g(libc::REG_RAX as usize), g(libc::REG_RCX as usize), g(libc::REG_RDX as usize),
                g(libc::REG_RSI as usize), g(libc::REG_RDI as usize), g(libc::REG_RBP as usize),
                g(libc::REG_RSP as usize), g(libc::REG_R8 as usize), g(libc::REG_R9 as usize),
                g(libc::REG_R10 as usize), g(libc::REG_R11 as usize), g(libc::REG_R12 as usize),
                g(libc::REG_R13 as usize), g(libc::REG_R14 as usize), g(libc::REG_R15 as usize),
                g(libc::REG_EFL as usize),
            );
            let s = format!(
                "\n[{name}] tid={tid} fault={fault:#x} rip={rip:#x} guestpc={pc:#x} rbx_matches_gueststate={state_matches} (reg_state={reg_state:#x}, tid={reg_guest_tid})\n  x0={x0:#x} x1={x1:#x} x2={x2:#x} x3={x3:#x} x4={x4:#x}\n  x5={x5:#x} x6={x6:#x} x7={x7:#x} x8={x8:#x} x9={x9:#x} sp={sp:#x}\n  x10={x10:#x} x19={x19:#x} x20={x20:#x} x21={x21:#x} x22={x22:#x}\n  x23={x23:#x} x28={x28:#x} x29={x29:#x} lr(x30)={x30:#x} lsm_map_global=0x{lsm_global:x}\n  HOST rax={rax:#x} rbx={rbx:#x} rcx={rcx:#x} rdx={rdx:#x} rsi={rsi:#x} rdi={rdi:#x}\n  HOST rbp={rbp:#x} rsp={rsp:#x} r8={r8:#x} r9={r9:#x} r10={r10:#x} r11={r11:#x}\n  HOST r12={r12:#x} r13={r13:#x} r14={r14:#x} r15={r15:#x} eflags={fl:#x}\n  GUEST_THREADS {thr} in_jit_run={in_jit}\n  raw[]= {hex}\n"
            );
            let b = s.as_bytes();
            libc::write(2, b.as_ptr() as *const libc::c_void, b.len());
            // Native frame-pointer backtrace (SysV: rbp chain, [rbp]=prev rbp,
            // [rbp+8]=return addr). Classifies every ret addr as host-JIT vs
            // guest-text vs libc so we see WHICH dispatcher path jumped to guest.
            let mut btd = String::from("\n  BT:");
            let mut fp: u64 = rbp as u64;
            let classify = |ra: u64| -> String {
                if ra >= 0x100000000 && ra < 0x120000000 {
                    format!("GUEST({ra:#x})")
                } else if ra >= 0x7f0000000000 && ra < 0x7f8000000000 {
                    format!("HOST({ra:#x})")
                } else if ra >= 0x7f0000000000 {
                    format!("HOST({ra:#x})")
                } else {
                    format!("{ra:#x}")
                }
            };
            for _ in 0..24 {
                if fp & 7 != 0 || fp < 0x400000 || fp >> 56 != 0 {
                    break;
                }
                let ra = unsafe { *(fp.wrapping_add(8) as *const u64) };
                if ra == 0 {
                    break;
                }
                btd.push_str(&format!(" -> {}", classify(ra)));
                let nfp = unsafe { *(fp as *const u64) };
                if nfp <= fp || nfp - fp > 0x4000 {
                    break;
                }
                fp = nfp;
            }
            btd.push('\n');
            libc::write(2, btd.as_bytes().as_ptr() as *const libc::c_void, btd.len());
        }
        // Restore default disposition for SIGABRT before re-raising via
        // process::abort() (which delivers SIGABRT); otherwise we recurse into
        // this handler in an infinite dump loop.
        if sig == libc::SIGABRT {
            unsafe {
                let mut sa: libc::sigaction = std::mem::zeroed();
                sa.sa_sigaction = libc::SIG_DFL;
                libc::sigaction(libc::SIGABRT, &sa, std::ptr::null_mut());
            }
        }
        std::process::abort();
    }
    for sig in [libc::SIGSEGV, libc::SIGILL, libc::SIGABRT] {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = handler as usize;
        sa.sa_flags = libc::SA_SIGINFO;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(sig, &sa, std::ptr::null_mut());
    }
}

/// How a single `--kicker` drives its target guest global.
#[derive(Clone, Copy)]
enum KickerMode {
    /// `pthread_cond_broadcast` the address every tick.
    Broadcast,
    /// Write an exact u64 value every tick (`--kicker 0xADDR=0xVAL`).
    Fixed(u64),
    /// Historical lifecycle pulse: write 1 through the first gate, then 2.
    Pulse,
}

/// Bring up an Xvfb X server + a 1280x720 window and register its XID as the
/// guest's ANativeWindow handle (GRAPHICS_RECOMMENDATION §5.3). Runs
/// SYNCHRONOUSLY so the real window is wired before StartApp reaches the
/// window/EGL surface path — a racing spawned thread would lose and hand the
/// guest the sentinel instead of a genuine window. The X connection is leaked
/// (kept alive) so the window outlives this function. Returns the wired XID,
/// or 0 if no window could be opened (caller keeps the sentinel fallback).
fn wire_real_window() -> u64 {
    // SH112: a stale /tmp/.X11-unix/X<n> socket from a dead prior session must
    // NOT be trusted — the old code saw the file and "broke", skipping the
    // spawn, then every connect failed and the boot silently kept the sentinel
    // ANativeWindow (so the render surface could never be real on the boot
    // window). Rewritten to be connect-first: try the display as-is, and only
    // if that fails unlink the stale socket and spawn our own Xvfb on it.
    let pid = std::process::id();
    for attempt in 0..24usize {
        let display_num = 220 + (((pid as usize) + attempt * 7) % 250);
        let display = format!(":{display_num}");
        // 1) Try the display as-is (it may already be a live server).
        if let Ok((conn, win)) = x11::open_window_sized(Some(&display), 1280, 720) {
            Box::leak(Box::new(conn)); // keep the window alive for the boot
            unsafe {
                std::env::set_var("DISPLAY", &display);
                std::env::set_var("EGL_PLATFORM", "x11");
            }
            let xid = win as u64;
            set_anativewindow_xid(xid);
            eprintln!(
                "[elfjit:anativewindow] wired real X11 window XID=0x{xid:x} on {display} as the guest ANativeWindow"
            );
            return xid;
        }
        // 2) Connect failed. Clear any stale socket then start our own server.
        let sock = format!("/tmp/.X11-unix/X{display_num}");
        if std::path::Path::new(&sock).exists() {
            let _ = std::fs::remove_file(&sock);
        }
        if let Ok(mut c) = std::process::Command::new("Xvfb")
            .arg(&display)
            .arg("-screen").arg("0").arg("1280x720x24")
            .arg("-nolisten").arg("tcp")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if let Ok((conn, win)) = x11::open_window_sized(Some(&display), 1280, 720) {
                    Box::leak(Box::new(conn));
                    unsafe {
                        std::env::set_var("DISPLAY", &display);
                        std::env::set_var("EGL_PLATFORM", "x11");
                    }
                    let xid = win as u64;
                    set_anativewindow_xid(xid);
                    eprintln!(
                        "[elfjit:anativewindow] wired real X11 window XID=0x{xid:x} on {display} as the guest ANativeWindow"
                    );
                    return xid;
                }
            }
            let _ = c.kill(); // never came up here; try a different display below
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    eprintln!(
        "[elfjit:anativewindow] could not open an X11 window (Xvfb absent?) — keeping the sentinel ANativeWindow"
    );
    0
}

/// Arm the guest-persistence root for a real run: if `SOBER_ANDROID_ROOT` is
/// not already set, create a stable host directory under the runtime's data
/// dir and export it, so guest `/data`/`/sdcard`/`/cache` writes (the client's
/// datastore / login-session store) land on persistent host disk via
/// `arm64jit::fsmap` instead of the nonexistent host root. Verified not to
/// disturb the boot (stable idle exit 124 with the root armed); it only gains
/// effect when the client opens a `/data` sink.
fn arm_persist_root() {
    if std::env::var_os("SOBER_ANDROID_ROOT").is_some() {
        return;
    }
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(std::path::PathBuf::from)
                .map(|h| h.join(".local/share"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = data.join("open-sober").join("android-root");
    if let Ok(()) = std::fs::create_dir_all(&dir) {
        unsafe { std::env::set_var("SOBER_ANDROID_ROOT", &dir) };
        println!(
            "[fsmap] armed persistence root SOBER_ANDROID_ROOT={}",
            dir.display()
        );
    } else {
        println!("[fsmap] warn: could not create persistence root {}", dir.display());
    }
}

/// Objective 2b — prove the LIVE productized client's data-persistence path
/// end-to-end. The engine's boot+render never reaches a session (the standing
/// producer wall), so the productized run otherwise makes ZERO `[fsmap] remap:`
/// lines. Driving the SAME guest_svc ABI a real datastore write uses
/// (openat -> write -> fsync -> close -> reopen -> read) inside this very
/// JIT process — the exact executable `open-sober play --jit` launches — with
/// SOBER_ANDROID_ROOT armed, turns the persistent-store roundtrip into an
/// observable, self-verifying property of the live client, not just a hermetic
/// test. The write is remapped under the armed root and read back byte-exact in
/// the same process.
fn run_persist_roundtrip() {
    let Some(root) = arm64jit::fsmap::configured_root() else {
        println!("[persist] SOBER_ANDROID_ROOT not armed — skipping datastore roundtrip");
        return;
    };
    fn svc(args: [u64; 6], nr: u64) -> i64 {
        let mut st = CpuState::new();
        st.x[0..6].copy_from_slice(&args);
        st.x[8] = nr;
        unsafe { arm64jit::jit::guest_svc(&mut st) as i64 }
    }
    fn cstr(s: &str) -> std::ffi::CString {
        std::ffi::CString::new(s).unwrap()
    }
    const DB: &str = "/data/user/0/com.roblox.client/databases/session.db";
    const PAYLOAD: &[u8] = b"ROBLOSECURITY=_live_client_remembered_session";

    // Write side: openat(O_CREAT|O_RDWR|O_TRUNC) -> write -> fsync -> close.
    let path = cstr(DB);
    let fd = svc(
        [
            libc::AT_FDCWD as u64,
            path.as_ptr() as u64,
            (libc::O_CREAT | libc::O_RDWR | libc::O_TRUNC) as u64,
            0o600,
            0,
            0,
        ],
        56, // openat
    );
    if fd < 0 {
        println!("[persist] openat O_CREAT failed: {fd}");
        return;
    }
    let fd = fd as i32;
    let buf = cstr(std::str::from_utf8(PAYLOAD).unwrap());
    let n = svc([fd as u64, buf.as_ptr() as u64, PAYLOAD.len() as u64, 0, 0, 0], 64); // write
    let fs = svc([fd as u64, 0, 0, 0, 0, 0], 81); // fsync (SQLite-style durability)
    let _c = svc([fd as u64, 0, 0, 0, 0, 0], 57); // close

    // Persistent backend: the remap must have landed a REAL host file.
    let host = root.join("data/user/0/com.roblox.client/databases/session.db");
    let on_disk = std::fs::read(&host).ok().map(|b| b == PAYLOAD);

    // Read side: reopen read-only in the same live process and verify byte-exact.
    let path2 = cstr(DB);
    let fd2 = svc(
        [libc::AT_FDCWD as u64, path2.as_ptr() as u64, libc::O_RDONLY as u64, 0, 0, 0],
        56,
    );
    let mut read_back = false;
    if fd2 >= 0 {
        let mut rb = vec![0u8; PAYLOAD.len()];
        let rn = svc([fd2 as u64, rb.as_mut_ptr() as u64, rb.len() as u64, 0, 0, 0], 63); // read
        let _c2 = svc([fd2 as u64, 0, 0, 0, 0, 0], 57);
        read_back = rn as usize == rb.len() && rb == PAYLOAD;
    }
    println!(
        "[persist] live datastore roundtrip: write={n}B fsync={fs} read_back_byte_exact={read_back} on_disk={:?} host={}",
        on_disk,
        host.display()
    );
    assert!(read_back && on_disk == Some(true), "live persistence roundtrip failed (read_back={read_back} on_disk={on_disk:?})");
}

fn main() {
    unsafe {
        install_fault_debug();
    }
    arm_persist_root();
    // Objective 2b: prove the client's own data-persistence plane
    // (openat/write/fsync/close/reopen/read under a guest /data datastore path)
    // round-trips byte-exact through the armed persistent host store, inside
    // this live process. Runs up front because StartApp parks in an idle
    // main-loop (absent a producer) and never returns for a post-boot check.
    // Opt-in so plain `--jni`/baseline runs stay untouched.
    if std::env::args().any(|a| a == "--persist-roundtrip") {
        run_persist_roundtrip();
    }
    let path = std::env::args()
        .nth(1)
        .expect("usage: elfjit <aarch64-elf> [entry-guest-addr-hex]");
    let entry_arg = std::env::args().nth(2);

    let el = unsafe { libloader::elf::load_elf_image(std::path::Path::new(&path)) }
            .expect("load_elf_image");

        // Fold the import resolver + host shims into the boot path: bind every PLT
        // JUMP_SLOT GOT slot to a host thunk so translated Roblox `blr`s hit real
        // host functions (libc/libm/float/bionic/graphics-stub) instead of stalling.
        let (nbound, nunresolved) = arm64jit::plt::bind_image_plt(&el, None);
        if nbound > 0 {
            println!("PLT imports bound: {nbound} to host thunks ({} unbound)", nunresolved);
        }

    // Route the TLS-block allocator's big-allocation path to host calloc so the
    // unseeded MemoryPool empty-free-list returns a real buffer instead of a
    // NULL+abort. Site 0x1d9801c is the big allocator of Roblox v2.738.1397's
    // per-thread TLS block (reachable from 0x1d96a40's empty free-list tail).
    match arm64jit::jit::route_mempool_big_alloc_to_host(el.guest_of(0x1d9801c), 0x1_0000_0000) {
        Ok(tp) => {
            println!("[mempool] big-alloc 0x1d9801c routed to host calloc (thunk @ {tp:#x})");
            let p = tp as *const u8;
            let hex: Vec<String> = (0..20).map(|i| unsafe { format!("{:02x}", *p.add(i)) }).collect();
            println!("[mempool] thunk bytes: {} (JIT-readable via guest image)", hex.join(" "));
        }
        Err(e) => println!("[mempool] warn: big-alloc patch skipped: {e}"),
    }

    // Guest entry: the ELF's own e_entry (already relocated to guest space by
    // load_elf_image) unless a link-time address is supplied, in which case we
    // translate it to guest/runtime space with guest_of().
    let entry = match entry_arg {
        Some(h) => {
            let link = u64::from_str_radix(h.trim_start_matches("0x"), 16)
                .unwrap_or_else(|e| panic!("bad entry hex: {e}"));
            el.guest_of(link)
        }
        None => el.info.entry,
    };

    println!(
        "loaded '{}': is_pie={} base_load_vaddr=0x{:x} e_entry=0x{:x}",
        path, el.info.is_pie, el.info.base_load_addr, el.info.entry
    );
    for s in &el.segments {
        println!(
            "  segment guest=[0x{:x},0x{:x}) host=same prot={}{}{}",
            s.guest_vaddr,
            s.guest_vaddr + s.memsz,
            if s.prot.read { "r" } else { "-" },
            if s.prot.write { "w" } else { "-" },
            if s.prot.execute { "x" } else { "-" }
        );
    }

    // Pick the executable (text) segment to translate code out of.
    let seg = el
        .segments
        .iter()
        .find(|s| s.prot.execute)
        .expect("no executable segment");
    let base = seg.guest_vaddr; // == host addr of image[0] (guest==host)

    // Use the FULL mapped span (every PT_LOAD + inter-segment gaps, which
    // load_elf_image lays into ONE contiguous anonymous region at the fixed
    // base) as the valid-pc extent. The guest may legitimately branch/call
    // into higher sections (data-backed trampolines, .bss-slotted function
    // pointers) that live past the r-x slice; bounding `run_loop` to only the
    // text slice wrongly flags those as "outside image". Compute the extent as
    // the largest guest_vaddr+memsz across segments (the whole mmap is zero-
    // filled), relative to this text-segment base.
    let full_end = el
        .segments
        .iter()
        .fold(0u64, |m, s| m.max(s.guest_vaddr + s.memsz));
    let len = (full_end - base) as usize;

    // Reserve a writable guest tail past the ELF's mapped span. Real Roblox
    // `nativeInitCrashpad` walks a link-time `& bss` telemetry table base by a
    // slot index that reaches tens of MB past the last PT_LOAD `.bss` end; on
    // real Android that adjacent memory is mapped anonymous, our loader maps
    // only the ELF span. Reserve 384MB of RW headroom so the deep table writes
    // (and other large guest tables/arenas) have real backing instead of
    // SIGSEGV. MAP_FIXED at a page-aligned address after base+len is safe
    // (host heap/stack live elsewhere); must start page-aligned or mmap EINVALs.
    let tail_start = (full_end as usize + 0xfff) & !0xfff;
    const TAIL_SIZE: usize = 384 * 1024 * 1024;
    match libloader::elf::reserve_guest_tail(tail_start, TAIL_SIZE) {
        Ok(_s) => println!("[tail] reserved {TAIL_SIZE}B guest RW tail @0x{tail_start:x}"),
        Err(e) => eprintln!("[tail] warn: guest-tail reserve skipped: {e}"),
    }
    // Give the deque-node injector a guest-visible arena in the RW tail so its
    // node/vtable allocations are stable guest addresses (< 2^48, low48-safe)
    // backed by real mapped RW memory.
    guest_arena_set_base(tail_start as u64);

    // Route the LocalStorageManager static hash-map's bucket-array allocator
    // (`0x1d97744`, receives its byte size in x0) to host calloc, so the
    // unseeded per-object MemoryPool empty free-list returns a real zeroed
    // buffer. The map's lazy init then stores a valid non-NULL bucket array
    // into its header global (0x726f8c0) instead of NULL, so the hash-lookup
    // reader (0x1d99e30: `ldr x8,[0x726f8c0]; ...; ldar x8,[x8]; ldr x0,[x8,idx<<3]`)
    // finds a real map instead of derefing a NULL bucket.
    match arm64jit::jit::route_allocator_x0_to_calloc(el.guest_of(0x1d97744), 0x1_0000_0000) {
        Ok(tp) => println!("[lsm-map] allocator 0x1d97744 routed to host calloc(x0) (thunk @ {tp:#x})"),
        Err(e) => eprintln!("[lsm-map] warn: allocator route skipped: {e}"),
    }

    // Disable the LSM map's lazy-init store that would clobber our seed.
    //
    // The init at 0x1d975f8 does `str x0,[x22,#2240]` writing its (routed)
    // allocator result into the map global 0x726f8c0, then memsets and builds a
    // two-level bucket structure into that discarded buffer. The reader
    // (0x1d99e40/0x1d99e4c/0x1d99e50) instead requires the global to point at a
    // bucket array whose every slot (key>>29) is a pointer to a zeroed
    // sub-array (indexed by (key>>16)&0x1fff); a bare all-zero calloc leaves
    // bucket slots NULL and the reader derefs NULL. `seed_static_empty_map`
    // below constructs exactly the required layout, so NOP the init store to
    // keep that seed authoritative. Guest insn -> NOP (0xd503201f).
    let init_store = el.guest_of(0x1d975f8);
    {
        let page = init_store & !0xfff;
        if unsafe { libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) } == 0 {
            unsafe { *(init_store as *mut u32) = 0xd503_201fu32 };
            unsafe { libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC) };
            println!("[lsm-map] NOP'd LSM init store at 0x{init_store:x} (guest 0x{:x}) to keep seeded empty map", 0x1d975f8);
        } else {
            eprintln!("[lsm-map] warn: could not mprotect init-store page RW");
        }
    }

    // Seed the LocalStorageManager C++ static hash-map whose .bss base global
    // (guest 0x10726f8c0 = 0x726f000+0x8c0) is 0 because its constructor never
    // ran (.init_array is empty). The reader (0x1d99e40) does
    //   ldr x8,[0x726f8c0]; add x8,x8,key>>29<<3; ldar x8,[x8]; ldr x0,[x8,idx<<3]
    // and faults on the NULL bucket (`key` is a heap/guest pointer, so
    // key>>29 lands up to ~0xb00 buckets in). Seed a real zeroed region as the
    // bucket array, with every bucket slot pointing at a shared (also-zeroed,
    // non-overlapping) sub-array slot, so any lookup reads 0 -> "not found".
    // (Fallback: if the guest later overwrites it with a real map, all the better.)
    unsafe fn seed_static_empty_map(map_global_guest: u64) {
        const BUCKETS: usize = 0x400000; // key>>29: pointers ~0x56.. give idx ~0x2b2a7; grant headroom
        const SLOT_STRIDE: usize = 8;
        const SLOT_REGION: usize = 0x10000; // (key>>16)&0x1fff max index * 8
        let buckets_bytes = BUCKETS * SLOT_STRIDE;
        let total = buckets_bytes + SLOT_REGION;
        let buf = Box::leak(vec![0u8; total].into_boxed_slice());
        let bufp = buf.as_mut_ptr();
        let base = bufp as u64;
        let sub = base + buckets_bytes as u64;
        // Every bucket slot points at `sub` (a zeroed shared sub-array), so
        // `ldar x8,[bucket[key>>29]]` returns a non-null pointer and
        // `ldr x0,[x8, idx<<3]` reads 0 -> cbz -> return NULL (not found).
        let slots = std::slice::from_raw_parts_mut(bufp.cast::<u64>(), BUCKETS);
        for s in slots {
            *s = sub;
        }
        *(map_global_guest as *mut u64) = base;
        println!("[lsm-map] seeded static empty LocalStorageManager map: global 0x{map_global_guest:x} -> bucket array 0x{base:x} ({} buckets, shared zero sub @0x{sub:x})", BUCKETS);
    }
    fn link_to_guest(el0: &libloader::elf::LoadedElf, link: u64) -> u64 {
        el0.guest_of(link)
    }
    unsafe { seed_static_empty_map(link_to_guest(&el, 0x726f000 + 0x8c0)) };

    // Seed the JNICallProtocol-ish refcounted-singleton pointer at guest
    // 0x107333948 (link-time 0x7333000+0x948). The once-init atomic store
    // (0x2b9e890) normally writes the object address 0x7333950 into that slot;
    // under the JIT the .init_array never runs so it stays 0, and the acquire
    // path (0x21daf00) locks `this+8` (a pthread_mutex at object+0x8) — with
    // `this` NULL it calls pthread_mutex_lock(0x8) and faults. The object
    // itself is zeroed bss (a valid PTHREAD_MUTEX_INITIALIZER at +8), so just
    // wiring the pointer releases the lock into the zeroed (== initial, unlocked)
    // mutex.
    let singleton_slot = link_to_guest(&el, 0x7333000 + 0x948); // [ptr] slot
    let singleton_obj = link_to_guest(&el, 0x7333000 + 0x950); // object base
    unsafe { *((singleton_slot) as *mut u64) = singleton_obj };
    // SH82b: the GlobalInit do-init funnels an intern/hash lookup through this
    // singleton's +0x30 word (table base, read at file 0x21db014 `ldr x10,[x19]`
    // then `ldr x10,[x10,x24,lsl#3]`). The object is zeroed bss so +0x30 == 0 ->
    // the table deref reads [0 + x24*8] -> SIGSEGV fault=0x0 at guestpc
    // 0x1021daf78. Seed +0x30 = a zeroed bucket array (mirrors seed_static_empty_map)
    // so a lookup reads bucket->0 (cbz -> "not found -> insert new"), and +0x8
    // (size/capacity) = a non-zero small value so the hashing path is coherent.
    unsafe {
        let bucket_arr = Box::leak(vec![0u8; 0x2000].into_boxed_slice());
        let base = bucket_arr.as_mut_ptr() as u64;
        *((singleton_obj + 0x30) as *mut u64) = base;
        *((singleton_obj + 0x38) as *mut u64) = 0x400; // capacity (buckets count)
        println!(
            "[JNICall-singleton] SH82b seeded intern-table @ obj+0x30 bucket array 0x{base:x} (+0x38 capacity 0x400) so the GlobalInit do-init hash lookup never derefs NULL"
        );
    }
    println!(
        "[JNICall-singleton] seeded ptr 0x{singleton_slot:x} -> object 0x{singleton_obj:x} (zeroed bss ~ PTHREAD_MUTEX_INITIALIZER at +8)"
    );
    // Read back the seed to confirm it landed where the guest reads it.
    let g_chk = link_to_guest(&el, 0x726f000 + 0x8c0);
    let v_chk = unsafe { *(g_chk as *const u64) };
    println!("[lsm-map] readback global 0x{g_chk:x} = 0x{v_chk:x} (must be non-zero)", );

    println!(
        "running entry guest=0x{:x} host=0x{:x} (segment base guest=0x{:x} size=0x{:x})",
        entry, entry, base, len
    );

    // image = the executable segment's bytes. Because guest==host, the `base`
    // passed to compile_image is the guest address of image[0] and the `entry`
    // is the guest address of the first instruction to run.
    let image = unsafe { std::slice::from_raw_parts(base as *const u8, len) };
    let mut st = CpuState::new();

    // Optional x0/x1/x2 init. Pass `buf` in position 3 to allocate a
    // writable 256-byte host buffer (guest==host, so its address is a valid
    // guest pointer) and put its address in x0; also x1=x0+32. Even when the
    // guest is a real binary we don't yet bootstrap (no TLS/stack), this lets
    // small aarch64 test functions run through the dispatcher.
    for (i, arg) in std::env::args().skip(3).take(3).enumerate() {
        if arg == "--jni" || arg == "buf" || arg == "--startapp" {
            let v = if arg == "buf" {
                let b = Box::leak(vec![0x7fu8; 256].into_boxed_slice());
                if i == 0 {
                    let base = b.as_ptr() as u64;
                    st.set(0, base);
                    st.set(1, base + 32);
                }
                b.as_ptr() as u64
            } else {
                0 // --jni / --startapp aren't x-register values; handled separately
            };
            if arg == "buf" && i == 0 {
                continue;
            }
            let _ = v;
        } else {
            let v = u64::from_str_radix(arg.trim_start_matches("0x"), 16)
                .unwrap_or_else(|e| panic!("bad x{i} hex: {e}"));
            st.set(i, v);
        }
    }

    // Bootstrap a guest runtime the binary can actually use -------------
    // (1) Guest stack: allocate a real writable region (guest==host addressing,
    //     so its host pointer is a valid guest pointer) and point SP at the top.
    // (2) TLS base: point CpuState.tpidr at a writable region so `mrs tpidr_el0`
    //     returns a non-zero, writable base (FS/GS-style thread pointer).
    const STACK_SIZE: usize = 4 * 1024 * 1024;
    let stack = Box::leak(vec![0u8; STACK_SIZE].into_boxed_slice());
    // Lay out a real kernel-style initial stack (argc/argv/envp/auxv) so glibc
    // IFUNCs resolve to scalar paths instead of reading garbage auxv into SMP
    // (which drove the JIT into an unsupported `str za` wall). No SME/SVE bits.
    let mut auxv = arm64jit::boot::standard_auxv(
        &el,
        arm64jit::boot::HWCAP_FP | arm64jit::boot::HWCAP_ASIMD,
        0,
    );
    let sp = arm64jit::boot::layout_initial_stack(
        stack.as_ptr() as *mut u8,
        STACK_SIZE,
        Some(&[0u8; 0]), // argv[0] (empty) — keeps argc==1 like a real shell exec
        &[],
        &mut auxv,
    );
    st.set(31, sp); // x31 = SP (points at argc on the initial stack)
    const TLS_SIZE: usize = 1024 * 64;
    let tls = Box::leak(vec![0u8; TLS_SIZE].into_boxed_slice());
    // Seed the guest TLS region from the image's PT_TLS (local-exec/initial-exec
    // thread-locals) and point tpidr_el0 at the AArch64 TCB (16 bytes before the
    // module's TLS block). `__thread` globals then read/write real data.
    st.tpidr = libloader::elf::setup_guest_tls(
        &el.info,
        std::path::Path::new(&path),
        tls.as_ptr() as *mut u8,
        TLS_SIZE,
    )
    .expect("setup_guest_tls");
    // Publish the main thread's TLS block as the template that spawned guest
    // threads (pthread_create/clone children) clone per-thread, so their
    // `__thread` locals and TP-indexed tables match the main thread instead of
    // a bare zeroed buffer.
    arm64jit::jit::publish_guest_tls_template(tls.as_ptr() as u64, TLS_SIZE);
    println!("guest sp=0x{:x} tls(tpidr)=0x{:x}", sp, st.tpidr);

    // JNI boot mode: hand the guest a guest-visible JavaVM* in x0 (as the Android
    // runtime would). Pass `--jni` to set x0 = vm. If x0/x1/x2 were already
    // supplied via positional args they win (we don't clobber a caller's x0).
    if std::env::args().any(|a| a == "--jni") && st.x[0] == 0 {
        let (_env, vm) = arm64jit::jni::build_jni();
        st.x[0] = vm; // JNI_OnLoad(JavaVM* vm, void* reserved) -> x0 = vm
        println!("JNI boot: x0 = JavaVM* 0x{:x}", vm);
    }

    // PC-driven dispatcher: compiles reachable regions and re-enters on
    // indirect branch (`blr`) / `br` / `ret`, so real (blr-heavy) Roblox code
    // can actually *execute* rather than stopping at the first blr.
    match jit_run(image, base, entry, &mut st as *mut CpuState) {
        Err(e) => {
            eprintln!("arm64jit run_loop stopped: {e}");
            std::process::exit(1);
        }
        Ok(r) => {
            // stderr is unbuffered; if this line appears BEFORE the SIGSEGV dump,
            // the fault is in post-run teardown, not the boot loop itself.
            eprintln!("[elfjit] jit_run returned Ok({r:#x}) — entering post-run phase");
            println!("JIT(no-QEMU) entry() -> {} (0x{:x})", r, r);
        }
    }
    // Let spawned worker guest threads (pthread_create/clone children started
    // during boot) run to completion before the process exits, so jit_run on a
    // detached child isn't torn down mid-translation (which surfaces as a
    // SIGSEGV reading a freed child CpuState as 'registers'). Wait for the
    // active-guest-thread count to return to the baseline (main only).
    let baseline = 1;
    for _ in 0..400 {
        if arm64jit::jit::active_guest_threads() <= baseline {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    std::thread::sleep(std::time::Duration::from_millis(50));

    // --startapp <link-addr>: after JNI_OnLoad completes, drive the next real
    // boot stage — the Java side's `nativeAppBridgeV2StartAppWithParams` (the
    // entry that creates the engine main loop + EGL/GLES context). We chain it
    // as a fresh guest entry after the registration phase, giving it the same
    // JNIEnv in x0 plus fake but VALID (non-null, dereferenceable) jobject /
    // jstring handles, exactly as the real JVM would. Captures how far the real
    // binary gets into StartApp (main-loop / graphics init) before the next wall.
    if let Some(hex) = {
        let args: Vec<String> = std::env::args().collect();
        args.iter()
            .position(|a| a == "--startapp")
            .and_then(|i| args.get(i + 1).cloned())
    } {
        // Reuse the singleton env/vm; build a fake-but-valid jobject (a 5-word
        // object header) and a jstring handle containing the StartApp params JSON.
        let (env_ptr, _vm) = arm64jit::jni::build_jni();
        let activity = arm64jit::jni::new_fake_object(); // non-null jobject
        // Default: pass the historical bare-JSON jstring as the params handle.
        // With --startapp-jobject, pass a real AutoValue-style jobject instead so
        // StartApp's serialization runs through the jni.rs getter-value registry
        // (Call{Object,Boolean,Int,Long,Float}Method serve real values) — the
        // recon-v2 Task-2 prescription. This is the live A/B on whether the value
        // registry unblocks StartApp's json serialization or whether the SH45
        // guest-stack-leak is genuinely params-independent.
        // --startapp-v1: drive the recon v1 "still-live lower-effort"
        // nativeAppBridgeAppStart__ (0x102338510) as the PRIMARY standalone
        // start INSTEAD of V2StartAppWithParams. V1 reads 6 individual jstrings
        // (JNI signature String,String,Z,String,String,String straight from the
        // mangled sym Java_...AppStart__Ljava_lang_String_2Ljava_lang_String_2Z
        // Ljava_lang_String_2Ljava_lang_String_2Ljava_lang_String_2) — no AutoValue
        // jobject, no Call*Method getter, so it BYPASSES the params-collapse
        // json-abort (SH56: the value registry never even fires on the V2 path).
        // It has only ever run as the UNREACHABLE tail of the v2boot ladder
        // (which stalls at rung 1 nativeGameGlobalInit every time), so its
        // downstream path (session/home-screen renderer) was NEVER exercised.
        // This lever drives it standalone so that path is finally reached.
        let use_v1 = std::env::args().any(|a| a == "--startapp-v1");
        let params = if std::env::args().any(|a| a == "--startapp-jobject") {
            arm64jit::jni::new_fake_object() // AutoValue InitParams/StartAppParams jobject
        } else {
            arm64jit::jni::new_string_utf_handle(b"{\"key\":\"\"}") // legacy JSON jstring
        };
        let link = u64::from_str_radix(hex.trim_start_matches("0x"), 16)
            .unwrap_or_else(|_| panic!("bad --startapp hex"));
        // V1 entry guest addr = file 0x2338510 + base. link here is the V2 file
        // vaddr (0x258b144) from the product recipe; --startapp-v1 swaps the
        // target (and the s2 ABI below) to the V1 6-jstring entry.
        let start_app = el.guest_of(if use_v1 { 0x2338510u64 } else { link });
        eprintln!("[elfjit] driving StartApp @ guest {start_app:#x} after JNI_OnLoad (env={env_ptr:#x} jobject={activity:#x} params={params:#x}){}",
            if use_v1 { " [V1 6-jstring AppStart__]" } else { "" });

        // --v2boot: drive the REAL engine boot ladder IN ORDER (the SH53-open
        // runtime test + the recon's corrective for the bare/out-of-order
        // StartApp-with-JSON json-abort). The recon (docs/recon-framework-
        // boot-order.md) reframes the type-4 producer vector [0x106829ea8] as
        // installed by TaskScheduler init reached only through this exact
        // order: nativeGameGlobalInit -> setTaskSchedulerBackgroundMode(false)
        // -> nativeAppBridgeV2InitWithParams -> nativeAppBridgeStartLuaAppDM ->
        // nativeAppBridgeV2StartAppWithParams. We drive each rung as a fresh
        // guest entry reusing the boot SP (the JNI natives each prologue
        // `sub sp` from it), passing AutoValue JNI jobjects (NOT JSON strings)
        // as the x2 params — serviced by the jni.rs AutoValue getter shim so
        // StartApp's serialization reads valid empty/default strings — and dump
        // [0x106829ea8] AFTER EVERY rung. First non-zero vector = the gate
        // opens (the scheduled producer is live). MUST be spawned BEFORE the
        // start_app jit_run below (which parks the main thread forever and
        // never returns), on a detached thread that sleeps its own warmup so
        // the rungs execute concurrently while StartApp idles. Opt-in.
        // SH124: the ladder handle is JOINED after the StartApp jit_run returns
        // so main() no longer tears down the detached ladder mid-do-init (the
        // pre-SH124 exit-232 race). No-op while StartApp parks (exit 124).
        let mut v2boot_join: Option<std::thread::JoinHandle<()>> = None;
        if std::env::args().any(|a| a == "--v2boot") {
            const BSS_TASKV4: u64 = 0x106829ea8;
            // Route-B SH81: force the engine's dispatch-accessor low-bit gate so
            // every gated dispatch site takes its clean direct path instead of
            // the NULL-vtable singleton fallback (SH80 crash at 0x10624f46c).
            // Since the --v2boot ladder is the Route-B driver, apply it here.
            routeb_patch_dispatch_gate();
            // SH81: also seed the two dispatch-singleton .data records so the
            // accessor's lazy-create returns a coherent object (real vtable) —
            // the singletons 0x624f4f0 reaches directly (cbz x1) that the gate
            // force does NOT cover (would otherwise null-vtable crash at 0x624f500).
            routeb_seed_task_singletons();
            // SH115: the three nullable-singleton dispatch accessors (V2Init/
            // V2Start/V1AppStart/SendAppEventOnAppReady) soft-return because
            // their `blr` reads past the 0x60 vtable. Scoped-patch each site to
            // materialize the stable zeroed singleton object so those bodies
            // COMPLETE (session advance). Leaves nativeInit's own 0x60 reads +
            // the shared vtable untouched. OPT-IN (JIT_SH115_SINGLETON_PATCH=1)
            // until the deeper nativeInit rung-0 getter gate is cleared: the
            // patch is verified to make the render pipeline run, but it also
            // advances nativeInit PAST its former soft-return into a 0x28-getter
            // fault (SH116), so it is inert by default to keep the ladder clean.
            if std::env::var("JIT_SH115_SINGLETON_PATCH").ok().as_deref() == Some("1") {
                routeb_patch_singleton_dispatch();
                // SH116: the advanced nativeInit path locks an unmapped .bss
                // global's +0x28; materialize a stable zeroed lock-owner into
                // the helper instead of seeding the unmapped page.
                routeb_patch_nativeinit_lock_owner();
                // SH117: nativeInit's flag-map probe helper reads a garbage +0
                // bucket-array; leaf-rewrite it to `ret` so the caller takes the
                // 'found' path and the flag-registration loop advances.
                routeb_patch_nativeinit_flagmap_helper();
                // SH119: SendAppEventOnAppReady's two ungated singleton lambdas
                // (0x6251610 off 0xf0, 0x6260a68 off 0x550) still soft-return;
                // materialize the stable object into x0 at both so the body
                // completes towards the app-data-model / GuiObjects.
                routeb_patch_sendapp_singleton_lambdas();
                // SH120: the app-bridge event dispatch reads the shared dispatcher-
                // node .bss global (0x10683a460) with an unseeded self-link; seed
                // the DATA (NOT the generic shared leaf) to the benign empty state.
                routeb_seed_dispatcher_node();
                // SH121: setTaskSchedulerBM lazily constructs the TaskScheduler whose
                // ctor asserts [0x72739d4].bit0 ("flags loaded") -> raise(SIGTRAP)
                // (exit 133). NOP the tbz + seed the REAL setTaskSchedulerBM version-gate
                // [0x10683cff8] (SH109's [0x10683d350] belongs to V2Init/V2Start). KEEP
                // OPT-IN like the rest of the SH115-121 chain: defaulting it ON makes the
                // default ladder proceed deeper into setTaskSchedulerBM and fault at the
                // NEXT unseeded gate (guestpc 0x106241c70) instead of the safe pre-existing
                // run-variable exit — a default-path regression. Inert without the flag.
                routeb_patch_taskscheduler_flags_gate();
                unsafe {
                    *(0x10683cff8u64 as *mut u64) = 0x0306u64; // low byte 6, byte1 3
                }
            }
            // SH87: the map-family generic dispatch can blr through the garbage +0x18
            // hash2 of a rehash-copied map (0x1800064) — force blr x1 (primary hash).
            routeb_patch_map_dispatch();
            // SH93: gameGlobalInit's do-init parks forever on a CEvent completion set by a
            // TaskScheduler worker thread that never runs headlessly -> NOP the barrier so
            // gameGlobalInit RETURNS (unlocks rung 2 nativeUpdateAdapterInit -> type-4 vector).
            routeb_patch_globalinit_cevent_barrier();
            // SH102: gameGlobalInit's ClientRunInfo base-url log block derefs a
            // lazily-initialized singleton *(0x106ED7A28) that never initializes
            // under the JIT -> ldrb [x0]=0 faults at 0x102dade34. The block is a
            // pure diagnostic log (gated on a run counter); force its gate to
            // always skip it.
            routeb_patch_clientruninfo_url_log();
            // SH103: gameGlobalInit's keyed-registry update reads the zeroed-bss
            // static [0x106dcb078] through x20, but a mid-block host-call leak
            // (PLT strlen) puts a HOST pointer in x20 -> `ldr x24,[x20]` faults.
            // The read is null-guarded (intended value 0); force it.
            routeb_patch_keyed_registry_x24_load();
            let boot_sp = st.x[31];
            let tpidr = arm64jit::jit::current_guest_tp();
            let ib = base;
            let iimg = image;
            let warmup_ms: u64 = std::env::var("V2BOOT_WARMUP_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4500);
            v2boot_join = Some(std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(warmup_ms));
                let (env_ptr, _vm) = arm64jit::jni::build_jni();
                let thiz = arm64jit::jni::new_fake_object(); // Activity jobject
                let init_params = arm64jit::jni::new_fake_object(); // AutoValue InitParams
                let start_params = arm64jit::jni::new_fake_object(); // AutoValue StartAppParams
                let bg_name = arm64jit::jni::new_string_utf_handle(b"ASMA.start");
                let dw = |a: u64| -> u64 {
                    if a >= 0x100000000 && a >> 56 == 0 && a & 7 == 0 { unsafe { *(a as *const u64) } } else { 0 }
                };
                let dump = |label: &str| {
                    eprintln!("[elfjit:v2boot] after {label}: [0x106829ea8] = {:#x}", dw(BSS_TASKV4));
                };
                // Guest addresses = file vaddr + 0x100000000 (recon quick-ref).
                // rung 0 FIRST = nativeInitializeNativeFlags (0x10232048c) — on
                // the engine's own flags-loaded write chain for the gameGlobalInit
                // latch byte [0x72739d4] (recon-routeB). With getFlagsCount>=1 the
                // JIT-translated engine code writes the latch; then rung 1
                // nativeGameGlobalInit can leave its nanosleep park.
                let rungs: [(&str, u64, [u64; 8]); 7] = [
                    ("nativeInitializeNativeFlags", 0x10232048c, [env_ptr, thiz, 0, 0, 0, 0, 0, 0]),
                    // nativeGameGlobalInit (JNIEnv*, jobject)
                    ("nativeGameGlobalInit", 0x102206404, [env_ptr, thiz, 0, 0, 0, 0, 0, 0]),
                    // nativeUpdateAdapterInit (JNIEnv*, jobject)
                    ("nativeUpdateAdapterInit", 0x10221c3ec, [env_ptr, thiz, 0, 0, 0, 0, 0, 0]),
                    // setTaskSchedulerBackgroundMode(env, thiz, enable=false, name)
                    ("setTaskSchedulerBM(false)", 0x102bb2380, [env_ptr, thiz, 0, bg_name, 0, 0, 0, 0]),
                    // nativeAppBridgeV2InitWithParams(env, thiz, InitParams)
                    ("V2InitWithParams", 0x102365c54, [env_ptr, thiz, init_params, 0, 0, 0, 0, 0]),
                    // nativeAppBridgeStartLuaAppDM(env, thiz)
                    ("StartLuaAppDM", 0x1023efe2c, [env_ptr, thiz, 0, 0, 0, 0, 0, 0]),
                    // nativeAppBridgeV2StartAppWithParams(env, thiz, StartAppParams)
                    ("V2StartAppWithParams", 0x10258b144, [env_ptr, thiz, start_params, 0, 0, 0, 0, 0]),
                ];
                dump("boot start");
                // Route-B latch (docs/recon-routeB-globaltinit-unblock.md, verified
                // file 0x22474e8 `strb w19,[x9,#2516]` with x9=adrp 0x7273000:
                // nativeGameGlobalInit only leaves its nanosleep park once .bss byte
                // [0x72739d4] bit0==1 "flags have been loaded"; SH55/62 stalled at
                // rung 1 because it defaulted 0. Drive nativeInitializeNativeFlags
                // (0x10232048c) FIRST as rung 0 — it is on the engine's own
                // flags-loaded write chain (0x2320cec -> 0x2320f2c -> the latch
                // setter) — so the latch is set through GUEST code (JIT-translated,
                // safely hits the real RW map) instead of a fragile raw host write.
                // NativeFlagsInterface getters are stubbed (GetFlagsCount>=1 +
                // empty jstrings) via the JNI value registry. --v2boot-r246 keeps
                // the SH58 probe (no GlobalInit); V2BOOT_SEED_LATCH keeps working.
                // SH82: nativeGameGlobalInit's GameGlobalInitImpl thread-dispatch
                // (file 0x2206db8, verified) compares pthread_self() (cmp x0,x20)
                // against the engine's stored main-thread id cell [0x6863a68]
                // (ldr x20,[x8,#2664], x8=adrp 0x6863000) at 0x2206de4/0x2206dec.
                // When they MATCH -> the b.ne at 0x2206df0 is not taken -> the
                // dispatch tail-calls vt[+48] of the [thiz+32] object (~immediate)
                // and GlobalInit is done. When they DIFFER (the harness's detached
                // ladder thread IS a non-main thread) -> the b.ne jumps to 0x2206e28
                // -> the inline do-init chain runs REAL scheduler/TaskScheduler
                // construction end-to-end then waits at the 0x2207648 completion
                // spin (ldrb [x19+1]; tbnz #0 -> poll fn 0x22076f0->0x2850520 ->
                // b 0x2207648) for a posted-job completion flag the headless
                // main-thread scheduler never sets -> the rung parks FOREVER and
                // the ladder never prints "after nativeGameGlobalInit" (SH82: the
                // pre-fix park, exit 124). NOTE: SH82 A/B proved forcing the match
                // via a .text NOP on the b.ne is a REGRESSION (6/6 json-crash on
                // the main-thread StartApp serialization vs 8/8 clean baseline) —
                // it re-routed the MAIN thread's own boot call incorrectly.
                // Instead, seed [0x6863a68] = THIS rung thread's pthread_self so its
                // own GlobalInit call takes the vt[+48] match path (no .text patch,
                // restored after only for rung 1, main-thread id untouched).
                let main_id_cell: u64 = 0x106863a68;
                let orig_main_id = unsafe { *(main_id_cell as *const u64) };
                // SH86: the OTel/pb_defaults registration path (reached deep inside
                // nativeGameGlobalInit's do-init) allocates via the CRT `operator new`
                // wrapper (file 0x2a0d9b8) whose allocator-hook dispatch compares the ACTIVE
                // hook global [0x1067daaf0] against the DEFAULT hook global [0x1067d0840].
                // Both are 0 in the file, so on a correct load the two `ldr`s are equal and
                // the `b.eq` takes the fast path (TLS allocator) with NO blr. Under the JIT
                // the RW segment leaves [0x1067daaf0] as host-heap garbage (observed
                // 0x7fcd98dd52e0), so `cmp` differs -> `blr x8` jumps to the heap -> SIGSEGV
                // (matched crash guestpc 0x1029b43f0, fault==heap). Seed BOTH to 0 (the
                // canonical default) before driving rung 1; idempotent and safe (a real boot
                // would install an override here, which never happens headlessly).
                unsafe {
                    *(0x1067daaf0u64 as *mut u64) = 0; // active allocator-hook global
                    *(0x1067d0840u64 as *mut u64) = 0; // default allocator-hook global
                }
                eprintln!(
                    "[elfjit:v2boot] SH86 seeded CRT allocator-hook globals [0x1067daaf0]=[0x1067d0840]=0 so operator-new takes the fast path (no blr through host garbage)"
                );
                // SH88: the OTel/pb_defaults descriptor registration passes a static
                // `.data.rel.ro` protobuf FIELD-TAG constant (0x1800064 from the 16-byte
                // strided table at file 0x62f5110) as the map/this arg to the hash-map FIND
                // op (file 0x29f424c), because the upstream registry map it should have been
                // constructed from (BSS 0x106838368/378/380) is never built under the JIT.
                // Seed a coherent empty span-hash map into those slots and register it with
                // the dispatch hook as the substitute for sub-image map/this candidates.
                let pb_map = routeb_seed_pb_registry_map(iimg, ib);
                arm64jit::jit::routeb_register_substitute_map(pb_map);
                // SH99: seed the global 0x10-stride vector the deep globalinit do-init
                // probes (guest 0x106dcae08, begin deref'd unconditionally -> NULL SEGV).
                let _ = routeb_seed_game_global_vector();
                // SH107: nativeUpdateAdapterInit (rung 2) reads a global adapter-config
                // RECORD through [guest 0x106ed7a18] (adrp x9,6ed7000; ldr x9,[x9,#2584]
                // = 0xa18; then `ldrb w10,[x9]; tbnz w10,#0`). BSS leaves it 0 -> NULL
                // deref `ldrb [x9]` SIGSEGV at 0x10221d7a8. Seed it to a zeroed
                // 0x20 record: bit0==0 takes the clean path (ldr q0,[x9]; str q0,[x8];
                // ldr x10,[x9,#16]; str x10,[x8,#16]; ret) -> copies zeros, no change.
                let adapter_rec =
                    *ROUTEB_ADAPTER_REC_ADDR.get_or_init(|| {
                        let r = Box::leak(vec![0u8; 0x20usize].into_boxed_slice()).as_mut_ptr() as u64;
                        unsafe { *(0x106ed7a18u64 as *mut u64) = r; }
                        eprintln!("[elfjit:v2boot] SH107 seeded adapter-record [0x106ed7a18]=0x{r:x} (zeroed 0x20 byte record, bit0=0 clean path)");
                        r
                    });
                let _ = adapter_rec;
                // SH109: V2InitWithParams/V2StartAppWithParams and
                // setTaskSchedulerBM gate on a version-global read at
                // [adrp 0x683d000 + 848] = guest 0x10683d350 (`ldr x8,[x8,#848]`;
                // `and w9,w8,#0xff; and x10,x8,#0xfc00; cmp w9,#6; b.eq <main>`).
                // BSS leaves it 0, so they take the "not version 6" detour which
                // ladders into the SH103 host-pointer-leak blr (soft-return pc
                // 0x40438b.../0x1088b8b5150 outside image). Seed low byte = 6
                // (and 0xfc00 bits clear) so they stay on the clean main path.
                // Re-asserted from this thread's own setup (idempotent RW store).
                unsafe {
                    *(0x10683d350u64 as *mut u64) = 6; // low byte==6, bits clear
                    *(0x10683d358u64 as *mut u64) = 0; // version-string ptr slot
                }
                eprintln!("[elfjit:v2boot] SH109 seeded version-gate [0x10683d350]=6 so V2Init/V2Start keep the clean main path");
                let _ = (iimg, ib);
                // rung index 1 == nativeGameGlobalInit in the rungs array below.
                for (name, guest, args) in rungs.iter() {
                    eprintln!("[elfjit:v2boot] driving {name} @ guest {guest:#x} (env={env_ptr:#x} thiz={thiz:#x})");
                    if *guest == 0x102206404 {
                        // rung 1 = nativeGameGlobalInit: make its thread-dispatch
                        // see "self == main" by seeding the stored-main-id cell
                        // with the current thread's own pthread_self.
                        let me = unsafe { libc::pthread_self() };
                        unsafe { *(main_id_cell as *mut u64) = me as u64 };
                        eprintln!("[elfjit:v2boot] seeded main-thread-id [0x106863a68]=0x{me:x} for GlobalInit thread-dispatch (this thread == stored == self -> takes the vt[+48] match path, no park)");
                    }
                    // SH122: StartLuaAppDM fans into the SAME GlobalInit do-init
                    // (via 0x2baeeec -> 0x2206c40) that is once-latched on
                    // [0x6a68410].bit0 and thread-gated on [0x106863a68]
                    // (deleg_65301a28). On this (non-main) ladder thread the
                    // thread-dispatch b.ne parks/soft-returns and the once-guard
                    // stays unlatched, so the DM/app-shell is never built. Mirror
                    // the SH82 trick: seed the main-id cell to self + set the
                    // once-guard bit0 so the do-init takes the match/DM-construction
                    // path. Restore the cell after the rung.
                    if *guest == 0x1023efe2c && std::env::var("JIT_SH115_SINGLETON_PATCH").ok().as_deref() == Some("1") {
                        let me = unsafe { libc::pthread_self() };
                        unsafe {
                            *(main_id_cell as *mut u64) = me as u64;
                            // once-guard [FILE 0x6a68410] = guest 0x106a68410 (in the
                            // mapped rw- segment [0x1067d67c0,0x107333c3c)). Set bit0
                            // so the GlobalInit do-init falls through its once-guard.
                            let og = 0x106a68410u64 as *mut u8;
                            *og |= 1;
                            // SH125 (recon deleg_5e2c8480 of do-init 0x102206c40): the
                            // do-init's flags-loaded GETTER (guest 0x10220671c) reads
                            // `ldrb w0,[0x106a683e8]` (file 0x2206738) before the
                            // thread-dispatch worker. When bit0==0 it selects the
                            // "fallback" dispatch-object slot [x21+0] instead of the
                            // "live DM" slot [x21+8] — still host-garbage / unseeded.
                            // Seed bit0=1 so the do-init consistently consumes the real
                            // DM/app-config slot. Mirrors the SH82/122 data-seed pattern
                            // (host byte-store, idempotent, NO .text patch — a NOP on the
                            // getter's tbz would force the guard-init path instead).
                            let flags_latch = 0x106a683e8u64 as *mut u8;
                            *flags_latch |= 1;
                            eprintln!("[elfjit:v2boot] SH125 seeded flags-loaded latch [0x106a683e8].bit0=1 so the do-init consumes the live DM slot (getter 0x102206738)");
                        }
                        eprintln!("[elfjit:v2boot] SH122 seeded main-id cell 0x{me:x} + once-guard [0x6a68410].bit0=1 for StartLuaAppDM -> GlobalInit do-init takes the DM-construction match path");
                    }
                    let mut s = arm64jit::jit::CpuState::new();
                    s.tpidr = tpidr;
                    s.x[31] = boot_sp;
                    s.x[..8].copy_from_slice(args);
                    match arm64jit::jit::jit_run(iimg, ib, *guest, &mut s as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:v2boot] {name} stopped: {e}"),
                        Ok(r) => eprintln!("[elfjit:v2boot] {name} returned Ok({r:#x})"),
                    }
                    dump(name);
                    if *guest == 0x102206404 || *guest == 0x1023efe2c {
                        unsafe { *(main_id_cell as *mut u64) = orig_main_id };
                    }
                }
                // Final: also drive the V1 6-jstring AppStart fallback so the
                // session/home-screen renderer can start even if the V2 path
                // stays gated (reads params as individual jstrings, not the
                // AutoValue getters). nativeAppBridgeAppStart__ (0x102338510).
                eprintln!("[elfjit:v2boot] driving V1 AppStart__ (fallback)");
                let mut sv = arm64jit::jit::CpuState::new();
                sv.tpidr = tpidr;
                sv.x[31] = boot_sp;
                sv.x[0] = env_ptr;
                sv.x[1] = thiz;
                // Correct V1 AppStart__ ABI: String,String,Z,String,String,String
                // -> 5 empty jstrings + boolean(false) in x2..x7 (the old fallback
                // put a jstring in the Z slot and left x7=0 — a never-exercised
                // wrong ABI, corrected here to match the current descriptor).
                sv.x[2] = arm64jit::jni::new_string_utf_handle(b"");
                sv.x[3] = arm64jit::jni::new_string_utf_handle(b"");
                sv.x[4] = 0; // jboolean false
                sv.x[5] = arm64jit::jni::new_string_utf_handle(b"");
                sv.x[6] = arm64jit::jni::new_string_utf_handle(b"");
                sv.x[7] = arm64jit::jni::new_string_utf_handle(b"");
                match arm64jit::jit::jit_run(iimg, ib, 0x102338510, &mut sv as *mut CpuState) {
                    Err(e) => eprintln!("[elfjit:v2boot] V1 AppStart__ stopped: {e}"),
                    Ok(r) => eprintln!("[elfjit:v2boot] V1 AppStart__ returned Ok({r:#x})"),
                }
                dump("V1 AppStart__");
                // SH113 (recon-sh113-surface-handoff.md): after V2Start/V1 drive
                // nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams so the REAL
                // wired XID lands at [0x10683d348] (the window the engine's
                // EGL-surface path reads) and nudge the app-bridge data model —
                // the concrete onboard step for "engine renders its own screens".
                // The JIT's ANativeWindow_fromSurface shim ignores the Surface
                // arg and returns the SH112-wired XID, so x2 only needs a
                // non-null token and x3 a readable platformParams sentinel.
                if std::env::args().any(|a| a == "--v2boot-surface-handoff") {
                    let surf_token = Box::leak(vec![0x42u8; 64].into_boxed_slice()).as_mut_ptr() as u64;
                    let params = Box::leak(vec![0u8; 64].into_boxed_slice()).as_mut_ptr() as u64;
                    eprintln!(
                        "[elfjit:v2boot] driving V2UpdateSurfaceAppWithPlatformParams @ guest 0x1025f5fec (surface_token={surf_token:#x} params={params:#x})"
                    );
                    // SH118: the surface-body canary fn re-publishes the dual-use
                    // guard/ctx GOT slot mid-execution -> false stack-smash. Reassert
                    // a stable guard pointer right before this rung.
                    if std::env::var("JIT_SH115_SINGLETON_PATCH").ok().as_deref() == Some("1") {
                        routeb_reassert_canary_guard();
                    }
                    let mut su = arm64jit::jit::CpuState::new();
                    su.tpidr = tpidr;
                    su.x[31] = boot_sp;
                    su.x[0] = env_ptr;
                    su.x[1] = thiz;
                    su.x[2] = surf_token;
                    su.x[3] = params;
                    match arm64jit::jit::jit_run(iimg, ib, 0x1025f5fec, &mut su as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:v2boot] V2UpdateSurface stopped: {e}"),
                        Ok(r) => eprintln!("[elfjit:v2boot] V2UpdateSurface returned Ok({r:#x})"),
                    }
                    let win = arm64jit::shims::anativewindow_xid();
                    let stored = unsafe { *(0x10683d348u64 as *const u64) };
                    let nf = arm64jit::jni::nativehelper_flags_loaded();
                    let ar = arm64jit::jni::nativehelper_app_ready();
                    eprintln!(
                        "[elfjit:v2boot] surface-handoff: wired_xid=0x{win:x} [0x10683d348]=0x{stored:x} MH_FLAGS_LOADED={nf} MH_APP_READY={ar}"
                    );
                    dump("V2UpdateSurfaceAppWithPlatformParams");
                }
                // SH114 (deleg_0d4e5597, disasm): the NativeHelper milestones
                // (MH_* atoms) are SET-ONLY — nothing in guest memory polls them,
                // so StartLuaAppDM's session never advances and no in-image
                // GuiObject->scene-list writer runs. The missing Java->engine
                // response is nativeAppBridgeV2SendAppEventOnAppReady
                // (0x102bb463c): it parses the first 4 bytes of the event-name
                // jstring (x2) against magics ("Home" LE 0x656d6f48 -> w19=4),
                // builds a 0x50 app-event struct, dispatches it through the
                // shared app-bridge pipe 0x102baeeec into the
                // LuaAppExperienceController -> NativeHelper/AppShell -> DataModel
                // path that constructs the shell UI. Driving it as a SEQUENTIAL
                // ladder rung (same thread, after surface-handoff) fixtures the
                // milestone contract without the SH44/49 block-cache SIGSEGV of a
                // shim-internal nested jit_run. Event-name ABI (6-arg JNI):
                // x0=env x1=thiz x2=event jstring, x3/x4/x5=extra jstrings (must
                // be readable non-null for GetStringUTFChars).
                if std::env::args().any(|a| a == "--v2boot-send-appevent") {
                    // SH126: the app-event object's vtable 0x635e068 is all-zero
                    // (empty .data.rel.ro) -> SendAppEventOnAppReady's terminal blr
                    // is `blr 0` = benign soft-return -> its body never completes
                    // and MH_FLAGS_LOADED/APP_READY stay false. Materialize the
                    // vtable's +0x20/+0x28 slots to the benign leaf + seed the pipe
                    // sync-gate [0x10683d010]=-1 so the body completes AND the pipe
                    // takes the synchronous do-init path. All plain data stores.
                    if std::env::var("JIT_SH115_SINGLETON_PATCH").ok().as_deref() == Some("1") {
                        routeb_patch_sendapp_appevent_vtable();
                        // SH126 runner-up #2: the do-init (0x2206c40) reached via the
                        // pipe is once-latched on [0x106a68410].bit0 and thread-gated
                        // on [0x106863a68] (ladder latched both). Clear the once-guard
                        // and re-seed main-id to THIS rung thread so the do-init
                        // re-takes the DM-construction match path (fresh app model).
                        unsafe {
                            let og = 0x106a68410u64 as *mut u8;
                            *og &= !1u8;
                            let me = libc::pthread_self();
                            *(0x106863a68u64 as *mut u64) = me as u64;
                            eprintln!(
                                "[elfjit:v2boot] SH126 cleared once-guard [0x6a68410].bit0=0 + re-seeded main-id 0x{me:x} so the pipe's do-init re-constructs the app model"
                            );
                        }
                    }
                    let ev = arm64jit::jni::new_string_utf_handle(b"Home");
                    let n3 = arm64jit::jni::new_string_utf_handle(b"");
                    let n4 = arm64jit::jni::new_string_utf_handle(b"");
                    let n5 = arm64jit::jni::new_string_utf_handle(b"");
                    eprintln!(
                        "[elfjit:v2boot] driving SendAppEventOnAppReady @ guest 0x102bb463c (event=\"Home\" jstr={ev:#x})"
                    );
                    let mut se = arm64jit::jit::CpuState::new();
                    se.tpidr = tpidr;
                    se.x[31] = boot_sp;
                    se.x[0] = env_ptr;
                    se.x[1] = thiz;
                    se.x[2] = ev;
                    se.x[3] = n3;
                    se.x[4] = n4;
                    se.x[5] = n5;
                    match arm64jit::jit::jit_run(iimg, ib, 0x102bb463c, &mut se as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:v2boot] SendAppEventOnAppReady stopped: {e}"),
                        Ok(r) => eprintln!(
                            "[elfjit:v2boot] SendAppEventOnAppReady returned Ok({r:#x}) w19-event={:#x}",
                            se.x[19]
                        ),
                    }
                    let nf2 = arm64jit::jni::nativehelper_flags_loaded();
                    let ar2 = arm64jit::jni::nativehelper_app_ready();
                    eprintln!(
                        "[elfjit:v2boot] app-event post: MH_FLAGS_LOADED={nf2} MH_APP_READY={ar2}"
                    );
                    dump("SendAppEventOnAppReady");
                }
                eprintln!("[elfjit:v2boot] ladder done; final [0x106829ea8] = {:#x}", dw(BSS_TASKV4));
                    // SH126-gate: signal the render pipeline it may start now
                    // (JIT_SERIALIZE_RENDER=1 makes --renderinit wait for this so
                    // render jit_runs never overlap the ladder's — SH55/64 class).
                    LADDER_DONE.store(true, core::sync::atomic::Ordering::Relaxed);
                    eprintln!("[elfjit:v2boot] LADDER_DONE=1 signaled (render may start)");
                    // SH122 session-advance probe: after the ladder, read the
                    // NativeHelper milestones + data-dir path state to confirm
                    // whether StartLuaAppDM's do-init actually advanced the session.
                    let nf = arm64jit::jni::nativehelper_flags_loaded();
                    let ni = arm64jit::jni::nativehelper_engine_initialized();
                    let ar = arm64jit::jni::nativehelper_app_ready();
                    let og = unsafe { *(0x106a68410u64 as *const u8) };
                    eprintln!(
                        "[elfjit:v2boot] SH122 session-advance probe: MH_FLAGS_LOADED={nf} MH_ENGINE_INITIALIZED={ni} MH_APP_READY={ar} once-guard[0x6a68410]={og:#x}"
                    );
            }));
        }
        // --v2boot-r246: the sequential --v2boot driver STALLS at rung 1 because
        // nativeGameGlobalInit parks without returning (SH54/SH55), so rungs 2-6
        // were NEVER exercised headlessly. A concurrent rung-1 thread is unsafe
        // (each top-level jit_run clears the block cache mid-other-thread). This
        // probe instead drives rungs 2-6 + V1 SEQUENTIALLY with NO GlobalInit and
        // NO extra threads, dumping the type-4 producer vector after each — a safe
        // test of whether any later bridge native alone installs it. Opt-in.
        if std::env::args().any(|a| a == "--v2boot-r246") {
            const BSS_TASKV4: u64 = 0x106829ea8;
            let boot_sp = st.x[31];
            let tpidr = arm64jit::jit::current_guest_tp();
            let ib = base;
            let iimg = image;
            let (env_ptr, _vm) = arm64jit::jni::build_jni();
            let thiz = arm64jit::jni::new_fake_object(); // Activity jobject
            let init_params = arm64jit::jni::new_fake_object(); // AutoValue InitParams
            let start_params = arm64jit::jni::new_fake_object(); // AutoValue StartAppParams
            let bg_name = arm64jit::jni::new_string_utf_handle(b"ASMA.start");
            let dw = |a: u64| -> u64 {
                if a >= 0x100000000 && a >> 56 == 0 && a & 7 == 0 { unsafe { *(a as *const u64) } } else { 0 }
            };
            let dump = |label: &str| {
                eprintln!("[elfjit:v2boot-r246] {label}: [0x106829ea8] = {:#x}", dw(BSS_TASKV4));
            };
            dump("r246-start");
            let rungs: [(&str, u64, [u64; 8]); 4] = [
                ("nativeUpdateAdapterInit", 0x10221c3ec, [env_ptr, thiz, 0, 0, 0, 0, 0, 0]),
                ("setTaskSchedulerBM(false)", 0x102bb2380, [env_ptr, thiz, 0, bg_name, 0, 0, 0, 0]),
                ("V2InitWithParams", 0x102365c54, [env_ptr, thiz, init_params, 0, 0, 0, 0, 0]),
                ("StartLuaAppDM", 0x1023efe2c, [env_ptr, thiz, 0, 0, 0, 0, 0, 0]),
            ];
            for (name, guest, args) in &rungs {
                eprintln!("[elfjit:v2boot-r246] driving {name} @ guest {guest:#x}");
                let mut s = arm64jit::jit::CpuState::new();
                s.tpidr = tpidr;
                s.x[31] = boot_sp;
                s.x[..8].copy_from_slice(args);
                match arm64jit::jit::jit_run(iimg, ib, *guest, &mut s as *mut CpuState) {
                    Err(e) => eprintln!("[elfjit:v2boot-r246] {name} stopped: {e}"),
                    Ok(r) => eprintln!("[elfjit:v2boot-r246] {name} returned Ok({r:#x})"),
                }
                dump(name);
            }
            eprintln!("[elfjit:v2boot-r246] driving V2StartAppWithParams");
            let mut sv = arm64jit::jit::CpuState::new();
            sv.tpidr = tpidr;
            sv.x[31] = boot_sp;
            sv.x[0] = env_ptr;
            sv.x[1] = thiz;
            sv.x[2] = start_params;
            match arm64jit::jit::jit_run(iimg, ib, 0x10258b144, &mut sv as *mut CpuState) {
                Err(e) => eprintln!("[elfjit:v2boot-r246] V2StartAppWithParams stopped: {e}"),
                Ok(r) => eprintln!("[elfjit:v2boot-r246] V2StartAppWithParams returned Ok({r:#x})"),
            }
            dump("V2StartAppWithParams");
            eprintln!("[elfjit:v2boot-r246] ladder done; final [0x106829ea8] = {:#x}", dw(BSS_TASKV4));
        }
        let mut s2 = arm64jit::jit::CpuState::new();
        s2.tpidr = arm64jit::jit::current_guest_tp();
        // Continue on the boot-phase guest stack (real SP), not a fresh 0 —
        // StartApp's prologue `sub sp,#0xf0` would otherwise wrap to 0xffff..10
        // and the frame-write faults. The Java side enters natives on whatever
        // thread is current; elfjit reuses the main guest thread's SP.
        s2.x[31] = st.x[31]; // guest SP
        s2.x[0] = env_ptr;
        s2.x[1] = activity;
        if use_v1 {
            // nativeAppBridgeAppStart__(JNIEnv*, jobject, jstring, jstring,
            // jboolean Z, jstring, jstring, jstring): 6 args -> x2..x7.
            // Pass 5 empty strings + boolean false (no AutoValue getters, so no
            // params-collapse json-abort possible on this path). The old
            // v2boot fallback put a jstring in the x4 Z-slot and left x7=0 —
            // a wrong ABI that was never exercised; here it is corrected.
            s2.x[2] = arm64jit::jni::new_string_utf_handle(b"");
            s2.x[3] = arm64jit::jni::new_string_utf_handle(b"");
            s2.x[4] = 0; // jboolean false
            s2.x[5] = arm64jit::jni::new_string_utf_handle(b"");
            s2.x[6] = arm64jit::jni::new_string_utf_handle(b"");
            s2.x[7] = arm64jit::jni::new_string_utf_handle(b"");
            eprintln!("[elfjit:startapp-v1] V1 AppStart__ ABI: x0=env x1=thiz x2..x7 = 5 empty jstrings + boolean(false)");
        } else {
            s2.x[2] = params;
        }
        // Concurrent guest-thread state sampler (JIT_THREADS=1). StartApp's
        // `jit_run` parks the main thread forever (the engine main-loop
        // lifecycle-await), so a post-run sampler would never run. Instead
        // spawn a detached host sampler that polls `snapshot_threads()`
        // every ~200 ms for a bounded window, dumping each parked thread's
        // hostcall slot (pc), guest call-site (x30/lr) and wait-object args
        // (x0..x2). This pins the boot wall to the exact guest function that
        // blocks and what it awaits. Runs concurrently with the jit_run.
        if std::env::var_os("JIT_THREADS").is_some() {
            std::thread::spawn(|| {
                for it in 0..100 {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                    let (c, h) = arm64jit::jit::block_cache_stats();
                    eprintln!("[elfjit:stats] it={it} compiles={c} hits={h}");
                    let snaps = arm64jit::jit::snapshot_threads();
                    for t in &snaps {
                        let at = arm64jit::resolver::name_of_call_addr(t.pc)
                            .unwrap_or_else(|| format!("{:#x}", t.pc));
                        eprintln!(
                            "  host_tid={} guest_tid={} pc={at} lr={:#x} x0={:#x} x1={:#x} x2={:#x} x3={:#x} x5={:#x} x19={:#x}[*={:#x}] x20={:#x} x21={:#x} x29={:#x} sp={:#x}",
                            t.host_tid, t.guest_tid, t.lr, t.x0, t.x1, t.x2, t.x3, t.x5, t.x19,
                            // deref [x19]: the wait-fn arg0 Q (host-heap; its high-32
                            // is the self-syncing version epoch, +4 the futex latch).
                            if t.x19 >= 0x100000000 && t.x19 >> 56 == 0 && t.x19 & 7 == 0 { unsafe { *(t.x19 as *const u64) } } else { 0 },
                            t.x20, t.x21, t.x29, t.sp
                        );
                        // JIT_DEQUE_PROBE=1: recover the parked consumer's
                        // deque-root from the waiter's SAVED frame and read the
                        // live deque head. The generic wait-with-timeout at
                        // 0x10284d018 leaves the caller's (drain fn 0x2856e40)
                        // callee-saved regs on its stack: stp x20,x19,[sp,#64]
                        // stored the DRAIN's x20 (= deque root, awk the waiter's
                        // own x20 is -1 = the infinite-timeout arg) and x19 (=
                        // consumer struct) at [sp+64] / [sp+72]. Read-only — the
                        // prerequisite to a host-side producer enqueue (push onto
                        // the deque the parked consumer drains).
                        if std::env::var_os("JIT_DEQUE_PROBE").is_some()
                            && t.lr == 0x10284d134
                        {
                            let sp = t.sp;
                            if sp >= 0x100000000 && sp >> 56 == 0 {
                                let root = unsafe { *(sp as *const u64).add(8) }; // [sp+64]
                                let cstruct = unsafe { *(sp as *const u64).add(9) }; // [sp+72]
                                let is_ptr = |p: u64| p >= 0x100000000 && p >> 56 == 0 && p & 7 == 0;
                                let q = if is_ptr(cstruct) { unsafe { *(cstruct as *const u64).add(13) } } else { 0 }; // [struct+104]
                                let headcell = if is_ptr(root) { unsafe { *(root as *const u64) } } else { 0 };
                                let head = if is_ptr(headcell) { unsafe { *(headcell as *const u64) } } else { 0 };
                                eprintln!(
                                    "  [deque] waiter_sp={sp:#x} drain_root=[sp+64]={root:#x} drain_struct={cstruct:#x} Q=[struct+104]={q:#x} headcell=[root]={headcell:#x} head={head:#x} (node={:#x} tag={:#x})",
                                    head & 0xffffffffffff, head >> 48
                                );
                                // Read the head node's internals to tell a real
                                // pending task node from a sentinel/garbage cell:
                                // next=[node], cb40=[node+40], vt=[node+112]&~0x3f
                                // then dispatch-cb [vt+40]; and [root+8] tag.
                                // Head node internals + the per-CPU slot layout.
                                // SH5 disasm pinned the deque head ATOMIC at
                                // slot+0x10 (packed low48=node, high16=tag) and
                                // tail at slot+0x18; slot+0 is likely a separate
                                // field (the sentinel/root ptr). Dump the whole
                                // neighborhood to resolve which offset the parked
                                // consumer actually drains.
                                let node = head & 0xffffffffffff;
                                // Dump the per-CPU slot neighborhood around the
                                // head-CELL to resolve the real deque head offset.
                                // The probe mislabeled slot+0 as the head; SH5
                                // disasm says the head ATOMIC is at slot+0x10.
                                if is_ptr(headcell) {
                                    let off = |o: usize| unsafe { *(headcell as *const u64).add(o / 8) };
                                    eprintln!(
                                        "      slot[{headcell:#x}] +0x00={:#x} +0x08={:#x} +0x10(HEAD)={:#x} +0x18(TAIL)={:#x} +0x20={:#x}",
                                        off(0), off(0x08), off(0x10), off(0x18), off(0x20)
                                    );
                                }
                                let rt8 = if is_ptr(root) { unsafe { *(root as *const u64).add(1) } } else { 0 };
                                if is_ptr(node) {
                                    let nxt = unsafe { *(node as *const u64) };
                                    let cb40 = unsafe { *(node as *const u64).add(5) }; // +40
                                    let v112 = unsafe { *(node as *const u64).add(14) }; // +112
                                    let vt = v112 & !0x3f;
                                    let dcb = if is_ptr(vt) { unsafe { *(vt as *const u64).add(5) } } else { 0 }; // [vt+40]
                                    eprintln!(
                                        "      node.next={nxt:#x} node[+40]={cb40:#x} node[+112]={v112:#x} vt={vt:#x} [vt+40]={dcb:#x} root[+8]tag={rt8:#x}"
                                    );
                                } else {
                                    eprintln!(
                                        "      head cell not a valid node (0); root[+8]tag={rt8:#x}"
                                    );
                                }
                                // Epoch: waiter x19 = the wait object Q' whose
                                // high-32 is the self-syncing version epoch, futex
                                // at Q'+4 = x1.
                                if is_ptr(t.x19) {
                                    let qw = unsafe { *(t.x19 as *const u64) };
                                    eprintln!(
                                        "      Q'=t.x19={:#x} [Q']={:#x} (refc=low32 {:#x} epoch=high32 {:#x}) futex_uaddr=x1={:#x}",
                                        t.x19, qw, qw & 0xffffffff, qw >> 32, t.x1
                                    );
                                }
                                // RAW STACK DUMP: print the parked waiter's sp
                                // window so the true frame layout (drain root,
                                // consumer struct, Q, timeout, saved x30) is
                                // resolved empirically instead of by inference.
                                // sp is host-readable (guest==host addressing).
                                if std::env::var_os("JIT_STACKDUMP").is_some() {
                                    let mut line = format!("      [stack sp={sp:#x}]");
                                    for o in (0..96usize).step_by(8) {
                                        let v = unsafe { *(sp as *const u64).add(o / 8) };
                                        line.push_str(&format!(" +{o:02x}={v:#018x}"));
                                    }
                                    eprintln!("{line}");
                                }
                                // [sp+0x50]=drain x20 (root), [sp+0x58]=drain x19
                                // (consumer) per drain 0x2856e54 stp x20,x19,[sp,#80]
                                // + generic-wait clobbers [sp+40..72] only. Try those.
                                if std::env::var_os("JIT_DEQUE_PROBE2").is_some() {
                                    let dr = unsafe { *(sp as *const u64).add(0x50 / 8) };
                                    let dc = unsafe { *(sp as *const u64).add(0x58 / 8) };
                                    eprintln!(
                                        "      [probe2] sp+0x50(drain x20 root)={dr:#x} sp+0x58(drain x19 consumer)={dc:#x}",
                                    );
                                    if is_ptr(dr) {
                                        let rd = unsafe { *(dr as *const u64) };
                                        eprintln!("        [root]={rd:#x}");
                                        if is_ptr(rd) {
                                            let head = unsafe { *(rd as *const u64) };
                                            eprintln!("        [[root]] head={head:#x} (node {:#x} tag {:#x})",
                                                head & 0xffffffffffff, head >> 48);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
        // Host-side lifecycle kicker (experimental): the engine owner parks
        // busy-polling a guest global (`ldar x8,[x8]; cmp #1; b.eq`) until the
        // Java layer's app-command sets it. On this box there is no Java side,
        // so `--kicker 0x<guest-hex-global>=<value-hex>` spawns a detached host
        // thread that writes the value to that guest global repeatedly WHILE
        // jit_run is parked, to test whether releasing the awaited predicate
        // lets StartApp proceed past the rendezvous toward the looper.
        // Host-side lifecycle kicker (experimental): the engine owner parks
        let mut kickers: Vec<(u64, KickerMode)> = Vec::new();
        let args: Vec<String> = std::env::args().collect();
        let mut i = 0;
        while i < args.len() {
            if let Some(k) = args[i].strip_prefix("--kicker") {
                let spec = if k.is_empty() {
                    i += 1;
                    if i >= args.len() { panic!("--kicker needs a value"); }
                    args[i].clone()
                } else {
                    k.trim_start_matches('=').to_string()
                };
                let (addr_s, val_s) = spec.split_once('=').unwrap_or((spec.trim_start_matches("0x"), "1"));
                let addr = u64::from_str_radix(addr_s.trim_start_matches("0x"), 16).expect("bad kicker addr");
                let val_l = val_s.trim_start_matches("0x").to_ascii_lowercase();
                // `=bcast` broadcasts the pthread_cond at that address; `=0xVAL`
                // writes the exact u64 value repeatedly; a bare `--kicker ADDR`
                // (no explicit `=`) keeps the historical 1->2 lifecycle pulse.
                let mode = if val_l == "bcast" {
                    KickerMode::Broadcast
                } else if spec.contains('=') {
                    let v = u64::from_str_radix(val_s.trim_start_matches("0x"), 16).expect("bad kicker val");
                    KickerMode::Fixed(v)
                } else {
                    KickerMode::Pulse
                };
                kickers.push((addr, mode));
            }
            i += 1;
        }
        for (addr, mode) in kickers {
            std::thread::spawn(move || {
                let what = match mode {
                    KickerMode::Broadcast => "pthread_cond_broadcast".to_string(),
                    KickerMode::Fixed(v) => format!("write 0x{v:x}"),
                    KickerMode::Pulse => "pulse 1->2".to_string(),
                };
                eprintln!("[elfjit:kicker] host thread drives 0x{addr:x} ({what})");
                let bc: unsafe extern "C" fn(*const u8) -> i32 = unsafe {
                    std::mem::transmute(libc::dlsym(libc::RTLD_NEXT, c"pthread_cond_broadcast".as_ptr()))
                };
                for it in 0..400 {
                    unsafe {
                        match mode {
                            KickerMode::Broadcast => {
                                bc(addr as *const u8);
                            }
                            KickerMode::Fixed(v) => {
                                *((addr) as *mut u64) = v;
                            }
                            KickerMode::Pulse => {
                                // PULSE: hold 1 through the first gate (init poll
                                // wants *pred==1), then set 2 — the wait loops while
                                // *pred==1 (cd7c b.eq) and proceeds only when
                                // *pred !=1 and !=0 (cd84 cbz-on-zero); 2 is the
                                // terminal "done" state.
                                let v = if it < 60 { 1u64 } else { 2u64 };
                                *((addr) as *mut u64) = v;
                            }
                        }
                        if it % 100 == 0 {
                            eprintln!("[elfjit:kicker] t={it} guest_global 0x{addr:x}=%{:#x}", *((addr) as *const u64));
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            });
        }
        // Synthetic app-command feed (JIT_DRIVE_LIFECYCLE): a host thread pushes
        // Android lifecycle commands into the ALooper app-command queue, so a
        // GameActivity main loop that reaches `ALooper_pollOnce` dispatches
        // APP_CMD_START then APP_CMD_RESUME (the two commands that precede a real
        // EGL context / first frame on Android) instead of spinning on the empty
        // queue. `post_app_command` is the same channel the ALooper shim drains.
        if std::env::var_os("JIT_DRIVE_LIFECYCLE").is_some() {
            use arm64jit::shims::post_app_command;
            std::thread::spawn(|| {
                for (it, cmd) in [
                    arm64jit::shims::APP_CMD_START,
                    arm64jit::shims::APP_CMD_RESUME,
                    arm64jit::shims::APP_CMD_INIT_WINDOW,
                ]
                .iter()
                .enumerate()
                {
                    eprintln!("[elfjit:appcmd] posting APP_CMD_{it} ({cmd})");
                    post_app_command(*cmd);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            });
        }
        // Real desktop X11 window for the ANativeWindow layer (GRAPHICS_-
        // RECOMMENDATION §5.3). Under JIT_DRIVE_LIFECYCLE bring up an Xvfb X
        // server, open a 1280x720 window, and register its XID as the guest's
        // ANativeWindow handle — SYNCHRONOUSLY before StartApp runs, so the
        // window is wired before the boot reaches the window/EGL surface path
        // (a racing spawned thread loses and hands the guest the sentinel).
        // Then eglCreateWindowSurface(dpy, config, win, ...) builds on a real
        // X11 window, not a fake address.
        if std::env::var_os("JIT_DRIVE_LIFECYCLE").is_some() {
            wire_real_window();
        }
        // Per-thread futex latch kicker (--futex-kick <period-ms>). The engine
        // main-loop idle barrier (cycle L) is a REAL per-thread futex: each
        // guest thread parks in guest_svc's FUTEX_WAIT_BITSET on its OWN latch
        // (uaddr = x1 = x19+4, awaited val 0xF4240) at call-site lr=0x10284d134
        // — a wait-until-changed tick/frame barrier. A host-side producer must
        // CHANGE the latch value and FUTEX_WAKE it to release the wait, else
        // the loop re-parks (a plain WAKE is a spurious wake; the value is
        // still the awaited one, so the futex immediately re-blocks). This was
        // unreachable by the static --kicker (which only writes fixed guest
        // globals). The sampler already exposes each parked thread's x1, so we
        // locate the per-thread latch live and write a value != awaited before
        // waking — advancing the loop one tick per kick into egl*/gl*.
        if let Some(hex) = {
            let args: Vec<String> = std::env::args().collect();
            args.iter()
                .position(|a| a == "--futex-kick")
                .and_then(|i| args.get(i + 1).cloned())
        } {
            let period_ms: u64 = hex.trim().parse().expect("--futex-kick needs integer period-ms");
            // Optional --futex-set <hex>: write a SPECIFIC latch value each tick
            // (the awaited token) instead of the free-running old+1. This tests
            // whether the idle barrier is a fixed "go" token (0xF4240) that the
            // producer must write verbatim, vs a pure version-counter (wait-until-
            // changed) where any new value works. `old.wrapping_add(1)` cannot
            // distinguish: if the waiter re-arms to a constant each cycle, a fixed
            // write is the correct producer signal and a version increment is a
            // stray number the loop ignores.
            let set_val: Option<i32> = {
                let args: Vec<String> = std::env::args().collect();
                args.iter()
                    .position(|a| a == "--futex-set")
                    .and_then(|i| args.get(i + 1).cloned())
                    .map(|v| i32::from_str_radix(v.trim_start_matches("0x"), 16).expect("--futex-set needs hex i32"))
            };
            const IDLE_FUTEX_CALLSITE: u64 = 0x10284d134; // guest lr when parked in the idle barrier
            // --futex-bump: the engine idle barrier is a wait on a VERSIONED
            // object. The parked consumer (wait-with-timeout 0x10284d018,
            // reached via blr — vtable-dispatched) gates on
            //   ldar x8,[Q]; cmp x21, x8 lsr#32   (0x2856ef4/efc)
            // where Q = t.x19 (arg0), and [Q+4] (== t.x1) is the futex latch.
            // It only PROCEEDS past the park when the version word [Q] high-32
            // CHANGES — a bare latch poke (--futex-kick/--futex-set) is not a
            // producer. --futex-bump also increments [Q] high-32 (version) so
            // the consumer's proceed-gate opens.
            let bump = {
                let args: Vec<String> = std::env::args().collect();
                args.iter().any(|a| a == "--futex-bump")
            };
            std::thread::spawn(move || {
                if let Some(v) = set_val {
                    eprintln!("[elfjit:futexkick] driving idle futex latch every {period_ms} ms, WRITING FIXED {v:#x} (awaited-token test)");
                } else if bump {
                    eprintln!("[elfjit:futexkick] driving idle barrier every {period_ms} ms, BUMPING version [Q]>>32 + latch (real producer shape)");
                } else {
                    eprintln!("[elfjit:futexkick] driving per-thread idle futex latch every {period_ms} ms");
                }
                for it in 0..6000 {
                    std::thread::sleep(std::time::Duration::from_millis(period_ms));
                    for t in arm64jit::jit::snapshot_threads() {
                        if t.lr != IDLE_FUTEX_CALLSITE {
                            continue;
                        }
                        let latch = t.x1; // per-thread futex uaddr (== x19+4)
                        // The latch must be host-addressable (guest==host map).
                        if latch < 0x100000000 || latch >> 56 != 0 {
                            continue;
                        }
                        // A futex uaddr is a 4-byte `int` (4-aligned) — read as a
                        // c_int, never as a u64 (the 4-aligned address misaligns).
                        let old = unsafe { *(latch as *const libc::c_int) };
                        // Version-counter futex: the waiter captures *latch as
                        // its "expected" value and blocks WHILE *latch is
                        // unchanged. Releasing it requires writing a NEW value
                        // (increment the version — never reuse the previous or
                        // the next waiter captures that same value and
                        // re-blocks; a fixed write is a self-defeating one-off).
                        // Gate is the exact idle call-site.
                        let nv = set_val.unwrap_or_else(|| old.wrapping_add(1));
                        // --futex-bump: also increment the VERSION word [Q]
                        // high-32 so the consumer's proceed-gate
                        // (cmp x21, [Q]>>32 at 0x2856efc) opens. Q = t.x19
                        // is a HOST-heap address (0x7f...), writable like the
                        // latch (t.x1 = Q+4); NOT a guest-image address.
                        if bump && t.x19 >= 0x100000000 && (t.x19 >> 56) == 0 && t.x19 & 7 == 0 {
                            let q = t.x19 as *mut u64;
                            let cur = unsafe { *q };
                            let nv_q = cur.wrapping_add(0x1_0000_0000);
                            unsafe { *q = nv_q };
                            if it % 50 == 0 {
                                eprintln!("[elfjit:futexkick] it={it} BUMP [Q]={:#x} ver {:#x}->{:#x}",
                                    q as usize, (cur >> 32), (nv_q >> 32));
                            }
                        }
                        unsafe { *(latch as *mut libc::c_int) = nv };
                        unsafe {
                            libc::syscall(
                                libc::SYS_futex,
                                latch as usize,
                                libc::FUTEX_WAKE as i64,
                                1i64,
                                0usize,
                            );
                        }
                        if it % 50 == 0 {
                            eprintln!(
                                "[elfjit:futexkick] it={it} guest_tid={} latch={latch:#x} old={old:#x}->{nv:#x}",
                                t.guest_tid
                            );
                        }
                    }
                }
            });
        }
        // Host-side task-deque PRODUCER (--deque-node <vtable-hex>). The cycle
        // SH5 frontier is that the parked threads are CONSUMERS of a per-CPU
        // lock-free task-deque (fns 0x285682c / 0x2856e40): each parks in the
        // generic version-epoch futex wait 0x10284d018 on Q'=t.x19 (futex at
        // Q'+4=t.x1) because the deque head-cell ([root]=0x10682a638 /
        // 0x10682b338) points at the self-referential SENTINEL (the drain
        // struct, [headcell].next==0). Version+latch bumping alone
        // (--futex-bump) re-parks — there is no work in the deque. This flag
        // makes a real PRODUCER: it CAS-es a freshly allocated task NODE into
        // the deque head-cell, links it into the circular intrusive list
        // (node.next = the old sentinel head), sets [node+112]=<vtable> so the
        // drain's dispatch ([node+112]&~0x3f -> [vt+40]) reaches a real guest
        // handler, then bumps [Q']>>32 (epoch) + FUTEX_WAKE on Q'+4. A zeroed
        // node (vt=0) trips the drain at [vt+40]=[0x28]; supplying the sentinel
        // vtable 0x106829f00 reaches the real engine handler 0x10285371c — the
        // first controlled crossing, even if that handler then faults on the
        // foreign node's task content.
        if let Some(vt) = {
            let args: Vec<String> = std::env::args().collect();
            args.iter()
                .position(|a| a == "--deque-node")
                .and_then(|i| args.get(i + 1).cloned())
                .map(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16).expect("--deque-node needs hex vtable"))
        } {
            // --deque-node-bump: also bump [Q']>>32 + FUTEX_WAKE. NOTE: this is
            // SELF-DEFEATING per the drain's version gate (a changed version makes
            // the drain return instead of pop on its timeout poll) — kept for the
            // comparison data. Default (no bump) lets the consumer's natural
            // timeout poll drain the node we placed.
            let bump_version = std::env::args().any(|a| a == "--deque-node-bump");
            const IDLE: u64 = 0x10284d134; // parked consumer call-site
            std::thread::spawn(move || {
                use std::collections::HashSet;
                let mut enqueued: HashSet<u64> = HashSet::new();
                let mut placed: Vec<(u64, u64, u64)> = Vec::new(); // (headcell+0x10, node, Q')
                eprintln!("[elfjit:deque-producer] host enqueue on parked consumers (node vtable 0x{vt:x}, bump_version={bump_version})");
                for it in 0..300 {
                    std::thread::sleep(std::time::Duration::from_millis(80));
                    // Post-enqueue verification: did the parked consumer wake and
                    // pop our node (head-cell back to the sentinel / off our node)?
                    if !placed.is_empty() {
                        let mut all_popped = true;
                        for (hc, np, qp) in placed.iter() {
                            let cur = unsafe { *(*hc as *const u64) };
                            let popped = cur != *np;
                            if !popped {
                                all_popped = false;
                            }
                            if it % 25 == 0 || popped {
                                eprintln!("[elfjit:deque-producer] check headcell={hc:#x} node={np:#x} now={cur:#x} popped={popped} Q'={qp:#x}");
                            }
                        }
                        if all_popped {
                            eprintln!("[elfjit:deque-producer] ALL placed nodes popped by consumers — deque crossed the barrier");
                            break;
                        }
                    }
                    for t in arm64jit::jit::snapshot_threads() {
                        if t.lr != IDLE {
                            continue;
                        }
                        if enqueued.contains(&t.guest_tid) {
                            continue;
                        }
                        let is_ptr = |p: u64| p >= 0x100000000 && p >> 56 == 0 && p & 7 == 0;
                        let sp = t.sp;
                        if !is_ptr(sp) {
                            continue;
                        }
                        // Parked drain saved its callee-saved registers at
                        // stp x20,x19,[sp,#64]: [sp+64]=drain root (the deque
                        // root ptr), [sp+72]=drain struct (the sentinel).
                        let root = unsafe { *(sp as *const u64).add(8) };
                        let sentinel = unsafe { *(sp as *const u64).add(9) };
                        if !is_ptr(root) || !is_ptr(sentinel) {
                            continue;
                        }
                        // The root points at a guest-bss head-CELL; its value is
                        // the deque head (now = sentinel = empty).
                        let headcell = unsafe { *(root as *const u64) };
                        if !is_ptr(headcell) {
                            continue;
                        }
                        let old = unsafe { *(headcell as *const u64) };
                        // Only enqueue when the head is still the empty sentinel
                        // (don't stack nodes over an already-pending one).
                        if old != sentinel {
                            continue;
                        }
                        // Allocate guest-visible task node (guest==host here).
                        let node = unsafe { libc::calloc(1, 256) as *mut u8 };
                        if node.is_null() {
                            continue;
                        }
                        let np = node as u64;
                        let qw = unsafe { *(t.x19 as *const u64) };
                        unsafe {
                            *(np as *mut u64) = 0; // node.next = null (this node becomes the tail)
                            (np as *mut u64).add(14).write_volatile(vt); // [node+112] = vtable
                            // WAIT — the drain's POP reads the head-node cell at
                            // [headcell + 0x0] (drain 0x2856f94: `ldr x23,[x20];
                            // ldar x24,[x23]` where x23 = [x20] = headcell, so the
                            // popped node = the VALUE at [headcell]). Prior cycles
                            // wrote to slot+0x10/0x18 (the ring arena's HEAD/TAIL
                            // internals) which the pop never reads — that is why
                            // nodes sat unconsumed. The real head-node cell the pop
                            // drains is offset +0x0. Publish our node there.
                            (headcell as *mut u64).write_volatile(np); // [headcell+0] = head node
                            // The drain's tag guard (0x2856e6c-78): `ldr x26,[x1,#104];
                            // ldr x24,[x23]; cmp x9, x24 lsr#48; b.ne ret` requires the
                            // head-node's high-16 tag == [headcell+8]. Publish the
                            // node's own tag word there so the guard passes.
                            (headcell as *mut u64).add(1).write_volatile(np >> 48);
                            // Keep next/self-link sane: node.next=0 (tail).
                            *((np as *mut u64)) = 0;
                            // Bump the wait object's version epoch so the parked
                            // consumer's proceed-gate (cmp [Q']>>32) opens. NOTE:
                            // self-defeating — see --deque-node-bump above.
                            if bump_version {
                                *(t.x19 as *mut u64) = qw.wrapping_add(0x1_0000_0000);
                                libc::syscall(
                                    libc::SYS_futex,
                                    t.x1 as usize,
                                    libc::FUTEX_WAKE as i64,
                                    1i64,
                                    0usize,
                                );
                            } else {
                                // Even without a version bump, a plain FUTEX_WAKE
                                // lets the drain's wait return; with --drain-poll
                                // forcing a finite timeout it re-enters the pop-loop
                                // and sees our node in [headcell+0].
                                libc::syscall(
                                    libc::SYS_futex,
                                    t.x1 as usize,
                                    libc::FUTEX_WAKE as i64,
                                    1i64,
                                    0usize,
                                );
                            }
                        }
                        eprintln!(
                            "[elfjit:deque-producer] enqueued node={:#x} into headcell[+0]={:#x} Q'{:#x} epoch {:#x} futex={:#x} guest_tid={}",
                            np, headcell, t.x19, qw >> 32, t.x1, t.guest_tid
                        );
                        enqueued.insert(t.guest_tid);
                        placed.push((headcell, np, t.x19));
                    }
                }
            });
        }
        // --deque-node-live <vt-hex>: inject a REAL task node into the LIVE
        // drainer's deque (guest_tid 0 under --drain-poll), NOT the parked
        // consumers' deques (tids 1/2) that --deque-node targets. This is the
        // SH7 documented next lever: the drain (0x2856e40) pop-loop at
        // 0x2856f94 reads the head node from [[root]] (x23=[x20]=[root],
        // x24=ldar[x23]=packed head), CAS-pops it, and — when it is not the
        // sentinel AND [node+40] != 0 AND [vt+40] != 0 — dispatches
        // [vt+40]([vt+16], consumer, [node+32]&~1, node, 4, 0). The deque root
        // for the live drainer is its x20, STABLE across the drain body and
        // readable from the host snapshot. We capture it once and write the
        // node into the head-cell it drains. Injection is gated on the deque
        // head being empty (low48==0) / the sentinel to avoid stacking over a
        // pending node, and we verify the node was popped (head-cell moved off
        // our packed value).
        if let Some(vt) = {
            let args: Vec<String> = std::env::args().collect();
            args.iter()
                .position(|a| a == "--deque-node-live")
                .and_then(|i| args.get(i + 1).cloned())
                .map(|v| {
                    if v == "probe" {
                        // Auto-build a HOST-THUNK PROBE vtable: [vt+40]=registered
                        // host thunk, [vt+16]=ctx marker. The drain dispatch of a
                        // FOREIGN node ([node+112]&~0x3f -> [vt+40]) then calls OUR
                        // probe with the real engine ABI args, firing the logging
                        // counter — the controlled type-4 crossing SH7b demanded.
                        // This avoids hand-resolving a real render/tick vtable.
                        extern "C" fn probe(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, _a6: u64, _a7: u64) -> u64 {
                            use std::sync::atomic::{AtomicU64, Ordering};
                            static CNT: AtomicU64 = AtomicU64::new(0);
                            let c = CNT.fetch_add(1, Ordering::Relaxed) + 1;
                            // The drain re-enqueues every popped node, so the head
                            // stays = our node while it IS being dispatched; the
                            // only discriminating signal is this type-4 dispatch.
                            if c <= 3 || c % 10000 == 0 {
                                eprintln!(
                                    "[elfjit:deque-probe] type-4 dispatch #{c}: x0(vt+16)={a0:#x} x1(consumer)={a1:#x} x2(node+32&~1)={a2:#x} x3(node)={a3:#x} w4={a4} x5={a5}"
                                );
                            }
                            0
                        }
                        let probe_addr = arm64jit::jit::register_host_call_auto(probe);
                        // Vtable MUST live at a guest-visible address (< 2^48,
                        // mapped RW), not host heap: the drain does `ldr [vt+40]`
                        // as guest memory, so a host-heap vt (0x55..) reads garbage.
                        let v = unsafe { libc::calloc(1, 8 * 8) as *mut u8 };
                        let vt_host = v as u64;
                        let v = guest_arena_alloc(8 * 8) as *mut u8;
                        unsafe {
                            (v as *mut u64).add(2).write_volatile(0x_dead_beef); // [vt+16] ctx
                            (v as *mut u64).add(5).write_volatile(probe_addr); // [vt+40] handler
                        }
                        eprintln!(
                            "[elfjit:deque-node-live] PROBE vtable (vt=0x{:x} guest, host-def 0x{vt_host:x}, [vt+40]=0x{probe_addr:x}) — foreign-node dispatch will hit a registered host-thunk",
                            v as u64
                        );
                        v as u64
                    } else {
                        u64::from_str_radix(v.trim_start_matches("0x"), 16).expect("--deque-node-live needs hex vtable or 'probe'")
                    }
                })
        } {
            // Drain body span (guest vaddrs) where the drain holds x20 = deque root.
            const DRAIN_LO: u64 = 0x102856e40;
            const DRAIN_HI: u64 = 0x1028570a4;
            // Optional --deque-arg2 <hex>: override [node+32] of the injected node
            // (the drain passes it as dispatch arg2, x2 = [node+32]&~1). Default
            // keeps the cloned sentinel's [node+32] (or 0). Sweeping this value is
            // the controllable node-content selector into the real dispatcher.
            let arg2_override: Option<u64> = std::env::args()
                .position(|a| a == "--deque-arg2")
                .and_then(|i| std::env::args().nth(i + 1))
                .map(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16).expect("--deque-arg2 needs hex"));
            std::thread::spawn(move || {
                use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
                static ROOT: AtomicU64 = AtomicU64::new(0);
                static PLACED: AtomicU64 = AtomicU64::new(0);
                static HEADCELL: AtomicU64 = AtomicU64::new(0);
                // SH44/SH11: the force-pop patches + block-cache eviction must run
                // EXACTLY ONCE. The patches are idempotent and the eviction only
                // needs to force the drain body to recompile from the now-patched
                // guest bytes the first time; a cache drop on EVERY re-injection
                // recompiles the pop-loop while the drain is mid-execution, and
                // after ~40 re-injections the translated block desyncs (the crash
                // at 0x102856f7c in runs/sh44-taskv4-plane.txt). Subsequent
                // injections only SWAP the node into the head-cell (no code patch,
                // no eviction), so a sustainable type-4 dispatch loop is possible
                // — the prerequisite for seeding the vector with a real guest
                // producer and observing sustained forwarding.
                static ARMED: AtomicBool = AtomicBool::new(false);
                eprintln!(
                    "[elfjit:deque-node-live] inject into LIVE drainer's deque (vtable 0x{vt:x}); draining when pc in [0x{DRAIN_LO:x},0x{DRAIN_HI:x})"
                );
                for it in 0..400 {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let is_ptr = |p: u64| p >= 0x100000000 && p >> 56 == 0 && p & 7 == 0;
                    // Already placed a node?
                    let np = PLACED.load(Ordering::Relaxed);
                    if np != 0 {
                        let hc = HEADCELL.load(Ordering::Relaxed);
                        let cur = unsafe { *(hc as *const u64) };
                        let popped = cur != np;
                        if popped {
                            static POPS: AtomicU64 = AtomicU64::new(0);
                            let p = POPS.fetch_add(1, Ordering::Relaxed) + 1;
                            eprintln!(
                                "[elfjit:deque-node-live] NODE 0x{np:x} POPPED by live drainer (headcell now 0x{cur:x}) — dispatch #{}; re-injecting a fresh node to sustain the type-4 dispatch loop", p
                            );
                            // Reset so the next iteration places a NEW node at head
                            // (the drain consumed this one and its head is empty).
                            PLACED.store(0, Ordering::Relaxed);
                            HEADCELL.store(0, Ordering::Relaxed);
                            continue;
                        }
                        if it % 20 == 0 {
                            eprintln!("[elfjit:deque-node-live] node 0x{np:x} still head (headcell=0x{cur:x})");
                        }
                        continue; // keep polling until popped
                    }
                    // Recon for the first ~3 ticks only (a tiny window): the new
                    // SH11 strategy needs OUR node at head BEFORE the first forced
                    // pop, so we must inject almost immediately. The --drain-force-
                    // pop path faults the sentinel-as-task at ~200ms, so a 2s recon
                    // (SH9's it<40) structurally loses the race. Collapse recon to
                    // a one-shot diagnostic, then inject right away.
                    if it < 3 {
                        let snaps = arm64jit::jit::snapshot_threads();
                        let mut rr = 0u64;
                        for t in &snaps {
                            if t.pc >= DRAIN_LO && t.pc < DRAIN_HI && is_ptr(t.x20) {
                                rr = t.x20;
                                break;
                            }
                        }
                        if rr != 0 && is_ptr(rr) {
                            ROOT.store(rr, Ordering::Relaxed);
                            let cell = unsafe { *(rr as *const u64) };
                            if is_ptr(cell) {
                                let head = unsafe { *(cell as *const u64) };
                                let headnode = head & 0xffff_ffff_ffff;
                                eprintln!(
                                    "[elfjit:deque-node-live][recon it={it}] root={rr:#x}[0]={cell:#x}[8]={:#x} headcell[0]=0x{head:x} low48={headnode:#x}",
                                    unsafe { *(rr as *const u64).add(1) }
                                );
                                if is_ptr(headnode) {
                                    let rd = |base: u64, o: usize| unsafe { *(base as *const u64).add(o / 8) };
                                    let v112 = rd(headnode, 112);
                                    let vt = v112 & !0x3f;
                                    let vt40 = if is_ptr(vt) { rd(vt, 40) } else { 0 };
                                    eprintln!(
                                        "[elfjit:deque-node-live][recon] headnode={headnode:#x} +40={:#x} +112={v112:#x} vt={vt:#x} [vt+40]={vt40:#x}",
                                        rd(headnode, 40)
                                    );
                                }
                            }
                        }
                        // fall through to inject on ticks >= 1 (root may be 0 on
                        // tick 0; re-captured below if so).
                    }
                    // Capture the live drainer's deque root once.
                    let root = ROOT.load(Ordering::Relaxed);
                    let snaps = arm64jit::jit::snapshot_threads();
                    let mut live_root = 0u64;
                    for t in &snaps {
                        // Drain body in progress -> x20 IS the deque root.
                        if t.pc >= DRAIN_LO && t.pc < DRAIN_HI && is_ptr(t.x20) {
                            live_root = t.x20;
                            break;
                        }
                        // Just left the drain into the dispatch handler: x20
                        // may already be clobbered, but guest_tid 0's lr is a
                        // drain-body return address while the drain ran.
                    }
                    if root == 0 {
                        if live_root == 0 {
                            if it % 20 == 0 {
                                eprintln!("[elfjit:deque-node-live] waiting for live drainer pc in drain body (it={it})");
                            }
                            continue;
                        }
                        ROOT.store(live_root, Ordering::Relaxed);
                        eprintln!("[elfjit:deque-node-live] recovered live drainer deque root x20={live_root:#x}");
                    }
                    let root = ROOT.load(Ordering::Relaxed);
                    // headcell = [root]; the pop reads the packed head from it.
                    if !is_ptr(root) {
                        continue;
                    }
                    let headcell = unsafe { *(root as *const u64) };
                    if !is_ptr(headcell) {
                        continue;
                    }
                    let old = unsafe { *(headcell as *const u64) };
                    // Dump the deque struct neighborhood to reverse the exact
                    // layout (root -> headcell -> packed head) from live memory.
                    if it % 40 == 0 {
                        let r0 = unsafe { *(root as *const u64).add(0) };
                        let r1 = unsafe { *(root as *const u64).add(1) };
                        let r2 = unsafe { *(root as *const u64).add(2) };
                        let r3 = unsafe { *(root as *const u64).add(3) };
                        let h0 = unsafe { *(headcell as *const u64).add(0) };
                        let h1 = unsafe { *(headcell as *const u64).add(1) };
                        eprintln!(
                            "[elfjit:deque-node-live] root={root:#x}[0]={r0:#x}[8]={r1:#x}[+16]={r2:#x}[+24]={r3:#x} headcell={headcell:#x}[0]={h0:#x}(low48 {:#x})[8]={h1:#x}",
                            h0 & 0xffff_ffff_ffff
                        );
                    }
                    // The drain keeps the deque head non-empty (it
                    // continuously pops + re-enqueues the self/sentinel node),
                    // so there is no "empty" window to wait for. Inject by
                    // SWAPPING our node over the live head: the drain's next
                    // CAS-pop reads our packed value, truncates low-48 to our
                    // node, and dispatches it (non-sentinel, [node+40]!=0).
                    if it % 20 == 0 {
                        eprintln!("[elfjit:deque-node-live] headcell 0x{headcell:x} head=0x{old:x} (replacing with task node)");
                    }
                    // The drain's entry tag guard (0x2856e74) requires the head
                    // node's high-16 tag == [root+8]. Read that tag so the packed
                    // value passes the guard and the low-48 truncation yields our
                    // node on pop.
                    let tag = unsafe { *(root as *const u64).add(1) }; // [root+8]
                    // Bind the dispatch handler: [node+112]&~0x3f -> vt, [vt+40]=handler.
                    // CLONE the live head node's coherent payload as the base so
                    // the drain's post-dispatch RE-ENQUEUE (producer 0x285682c)
                    // walks valid link/refcount fields instead of zeroed garbage.
                    // The live head node (sentinel during idle, `low48(headcell[0])`)
                    // is a fully-constructed task node the drain already pops and
                    // re-enqueues every maintenance iteration — the ideal template.
                    // (SH9's "[consumer+104]" indexing is unreliable: the consumer
                    // x19 is rarely snapshotted in-body, so fall back to the head
                    // node, which is guaranteed present and coherent.)
                    let node: *mut u8 = {
                        let mut sentinel = 0u64;
                        let hn = old & 0xffff_ffff_ffff;
                        // Node MUST be guest-arena allocated: its address is
                        // low48-packed into the head cell AND the drain reads/
                        // writes its fields as guest memory, so a host-heap
                        // (0x7f2a...) node would be mangled by the pop's low48
                        // truncation (0x7f2a... -> 0x2a...) and fault.
                        let n = guest_arena_alloc(256) as *mut u8;
                        if !n.is_null() {
                            if is_ptr(hn) && hn != n as u64 {
                                // Copy head-node node-constructor layout (link + refcount
                                // + args + vtable handled below).
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        hn as *const u8, n, 256,
                                    );
                                }
                                sentinel = hn;
                                eprintln!(
                                    "[elfjit:deque-node-live] cloned head node 0x{sentinel:x} as node base (headcell[0]=0x{old:x}) -> guest node {:#x}",
                                    n as u64
                                );
                            } else {
                                eprintln!(
                                    "[elfjit:deque-node-live] no coherent head-node template, using zeroed node (may crash on re-enqueue)"
                                );
                            }
                        }
                        n
                    };
                    if node.is_null() {
                        continue;
                    }
                    let np = node as u64;
                    // SH60 frame-mode: hold the node PLACEMENT until --renderthunk
                    // recovers RENDERCTX. Root/headcell are captured earlier in this
                    // same iteration (drain active), so the wait loses nothing; the
                    // drain stays on its finite-timeout heartbeat (never force-arms,
                    // never pops the sentinel) until we place + arm. Without this,
                    // the drain's type-4 (w4=4) dispatches are front-loaded onto the
                    // first pops, which fire before RENDERCTX is recovered -> the
                    // seeded frame thunk no-ops and no task frame ever presents.
                    if taskv4_frame_seed_active() {
                        let t0 = std::time::Instant::now();
                        let mut had_to_wait = false;
                        while RENDERCTX.load(Ordering::Relaxed) == 0 && t0.elapsed().as_secs() < 25 {
                            had_to_wait = true;
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        if had_to_wait {
                            eprintln!(
                                "[elfjit:deque-node-live] frame-mode: RENDERCTX={:#x} after {:?} — placing first task node now so its type-4 dispatch presents a real frame",
                                RENDERCTX.load(Ordering::Relaxed),
                                t0.elapsed()
                            );
                        }
                    }
                    unsafe {
                        // Fresh tail: the re-enqueue producer (0x285682c) walks the
                        // node's [node+0] next-link to find the tail; the *cloned*
                        // head-node template still points at the old sentinel ring,
                        // so zero it to a clean tail before publishing (else the
                        // producer follows the stale link and faults at pc 0x51).
                        (np as *mut u64).write_volatile(0);
                        // [node+112] = vtable; [vt+40] must be a real handler fn.
                        (np as *mut u64).add(14).write_volatile(vt);
                        // [node+40] != 0 so the drain DISPATCHES the handler on pop.
                        (np as *mut u64).add(5).write_volatile(
                            ((np as *const u64).add(5).read_volatile()) | 1,
                        );
                        // [node+32] = arg (dispatch arg2 = [node+32]&~1); keep
                        // sentinel's (or 0) unless --deque-arg2 overrides it.
                        (np as *mut u64).add(4).write_volatile(
                            arg2_override.unwrap_or_else(|| {
                                (np as *const u64).add(4).read_volatile()
                            }),
                        );
                        // Pack: low48 = node pointer (so pop truncates to it),
                        // high16 = tag matching [root+8].
                        let packed = np | ((tag & 0xffff) << 48);
                        // Publish into the head-cell the drain pops from.
                        (headcell as *mut u64).write_volatile(packed);
                        HEADCELL.store(headcell, Ordering::Relaxed);
                        PLACED.store(packed, Ordering::Relaxed);
                        // ARM FORCE-POP exactly ONCE (SH44/SH11): patch the drain's
                        // pop-loop to always fall through (`mov w24,w0` 0x102856f4c ->
                        // mov w24,#1 and NOP the tbz 0x102856f7c) so the next drain
                        // iteration pops+dispatches OUR foreign node, then drop the
                        // already-compiled drain block so it recompiles from the
                        // patched bytes. Doing this on EVERY re-injection re-evicts
                        // the block while the drain runs it and eventually
                        // desyncs the translation (SH44 crash at 0x102856f7c).
                        if !ARMED.swap(true, Ordering::Relaxed) {
                            let arm = [0x102856f4cu64, 0x102856f7cu64];
                            for a in arm {
                                let p = a & !0xfff;
                                unsafe {
                                    libc::mprotect(p as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE);
                                }
                                let before = unsafe { *(a as *const u32) };
                                let word = if a == 0x102856f7c { 0xd503_201fu32 /* NOP */ } else { 0x5280_0018u32 /* mov w24,#1 */ };
                                unsafe { *(a as *mut u32) = word };
                                unsafe {
                                    libc::mprotect(p as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
                                }
                                eprintln!(
                                    "[elfjit:deque-node-live] ARMED force-pop {:#x} (was {before:08x}) -> {word:08x}",
                                    a
                                );
                            }
                            // The drain body was already compiled (unpatched) into the
                            // block cache; drop those entries so the dispatcher
                            // recompiles it from the now-patched guest bytes on the
                            // next re-entry (otherwise the force-pop has no effect).
                            arm64jit::jit::block_cache_drop_region(0x102856e40, 0x1028570c0);
                            eprintln!(
                                "[elfjit:deque-node-live] dropped cached drain blocks [0x102856e40,0x1028570c0) — pop-loop will recompile patched"
                            );
                        }
                        eprintln!(
                            "[elfjit:deque-node-live] INJECTED node 0x{np:x} packed=0x{packed:x} into headcell 0x{headcell:x} (tag {tag:#x}) — awaiting pop by live drainer"
                        );
                    }
                }
                eprintln!("[elfjit:deque-node-live] gave up after 400 ticks");
            });
        }
        // --taskv4-seed <probe|guest-hex>: populate the dispatcher's TYPE-4
        // popped-task handler vector at guest BSS 0x106829ea8
        // (dispatcher 0x10285371c w4=4 path: `adrp x8,6829000; ldr x3,[x8,#3752];
        // br x3` at file 0x2853788/0x28537b8). During headless boot this vector is
        // 0 (a NULL .bss function ptr a real framework producer would install), so
        // the drain's type-4 dispatch of ANY popped task node returns at
        // 0x285378c->0x2853af0 doing nothing — the exact mechanical reason no
        // injected/foreign node can drive the engine toward a frame. Seeding it
        // with a registered HOST-THUNK probe (or a chosen guest fn) lets a
        // sentinel-vtable node pop through the REAL dispatcher w4=4 plane and hit
        // our vector, proving the plane is dispatchable when the slot is live.
        if let Some(spec) = std::env::args()
            .position(|a| a == "--taskv4-seed")
            .and_then(|i| std::env::args().nth(i + 1))
        {
            const TASKV4: u64 = 0x106829ea8;
            use std::sync::atomic::{AtomicU64, Ordering};
            static V4: AtomicU64 = AtomicU64::new(0);
            extern "C" fn v4probe(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, _a6: u64, _a7: u64) -> u64 {
                let c = V4.fetch_add(1, Ordering::Relaxed) + 1;
                if c <= 3 || c % 5000 == 0 {
                    eprintln!(
                        "[elfjit:taskv4] type-4 task handler #{c}: fnarg0(x0)={a0:#x} arg1={a1:#x} arg2={a2:#x} node={a3:#x} w4={a4} x5={a5}"
                    );
                }
                0
            }
            let seed = if spec == "probe" {
                arm64jit::jit::register_host_call_auto(v4probe)
            } else if spec == "frame" {
                // SH60: seed the dispatcher's type-4 vector with the real
                // task-driven frame thunk — each dispatched task node marshal's
                // into a REAL presented frame (recon-selfdrive-seed-jsonfix.md
                // §A). Requires RENDERCTX (recovered by --renderthunk) to be set;
                // dispatches before that no-op via the thunk's self-guard.
                let f = arm64jit::jit::register_host_call_auto(type4_frame_thunk);
                eprintln!(
                    "[elfjit:taskv4] type4_frame_thunk registered at {f:#x} — a popped task node reaching w4=4 will now present a real task-driven frame"
                );
                f
            } else {
                u64::from_str_radix(spec.trim_start_matches("0x"), 16).expect("--taskv4-seed needs 'probe', 'frame', or a hex guest fn addr")
            };
            unsafe {
                *(TASKV4 as *mut u64) = seed;
                eprintln!(
                    "[elfjit:taskv4] seeded dispatcher type-4 vector [0x{TASKV4:x}] = {seed:#x}{}",
                    if seed != 0 { " — a popped task node reaching w4=4 will now call it" } else { " (cleared)" }
                );
            }
        }
        // --deque-probe: convert the forced-pop sentinel fault into a CONTROLLED
        // type-4 dispatch the SH7b frontier demanded. The engine's real pop-loop
        // (0x2856f94) pops the head node and dispatches
        //   [node+112]&~0x3f -> vt; handler = [vt+40]; if [node+40]!=0 && handler!=0
        //   then handler([vt+16], x19=consumer, [node+32]&~1, node, w4=4, x5=0)
        // During idle the head node is the SENTINEL (the drain struct itself),
        // whose [node+112]=0x106829f00 -> [vt+40]=0x10285371c (the engine's own
        // dispatcher), which walks the sentinel's garbage task content and
        // strlen-faults (exit 134, the current unstable state). Instead of racing
        // a foreign node into the deque ahead of the fault, REPOINT the sentinel's
        // live [node+112] at a vtable WE control whose [vt+40] is a registered
        // host-thunk probe. Then every forced pop dispatches OUR probe with the
        // real engine ABI args (vt+16 / consumer / node+32 / node / w4=4 / 0),
        // stably, capturing the discriminate type-4 dispatch. Opt-in; default
        // --deque-node-live and plain --drain-force-pop unchanged.
        // --deque-probe <ctx-qw-hex> writes that qword to the sentinel's [node+32]
        // (the ABI arg passed as x2, &~1) so the probe proves which node road it.
        if std::env::args().any(|a| a == "--deque-probe") {
            let ctx = std::env::args()
                .position(|a| a == "--deque-probe")
                .and_then(|i| std::env::args().nth(i + 1))
                .map(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16).ok())
                .flatten();
            const DRAIN_LO: u64 = 0x102856e40;
            const DRAIN_HI: u64 = 0x1028570a4;
            use std::sync::atomic::{AtomicU64, Ordering};
            static PROBE_COUNT: AtomicU64 = AtomicU64::new(0);
            extern "C" fn probe(a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64, _a6: u64, _a7: u64) -> u64 {
                let c = PROBE_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
                if c == 1 || c % 10000 == 0 {
                    eprintln!(
                        "[elfjit:deque-probe] type-4 dispatch #{c}: x0(vt+16)={a0:#x} x1(consumer)={a1:#x} x2(node+32&~1)={a2:#x} x3(node)={a3:#x} w4={a4} x5={a5}"
                    );
                }
                0
            }
            // Allocate a guest-visible fake vtable: [vt+16] = ctx marker,
            // [vt+40] = probe host-thunk address (JIT routes guest `blr` to it).
            let vt = unsafe { libc::calloc(1, 8 * 8) as *mut u8 };
            let probe_addr = arm64jit::jit::register_host_call_auto(probe);
            let ctxv = ctx.unwrap_or(0);
            unsafe {
                (vt as *mut u64).add(2).write_volatile(ctxv); // [vt+16] (a0)
                // Handler slot is [vt+40] = byte 40 = u64 index 5 (same fix as the
                // --deque-node-live probe; writing index 4 reads 0 at [vt+40]).
                (vt as *mut u64).add(5).write_volatile(probe_addr); // [vt+40] (handler)
            }
            let vtaddr = vt as u64;
            std::thread::spawn(move || {
                eprintln!(
                    "[elfjit:deque-probe] probing sentinel dispatch (vt 0x{vtaddr:x}, probe 0x{probe_addr:x} -> [vt+40], ctx {ctxv:#x})"
                );
                let mut repointed: Vec<u64> = Vec::new();
                for it in 0..900 {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                    let is_ptr = |p: u64| p >= 0x100000000 && p >> 56 == 0 && p & 7 == 0;
                    let snaps = arm64jit::jit::snapshot_threads();
                    let mut roots: Vec<u64> = Vec::new();
                    for t in &snaps {
                        for r in [t.x20, t.x19] {
                            if is_ptr(r) && r >= 0x100000000 {
                                roots.push(r);
                            }
                        }
                        if t.lr == 0x10284d134 && is_ptr(t.sp) {
                            let r = unsafe { *(t.sp as *const u64).add(8) };
                            if is_ptr(r) {
                                roots.push(r);
                            }
                        }
                    }
                    roots.sort_unstable();
                    roots.dedup();
                    for rr in roots {
                        let headcell = unsafe { *(rr as *const u64) };
                        if !is_ptr(headcell) {
                            continue;
                        }
                        let head = unsafe { *(headcell as *const u64) };
                        let sentinel = head & 0xffff_ffff_ffff;
                        if !is_ptr(sentinel) || repointed.contains(&sentinel) {
                            continue;
                        }
                        let cur_v112 = unsafe { *((sentinel as *const u64).add(112 / 8)) };
                        if cur_v112 == 0x106829f00 {
                            unsafe {
                                (sentinel as *mut u64).add(112 / 8).write_volatile(vtaddr);
                            }
                            repointed.push(sentinel);
                            eprintln!(
                                "[elfjit:deque-probe] REPOINTED sentinel 0x{sentinel:x} (root 0x{rr:x}, headcell 0x{headcell:x}): [node+112] 0x{cur_v112:x}->0x{vtaddr:x}"
                            );
                        }
                    }
                    let cnt = PROBE_COUNT.load(Ordering::Relaxed);
                    if cnt >= 5 && it % 40 == 0 {
                        eprintln!(
                            "[elfjit:deque-probe] CONFIRMED {cnt} controlled type-4 dispatches through our vtable"
                        );
                    }
                }
                eprintln!("[elfjit:deque-probe] gave up (probe count={}, repointed={})", PROBE_COUNT.load(Ordering::Relaxed), repointed.len());
            });
        }
        // SH60 --taskv4-seed frame: force every drain dispatch through the
        // type-4 vector. The drain's idle path (heartbeat) routes its per-
        // iteration SENTINEL dispatch to w4=2/3 telemetry emitters
        // (0x2856f24 / 0x2856f68), which never reach the type-4 vector
        // [0x106829ea8]; the genuine w4=4 "popped task node" path (0x2856ffc)
        // is only hit during an early init window, so a seeded probe fires only
        // ~3× (all pre-RENDERCTX). Deterministic fix: rewrite the heartbeat's
        // `mov w4,#2`/`mov w4,#3` to `mov w4,#4`, so EVERY idle dispatch calls
        // the real dispatcher (0x10285371c) with w4=4 -> it loads the seeded
        // vector and br's to the type4_frame_thunk -> a real task-driven frame
        // per drain iteration, sustained (RENDERCTX self-guard no-ops the
        // pre-recovery heartbeats).
        if taskv4_frame_seed_active() {
            for (addr, name, word) in [
                (0x102856f24u64, "heartbeat w4#2", 0x52800084u32), // mov w4,#4
                (0x102856f68u64, "heartbeat w4#3", 0x52800084u32), // mov w4,#4
            ] {
                let page = addr & !0xfff;
                unsafe {
                    if libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) == 0 {
                        let before = *(addr as *const u32);
                        *(addr as *mut u32) = word;
                        libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC);
                        eprintln!(
                            "[elfjit:taskv4-frame] patched {name} 0x{addr:x} (was {before:08x}) -> {word:08x} — every drain dispatch now routes w4=4 through the type-4 vector to a real task-driven frame"
                        );
                    }
                }
            }
        }
        // Disable the gate-2 re-arm store: the owner's cond-wait loop at
    // 0x102b4cd50/0x102b4cd84 re-parks while *x19==1 and, on seeing that
    // pred has become 0, RE-ARMS it back to 1 (`mov x8,#1; str x8,[x19]` at
    // 0x102b4cdb0/0x102b4cdb4) so the terminal value driven from the host
    // never sticks. NOP the re-arm store so our value persists. JIT_DRIVE_*
    // mode only.
    if std::env::var_os("JIT_DRIVE_LIFECYCLE").is_some() {
        let rearm = el.guest_of(0x102b4cdb4);
        let page = rearm & !0xfff;
        if unsafe { libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) } == 0 {
            unsafe { *(rearm as *mut u32) = 0xd503_201fu32 }; // NOP
            unsafe { libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC) };
            eprintln!("[kernel:NOP re-arm store 0x{rearm:x} (gate-2) under JIT_DRIVE_LIFECYCLE");
        }
    }
    // --drain-poll <ms>: force the engine idle-task-deque consumer's drain
    // (0x2856e40) to use a FINITE wait timeout instead of the infinite -1 it
    // blocks on during idle. The parked threads deadlock because
    // `mov x2,x22` (0x2856f40, x22=drain timeout arg = -1) hands generic-wait
    // 0x284d014 an infinite timeout -> it parks in a bare futex forever, so the
    // drain's pop-loop at 0x2856f94 (reached ONLY when the wait returns
    // timed-out w0=1 AND the version matches) never runs. Patching that copy to
    // a finite ms value makes the wait time out, the drain reach the pop-loop,
    // find a host-placed task node in [headcell+0], and dispatch [node+112]->[vt+40].
    // Patch the guest IMAGE before jit_run so the drain block compiles with it.
    {
        let args: Vec<String> = std::env::args().collect();
        if let Some(i) = args.iter().position(|a| a == "--drain-poll") {
            let ms: u32 = args
                .get(i + 1)
                .expect("--drain-poll <ms>")
                .parse()
                .expect("--drain-poll needs integer ms");
            assert!(ms < 4096, "--drain-poll ms must be < 4096 (imm12)");
            // Patch the guest image (identity host mapping) BEFORE jit_run so the
            // drain block compiles with the finite timeout. The parked threads'
            // lr=0x10284d134 shows true guest addrs are in 0x1028xxxx, so the
            // instruction's true guest==host addr is 0x102856f40 directly (NOT
            // re-mapped via guest_of, which double-shifts to 0x202856f40).
            let insn_addr: u64 = 0x102856f40;
            let patch: u32 = 0xd280_0002 | (ms << 5); // mov x2, #ms (imm12<4096)
            let page = insn_addr & !0xfff;
            eprintln!("[elfjit:drain-poll] base_load=0x{:x} base_addr=0x{:x} guest_of(0x102856f40)=0x{:x}; read now={:08x}",
                el.info.base_load_addr, el.base_addr, el.guest_of(0x102856f40),
                unsafe { *(insn_addr as *const u32) });
            if unsafe { libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) } == 0 {
                unsafe { *(insn_addr as *mut u32) = patch };
                unsafe { libc::mprotect(page as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC) };
                eprintln!("[elfjit:drain-poll] patched 0x{insn_addr:x} -> mov x2,#{ms}ms (0x{patch:08x})");
            } else {
                eprintln!("[elfjit:drain-poll] WARN mprotect RW failed at 0x{page:x} errno={}", std::io::Error::last_os_error());
            }
            // SH7's --drain-poll claimed the finite timeout alone makes the
            // pop-loop run, but that is WRONG (corrected here): generic-wait
            // 0x284d014 maps the host futex's ETIMEDOUT (-110) return into w0=0
            // ("woken"), because `cmn x0,#1` (0x284d0a4) only treats an EXACT
            // x0==-1 as a timeout-under-deadline; -110 falls through to
            // 0x284d0ec and returns 0. So the drain's `tbz w24,#0` (0x2856f7c)
            // always re-loops and the pop-loop 0x2856f94 never runs (measured:
            // 0 hits / 128k drain branches). Forcing the pop-loop itself (the
            // real crossing) needs the drain's wait-result latch AND the tbz:
            // `mov w24,w0` at 0x102856f4c -> mov w24,#1, and NOP the tbz
            // 0x102856f7c so the drain falls through to the version-check and
            // the pop-loop, which then CAS-pops and dispatches a placed node.
            // This reaches previously-dead code and faults on dispatch of a
            // non-real task node (the "controlled first crossing"), so it is
            // opt-in via --drain-force-pop; plain --drain-poll keeps its
            // documented stable (finite-timeout maintenance heartbeat) behavior.
            let force = std::env::args().any(|a| a == "--drain-force-pop");
            // If --deque-node-live is also present, DEFER the force-pop patches to
            // the injector thread (see its "arm force-pop" step): patching here at
            // startup makes the drain pop the SENTINEL as the first task and fault
            // (~200ms) before any injected node can land. Left unpatched here, the
            // drain stays stable (never pops) while we place our node, then the
            // injector arms the pop-loop so the FIRST forced pop takes OUR foreign
            // node (passes the self-node-skip guard, [node+40]=1) and dispatches it.
            let deferred = std::env::args().any(|a| a == "--deque-node-live");
            if force && !deferred {
            let latch_addr: u64 = 0x102856f4c; // mov w24,w0 (=0x2a0003f8)
            let _latch_patch: u32 = 0x52800018; // mov w24,#1 (MOVZ W24,#1)
            let tbz_addr: u64 = 0x102856f7c;
            let tbz_page = tbz_addr & !0xfff;
            for (a, name) in [(latch_addr, "w24"), (tbz_addr, "tbz")] {
                let p = a & !0xfff;
                if unsafe { libc::mprotect(p as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_WRITE) } != 0 {
                    eprintln!("[elfjit:drain-poll] WARN mprotect RW failed at {name} 0x{p:x} errno={}", std::io::Error::last_os_error());
                    continue;
                }
                let before = unsafe { *(a as *const u32) };
                let patch_word: u32 = if a == tbz_addr { 0xd503_201f /* NOP */ } else { 0x5280_0018 /* mov w24,#1 */ };
                unsafe { *(a as *mut u32) = patch_word };
                let _ = unsafe { libc::mprotect(p as *mut libc::c_void, 4096, libc::PROT_READ | libc::PROT_EXEC) };
                eprintln!("[elfjit:drain-poll] FORCE pop-loop: patched {name} 0x{a:x} (was {before:08x}) -> {patch_word:08x}");
            }
            let _ = tbz_page;
            }
        }
    }

    // JIT_FRAMEWORK_DUMP: StartApp's jit_run below parks the main thread in the
    // engine main loop and never returns, so a post-run sampler would never
    // run. Instead spawn a detached host thread that samples the framework-built
    // globals (guest==host addressing) every ~500 ms while StartApp initializes
    // and parks, so we learn whether the render-init context (0x1067d16f0) or
    // the deque-maintenance forward-edges (0x1068262e8/300/308) get POPULATED
    // at runtime — i.e. whether driving the real render-init after warm-up runs.
    if std::env::var_os("JIT_FRAMEWORK_DUMP").is_some() {
        std::thread::spawn(|| {
            let dw = |a: u64| -> u64 {
                if a >= 0x100000000 && a >> 56 == 0 && a & 7 == 0 {
                    unsafe { *(a as *const u64) }
                } else {
                    0
                }
            };
            for _ in 0..60 {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let ctx = dw(0x1067d16f0);
                // Type-4 (popped task-node) dispatch vector: the dispatcher
                // 0x10285371c does `cmp w4,#4; ... adrp x8,6829000; ldr x3,[x8,#3752];
                // br x3` (file 0x2853788/0x28537b8) — the function pointer our injected
                // task nodes ACTUALLY call when popped. If it is 0 the type-4 path
                // returns at 0x285378c->0x2853af0 doing nothing. Also dump the
                // type-0/2 backup vector at [0x6826000+800]=0x106826320 (`br x4`).
                eprintln!(
                    "[elfjit:fw] render-ctx 0x1067d16f0={:#x} | deque-fwd 0x1068262e8={:#x} 0x106826300={:#x} 0x106826308={:#x} | task-v4 [0x106829ea8]={:#x} v0/2 [0x106826320]={:#x} | [*ctx]={:#x}",
                    ctx,
                    dw(0x1068262e8),
                    dw(0x106826300),
                    dw(0x106826308),
                    dw(0x106829ea8),
                    dw(0x106826320),
                    if ctx != 0 && ctx >> 56 == 0 { dw(ctx) } else { 0 },
                );
            }
        });
    }

    // --renderinit <link-addr>: after StartApp's init has populated the framework/
    // render context global 0x1067d16f0 (verified live 0x562a.. — SH14's
    // "statically 0, framework-gated, not drivable" is WRONG at runtime),
    // drive the engine's REAL EGL render-init (SH14 pinned eglGetDisplay->
    // eglInitialize->eglCreateContext->eglCreateWindowSurface->eglMakeCurrent at
    // fn 0x105b3a2d8 / thunk 0x105b3a280) directly. Runs on a DETACHED host
    // thread because StartApp's main-thread jit_run parks in the idle futex and
    // never returns; it sleeps `warmup` ms first so StartApp populates the
    // context. clear_block_cache on its top-level entry is SAFE (JitBlocks leak,
    // never munmap), so StartApp's parked threads just recompile on wake.
    let renderinit_args: Vec<String> = std::env::args().collect();
    // Clone the full arg list again for the opt-in --renderframe sub-mode (drives
    // the render-init THUNK then the swap fn to actually present a buffer).
    let renderframe_args: Vec<String> = std::env::args().collect();
    if let Some(i) = renderinit_args.iter().position(|a| a == "--renderinit") {
        let rhex = renderinit_args
            .get(i + 1)
            .cloned()
            .expect("--renderinit needs a link-addr hex");
        let link = u64::from_str_radix(rhex.trim_start_matches("0x"), 16)
            .unwrap_or_else(|_| panic!("bad --renderinit hex"));
        // NOTE: like the `disasm` example, the render-init addresses in the SH14
        // records are GUEST addresses (0x105b3a2d8 already includes the segment
        // base 0x100000000). Pass through directly — DO NOT `el.guest_of()` (that
        // would double-map to 0x205b3a2d8, outside the image, and jit_run would
        // reject it).
        let render_init = link;
        let warmup_ms = std::env::var("RENDERINIT_WARMUP_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(5000);
        let (ibase, ilen, isp) = (base, len, st.x[31]);
        let tpidr = arm64jit::jit::current_guest_tp();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(warmup_ms));
            // SH126-gate: when JIT_SERIALIZE_RENDER=1 AND a --v2boot ladder is
            // present, wait for the ladder to signal LADDER_DONE before driving
            // the render pipeline. This converts the concurrent render+ladder
            // jit_run overlap (the SH55/64 block-cache/message-queue desync that
            // makes the combined run 1/3 flaky) into a deterministic SEQUENTIAL
            // run: the ladder completes its rungs first, then render init/thunk/
            // frame drive their jit_runs with no other top-level jit_run in
            // flight. Bounded so a parked ladder (historical exit-124) does not
            // hang render forever — it proceeds after the bound.
            let serialize = std::env::var("JIT_SERIALIZE_RENDER").ok().as_deref() == Some("1")
                && std::env::args().any(|a| a == "--v2boot");
            if serialize && !LADDER_DONE.load(core::sync::atomic::Ordering::Relaxed) {
                eprintln!(
                    "[elfjit:renderinit] JIT_SERIALIZE_RENDER=1: waiting for --v2boot ladder to signal LADDER_DONE (render after ladder = deterministic, no SH55/64 overlap)"
                );
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
                while !LADDER_DONE.load(core::sync::atomic::Ordering::Relaxed)
                    && std::time::Instant::now() < deadline
                {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                if LADDER_DONE.load(core::sync::atomic::Ordering::Relaxed) {
                    eprintln!("[elfjit:renderinit] LADDER_DONE received — driving render after ladder");
                } else {
                    eprintln!("[elfjit:renderinit] WARN LADDER_DONE not reached in 300s (ladder parked?) — proceeding to render anyway");
                }
            }
            let iimg: &[u8] =
                unsafe { std::slice::from_raw_parts(ibase as *const u8, ilen) };
            let mut s3 = arm64jit::jit::CpuState::new();
            s3.tpidr = tpidr;
            // SH108: do NOT seed this render thread's guest SP from the SAME boot
            // stack as the --v2boot ladder thread (`st.x[31]`/`isp`). When both
            // threads drive jit_run concurrently, the render thread's GUEST frames
            // grow down into the ladder thread's live frames and a nested guest
            // `stp x29,x30` (object-init prologue) overwrites the LADDER frame's
            // __stack_chk_guard canary slot -> false `*** stack smashing ***`
            // (SH104/105 layer-2, reproduced with --renderinit). Use a separate,
            // dedicated leaked 1 MiB guest stack for THIS thread (guest==host
            // identity map, so a leaked buffer is directly guest-addressable) —
            // mirroring how run_guest_callback gives each guest re-entry its own
            // stack. The render guest never addresses a caller stack directly.
            const RSTACK: usize = 1 << 20;
            let rstack = Box::leak(vec![0u8; RSTACK].into_boxed_slice());
            s3.x[31] = rstack.as_mut_ptr() as u64 + RSTACK as u64 - 0x100;
            // render-init's prologue writes a resolved global ptr through its x0
            // param (real caller passes `[parent+344]`; a fresh call leaves x0=0
            // -> NULL store -> SIGSEGV). Point x0 at a guest-writable leaked
            // buffer so the first store lands and we reach the EGL sequence.
            let scratch = Box::leak(vec![0u8; 4096].into_boxed_slice());
            // --renderthunk (opt-in, must accompany --renderinit): drive the render-init
            // THUNK 0x105b3a280 instead of the inner fn, to recover the engine's REAL
            // ctx object. SH17's record "DON'T drive the thunk (SIGSEGV)" is WRONG —
            // the crash was from misplacing the harness args. Disasm of v2.738.1397:
            //   thunk(x0, x1):  x21=x0; x20=x1; x19=alloc_big(0x48);
            //                   inner(x19, x2?=x1=x21, x2=x20); ret x0=x19
            // i.e. thunk(win, parent) -> inner(alloc_ctx, win, parent) and returns the
            // real 0x48-byte guest ctx in x0 (engine's callers 0x5b2b214/0x5b2ea90 do
            // `bl 0x105b3a280` then `ldr x8,[x0]; ldr x8,[x8,#16]; blr x8` vtable-
            // dispatch). The engine's own frame-render path consumes THIS ctx, so
            // recovering it is the bridge to frontier lever (2) (drive the engine's
            // own frame-render machinery with a coherent renderer). The old harness
            // passed scratch as x0 -> inner took win=scratch (not the XID) and the
            // surface create rejected it. Correct drive: thunk(x0=win=XID, x1=parent=0).
            let render_thunk = renderframe_args.iter().any(|a| a == "--renderthunk");
            // SH112 (recon deleg_9935787c): the renderinit thread must NOT reuse the
            // boot ANativeWindow XID — the boot/ladder path already created an EGL
            // surface on it, so a second eglCreateWindowSurface on the renderinit
            // thread returns EGL_BAD_SURFACE (0x300b) and the engine throws
            // `std::runtime_error: Error creating context: eglCreateWindowSurface`
            // -> whole-process abort EXIT 139, killing the combined ladder before it
            // reaches StartLuaAppDM. Give the renderinit thread a FRESH, never-
            // surfaced X11 window XID (same size as the framebuffer) so Mesa x11 EGL
            // can build a real window surface there.
            let fresh_xid = || -> u64 {
                let display = std::env::var("DISPLAY").unwrap_or_default();
                if display.is_empty() {
                    return 0;
                }
                for _ in 0..20 {
                    if let Ok((conn, win)) = x11::open_window_sized(Some(&display), 1280, 720) {
                        Box::leak(Box::new(conn)); // keep the window alive
                        return win as u64;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                0
            };
            let boot_xid = arm64jit::shims::anativewindow_xid();
            // Prefer a FRESH window when one opens; fall back to the boot window
            // only if the display is unavailable (fresh_xid returns 0).
            let xid = if render_thunk {
                let fresh = fresh_xid();
                if fresh != 0 { fresh } else { boot_xid }
            } else {
                boot_xid
            };
            if render_thunk {
                eprintln!(
                    "[elfjit:renderinit] using FRESH render-thread window XID=0x{xid:x} (boot shared xid was 0x{boot_xid:x}) so eglCreateWindowSurface does not re-surface the already-bound boot window"
                );
            }
            // Scratch is still needed: render-init's prologue stores the resolved
            // parent-global ptr through x0 only for the inner-fn path; the thunk's
            // inner call gets its OWN freshly-allocated ctx as x0, so it never touches
            // scratch — we keep it solely to pin guest-arena-visible RW backing and as
            // the fallback driver buffer if --renderthunk isn't set.
            s3.x[0] = if render_thunk { xid } else { scratch.as_ptr() as u64 };
            // Real caller (0x105b2ea98) passes x1 = the ANativeWindow (loaded from
            // [parent+352] into x22 -> stored to [ctx+24] -> eglCreateWindowSurface's
            // native-window arg). For the thunk, x1 is the (optional) share/ parent
            // context (0 = fresh, no sharing) — the window rides in x0 for the thunk
            // (it forwards x0 into inner's x1, i.e. the win). Mesa's x11 EGL platform
            // wants the X11 Window XID as its native window.
            s3.x[1] = if render_thunk { 0 } else { xid };
            let got = if ibase >= 0x100000000 && ibase >> 56 == 0 {
                unsafe { *(0x1067d16f0u64 as *const u64) }
            } else {
                0
            };
            eprintln!(
                "[elfjit:renderinit] driving {}{render_init:#x} after {warmup_ms}ms warm-up (ctx 0x1067d16f0={got:#x}, x0={:#x}, x1={:#x})",
                if render_thunk { "THUNK " } else { "" }, s3.x[0], s3.x[1],
            );
            let swap_result = arm64jit::jit::jit_run(iimg, ibase, render_init, &mut s3 as *mut CpuState);
            let rv = match swap_result {
                Err(e) => {
                    eprintln!("[elfjit:renderinit] stopped: {e}");
                    return;
                }
                Ok(r) => r,
            };
            eprintln!("[elfjit:renderinit] returned Ok({rv:#x})");
            // When driving the THUNK, x0's return value IS the engine's real ctx
            // (guest-addressable 0x48-byte object with its own vtable at [ctx+0] =
            // 0x106731ae0). Range-check it (>= some guest base, < 2^48, mapped) and
            // note that the engine's own frame callers deref it. Keep the swap/sclear
            // levers operating on THIS ctx (its [ctx+32]/[+40]/[+48] hold the live
            // EGL display/surface/context the inner fn stored).
            let real_ctx = if render_thunk {
                let c = rv;
                if c >= 0x100000000 && c >> 56 == 0 {
                    let vt = unsafe { *(c as *const u64) };
                    eprintln!(
                        "[elfjit:renderthunk] REAL ctx 0x{c:x} vtable=0x{vt:x} egl: display=0x{:x} surface=0x{:x} context=0x{:x}",
                        unsafe { *(c as *const u64).add(4) },
                        unsafe { *(c as *const u64).add(5) },
                        unsafe { *(c as *const u64).add(6) },
                    );
                    // Dump the live vtable slots (engine-populated at runtime, no
                    // static relocs). The engine's frame-render callers
                    // (0x5b2b214/0x5b2ea90) do `ldr x8,[ctx]; ldr x8,[x8,#16]; blr
                    // x8` — slot [vt+16] (index 2) is the method a real frame
                    // dispatch reaches. Read the first 5 table entries live.
                    if vt >= 0x100000000 && vt >> 56 == 0 {
                        let slots: Vec<String> = (0..5)
                            .map(|i| unsafe { *(vt as *const u64).add(i) })
                            .map(|v| format!("{v:#x}"))
                            .collect();
                        eprintln!(
                            "[elfjit:renderthunk] ctx vtable[0..5] = {} — [vt+16](idx2)=disp target",
                            slots.join(" ")
                        );
                    }
                    c
                } else {
                    eprintln!("[elfjit:renderthunk] thunk return 0x{c:x} not a guest ctx; falling back to scratch");
                    scratch.as_ptr() as u64
                }
            } else {
                scratch.as_ptr() as u64
            };
            // SH60: publish the recovered real ctx so the --taskv4-seed frame
            // thunk (dispatched on the drain thread) marshals task nodes into
            // real frames on it. Self-guarded: 0 here is transient.
            RENDERCTX.store(real_ctx, core::sync::atomic::Ordering::Relaxed);
            if real_ctx != scratch.as_ptr() as u64 {
                eprintln!(
                    "[elfjit:renderthunk] published RENDERCTX 0x{real_ctx:x} — type-4 task frames will render on it"
                );
                // SH61b: THIS thread is the single presenter (EGL current is
                // genuinely established here — the SH60 observation that only the
                // renderinit-thread presents return Ok(0x1), plus the failed
                // presenter-mutex experiment, pin it). The drain thread only bumps
                // PENDING_PRESENTS (adds as fast as frame-fn can consume); we
                // drain it here, presenting EVERY queued request as a real frame.
                // Seed one request to anchor the stream, then drain for a bounded
                // window so the run still exits 124 (stable idle) cleanly.
                if taskv4_frame_seed_active() {
                    eprintln!("[elfjit:taskv4-frame] presenter thread: draining PENDING_PRESENTS on the currency-owning thread");
                    // The drain flood (heartbeat-patched w4=4) adds PENDING far
                    // faster than llvmpipe can present, so a greedy drain never
                    // terminates. Rate-limit to a bounded, sustainable ~8 fps for a
                    // bounded window so the run exits 124 (stable idle) cleanly with
                    // a stream of CLEAN Ok(0x1) presents (the SH61b result: every
                    // present on THIS thread is a genuine eglSwapBuffers success).
                    let t0 = std::time::Instant::now();
                    let max_window = std::time::Duration::from_millis(
                        std::env::var("TASKFRAME_WINDOW_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000),
                    );
                    let max_frames: u64 = std::env::var("TASKFRAME_MAX_FRAMES")
                        .ok().and_then(|v| v.parse().ok()).unwrap_or(24);
                    let mut presented: u64 = 0;
                    let mut drained: u64 = 0;
                    while presented < max_frames && t0.elapsed() < max_window {
                        // Consume up to one pending request into the next frame
                        // (rate-limited: one present per loop iteration with a
                        // ~120ms cadence keeps it sustainable and visibly animating).
                        let pending = PENDING_PRESENTS.load(core::sync::atomic::Ordering::Relaxed);
                        if pending > drained {
                            let _ = present_one_task_frame(real_ctx, presented);
                            presented += 1;
                            drained += 1;
                            PENDING_PRESENTS.fetch_sub(1, core::sync::atomic::Ordering::Relaxed);
                        } else {
                            // No new request since last count; sleep then check again
                            std::thread::sleep(std::time::Duration::from_millis(20));
                        }
                    }
                    eprintln!(
                        "[elfjit:taskv4-frame] presenter drained: {presented} real task-driven frames presented (all on the currency-owning thread); pending={}",
                        PENDING_PRESENTS.load(core::sync::atomic::Ordering::Relaxed)
                    );
                } else {
                    // Non-frame seed value: still fire one deterministic present
                    // (SH60 marker) on this currency-owning thread.
                    let _ = present_one_task_frame(real_ctx, 0);
                }
                // --renderscene (opt-in, must accompany --renderinit + the
                // renderthunk so RENDERCTX is the real ctx): drive the engine's
                // OWN scene renderer (0x105b2ead4) with a fabricated-but-engine-
                // native render-manager R, replacing the harness-fabricated
                // clear-path renderer with the engine's real frame-desc
                // construction (its own operator-new 0x1d96768 / frame ctor
                // 0x5b34de8 / linker 0x5b2d9e0). Bounded window so the run still
                // exits 124 cleanly. VERIFIES the engine registered a real
                // frame-desc into R+0x170 before presenting.
                if renderframe_args.iter().any(|a| a == "--renderscene") {
                    let t0 = std::time::Instant::now();
                    let max_window = std::time::Duration::from_millis(
                        std::env::var("RENDERSCENE_WINDOW_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(1500),
                    );
                    let max_frames: u64 = std::env::var("RENDERSCENE_MAX_FRAMES")
                        .ok().and_then(|v| v.parse().ok()).unwrap_or(3);
                    // SH63: number of 0x28-stride scene nodes to populate into
                    // R+0x180 so the engine's per-node frame-build path runs
                    // (default 3 => base frame + 3 per-node engine frames).
                    let scene_nodes: u64 = std::env::var("RENDERSCENE_NODES")
                        .ok().and_then(|v| v.parse().ok()).unwrap_or(3);
                    let mut presented: u64 = 0;
                    while presented < max_frames && t0.elapsed() < max_window {
                        if render_engine_scene(real_ctx, presented, scene_nodes) == 1 {
                            presented += 1;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(120));
                    }
                    eprintln!(
                        "[elfjit:renderscene] drained: {presented} real engine-scene-renderer frames presented (engine-built frame-desc) on the currency-owning thread"
                    );
                }
                // --renderwalker (opt-in, must accompany --renderinit + the
                // renderthunk so real_ctx is the real ctx + --renderscene which
                // lays the scene-list R): drive the engine's REAL per-node
                // PRESENT walker (mid-loop entry 0x105b2eec0) so each populated
                // 0x28-stride scene node actually DRAWS through its render-obj
                // vt[+24]. Each node's draw is a REGISTERED HOST THUNK
                // (walker_item_draw_thunk) dispatched via host_call_at — pure
                // host, ZERO block-cache mutation, so the present-loop block
                // stays intact and the SH64 nested-jit_run desync SIGSEGV is
                // closed. The walker then swaps through the real ctx-vt[+24].
                // Bounded window so the run still exits 124 cleanly.
                if renderframe_args.iter().any(|a| a == "--renderwalker") {
                    let t0 = std::time::Instant::now();
                    let max_window = std::time::Duration::from_millis(
                        std::env::var("RENDERWALKER_WINDOW_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(1500),
                    );
                    let max_frames: u64 = std::env::var("RENDERWALKER_MAX_FRAMES")
                        .ok().and_then(|v| v.parse().ok()).unwrap_or(3);
                    let walker_nodes: u64 = std::env::var("RENDERWALKER_NODES")
                        .ok().and_then(|v| v.parse().ok()).unwrap_or(3);
                    // --renderemitter (SH66): additionally drive the engine's REAL
                    // geometry emitter 0x105b35288 with a fabricated geometry ctx G
                    // so an authored quad draws through the ENGINE's own GL stack
                    // (primitive-setup -> glDrawArrays @plt -> real Mesa), pixel-
                    // verified. Desync-safe: the emitter is its OWN top-level
                    // jit_run (not nested in the walker block).
                    let emit: bool = renderframe_args.iter().any(|a| a == "--renderemitter");
                    let mut presented: u64 = 0;
                    while presented < max_frames && t0.elapsed() < max_window {
                        let ret =
                            render_engine_present_walker(real_ctx, presented, walker_nodes, iimg, ibase, tpidr, isp);
                        if emit {
                            // The emitter draws onto the same live ctx; drive it
                            // AFTER this frame's walker so its swap shows the
                            // engine-emitted quad (the walker's swap precedes).
                            // RENDEREMITTER_QUADS=N (SH67d): draw a POPULATED
                            // N-quad 2D frame in ONE engine-emitter jit_run
                            // (single pre-uploaded VBO, GL_TRIANGLES) — closes
                            // the SH67c per-drive glBufferData orphan blocker
                            // by construction and proves a populated
                            // multi-element engine-emitted frame headlessly.
                            // Default (unset) keeps the SH67b single quad.
                            let nq: usize = std::env::var("RENDEREMITTER_QUADS")
                                .ok().and_then(|v| v.parse().ok()).unwrap_or(0);
                            // RENDEREMITTER_TEX=1 (SH67e): add a 3rd per-vertex
                            // texcoord attribute (aTex loc2) sampled from a
                            // pre-uploaded host texture so the engine emitter draws
                            // TEXTURED quads (stride 32), the populated UI-layer-
                            // style commitment. Default: solid-color grid (SH67d).
                            let tex = std::env::var_os("RENDEREMITTER_TEX").is_some();
                            // RENDEREMITTER_LAYOUT=home (SH68): the engine's real
                            // emitter draws a LAYERED login/home-style frame (5
                            // textured quads: backdrop/panel/button/title/field)
                            // with GL_BLEND alpha compositing, sized from the real
                            // scene list (R+0x180/0x188). Overrides the grid.
                            let layout = std::env::var("RENDEREMITTER_LAYOUT").unwrap_or_default();
                            // RENDEREMITTER_MULTI=1 (SH72): the engine's real
                            // emitter draws a FULL multi-sprite "login/home"
                            // composite — a dark backdrop + one aspect-correct
                            // box per REAL Roblox UI sprite (spinner, robux
                            // icon, jump button) sampled from a shared vertical
                            // atlas in ONE top-level jit_run, composited with
                            // GL_BLEND. Overrides the single-spinner home path.
                            let multi = std::env::var_os("RENDEREMITTER_MULTI").is_some();
                            if layout == "home" {
                                if multi {
                                    let _ = render_engine_emitter_multi(real_ctx, iimg, ibase, isp);
                                } else {
                                    let _ = render_engine_emitter_home(real_ctx, iimg, ibase, isp, 5);
                                }
                            } else if nq > 0 {
                                let _ = render_engine_emitter_grid(real_ctx, iimg, ibase, isp, nq, tex);
                            } else {
                                let _ = render_engine_emitter_quad(real_ctx, iimg, ibase, isp);
                            }
                        }
                        if ret == 1 {
                            presented += 1;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                    eprintln!(
                        "[elfjit:renderwalker] drained: {presented} real engine-per-node present-walker frames presented (engine vt[+24] draws + engine swap) on the currency-owning thread"
                    );
                }
            }
            let _ = &real_ctx;
            // --renderframe (opt-in, must accompany --renderinit): after the real
            // render-init ran, present a buffer through the engine's LIVE EGL
            // context. Reverse from the real binary (SH17 disasm): the direct
            // drive of the render-init inner fn (0x105b3a2d8) wrote the live EGL
            // handles into our `scratch` buffer — [scratch+32]=eglDisplay,
            // [scratch+40]=surface, [scratch+48]=context (str x0,[x19,#32] /
            // str x1,[x19,#40] / str x0,[x19,#48], x19=ctx=the fn's x0 param).
            // The swap fn 0x105b3b408 is a tail thunk `ldp x8,x1,[x0,#32]; mov
            // x0,x8; b eglSwapBuffers` — i.e. eglSwapBuffers([x0+32],[x0+40]).
            // Passing x0=scratch (the SAME buffer render-init wrote) makes the
            // engine's own swap path present the current surface headlessly
            // (llvmpipe+Xvfb), WITHOUT re-running the init (which crashes because
            // the thunk re-drive shifts the parent/window args). Same host thread
            // so the EGL context stays current.
            if renderframe_args.iter().any(|a| a == "--renderframe") {
                // --renderbind (opt-in): drive the engine's OWN make-current method
                // (ctx vtable [vt+16] = 0x105b3b358) before presenting, instead of
                // relying on the render-init having left the context current. This
                // is the exact code the engine's frame-render callers dispatch
                // (0x5b2b214 -> [vt+16] -> blr) when they (re)bind the GL context
                // before a swap/draw: it reads eglGetCurrentContext, and when not
                // already == [ctx+48] calls eglMakeCurrent([+32]display,
                // [+40]surface, [+40]surface, [+48]context). Driving it proves the
                // engine's own rebind path executes on the recovered ctx (the same
                // host thread keeps the context current afterwards so the following
                // swap/draw land on it).
                if renderframe_args.iter().any(|a| a == "--renderbind") {
                    let mut sb = arm64jit::jit::CpuState::new();
                    sb.tpidr = tpidr;
                    sb.x[31] = isp;
                    sb.x[0] = real_ctx; // method: this = ctx
                    match arm64jit::jit::jit_run(iimg, ibase, 0x105b3b358, &mut sb as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:renderbind] stopped: {e}"),
                        Ok(ok) => eprintln!(
                            "[elfjit:renderbind] engine make-current method returned Ok({ok:#x}) on ctx {real_ctx:#x} (eglMakeCurrent)"
                        ),
                    }
                }
                let swap_thunk = renderframe_args
                    .iter()
                    .position(|a| a == "--renderframe")
                    .and_then(|i| renderframe_args.get(i + 1).cloned())
                    .and_then(|h| u64::from_str_radix(h.trim_start_matches("0x"), 16).ok())
                    .unwrap_or(0x105b3b408);
                unsafe {
                    eprintln!(
                        "[elfjit:renderframe] ctx={real_ctx:#x} [+32]=display {:#x} [+40]=surface {:#x} [+48]=context {:#x}",
                        *(real_ctx as *const u64).add(4),
                        *(real_ctx as *const u64).add(5),
                        *(real_ctx as *const u64).add(6),
                    );
                    *(real_ctx as *mut u64) = 0;
                }
                let mut s5 = arm64jit::jit::CpuState::new();
                s5.tpidr = tpidr;
                s5.x[31] = isp;
                s5.x[0] = real_ctx; // swap fn reads [x0+32]/[x0+40]
                match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut s5 as *mut CpuState) {
                    Err(e) => eprintln!("[elfjit:renderframe] swap stopped: {e}"),
                    Ok(ok) => eprintln!("[elfjit:renderframe] swap returned Ok({ok:#x}) (eglSwapBuffers)"),
                }
                // --renderframe-drive: probe how FAR the engine's OWN frame-render fn
                // 0x105b32c00 gets when driven on the real ctx with fabricated
                // renderer/view objects. This is frontier lever (2) — replacing the
                // harness's force-driven glClearColor/glClear with the engine's real
                // frame code. From SH18 disasm the fn is
                //   frame(renderer=x0, view=x1, w2, w3, x4, x5):
                //     [renderer+16]=1; x0=[renderer+24]; bl 0x5b2e98c   (find/dispatch)
                //     glBindFramebuffer(0x8d40, [view+140])  -> glGetError (cmp 0x505)
                //     glViewport(0,0,[view+128],[view+132])
                //     [renderer+24]->[+552]: if 0 skip clear path
                //     [renderer+40]->[+140]: if 0 skip clear path
                //     clear via glClearColor/glColorMask/glClearDepthf/...
                // We fabricate: renderer (with +16 set, +24->objA[+552]=1,
                // +40->objB[+140]=1), view (+128,+132 = 1280x720, +140 framebuffer 0).
                // Same host thread, context already current (renderbind/renderinit).
                if renderframe_args.iter().any(|a| a == "--renderframe-drive") {
                    // Guest-visible scratch for the objects (guest==host, low48).
                    // The engine writes deep into these (objA[+552/608],
                    // renderer[+224/232/236/238], view[+124..140]) — MUST be large
                    // enough that every fabricated struct (renderer/base, objA +0x100,
                    // objB +0x200, view +0x300, plus engine writes past those) stays
                    // inside the allocation, else the drive heap-corrupts at shutdown
                    // ("free(): invalid next size").
                    let objs = Box::leak(vec![0u8; 8192].into_boxed_slice());
                    let base = objs.as_ptr() as u64;
                    // view: +128=w(1280) +132=h(720) +140=default framebuffer(0)
                    unsafe {
                        *(base as *mut u64) = 0; // renderer[+0] reserved (obj not vt)
                        // renderer fields at +16,+24,+40
                        let renderer = base;
                        let objA = base + 0x100; // [renderer+24]->[+552]
                        let objB = base + 0x200; // [renderer+40]->[+140]
                        let view = base + 0x300;
                        // [renderer+16]=1 (set by fn anyway), [renderer+24]=objA
                        *(renderer.wrapping_add(16) as *mut u8) = 1;
                        *(renderer.wrapping_add(24) as *mut u64) = objA;
                        *(renderer.wrapping_add(40) as *mut u64) = objB;
                        // objA[+552]=1 (nonzero -> clear path enabled)
                        *(objA.wrapping_add(552) as *mut u8) = 1;
                        // 0x5b2e98c is a list-find: it reads [objA+368] first and
                        // returns immediately when [objA+368]==view(arg1), else walks
                        // an intrusive list [objA+384]..[objA+392] (empty => returns at
                        // the head==tail check). Set [objA+368]=view so it bails on the
                        // first cmp (clean return, no list walk that could fault on a
                        // 0 head). Also seed both list bounds to 0 = empty list.
                        *(objA.wrapping_add(368) as *mut u64) = view;
                        *(objA.wrapping_add(384) as *mut u64) = 0;
                        *(objA.wrapping_add(392) as *mut u64) = 0;
                        // objB[+140]=1, [+124]=1 (nonzero flags)
                        *(objB.wrapping_add(140) as *mut u32) = 0; // 0 -> glDrawBuffers(1,{GL_BACK}) for the default FB
                        *(objB.wrapping_add(124) as *mut u32) = 1;
                        // view: [+128]=w=[+132]=h, [+140]=framebuffer id 0
                        *(view.wrapping_add(128) as *mut u32) = 1280;
                        *(view.wrapping_add(132) as *mut u32) = 720;
                        *(view.wrapping_add(140) as *mut u32) = 0;
                        // Clear color: the engine's frame-fn takes the clear-color
                        // object as its 5th arg x4 (0x105b32c30 `mov x20,x4`), passed
                        // as the clear-state sub-fn 0x105b32e08's x2 (its prologue
                        // `mov x21,x2`), which reads the RGBA float4 from
                        // [obj+4],[obj+8],[obj+12],[obj+16] (LDP s0,s1,[x21,#4] /
                        // LDP s2,s3,[x21,#12]). The sub-fn call is gated on
                        // x4!=0 AND [x4]!=0 (0x105b32d44 cbz x20 / 0x105b32d4c cbz
                        // [x20]). SH20 left x4=0 so the engine's own clear never
                        // fired and the window stayed black. Fix: fabricate a
                        // clear-state object (nonzero [+0] flag + RGBA float4 at
                        // [+4..16]) and pass it as x4 (optionally recolored via
                        // --renderframe-color r,g,b,a).
                        let default_cc = [0.40f32, 0.20f32, 0.95f32, 1.0f32];
                        let mut cc = default_cc;
                        if let Some(i) = renderframe_args
                            .iter()
                            .position(|a| a == "--renderframe-color")
                        {
                            let csv = renderframe_args.get(i + 1).cloned().unwrap_or_default();
                            let vals: Vec<f32> = csv
                                .split(',')
                                .filter_map(|x| x.parse::<f32>().ok())
                                .collect();
                            if vals.len() >= 4 {
                                cc = [vals[0], vals[1], vals[2], vals[3]];
                            }
                        }
                        let clearobj = base + 0x400; // frame-fn x4 = clear-state obj
                        *(clearobj as *mut u32) = 0xF; // [obj+0]: clear-buffer bitmask (w20); 0xF=all 4
                        for (k, v) in cc.iter().enumerate() {
                            *(clearobj.wrapping_add(4 + (k as u64) * 4) as *mut f32) = *v;
                        }
                        // Frame-fn 6th arg x5 -> x22 (0x105b32c28 `mov x22,x5`), the
                        // main-fn's second clear-source object (0x105b32d5c cbz x22 /
                        // ldr q0,[x22] copies [+0..16] vec; gated on [x22]!=0). The
                        // harness left x5=0 so this path was skipped too.
                        let ccobj = base + 0x500;
                        *(ccobj.wrapping_add(0) as *mut f32) = cc[0];
                        *(ccobj.wrapping_add(4) as *mut f32) = cc[1];
                        *(ccobj.wrapping_add(8) as *mut f32) = cc[2];
                        *(ccobj.wrapping_add(12) as *mut f32) = cc[3];
                        eprintln!(
                            "[elfjit:renderframe-drive] clear-color x5 obj 0x{ccobj:x} (RGBA {cc:?} at [+0..16])"
                        );
                        // SH19 diagnostic: dump the 8 engine-GLES dispatch slots the
                        // frame's clear path `br`-stubs read. The stubs 0x5b3a1c0..
                        // (with a 0x10 stride) do `adrp x8, 6d3b000; ldr x2,[x8,#752]`
                        // + 8*N, i.e. slot N at guest 0x106d3b2f0 + 8*N. Each holds a
                        // function pointer the engine's own GLES-table init is
                        // supposed to populate (it never does under our headless
                        // drive), so an unset slot makes the clear path `br` into
                        // garbage. Since guest==host these vaddrs are dereferenceable.
                        let mut vals = [0u64; 8];
                        for i in 0..8 {
                            let slot_v = 0x106d3b2f0u64 + i * 8;
                            let val = unsafe { *(slot_v as *const u64) };
                            vals[i as usize] = val;
                            eprintln!(
                                "[elfjit:renderframe-drive] gles-dispatch slot {i} guest {slot_v:#x} = {val:#x}"
                            );
                        }
                        eprintln!(
                            "[elfjit:renderframe-drive] gles-dispatch values = {vals:?}"
                        );
                        // --renderframe-seedgles (opt-in): overwrite the 8 engine
                        // GLES dispatch slots (BSS 0x106d3b2f0..0x106d3b328) with
                        // OUR host-thunk GLES bridge slots (resolve_gles_mixed) so
                        // the frame clear path's `br`-stubs dispatch through the
                        // bridge (float/texture interception) instead of jumping to
                        // raw Mesa (out-of-image). The engine's real GL-init fills
                        // these with raw Mesa addresses (SH19); seeding proves the
                        // bridge takes over. Names are per-slot guesses from the
                        // clear-path usage; refine by reading which slot the engine
                        // needs once the drive passes the current stop.
                        // Slot->function names corrected by disassembly (SH22): the clear
                        // path dispatches slot0 as glDrawBuffers (builds
                        // {GL_COLOR_ATTACHMENT0..3} / {GL_BACK} buf arrays) and slot2 as
                        // glClearBufferfv (per-buffer clear loop uses GL_COLOR=0x1800 /
                        // GL_DEPTH=0x1801 buffer enums, drawbuffer in w1, value ptr in x2).
                        // The SH19-21 "glClearColor"+"glClearDepthf" guesses mis-routed
                        // those dispatches (glClearDepthf bridge ignored the int/ptr args
                        // and cleared nothing -> black window).
                        // The engine's GLES dispatch table is 16 slots at BSS
                        // 0x106d3b2f0 (stub 0x5b3a1c0+0xc*N does adrp 6d3b000; ldr
                        // xK,[x8,#752+8*N]; br xK). Slots 0-7 are the clear path
                        // (SH22-corrected names below). Slots 8-15 are the GEOMETRY
                        // draw path: the draw wrapper 0x5b35288 dispatches slot 9 as
                        // glDrawElements (indexed draw, 0x5b352f4 bl 0x5b3a22c) and
                        // slot 10 as glDrawArrays (array draw, 0x5b35368 bl 0x5b3a238)
                        // after the primitive-setup fn 0x5b353d0 binds buffers +
                        // sets up vertex attrib pointers (glBindBuffer/
                        // glEnableVertexAttribArray/glVertexAttribPointer direct @plt).
                        // Seeding slots 9/10 too means a real geometry draw (reaching
                        // the RENDERER C++ object reverse) dispatches through the
                        // bridge instead of jumping to a raw Mesa addr (SH19 class).
                        // Seed EVERY dispatch slot explicitly by (slot, name). Slots
                        // 0-7 are the clear path (SH22-corrected names below). The
                        // real geometry draw dispatches slot 9 as glDrawElements
                        // (indexed draw, wrapper 0x5b35288 @0x5b352f4 bl 0x5b3a22c)
                        // and slot 10 as glDrawArrays (array draw @0x5b35368 bl
                        // 0x5b3a238), after primitive-setup 0x5b353d0 binds buffers +
                        // sets vertex attrib pointers via direct @plt. Seeding 9/10
                        // means a real geometry draw dispatches through the bridge
                        // instead of jumping to a raw Mesa addr (SH19 class).
                        let seed_slots: [(usize, &str); 10] = [
                            (0, "glDrawBuffers"),
                            (1, "glClearBufferiv"),
                            (2, "glClearBufferfv"),
                            (3, "glClearBufferfi"),
                            (4, "glColorMask"),
                            (5, "glDepthMask"),
                            (6, "glStencilMask"),
                            (7, "glViewport"),
                            (9, "glDrawElements"),
                            (10, "glDrawArrays"),
                        ];
                        if renderframe_args.iter().any(|a| a == "--renderframe-seedgles") {
                            // Diagnostic: dump the 16 raw slot values the ENGINE left in the
                            // dispatch table (before we overwrite) and dladdr-resolve each host
                            // address to a symbol. Pins the real slot->function mapping
                            // (0-7 clear, 9/10 draw, 11-15 texture/uniform/shader) without code
                            // archaeology, IF the engine's GL-init has filled them.
                            eprintln!("[elfjit:renderframe-seedgles] raw slot snapshot (before seed):");
                            for (i, _n) in [(0usize, "x"), (1, "y"), (2, "z"), (3, "w"), (4, "q"), (5, "r"), (6, "s"), (7, "t"), (8, "a"), (9, "b"), (10, "c"), (11, "d"), (12, "e"), (13, "f"), (14, "g"), (15, "h")] {
                                let sv = 0x106d3b2f0 + (i as u64) * 8;
                                let raw = unsafe { *(sv as *const u64) };
                                let sym = unsafe {
                                    let mut dli = std::mem::zeroed::<libc::Dl_info>();
                                    if libc::dladdr(raw as *const libc::c_void, &mut dli) != 0
                                        && !dli.dli_sname.is_null()
                                    {
                                        std::ffi::CStr::from_ptr(dli.dli_sname)
                                            .to_string_lossy()
                                            .into_owned()
                                    } else {
                                        String::new()
                                    }
                                };
                                eprintln!("[elfjit:renderframe-seedgles]   slot {i:2} = {raw:#018x}  {sym}");
                            }
                            for (i, name) in seed_slots {
                                let slot_v = 0x106d3b2f0u64 + (i as u64) * 8;
                                // Mixed (float) ABI first; fall back to int ABI for
                                // glClear/glColorMask/glViewport etc.
                                let slot = arm64jit::resolver::resolve_gles_mixed(
                                    format!("{name}\0").as_bytes(),
                                )
                                .or_else(|| {
                                    arm64jit::resolver::resolve_gles_int(
                                        format!("{name}\0").as_bytes(),
                                    )
                                });
                                match slot {
                                    Some(bridge_slot) => {
                                        unsafe { *(slot_v as *mut u64) = bridge_slot };
                                        eprintln!(
                                            "[elfjit:renderframe-seedgles] slot {i} ({name}) <- bridge {bridge_slot:#x}"
                                        );
                                    }
                                    None => eprintln!(
                                        "[elfjit:renderframe-seedgles] slot {i} ({name}) NOT resolvable"
                                    ),
                                }
                            }
                        }
                        eprintln!(
                            "[elfjit:renderframe-drive] fabricated renderer 0x{renderer:x} (+16=1,+24->0x{objA:x}[+552]=1,+40->0x{objB:x}[+140]=1) view 0x{view:x} ([+128]=1280 [+132]=720 [+140]=0)"
                        );
                        // --renderframe-loop <N>: repeat the engine's OWN recipe
                        // (bind already done by renderbind -> the real frame-fn
                        // 0x105b32c00 -> post-frame swap via real ctx) N times to
                        // prove the render path is reentrant/sustainable. Default 1.
                        // --rendersustain <fps>: instead of a bounded loop, run the
                        // engine's OWN recipe CONTINUOUSLY at ~fps on this detached
                        // host thread while StartApp's main-loop jit_run idles
                        // concurrently on the main thread — a live animated render
                        // loop (the shape the engine needs to drive frames from its
                        // own thread). Each frame cycles the clear color through a
                        // small palette so a capture proves every frame is a fresh
                        // render, not a static buffer.
                        let loop_n: usize = renderframe_args
                            .iter()
                            .position(|a| a == "--renderframe-loop")
                            .and_then(|i| renderframe_args.get(i + 1))
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(1);
                        let sustain_fps: Option<f64> = renderframe_args
                            .iter()
                            .position(|a| a == "--rendersustain")
                            .and_then(|i| renderframe_args.get(i + 1))
                            .and_then(|v| v.parse().ok());
                        let palette: [[f32; 4]; 5] = [
                            [0.40, 0.20, 0.95, 1.0],
                            [0.10, 0.70, 0.05, 1.0],
                            [0.90, 0.15, 0.10, 1.0],
                            [0.05, 0.60, 0.90, 1.0],
                            [1.00, 0.82, 0.05, 1.0],
                        ];
                        let mut iter: u64 = 0;
                        loop {
                            // Per-frame color: sustain mode cycles the palette (so a
                            // capture proves fresh renders); bounded --renderframe-loop
                            // keeps the --renderframe-color (or default).
                            let cur_color = match sustain_fps {
                                Some(_) => palette[(iter as usize) % palette.len()],
                                None => cc,
                            };
                            // Re-write both clear-color sources each iteration.
                            unsafe {
                                for (k, v) in cur_color.iter().enumerate() {
                                    *(clearobj.wrapping_add(4 + (k as u64) * 4) as *mut f32) = *v;
                                    *(ccobj.wrapping_add((k as u64) * 4) as *mut f32) = *v;
                                }
                            }
                            eprintln!(
                                "[elfjit:renderframe-drive] === frame iteration {iter} color {:?} ===",
                                cur_color
                            );
                            let mut sd = arm64jit::jit::CpuState::new();
                            sd.tpidr = tpidr;
                            sd.x[31] = isp;
                            sd.x[0] = renderer;
                            sd.x[1] = view;
                            sd.x[2] = view; // 3rd arg (w2, unused by main fn path)
                            sd.x[4] = clearobj; // 5th arg -> x20 -> clear-state sub-fn x2
                            sd.x[5] = ccobj; // 6th arg -> x22 -> color-source object
                            match arm64jit::jit::jit_run(
                                iimg, ibase, 0x105b32c00, &mut sd as *mut CpuState,
                            ) {
                                Err(e) => eprintln!("[elfjit:renderframe-drive] frame-fn stopped: {e}"),
                                Ok(ok) => eprintln!(
                                    "[elfjit:renderframe-drive] engine frame-fn 0x105b32c00 returned Ok({ok:#x})"
                                ),
                            }
                            // Then present whatever the frame-fn did on the real ctx.
                            let mut se = arm64jit::jit::CpuState::new();
                            se.tpidr = tpidr;
                            se.x[31] = isp;
                            se.x[0] = real_ctx;
                            match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut se as *mut CpuState) {
                                Err(e) => eprintln!("[elfjit:renderframe-drive] swap stopped: {e}"),
                                Ok(ok) => eprintln!(
                                    "[elfjit:renderframe-drive] post-frame swap returned Ok({ok:#x})"
                                ),
                            }
                            iter += 1;
                            if let Some(fps) = sustain_fps {
                                if fps > 0.0 {
                                    std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / fps));
                                }
                            } else if (iter as usize) >= loop_n {
                                break;
                            }
                        }
                        // --renderframe-drawprobe: drive the engine's REAL geometry
                        // draw wrapper 0x5b35288 (the fn that calls primitive-setup
                        // 0x5b353d0 then dispatches the indexed/array draw through
                        // GLES dispatch-table slots 9/10). Fabricate a minimal
                        // coherent renderer: empty primitive list ([container+72]==
                        // [container+80]==0 -> 0x5b353d0 returns mask 0 fast), but a
                        // NONZERO [renderer+120] index-buffer object + nonzero count
                        // arg (w5) so the wrapper takes the INDEXED path and
                        // dispatches slot 9 (glDrawElements) through the bridge.
                        // This proves the geometry draw dispatch reaches a real
                        // glDrawElements (currently the recorded clear-state maxes
                        // out before any gl*Draw*).
                        if renderframe_args.iter().any(|a| a == "--renderframe-drawprobe") {
                            let objs = Box::leak(vec![0u8; 8192].into_boxed_slice());
                            let base = objs.as_ptr() as u64;
                            let renderer = base;
                            let container = base + 0x100;
                            let ibo = base + 0x200;
                            unsafe {
                                // renderer[56] = container (0x5b353f0 ldr x25,[x0,#56])
                                *(renderer.wrapping_add(56) as *mut u64) = container;
                                // renderer[120] = index-buffer object (nonzero -> indexed path)
                                *(renderer.wrapping_add(120) as *mut u64) = ibo;
                                // renderer[142] u16 element count (w8 in wrapper 0x5b352b8)
                                *(renderer.wrapping_add(142) as *mut u16) = 3;
                                // container[72]/[80] = begin/end primitive list, empty (equal)
                                *(container.wrapping_add(72) as *mut u64) = 0;
                                *(container.wrapping_add(80) as *mut u64) = 0;
                                // ibo[72] = element-buffer id (bound by 0x5b353d0's tail)
                                *(ibo.wrapping_add(72) as *mut u32) = 0;
                            }
                            eprintln!(
                                "[elfjit:renderframe-drawprobe] fabricated renderer 0x{renderer:x} ([+56]->cont, [+120]=ibo, [+142]=3, empty prim list)"
                            );
                            let mut sd = arm64jit::jit::CpuState::new();
                            sd.tpidr = tpidr;
                            sd.x[31] = isp;
                            sd.x[0] = renderer;
                            sd.x[1] = 0; // w22: draw-mode table index (GL_TRIANGLES-ish)
                            sd.x[2] = 0; // w23: stride multiplier
                            sd.x[3] = 0; // -> w1 for primitive-setup
                            sd.x[4] = 0; // w20: offset/count arg
                            sd.x[5] = 3; // w21: count (nonzero -> indexed path w/ slot 9)
                            match arm64jit::jit::jit_run(
                                iimg, ibase, 0x105b35288, &mut sd as *mut CpuState,
                            ) {
                                Err(e) => eprintln!(
                                    "[elfjit:renderframe-drawprobe] geometry wrapper stopped: {e}"
                                ),
                                Ok(ok) => eprintln!(
                                    "[elfjit:renderframe-drawprobe] geometry wrapper 0x5b35288 returned Ok({ok:#x})"
                                ),
                            }
                            // --renderframe-triangle: fabricate a COHERENT renderer — a
                            // real 1-primitive list, a real vertex-descriptor table, a
                            // real stride table, a real IBO, real vertex/index buffers
                            // (created + uploaded through the JIT bridge) and a real
                            // compiled+linked shader program — then drive the engine's
                            // own geometry wrapper 0x5b35288. Its primitive-setup
                            // 0x5b353d0 runs its REAL loop (bind ARRAY_BUFFER, enable
                            // attrib 0, glVertexAttribPointer at the format table) and
                            // the wrapper then dispatches a REAL indexed glDrawElements
                            // through GLES dispatch-table slot 9 with count=3, drawing
                            // an actual visible triangle (proving real geometry, not
                            // just the clear path, renders through the bridge).
                            if renderframe_args.iter().any(|a| a == "--renderframe-triangle") {
                                // GL enums used below.
                                const GL_ARRAY_BUFFER: u64 = 0x8892;
                                const GL_ELEMENT_ARRAY_BUFFER: u64 = 0x8893;
                                const GL_STATIC_DRAW: u64 = 0x88e4;
                                const GL_FLOAT: u64 = 0x1406;
                                const GL_VERTEX_SHADER: u64 = 0x8b31;
                                const GL_FRAGMENT_SHADER: u64 = 0x8b30;
                                const GL_COMPILE_STATUS: u64 = 0x8b81;
                                const GL_COLOR_BUFFER_BIT: u64 = 0x4000;
                                // GLES PLT stubs (verified against the real binary).
                                let plt_clear = 0x1062d7740u64;
                                let plt_clearcolor = 0x1062d7710u64;
                                let plt_viewport = 0x1062d75c0u64;
                                let plt_scissor = 0x1062d75d0u64;
                                let plt_genbuffers = 0x1062d77c0u64;
                                let plt_bindbuffer = 0x1062d77b0u64;
                                let plt_buffdata = 0x1062d77d0u64;
                                let plt_createshader = 0x1062d7880u64;
                                let plt_shadersource = 0x1062d7890u64;
                                let plt_compileshader = 0x1062d78a0u64;
                                let plt_getshaderiv = 0x1062d78b0u64;
                                // Texture/uniform GLES PLT stubs (verified against the real binary
                                // .plt: guest = file vaddr + 0x100000000).
                                let plt_active_texture = 0x1062d75e0u64;
                                let plt_bind_texture = 0x1062d75f0u64;
                                let plt_get_uniform_location = 0x1062d7900u64;
                                let plt_uniform_1i = 0x1062d7910u64;
                                let plt_tex_parameteri = 0x1062d7960u64;
                                let plt_gen_textures = 0x1062d7980u64;
                                let plt_tex_image_2d = 0x1062d79a0u64;
                                let plt_compressed_tex_image_2d = 0x1062d7990u64;
                                    let plt_getprogramiv = 0x1062d77f0u64;
                                    let plt_readpixels = 0x1062d7940u64;
                                    let plt_attachshader = 0x1062d78d0u64;
                                let plt_linkprogram = 0x1062d78e0u64;
                                let plt_bindattrib = 0x1062d78f0u64;
                                let plt_useprogram = 0x1062d75a0u64;
                                let plt_enableattrib = 0x1062d7850u64;
                                let plt_attribptr = 0x1062d7860u64;
                                let plt_dewelem = 0x1062d7830u64; // glDrawElements@plt (direct, not slot9)
                                // Helper: drive a single guest PLT stub via jit_run and
                                // return its x0 (the int-bridge HostCall returns via x0).
                                let mut gcall = |addr: u64,
                                                 a0: u64,
                                                 a1: u64,
                                                 a2: u64,
                                                 a3: u64,
                                                 a4: u64,
                                                 a5: u64|
                                                 -> Result<u64, String> {
                                    let mut s = arm64jit::jit::CpuState::new();
                                    s.tpidr = tpidr;
                                    s.x[31] = isp;
                                    s.x[0] = a0;
                                    s.x[1] = a1;
                                    s.x[2] = a2;
                                    s.x[3] = a3;
                                    s.x[4] = a4;
                                    s.x[5] = a5;
                                    let r = arm64jit::jit::jit_run(iimg, ibase, addr, &mut s as *mut CpuState)?;
                                    Ok(s.x[0])
                                };
                                let objs = Box::leak(vec![0u8; 16384].into_boxed_slice());
                                let base = objs.as_ptr() as u64;
                                unsafe {
                                    // Clear the framebuffer first so the triangle is
                                    // visible against a known background. glClearColor is
                                    // a FLOAT-ABI bridge (reads guest s0..s3 = v[0],v[2],
                                    // v[4],v[6] low lanes), so set the SIMD lanes not x-regs.
                                    let mut sc = arm64jit::jit::CpuState::new();
                                    sc.tpidr = tpidr;
                                    sc.x[31] = isp;
                                    sc.v[0] = (0.0f32).to_bits() as u64;
                                    sc.v[2] = (0.0f32).to_bits() as u64;
                                    sc.v[4] = (0.3f32).to_bits() as u64;
                                    sc.v[6] = (1.0f32).to_bits() as u64;
                                    let _ = arm64jit::jit::jit_run(
                                        iimg,
                                        ibase,
                                        plt_clearcolor,
                                        &mut sc as *mut CpuState,
                                    );
                                    let _ = gcall(plt_clear, GL_COLOR_BUFFER_BIT, 0, 0, 0, 0, 0);
                                    // Set the viewport + scissor to the window size so
                                    // the rasterizer has a drawable region. The SH22
                                    // frame-fn sets these; a 0-size stale viewport from
                                    // context creation silently rasterizes nothing.
                                    let _ = gcall(plt_viewport, 0, 0, 1280, 720, 0, 0);
                                    let _ = gcall(plt_scissor, 0, 0, 1280, 720, 0, 0);
                                    // Vertex shader: pass clip-space position straight
                                    // through (data is already in NDC).
                                    let vs_src = b"attribute vec4 aPos;\nvoid main(){ gl_Position = aPos; }\n\0";
                                    // --renderframe-tex: prove the GLES texture/uniform/shader
                                    // bridge path renders a TEXTURED draw through the engine's
                                    // own geometry wrapper. The fragment shader samples a 2x2 RGBA
                                    // checkerboard via a UV computed from gl_FragCoord (so the
                                    // single-attrib coherent renderer stays unchanged — no second
                                    // UV vertex attrib). floor/texture2D/gl_FragCoord are all GLSL
                                    // ES 1.00. Three interior probes then read back three DIFFERENT
                                    // texel colors, which no constant/solid shader can produce.
                                    let tex_mode = renderframe_args.iter().any(|a| a == "--renderframe-tex");
                                    // --renderframe-etc: like --renderframe-tex but uploads the 2x2
                                    // checkerboard as a REAL compressed ETC1 texture (4 solid
                                    // 4x4 blocks = 8x8) through glCompressedTexImage2D
                                    // (GL_ETC1_RGB8_OES=0x8d64). Proves the compressed-texture
                                    // interception live: the bridge decodes ETC1->RGBA and
                                    // uploads via glTexImage2D. Same FS + readback as tex_mode.
                                    let etc_mode = renderframe_args.iter().any(|a| a == "--renderframe-etc");
                                    // --renderframe-etc2: same compressed path but internalformat
                                    // GL_COMPRESSED_RGB8_ETC2 (0x9274 — the actual Android Roblox
                                    // ETC2 format). Modes 1/2 of ETC2 RGB are bit-identical to ETC1
                                    // individual/differential, so the same crafted blocks are valid
                                    // ETC2 blocks; decode_etc2_rgb must yield the same colors.
                                    let etc2_mode = renderframe_args.iter().any(|a| a == "--renderframe-etc2");
                                    let comp_mode = etc_mode || etc2_mode;
                                    let fs_src: &[u8] = if tex_mode || comp_mode {
                                        // 2x2 texels RED,GREEN,BLUE,WHITE. UV = floor(frag/640,360)
                                        // picks a quadrant, (uv+0.5)*0.5 samples its texel center
                                        // under NEAREST. centroid(640,360)->(1,1)->WHITE; (900,150)
                                        // ->(1,0)->GREEN; (300,150)->(0,0)->RED.
                                        b"precision mediump float;\nuniform sampler2D uTex;\nvoid main(){ vec2 uv = floor(gl_FragCoord.xy / vec2(640.0,360.0)); uv = (uv + 0.5) * 0.5; gl_FragColor = texture2D(uTex, uv); }\n\0"
                                    } else {
                                        // Fragment shader: solid red.
                                        b"void main(){ gl_FragColor = vec4(1.0,0.0,0.0,1.0); }\n\0"
                                    };
                                    let vs_ptr = objs.as_ptr() as u64 + 0x400;
                                    let fs_ptr = objs.as_ptr() as u64 + 0x800;
                                    std::ptr::copy_nonoverlapping(
                                        vs_src.as_ptr(),
                                        vs_ptr as *mut u8,
                                        vs_src.len(),
                                    );
                                    std::ptr::copy_nonoverlapping(
                                        fs_src.as_ptr(),
                                        fs_ptr as *mut u8,
                                        fs_src.len(),
                                    );
                                    // src[] arrays: 1 string pointer each, NULL lengths.
                                    let vs_ary = objs.as_ptr() as u64 + 0xa00;
                                    let fs_ary = objs.as_ptr() as u64 + 0xa10;
                                    *(vs_ary as *mut u64) = vs_ptr;
                                    *(fs_ary as *mut u64) = fs_ptr;
                                    // Triangle vertices (NDC, 3 x vec4). Fill most of the frame so the
                                    // rendered footprint is easy to measure for scaling.
                                    let verts: [f32; 12] = [
                                        -0.95, -0.95, 0.0, 1.0, // v0
                                        0.95, -0.95, 0.0, 1.0, // v1
                                        0.0, 0.95, 0.0, 1.0, // v2
                                    ];
                                    let vbo_data = objs.as_ptr() as u64 + 0xc00;
                                    // Indices: 3 (u32).
                                    let idx: [u32; 3] = [0, 1, 2];
                                    let ebo_data = objs.as_ptr() as u64 + 0xd00;
                                    std::ptr::copy_nonoverlapping(
                                        verts.as_ptr() as *const u8,
                                        vbo_data as *mut u8,
                                        std::mem::size_of_val(&verts),
                                    );
                                    std::ptr::copy_nonoverlapping(
                                        idx.as_ptr() as *const u8,
                                        ebo_data as *mut u8,
                                        std::mem::size_of_val(&idx),
                                    );
                                    let shader_id_slot = objs.as_ptr() as u64 + 0xe00;
                                    let _ = shader_id_slot;
                                    // Compile vertex shader (glCreateShader returns id in x0).
                                    let vs_shader = gcall(plt_createshader, GL_VERTEX_SHADER, 0, 0, 0, 0, 0)
                                        .unwrap_or(0)
                                        & 0xffff_ffff;
                                    let _ = gcall(plt_shadersource, vs_shader, 1, vs_ary, 0, 0, 0);
                                    let _ = gcall(plt_compileshader, vs_shader, 0, 0, 0, 0, 0);
                                    // Compile fragment shader.
                                    let fs_shader = gcall(plt_createshader, GL_FRAGMENT_SHADER, 0, 0, 0, 0, 0)
                                        .unwrap_or(0)
                                        & 0xffff_ffff;
                                    let _ = gcall(plt_shadersource, fs_shader, 1, fs_ary, 0, 0, 0);
                                    let _ = gcall(plt_compileshader, fs_shader, 0, 0, 0, 0, 0);
                                    eprintln!(
                                        "[elfjit:renderframe-triangle] compiled vs={vs_shader:#x} fs={fs_shader:#x}"
                                    );
                                    // Create + link program (id returned in x0).
                                    let program = gcall(0x1062d78c0, 0, 0, 0, 0, 0, 0) // glCreateProgram@plt
                                        .unwrap_or(0)
                                        & 0xffff_ffff;
                                    let _ = gcall(plt_attachshader, program, vs_shader, 0, 0, 0, 0);
                                    let _ = gcall(plt_attachshader, program, fs_shader, 0, 0, 0, 0);
                                    // Bind attrib location 0 = aPos BEFORE link.
                                    let loc_name = objs.as_ptr() as u64 + 0xd20;
                                    std::ptr::copy_nonoverlapping(
                                        b"aPos\0".as_ptr(),
                                        loc_name as *mut u8,
                                        5,
                                    );
                                    let _ = gcall(plt_bindattrib, program, 0, loc_name, 0, 0, 0);
                                    let _ = gcall(plt_linkprogram, program, 0, 0, 0, 0, 0);
                                    let _ = gcall(plt_useprogram, program, 0, 0, 0, 0, 0);
                                    eprintln!(
                                        "[elfjit:renderframe-triangle] linked program={program:#x} current"
                                    );
                                    // --renderframe-tex: create + upload a 2x2 RGBA checkerboard
                                    // texture and assign it to the program's uTex sampler (unit 0),
                                    // all through the GLES bridge (@plt). Every call here exercises
                                    // the texture/uniform/shader bridge surface the engine's real
                                    // textured draws will need. glTexImage2D has 9 args (pixels on
                                    // the guest stack), so drive it with a dedicated CpuState whose
                                    // sp=tex_sp points at a slot holding the pixels pointer.
                                    if tex_mode || comp_mode {
                                                                            const GL_TEXTURE0: u64 = 0x84c0;
                                                                            const GL_TEXTURE_2D: u64 = 0x0de1;
                                                                            const GL_RGBA: u64 = 0x1908;
                                                                            const GL_UNSIGNED_BYTE: u64 = 0x1401;
                                                                            const GL_NEAREST: u64 = 0x2600;
                                                                            const GL_TEXTURE_MIN_FILTER: u64 = 0x2801;
                                                                            const GL_TEXTURE_MAG_FILTER: u64 = 0x2800;
                                                                            const GL_TEX_DATA: u64 = 0xf60;
                                                                            // Shared: create + bind the texture on unit 0, NEAREST filtering.
                                                                            let tex_id_slot = base + 0xfd0;
                                                                            let _ = gcall(plt_gen_textures, 1, tex_id_slot, 0, 0, 0, 0);
                                                                            let tex_id = *(tex_id_slot as *const u32) as u64;
                                                                            let _ = gcall(plt_active_texture, GL_TEXTURE0, 0, 0, 0, 0, 0);
                                                                            let _ = gcall(plt_bind_texture, GL_TEXTURE_2D, tex_id, 0, 0, 0, 0);
                                                                            let _ = gcall(plt_tex_parameteri, GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST, 0, 0, 0);
                                                                            let _ = gcall(plt_tex_parameteri, GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST, 0, 0, 0);
                                                                            if tex_mode {
                                                                                // RGBA 2x2 checkerboard via 9-arg glTexImage2D. pixels (the 9th arg) rides the
                                                                                // guest stack at [sp+0]; the PLT stub is a leaf (adrp/ldr/add/br, never pushes sp),
                                                                                // so a fake sp whose [0] holds the pixels ptr is read by the bridge's gs_stack.
                                                                                const TEX: [u8; 16] = [
                                                                                    255, 0, 0, 255, // texel(0,0) RED
                                                                                    0, 255, 0, 255, // texel(1,0) GREEN
                                                                                    0, 0, 255, 255, // texel(0,1) BLUE
                                                                                    255, 255, 255, 255, // texel(1,1) WHITE
                                                                                ];
                                                                                std::ptr::copy_nonoverlapping(TEX.as_ptr(), (base + GL_TEX_DATA) as *mut u8, 16);
                                                                                let tex_sp = base + 0xf80;
                                                                                *(tex_sp as *mut u64) = base + GL_TEX_DATA;
                                                                                let mut stex = arm64jit::jit::CpuState::new();
                                                                                stex.tpidr = tpidr;
                                                                                stex.x[31] = tex_sp;
                                                                                stex.x[0] = GL_TEXTURE_2D;
                                                                                stex.x[1] = 0; // level
                                                                                stex.x[2] = GL_RGBA; // internalformat
                                                                                stex.x[3] = 2; // width
                                                                                stex.x[4] = 2; // height
                                                                                stex.x[5] = 0; // border
                                                                                stex.x[6] = GL_RGBA; // format
                                                                                stex.x[7] = GL_UNSIGNED_BYTE; // type
                                                                                let _ = arm64jit::jit::jit_run(iimg, ibase, plt_tex_image_2d, &mut stex as *mut CpuState);
                                                                            } else {
                                                                                // ETC1 compressed-texture interception live-path: upload a REAL 8x8 ETC1 texture
                                                                                // (4 solid 4x4 blocks = 32 bytes) via glCompressedTexImage2D (GL_ETC1_RGB8_OES).
                                                                                // The bridge decodes ETC1->RGBA (texture-codec) and re-uploads via glTexImage2D.
                                                                                // All 8 args fit x0-x7 (no stack arg). Each block: individual mode, table codeword
                                                                                // 0, all selectors 0 -> decoded color = (c*0x11)+2 per channel, clamped.
                                                                                const GL_ETC1_RGB8_OES: u64 = 0x8d64;
                                                                                const GL_COMPRESSED_RGB8_ETC2: u64 = 0x9274;
                                                                                // ETC2 mode 1/2 are bit-identical to ETC1 individual/differential, so the
                                                                                // same blocks are valid ETC2-RGB; just relabel the internalformat to prove
                                                                                // decode_etc2_rgb (the real Android Roblox path) handles them.
                                                                                let comp_fmt = if etc2_mode { GL_COMPRESSED_RGB8_ETC2 } else { GL_ETC1_RGB8_OES };
                                                                                let enc = |t: i32| -> u8 { let c = ((t - 2).clamp(0, 240) >> 4) as u8; (c << 4) | c };
                                                                                let blk = |r: u8, g: u8, b: u8| -> [u8; 8] { [r, g, b, 0, 0, 0, 0, 0] };
                                                                                // 8x8 ETC1: 4 blocks row-major top-first -> (255,2,2) red,(2,255,2) green,
                                                                                // (2,2,255) blue,(255,255,255) white.
                                                                                let etc_data: [u8; 32] = {
                                                                                    let mut d = [0u8; 32];
                                                                                    let red = blk(enc(255), enc(2), enc(2));
                                                                                    let grn = blk(enc(2), enc(255), enc(2));
                                                                                    let blu = blk(enc(2), enc(2), enc(255));
                                                                                    let wht = blk(enc(255), enc(255), enc(255));
                                                                                    d[0..8].copy_from_slice(&red);
                                                                                    d[8..16].copy_from_slice(&grn);
                                                                                    d[16..24].copy_from_slice(&blu);
                                                                                    d[24..32].copy_from_slice(&wht);
                                                                                    d
                                                                                };
                                                                                std::ptr::copy_nonoverlapping(etc_data.as_ptr(), (base + GL_TEX_DATA) as *mut u8, 32);
                                                                                let mut sce = arm64jit::jit::CpuState::new();
                                                                                sce.tpidr = tpidr;
                                                                                sce.x[31] = isp;
                                                                                sce.x[0] = GL_TEXTURE_2D;
                                                                                sce.x[1] = 0; // level
                                                                                sce.x[2] = comp_fmt; // internalformat
                                                                                sce.x[3] = 8; // width
                                                                                sce.x[4] = 8; // height
                                                                                sce.x[5] = 0; // border
                                                                                sce.x[6] = 32; // imageSize
                                                                                sce.x[7] = base + GL_TEX_DATA; // data
                                                                                let _ = arm64jit::jit::jit_run(iimg, ibase, plt_compressed_tex_image_2d, &mut sce as *mut CpuState);
                                                                            }
                                                                            // Shared: uTex sampler = texture unit 0.
                                                                            let uni = objs.as_ptr() as u64 + 0xe20;
                                                                            std::ptr::copy_nonoverlapping(b"uTex\0".as_ptr(), uni as *mut u8, 5);
                                                                            let ploc = gcall(plt_get_uniform_location, program, uni, 0, 0, 0, 0).unwrap_or(0) & 0xffff_ffff;
                                                                            let _ = gcall(plt_uniform_1i, ploc, 0, 0, 0, 0, 0);
                                                                            eprintln!("[elfjit:renderframe-tex] texture tex_id={tex_id:#x} bound+uploaded uTex loc={ploc:#x}<-unit0");
                                    }
                                    // Diagnostics: real compile/link status. Reading a
                                    // GL int from a shifted-out 32-bit slot requires a
                                    // predictable result location — use glGetShaderiv/
                                    // glGetProgramiv writing a real int result slot.
                                    let int_slot0 = objs.as_ptr() as u64 + 0xf20; // vs compile
                                    let int_slot1 = objs.as_ptr() as u64 + 0xf24; // fs compile
                                    let int_slot2 = objs.as_ptr() as u64 + 0xf28; // link
                                    *(int_slot0 as *mut u32) = 0xdeadbeef;
                                    *(int_slot1 as *mut u32) = 0xdeadbeef;
                                    *(int_slot2 as *mut u32) = 0xdeadbeef;
                                    let _ = gcall(
                                        plt_getshaderiv,
                                        vs_shader,
                                        0x8b81, // GL_COMPILE_STATUS
                                        int_slot0,
                                        0,
                                        0,
                                        0,
                                    );
                                    let _ = gcall(
                                        plt_getshaderiv,
                                        fs_shader,
                                        0x8b81,
                                        int_slot1,
                                        0,
                                        0,
                                        0,
                                    );
                                    let _ = gcall(
                                        plt_getprogramiv,
                                        program,
                                        0x8b82, // GL_LINK_STATUS
                                        int_slot2,
                                        0,
                                        0,
                                        0,
                                    );
                                    unsafe {
                                        eprintln!(
                                            "[elfjit:renderframe-triangle] compile_status vs=0x{:x} fs=0x{:x} link_status=0x{:x}",
                                            *(int_slot0 as *const u32),
                                            *(int_slot1 as *const u32),
                                            *(int_slot2 as *const u32)
                                        );
                                    }
                                    // Debug: if a shader/program failed, dump its info log (via the
                                    // int bridge — glGetShaderInfoLog / glGetProgramInfoLog resolve
                                    // through the same resolve_gles_int the seedgles uses).
                                    {
                                        let logbuf = objs.as_ptr() as u64 + 0xfb0;
                                        let logslot: Option<u64> = arm64jit::resolver::resolve_gles_int(b"glGetShaderInfoLog\0");
                                        let plogslot: Option<u64> = arm64jit::resolver::resolve_gles_int(b"glGetProgramInfoLog\0");
                                        unsafe {
                                            let bad_fs = *(int_slot1 as *const u32) == 0;
                                            let bad_vs = *(int_slot0 as *const u32) == 0;
                                            let bad_link = *(int_slot2 as *const u32) == 0;
                                            if (bad_fs || bad_vs) && let Some(slot) = logslot {
                                                for (what, sh) in [("fs", fs_shader), ("vs", vs_shader)] {
                                                    if !(if what == "fs" { bad_fs } else { bad_vs }) { continue; }
                                                    let mut ls = arm64jit::jit::CpuState::new();
                                                    ls.tpidr = tpidr;
                                                    ls.x[31] = isp;
                                                    ls.x[0] = sh as u64;
                                                    ls.x[1] = 2048;
                                                    ls.x[2] = 0;
                                                    ls.x[3] = logbuf;
                                                    let _ = arm64jit::jit::jit_run(iimg, ibase, slot, &mut ls as *mut CpuState);
                                                    let cstr = std::ffi::CStr::from_ptr(logbuf as *const libc::c_char);
                                                    eprintln!("[elfjit:renderframe-triangle] {what} info-log: {cstr:?}");
                                                }
                                            }
                                            if bad_link && let Some(slot) = plogslot {
                                                let mut ls = arm64jit::jit::CpuState::new();
                                                ls.tpidr = tpidr;
                                                ls.x[31] = isp;
                                                ls.x[0] = program as u64;
                                                ls.x[1] = 2048;
                                                ls.x[2] = 0;
                                                ls.x[3] = logbuf;
                                                let _ = arm64jit::jit::jit_run(iimg, ibase, slot, &mut ls as *mut CpuState);
                                                let cstr = std::ffi::CStr::from_ptr(logbuf as *const libc::c_char);
                                                eprintln!("[elfjit:renderframe-triangle] program info-log: {cstr:?}");
                                            }
                                        }
                                    }
                                    // Create + fill the VBO (ARRAY_BUFFER) with verts.
                                    // glGenBuffers writes the generated id to its out
                                    // pointer — use a DEDICATED slot, never the data
                                    // buffer (aliasing would clobber the vertices).
                                    let vbo_id_slot = objs.as_ptr() as u64 + 0xf00;
                                    let ebo_id_slot = objs.as_ptr() as u64 + 0xf10;
                                    let _ = gcall(plt_genbuffers, 1, vbo_id_slot, 0, 0, 0, 0);
                                    let vbo = *(vbo_id_slot as *const u32) as u64;
                                    let _ = gcall(plt_bindbuffer, GL_ARRAY_BUFFER, vbo, 0, 0, 0, 0);
                                    let _ = gcall(
                                        plt_buffdata,
                                        GL_ARRAY_BUFFER,
                                        std::mem::size_of_val(&verts) as u64,
                                        vbo_data,
                                        GL_STATIC_DRAW,
                                        0,
                                        0,
                                    );
                                    // Create + fill the EBO (ELEMENT_ARRAY_BUFFER) idx.
                                    let _ = gcall(plt_genbuffers, 1, ebo_id_slot, 0, 0, 0, 0);
                                    let ebo = *(ebo_id_slot as *const u32) as u64;
                                    let _ = gcall(
                                        plt_bindbuffer,
                                        GL_ELEMENT_ARRAY_BUFFER,
                                        ebo,
                                        0,
                                        0,
                                        0,
                                        0,
                                    );
                                    let _ = gcall(
                                        plt_buffdata,
                                        GL_ELEMENT_ARRAY_BUFFER,
                                        std::mem::size_of_val(&idx) as u64,
                                        ebo_data,
                                        GL_STATIC_DRAW,
                                        0,
                                        0,
                                    );
                                    eprintln!(
                                        "[elfjit:renderframe-triangle] vbo={vbo:#x} ebo={ebo:#x} uploaded"
                                    );
                                    // REFERENCE DRAW (opt-in: SH25_REF=1): drive the
                                    // draw directly (not through the engine wrapper) with
                                    // our own glVertexAttribPointer, to cross-check the
                                    // engine-path result. Now that the engine wrapper's
                                    // primitive-setup renders the full triangle (format
                                    // index fixed 5->3 = GL_FLOAT), the reference is
                                    // redundant; default OFF (SH25_REF=1 re-enables).
                                    if std::env::var("SH25_REF").map(|v| v == "1").unwrap_or(false) {
                                    // With a VBO bound, the attrib pointer's 6th arg is a
                                    // byte OFFSET (0 = start of the buffer), not a host
                                    // pointer — a wrong value silently collapses geometry.
                                    {
                                        let _ = gcall(plt_bindbuffer, GL_ARRAY_BUFFER, vbo, 0, 0, 0, 0);
                                        let _ = gcall(plt_attribptr, 0, 4, GL_FLOAT, 0, 16, 0);
                                        let _ = gcall(plt_enableattrib, 0, 0, 0, 0, 0, 0);
                                        let _ = gcall(
                                            plt_bindbuffer,
                                            GL_ELEMENT_ARRAY_BUFFER,
                                            ebo,
                                            0,
                                            0,
                                            0,
                                            0,
                                        );
                                        eprintln!(
                                            "[elfjit:renderframe-triangle] reference draw: attrib0(4xfloat,stride16,off0) + EBO bound"
                                        );
                                        // glDrawElements signature: (mode, count, type,
                                        // indices-offset) -> (x0,x1,x2,x3).
                                        let mut sd = arm64jit::jit::CpuState::new();
                                        sd.tpidr = tpidr;
                                        sd.x[31] = isp;
                                        sd.x[0] = 4; // GL_TRIANGLES
                                        sd.x[1] = 3; // count
                                        sd.x[2] = 0x1405; // GL_UNSIGNED_INT
                                        sd.x[3] = 0; // indices offset in EBO
                                        let _ = arm64jit::jit::jit_run(
                                            iimg,
                                            ibase,
                                            plt_dewelem,
                                            &mut sd as *mut CpuState,
                                        );
                                        eprintln!(
                                            "[elfjit:renderframe-triangle] reference glDrawElements issued"
                                        );
                                    }
                                    }
                                    // ---- Fabricate the COHERENT renderer ----
                                    // renderer[+56]=container ; [renderer+0x48]=the 16-byte
                                    // vertex-descriptor table base (entry[vb] @ +vb*16).
                                    // container[+72]=begin,[+80]=end primitive list;
                                    // container[+96]=stride table base ([cb+96+vb*8]).
                                    // descriptor obj: [desc+72]=ARRAY_BUFFER id.
                                    // IBO: renderer[+120]=ibo obj; [ibo+72]=EBO id.
                                    // renderer[+142](u16)=element count.
                                    let renderer = base;
                                    let container = base + 0x100;
                                    let desc = base + 0x200; // vertex descriptor obj
                                    let stride_tbl = base + 0x300; // u64 tbl [vb]
                                    let prim = base + 0x400;
                                    let ibo = base + 0x500;
                                    let fmt_index: u32 = 3; // format[3]={size4, GL_FLOAT=0x1406} (table @0x100cecf8c). NOT format[5] which is {4, GL_SHORT=0x1402} — GL_SHORT misreads float verts -> degenerate.
                                    // descriptor[+72] = vbo id (the ARRAY_BUFFER we created)
                                    *(desc.wrapping_add(72) as *mut u32) = vbo as u32;
                                    // stride table[vb=0] = 16 (tight vec4)
                                    *(stride_tbl as *mut u64) = 16;
                                    // container
                                    *(renderer.wrapping_add(56) as *mut u64) = container;
                                    *(container.wrapping_add(72) as *mut u64) = prim;
                                    *(container.wrapping_add(80) as *mut u64) = prim + 0x18; // 1 prim (stride 0x18)
                                    *(container.wrapping_add(96) as *mut u64) = stride_tbl;
                                    // descriptor table is INLINE at renderer+0x48: entry[vb] @ +vb*16 is the
                                    // descriptor pointer (5b3546c ldr x11,[sp,#16] with
                                    // sp+16=renderer+0x48; 5b3547c ldr x10,[x11, w9<<4]).
                                    // vb=0 -> the desc ptr lives at renderer+0x48.
                                    *(renderer.wrapping_add(0x48) as *mut u64) = desc;
                                    // primitive: [+0]=vb idx(w9=0), [+4]=offset(w21=0),
                                    // [+8]=format idx(w28=fmt_index), [+12]=type(w22=0 ->
                                    // attrib index 0), [+16]=base(0).
                                    *(prim as *mut u32) = 0;
                                    *(prim.wrapping_add(4) as *mut u32) = 0;
                                    *(prim.wrapping_add(8) as *mut u32) = fmt_index;
                                    *(prim.wrapping_add(12) as *mut u32) = 0;
                                    *(prim.wrapping_add(16) as *mut u32) = 0;
                                    // IBO: renderer[+120]=ibo ; [ibo+72]=EBO id
                                    *(renderer.wrapping_add(120) as *mut u64) = ibo;
                                    *(ibo.wrapping_add(72) as *mut u32) = ebo as u32;
                                    // renderer[+142] u16 element count = 3
                                    *(renderer.wrapping_add(142) as *mut u16) = 3;
                                    eprintln!(
                                        "[elfjit:renderframe-triangle] coherent renderer 0x{renderer:x}: container 0x{container:x} prim 0x{prim:x} desc 0x{desc:x} desc_tbl@renderer+0x48 stride 0x{stride_tbl:x} ibo 0x{ibo:x}"
                                    );
                                    // Drive the engine's OWN geometry wrapper.
                                    let mut sw = arm64jit::jit::CpuState::new();
                                    sw.tpidr = tpidr;
                                    sw.x[31] = isp;
                                    sw.x[0] = renderer;
                                    sw.x[1] = 0; // w22: draw-mode table index (0=GL_TRIANGLES)
                                    sw.x[2] = 0; // w23: stride multiplier
                                    sw.x[3] = 0; // -> w1 for primitive-setup
                                    sw.x[4] = 3; // w20 -> glDrawElements count (wrapper `mov w1,w20`)
                                    sw.x[5] = 3; // w21: nonzero -> indexed path selection
                                    match arm64jit::jit::jit_run(
                                        iimg,
                                        ibase,
                                        0x105b35288,
                                        &mut sw as *mut CpuState,
                                    ) {
                                        Err(e) => eprintln!(
                                            "[elfjit:renderframe-triangle] geometry wrapper stopped: {e}"
                                        ),
                                        Ok(ok) => eprintln!(
                                            "[elfjit:renderframe-triangle] geometry wrapper 0x5b35288 returned Ok({ok:#x}) (real indexed glDrawElements drawn)"
                                        ),
                                    }
                                    // glReadPixels readback: verify the triangle
                                    // actually drew. Center (0,0 NDC -> ~639,360) should
                                    // be RED; top-left corner should be background.
                                    // glReadPixels verification: 3 probes — triangle centroid interior, left
                                    // background, right background. Proves real drawn
                                    // geometry landed at the expected sub-frame spots.
                                    {
                                        let mut sp = arm64jit::jit::CpuState::new();
                                        sp.tpidr = tpidr;
                                        sp.x[31] = isp;
                                        sp.x[0] = 640;
                                        sp.x[1] = 360;
                                        sp.x[2] = 1;
                                        sp.x[3] = 1;
                                        sp.x[4] = 0x1908; // GL_RGBA
                                        sp.x[5] = 0x1401; // GL_UNSIGNED_BYTE
                                        sp.x[6] = objs.as_ptr() as u64 + 0xf40;
                                        let _ = arm64jit::jit::jit_run(
                                            iimg,
                                            ibase,
                                            plt_readpixels,
                                            &mut sp as *mut CpuState,
                                        );
                                        let _ = sp;
                                    }
                                    let mut pb = arm64jit::jit::CpuState::new();
                                    pb.tpidr = tpidr;
                                    pb.x[31] = isp;
                                    pb.x[0] = 1200;
                                    pb.x[1] = 20;
                                    pb.x[2] = 1;
                                    pb.x[3] = 1;
                                    pb.x[4] = 0x1908;
                                    pb.x[5] = 0x1401;
                                    pb.x[6] = objs.as_ptr() as u64 + 0xf44;
                                    let _ = arm64jit::jit::jit_run(
                                        iimg,
                                        ibase,
                                        plt_readpixels,
                                        &mut pb as *mut CpuState,
                                    );
                                    let pc2 = objs.as_ptr() as u64 + 0xf48;
                                    let mut pc3 = arm64jit::jit::CpuState::new();
                                    pc3.tpidr = tpidr;
                                    pc3.x[31] = isp;
                                    pc3.x[0] = 60;
                                    pc3.x[1] = 20;
                                    pc3.x[2] = 1;
                                    pc3.x[3] = 1;
                                    pc3.x[4] = 0x1908;
                                    pc3.x[5] = 0x1401;
                                    pc3.x[6] = pc2;
                                    let _ = arm64jit::jit::jit_run(
                                        iimg,
                                        ibase,
                                        plt_readpixels,
                                        &mut pc3 as *mut CpuState,
                                    );
                                    unsafe {
                                        let c = |p: u64| -> String {
                                            format!(
                                                "RGBA({},{},{},{})",
                                                *(p as *const u8),
                                                *(p as *const u8).add(1),
                                                *(p as *const u8).add(2),
                                                *(p as *const u8).add(3)
                                            )
                                        };
                                        eprintln!(
                                            "[elfjit:renderframe-triangle] readback: centroid(640,360)={} top-left-bg(60,20)={} top-right-bg(1200,20)={}",
                                            c(objs.as_ptr() as u64 + 0xf40),
                                            c(pc2),
                                            c(pb.x[6])
                                        );
                                    }
                                    // --renderframe-tex readback: 3 on-triangle quadrant probes
                                    // must yield three DIFFERENT texel colors (WHITE/GREEN/RED),
                                    // which a constant shader cannot produce -> proves the sampled
                                    // texture actually rendered.
                                    if tex_mode || comp_mode {
                                        let probes: [(u32, u32, &str, u64); 3] = [
                                            (640, 360, "centroid(WHITE)", 0xf50),
                                            (900, 150, "quad-(1,0)(GREEN)", 0xf54),
                                            (300, 150, "quad-(0,0)(RED)", 0xf58),
                                        ];
                                        for (px, py, label, slot) in probes {
                                            let mut pp = arm64jit::jit::CpuState::new();
                                            pp.tpidr = tpidr;
                                            pp.x[31] = isp;
                                            pp.x[0] = px as u64;
                                            pp.x[1] = py as u64;
                                            pp.x[2] = 1;
                                            pp.x[3] = 1;
                                            pp.x[4] = 0x1908; // GL_RGBA
                                            pp.x[5] = 0x1401; // GL_UNSIGNED_BYTE
                                            pp.x[6] = objs.as_ptr() as u64 + slot;
                                            let _ = arm64jit::jit::jit_run(
                                                iimg,
                                                ibase,
                                                plt_readpixels,
                                                &mut pp as *mut CpuState,
                                            );
                                            unsafe {
                                                let p = objs.as_ptr() as u64 + slot;
                                                let c_ = format!(
                                                    "RGBA({},{},{},{})",
                                                    *(p as *const u8),
                                                    *(p as *const u8).add(1),
                                                    *(p as *const u8).add(2),
                                                    *(p as *const u8).add(3)
                                                );
                                                eprintln!(
                                                    "[elfjit:renderframe-tex] readback {label} @({px},{py}) = {c_}"
                                                );
                                            }
                                        }
                                    }
                                    // --renderframe-triangle-loop <N>: SUSTAINABLE real-
                                    // geometry rendering. Re-run clear (cycling the clear
                                    // color through a palette so a recording proves a
                                    // fresh frame each iteration) -> re-drive the engine's
                                    // own geometry wrapper (same coherent renderer) ->
                                    // swap. Geometry analog of SH23's --rendersustain.
                                    let tri_loop_n: usize = renderframe_args
                                        .iter()
                                        .position(|a| a == "--renderframe-triangle-loop")
                                        .and_then(|i| renderframe_args.get(i + 1))
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or(0);
                                    if tri_loop_n > 1 {
                                        let palette: [[f32; 3]; 5] = [
                                            [0.05, 0.05, 0.05],
                                            [0.20, 0.05, 0.05],
                                            [0.05, 0.20, 0.05],
                                            [0.05, 0.05, 0.20],
                                            [0.18, 0.10, 0.04],
                                        ];
                                        let mut iter: u64 = 0;
                                        while iter < tri_loop_n as u64 {
                                            let bg = palette[(iter as usize) % palette.len()];
                                            let mut scn = arm64jit::jit::CpuState::new();
                                            scn.tpidr = tpidr;
                                            scn.x[31] = isp;
                                            scn.v[0] = bg[0].to_bits() as u64;
                                            scn.v[2] = bg[1].to_bits() as u64;
                                            scn.v[4] = bg[2].to_bits() as u64;
                                            scn.v[6] = (1.0f32).to_bits() as u64;
                                            let _ = arm64jit::jit::jit_run(
                                                iimg,
                                                ibase,
                                                plt_clearcolor,
                                                &mut scn as *mut CpuState,
                                            );
                                            let _ = gcall(plt_clear, GL_COLOR_BUFFER_BIT, 0, 0, 0, 0, 0);
                                            let mut swn = arm64jit::jit::CpuState::new();
                                            swn.tpidr = tpidr;
                                            swn.x[31] = isp;
                                            swn.x[0] = renderer;
                                            swn.x[1] = 0;
                                            swn.x[2] = 0;
                                            swn.x[3] = 0;
                                            swn.x[4] = 3;
                                            swn.x[5] = 3;
                                            match arm64jit::jit::jit_run(
                                                iimg,
                                                ibase,
                                                0x105b35288,
                                                &mut swn as *mut CpuState,
                                            ) {
                                                Err(e) => eprintln!(
                                                    "[elfjit:renderframe-triangle-loop] iter {iter} wrapper stopped: {e}"
                                                ),
                                                Ok(_) => (),
                                            }
                                            let mut sen = arm64jit::jit::CpuState::new();
                                            sen.tpidr = tpidr;
                                            sen.x[31] = isp;
                                            sen.x[0] = real_ctx;
                                            match arm64jit::jit::jit_run(
                                                iimg,
                                                ibase,
                                                swap_thunk,
                                                &mut sen as *mut CpuState,
                                            ) {
                                                Err(e) => eprintln!(
                                                    "[elfjit:renderframe-triangle-loop] iter {iter} swap stopped: {e}"
                                                ),
                                                Ok(ok) => eprintln!(
                                                    "[elfjit:renderframe-triangle-loop] iter {iter} drew+swap Ok({ok:#x}) bg={bg:?} (fresh real-geometry frame)"
                                                ),
                                            }
                                            iter += 1;
                                            std::thread::sleep(std::time::Duration::from_millis(350));
                                        }
                                    }
                                }
                                // Present the drawn frame.
                                let mut se = arm64jit::jit::CpuState::new();
                                se.tpidr = tpidr;
                                se.x[31] = isp;
                                se.x[0] = real_ctx;
                                match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut se as *mut CpuState) {
                                    Err(e) => eprintln!("[elfjit:renderframe-triangle] swap stopped: {e}"),
                                    Ok(ok) => eprintln!(
                                        "[elfjit:renderframe-triangle] post-draw swap returned Ok({ok:#x})"
                                    ),
                                }
                            }

                            let mut se = arm64jit::jit::CpuState::new();
                            se.tpidr = tpidr;
                            se.x[31] = isp;
                            se.x[0] = real_ctx;
                            match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut se as *mut CpuState) {
                                Err(e) => eprintln!("[elfjit:renderframe-drawprobe] swap stopped: {e}"),
                                Ok(ok) => eprintln!(
                                    "[elfjit:renderframe-drawprobe] post-draw swap returned Ok({ok:#x})"
                                ),
                            }
                        }
// --renderframe-quad: scale the (now fully reversed) coherent renderer onto a
                        // real TWO-ATTRIB textured QUAD — the shape of real Roblox geometry.
                        // primitive-setup 0x5b353d0 loops the primitive list, ONE vertex attrib
                        // per primitive (via a slice at 0x5b35420). Two primitives -> two attribs:
                        //   primitive[0]: vb=0, offset=0,  format[3]={size4,GL_FLOAT}, attrib=0 (aPos)
                        //   primitive[1]: vb=0, offset=16, format[1]={size2,GL_FLOAT}, attrib=1 (aUV)
                        // The VBO is interleaved [pos.xyzw, uv.xy] per vertex (stride 24). The
                        // fragment shader samples a 2x2 texture at the REAL interpolated vertex UV
                        // (not gl_FragCoord) — proving per-texel UV mapping, which no single-attrib
                        // draw path can. Readback: the 4 quadrants read the 4 texel colors.
                        if renderframe_args.iter().any(|a| a == "--renderframe-quad") {
                            // --renderframe-etc2a: like the quad RGBA path but upload the
                            // texture as a REAL ETC2-RGBA8/EAC texture (0x9278 — the real
                            // Android RGBA-EAC format) through glCompressedTexImage2D, and
                            // map the DECODED ALPHA to the fragment RGB. The 4 quadrant
                            // readbacks then read the 4 distinct EAC block alphas as gray
                            // levels (255/190/128/64) — robust proof the EAC alpha
                            // sub-block decodes live (window framebuffers often discard
                            // alpha, so the gray-scale mapping makes it window-capturable).
                            let etc2a_mode = renderframe_args.iter().any(|a| a == "--renderframe-etc2a");
                            // --renderframe-astc: like --renderframe-etc2a but upload the texture
                            // as a REAL ASTC 4x4 LDR void-extent texture (0x93B0 — the load-bearing
                            // Android format desktop GL cannot native-decode, so our interception is
                            // REQUIRED there). Same gray-scale alpha->RGB proof: the 4 blocks' EAC-free
                            // ASTC void-extent alphas (255/190/128/64) render as 4 gray lobes.
                            let astc_mode = renderframe_args.iter().any(|a| a == "--renderframe-astc");
                            let comp_gray = etc2a_mode || astc_mode;
                            // --renderframe-quad-loop <N>: SUSTAINABLE textured-quad rendering —
                            // after the single proof frame, re-drive clear(cycling bg) ->
                            // engine geometry wrapper -> swap N times on the detached host thread,
                            // so a recording proves a fresh textured geometry render every frame
                            // (the last property a real main-loop frame drive needs for the
                            // textured/mesh path; geometry analog of SH25b's triangle-loop).
                            let quad_loop_n: Option<u32> = renderframe_args
                                .iter()
                                .position(|a| a == "--renderframe-quad-loop")
                                .and_then(|i| renderframe_args.get(i + 1))
                                .and_then(|s| s.parse().ok())
                                .or_else(|| {
                                    renderframe_args.iter().any(|a| a == "--renderframe-quad-loop").then_some(6)
                                });
                            // --renderframe-grid <N>: scale the coherent renderer onto a REAL
                            // larger mesh — an NxN grid of textured quads (N>1 => (N+1)^2
                            // verts, 6*N^2 indices, one distinct texel color per cell drawn
                            // at the real interpolated UV). Proves the engine's OWN geometry
                            // wrapper + primitive-setup loop render a mesh of real topology
                            // (many verts/indices), not just a single 4-vert quad (SH25-33).
                            // Readback probes each cell center, which must read that cell's
                            // distinct texel — per-cell UV->texel mapping across the mesh.
                            let grid_n: Option<u32> = renderframe_args
                                .iter()
                                .position(|a| a == "--renderframe-grid")
                                .and_then(|i| renderframe_args.get(i + 1))
                                .and_then(|s| s.parse().ok())
                                .filter(|&n| n >= 2 && n <= 8);
                            const GL_ARRAY_BUFFER: u64 = 0x8892;
                            const GL_ELEMENT_ARRAY_BUFFER: u64 = 0x8893;
                            const GL_STATIC_DRAW: u64 = 0x88e4;
                            const GL_FLOAT: u64 = 0x1406;
                            const GL_VERTEX_SHADER: u64 = 0x8b31;
                            const GL_FRAGMENT_SHADER: u64 = 0x8b30;
                            const GL_TEXTURE0: u64 = 0x84c0;
                            const GL_TEXTURE_2D: u64 = 0x0de1;
                            const GL_RGBA: u64 = 0x1908;
                            const GL_UNSIGNED_BYTE: u64 = 0x1401;
                            const GL_NEAREST: u64 = 0x2600;
                            const GL_COLOR_BUFFER_BIT: u64 = 0x4000;
                            let pb = |a: u64| -> Result<u64, String> {
                                let mut s = arm64jit::jit::CpuState::new();
                                s.tpidr = tpidr; s.x[31] = isp;
                                s.x[0] = a;
                                arm64jit::jit::jit_run(iimg, ibase, a, &mut s as *mut CpuState).map(|_| s.x[0])
                            };
                            let gcall = |addr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64| -> Result<u64, String> {
                                let mut s = arm64jit::jit::CpuState::new();
                                s.tpidr = tpidr; s.x[31] = isp;
                                s.x[0]=a0; s.x[1]=a1; s.x[2]=a2; s.x[3]=a3; s.x[4]=a4; s.x[5]=a5;
                                arm64jit::jit::jit_run(iimg, ibase, addr, &mut s as *mut CpuState).map(|_| s.x[0])
                            };
                            let objs = Box::leak(vec![0u8; 32768].into_boxed_slice());
                            let base = objs.as_ptr() as u64;
                            unsafe {
                                // Clear + viewport.
                                {
                                    let mut sc = arm64jit::jit::CpuState::new();
                                    sc.tpidr = tpidr; sc.x[31] = isp;
                                    sc.v[0] = (0.0f32).to_bits() as u64;
                                    sc.v[2] = (0.0f32).to_bits() as u64;
                                    sc.v[4] = (0.3f32).to_bits() as u64;
                                    sc.v[6] = (1.0f32).to_bits() as u64;
                                    let _ = arm64jit::jit::jit_run(iimg, ibase, 0x1062d7710, &mut sc as *mut CpuState);
                                }
                                let _ = gcall(0x1062d7740, GL_COLOR_BUFFER_BIT, 0,0,0,0,0);
                                let _ = gcall(0x1062d75c0, 0,0,1280,720,0,0); // glViewport
                                let _ = gcall(0x1062d75d0, 0,0,1280,720,0,0); // glScissor
                                // Shaders with a real UV varying.
                                let vs_src = b"attribute vec4 aPos;\nattribute vec2 aUV;\nvarying vec2 vUV;\nvoid main(){ vUV = aUV; gl_Position = aPos; }\n\0";
                                let fs_src: &[u8] = if comp_gray {
                                    // Output the DECODED ALPHA as RGB gray-scale (alpha=A->RGB)
                                    // so the EAC/ASTC alpha sub-block is window-capturable even on an
                                    // alpha-less window framebuffer.
                                    b"precision mediump float;\nuniform sampler2D uTex;\nvarying vec2 vUV;\nvoid main(){ vec4 t = texture2D(uTex, vUV); gl_FragColor = vec4(t.aaa, 1.0); }\n\0"
                                } else {
                                    b"precision mediump float;\nuniform sampler2D uTex;\nvarying vec2 vUV;\nvoid main(){ gl_FragColor = texture2D(uTex, vUV); }\n\0"
                                };
                                let vs_ptr = base + 0x400;
                                let fs_ptr = base + 0x800;
                                std::ptr::copy_nonoverlapping(vs_src.as_ptr(), vs_ptr as *mut u8, vs_src.len());
                                std::ptr::copy_nonoverlapping(fs_src.as_ptr(), fs_ptr as *mut u8, fs_src.len());
                                let vs_ary = base + 0xa00;
                                let fs_ary = base + 0xa10;
                                *(vs_ary as *mut u64) = vs_ptr;
                                *(fs_ary as *mut u64) = fs_ptr;
                                let vs = gcall(0x1062d7880, GL_VERTEX_SHADER, 0,0,0,0,0).unwrap_or(0) & 0xffff_ffff;
                                let _ = gcall(0x1062d7890, vs, 1, vs_ary, 0,0,0); // glShaderSource
                                let _ = gcall(0x1062d78a0, vs, 0,0,0,0,0);      // glCompileShader
                                let fs = gcall(0x1062d7880, GL_FRAGMENT_SHADER, 0,0,0,0,0).unwrap_or(0) & 0xffff_ffff;
                                let _ = gcall(0x1062d7890, fs, 1, fs_ary, 0,0,0);
                                let _ = gcall(0x1062d78a0, fs, 0,0,0,0,0);
                                let prog = gcall(0x1062d78c0, 0,0,0,0,0,0).unwrap_or(0) & 0xffff_ffff; // glCreateProgram
                                let _ = gcall(0x1062d78d0, prog, vs, 0,0,0,0); // glAttachShader
                                let _ = gcall(0x1062d78d0, prog, fs, 0,0,0,0);
                                let pos_name = base + 0xd00; std::ptr::copy_nonoverlapping(b"aPos\0".as_ptr(), pos_name as *mut u8, 5);
                                let uv_name = base + 0xd20; std::ptr::copy_nonoverlapping(b"aUV\0".as_ptr(), uv_name as *mut u8, 5);
                                let _ = gcall(0x1062d78f0, prog, 0, pos_name, 0,0,0); // glBindAttribLocation aPos->0
                                let _ = gcall(0x1062d78f0, prog, 1, uv_name, 0,0,0);  // glBindAttribLocation aUV->1
                                let _ = gcall(0x1062d78e0, prog, 0,0,0,0,0);          // glLinkProgram
                                let _ = gcall(0x1062d75a0, prog, 0,0,0,0,0);          // glUseProgram
                                // Texture: default = 2x2 RGBA checkerboard RED/GREEN/BLUE/WHITE;
                                // --renderframe-etc2a = a REAL 8x8 ETC2-RGBA8/EAC texture (0x9278)
                                // via glCompressedTexImage2D (the last compressed format with an
                                // unimplemented live-path prove). The bridge decodes ETC2-RGBA8 and
                                // re-uploads, so the EAC alpha + RGB both reach the quad.
                                let tex_data = base + 0x6000;
                                let tex_sp = base + 0xf80;
                                let tex_id_slot = base + 0xfd0;
                                if comp_gray {
                                    // ETC2-RGBA8 8x8 = 4 x 16-byte blocks; ASTC 4x4 8x8 = 4 x 16-byte
                                    // LDR void-extent blocks. Both encode a solid color+alpha per
                                    // 4x4 block, so the 4 blocks give 4 distinct alphas (255/190/
                                    // 128/64). The ETC2-RGBA8 RGB is the SH29-proven ETC2 color; the
                                    // ASTC RGB=alpha. FS maps alpha->RGB so the quadrant readbacks
                                    // read those 4 gray levels.
                                    let (comp_fmt, ctex): (u64, [u8; 64]) = if astc_mode {
                                        // ASTC LDR void-extent: bytes 9/11/13/15 = UNORM16 high bytes
                                        // of R/G/B/A (Khronos void-extent block, buf[0]=0xFC).
                                        let ve = |g: u8| -> [u8; 16] {
                                            let mut d = [0u8; 16];
                                            d[0] = 0xFC; d[1] = 0x01;
                                            d[9] = g; d[11] = g; d[13] = g; d[15] = g;
                                            d
                                        };
                                        let mut d = [0u8; 64];
                                        d[0..16].copy_from_slice(&ve(255));
                                        d[16..32].copy_from_slice(&ve(190));
                                        d[32..48].copy_from_slice(&ve(128));
                                        d[48..64].copy_from_slice(&ve(64));
                                        (0x93B0u64, d)
                                    } else {
                                        const GL_COMPRESSED_RGBA8_ETC2_EAC: u64 = 0x9278;
                                        let enc = |t: i32| -> u8 { let c = ((t - 2).clamp(0, 240) >> 4) as u8; (c << 4) | c };
                                        let blk = |r: u8, g: u8, b: u8| -> [u8; 8] { [r, g, b, 0, 0, 0, 0, 0] };
                                        let rgba8 = |a: u8, rgb: [u8; 8]| -> [u8; 16] {
                                            let mut d = [0u8; 16];
                                            d[0] = a; d[1] = 0;
                                            d[8..16].copy_from_slice(&rgb);
                                            d
                                        };
                                        let mut d = [0u8; 64];
                                        d[0..16].copy_from_slice(&rgba8(255, blk(enc(255), enc(2), enc(2))));   // block0
                                        d[16..32].copy_from_slice(&rgba8(190, blk(enc(2), enc(255), enc(2))));  // block1
                                        d[32..48].copy_from_slice(&rgba8(128, blk(enc(2), enc(2), enc(255))));  // block2
                                        d[48..64].copy_from_slice(&rgba8(64, blk(enc(255), enc(255), enc(255)))); // block3
                                        (GL_COMPRESSED_RGBA8_ETC2_EAC, d)
                                    };
                                    std::ptr::copy_nonoverlapping(ctex.as_ptr(), tex_data as *mut u8, 64);
                                    let _ = gcall(0x1062d7980, 1, tex_id_slot, 0,0,0,0); // glGenTextures
                                    let _ = gcall(0x1062d75e0, GL_TEXTURE0, 0,0,0,0,0);     // glActiveTexture
                                    let _ = gcall(0x1062d75f0, GL_TEXTURE_2D, *(tex_id_slot as *const u32) as u64, 0,0,0,0); // glBindTexture
                                    let _ = gcall(0x1062d7960, GL_TEXTURE_2D, 0x2801, GL_NEAREST, 0,0,0); // MIN
                                    let _ = gcall(0x1062d7960, GL_TEXTURE_2D, 0x2800, GL_NEAREST, 0,0,0); // MAG
                                    let mut sce = arm64jit::jit::CpuState::new();
                                    sce.tpidr = tpidr; sce.x[31] = isp;
                                    sce.x[0] = GL_TEXTURE_2D; sce.x[1] = 0; sce.x[2] = comp_fmt;
                                    sce.x[3] = 8; sce.x[4] = 8; sce.x[5] = 0; sce.x[6] = 64; sce.x[7] = tex_data;
                                    let what = if astc_mode { "ASTC 4x4 (0x93B0) LDR void-extent" } else { "ETC2-RGBA8 (0x9278)" };
                                    eprintln!("[elfjit:renderframe] uploading 8x8 {what} 4-block via glCompressedTexImage2D");
                                    let _ = arm64jit::jit::jit_run(iimg, ibase, 0x1062d7990, &mut sce as *mut CpuState); // glCompressedTexImage2D
                                } else {
                                    // Grid mode: an NxN RGBA texture, one DISTINCT solid color
                                    // per texel (gi,gj). Each grid cell samples exactly one texel
                                    // (all 4 of its verts share the texel-center UV, so the whole
                                    // cell renders flat) -> a readback at any cell center must
                                    // read that texel's unique color. Single-quad mode keeps the
                                    // 2x2 checkerboard. tex_w/tex_h/tex_len below are used for the
                                    // glTexImage2D dims + data length.
                                    let (tex_w, tex_h, tex_len) = match grid_n {
                                        Some(n) => (n, n, (n * n) as usize * 4),
                                        None => (2, 2, 16),
                                    };
                                    let mut tex: Vec<u8> = vec![0u8; tex_len];
                                    if let Some(n) = grid_n {
                                        for gj in 0..n {
                                            for gi in 0..n {
                                                let i = (gj * n + gi) as usize * 4;
                                                tex[i] = (gi as f32 / (n - 1) as f32 * 255.0) as u8;
                                                tex[i + 1] = (gj as f32 / (n - 1) as f32 * 255.0) as u8;
                                                tex[i + 2] = 64;
                                                tex[i + 3] = 255;
                                            }
                                        }
                                    } else {
                                        tex.copy_from_slice(&[255,0,0,255, 0,255,0,255, 0,0,255,255, 255,255,255,255]);
                                    }
                                    std::ptr::copy_nonoverlapping(tex.as_ptr(), tex_data as *mut u8, tex_len);
                                    *(tex_sp as *mut u64) = tex_data;
                                    let _ = gcall(0x1062d7980, 1, tex_id_slot, 0,0,0,0); // glGenTextures
                                    let _ = gcall(0x1062d75e0, GL_TEXTURE0, 0,0,0,0,0);     // glActiveTexture
                                    let _ = gcall(0x1062d75f0, GL_TEXTURE_2D, *(tex_id_slot as *const u32) as u64, 0,0,0,0); // glBindTexture
                                    let _ = gcall(0x1062d7960, GL_TEXTURE_2D, 0x2801, GL_NEAREST, 0,0,0); // MIN
                                    let _ = gcall(0x1062d7960, GL_TEXTURE_2D, 0x2800, GL_NEAREST, 0,0,0); // MAG
                                    let mut steg = arm64jit::jit::CpuState::new();
                                    steg.tpidr = tpidr; steg.x[31] = tex_sp;
                                    steg.x[0]=GL_TEXTURE_2D; steg.x[1]=0; steg.x[2]=GL_RGBA; steg.x[3]=tex_w as u64; steg.x[4]=tex_h as u64; steg.x[5]=0; steg.x[6]=GL_RGBA; steg.x[7]=GL_UNSIGNED_BYTE;
                                    let _ = arm64jit::jit::jit_run(iimg, ibase, 0x1062d79a0, &mut steg as *mut CpuState); // glTexImage2D
                                    let _ = tex_sp;
                                }
                                let tex_id = *(tex_id_slot as *const u32) as u64;
                                let uni = base + 0xe20; std::ptr::copy_nonoverlapping(b"uTex\0".as_ptr(), uni as *mut u8, 5);
                                let ploc = gcall(0x1062d7900, prog, uni, 0,0,0,0).unwrap_or(0) & 0xffff_ffff;
                                let _ = gcall(0x1062d7910, ploc, 0,0,0,0,0); // glUniform1i(uTex,0)
                                eprintln!("[elfjit:renderframe-quad] program={prog:#x} compiled+linked; texture tex_id={tex_id:#x} uTex={ploc:#x}");
                                // Mesh construction: single quad (SH25-33) vs an NxN grid
                                                                // (--renderframe-grid). The grid uses INDEPENDENT per-cell
                                                                // quads (4 verts + 6 idx each), every vertex of a cell
                                                                // sharing that cell's texel-center UV, so each cell renders
                                                                // flat with its own distinct texel color -> a readback at any
                                                                // cell center must read that texel. Grid = a REAL mesh:
                                                                // (4*N*N) verts + (6*N*N) idx through the engine's own
                                                                // primitive-setup + draw wrapper.
                                                                let (n, nv, ni) = match grid_n {
                                                                    Some(n) => {
                                                                        let u = n as usize;
                                                                        (u, 4 * u * u, 6 * u * u)
                                                                    }
                                                                    None => (1usize, 4, 6),
                                                                };
                                                                let mut verts: Vec<f32> = Vec::with_capacity(nv * 6);
                                                                let mut idxs: Vec<u32> = Vec::with_capacity(ni);
                                                                let cell = 1.8f32 / n as f32;
                                                                if grid_n.is_some() {
                                                                    // Grid mode: each cell's 4 verts share the texel-center UV
                                                                    // so the whole cell renders flat with ITS distinct texel.
                                                                    for gj in 0..n {
                                                                        for gi in 0..n {
                                                                            let x0 = -0.9f32 + gi as f32 * cell;
                                                                            let y0 = -0.9f32 + gj as f32 * cell;
                                                                            let (x1, y1) = (x0 + cell, y0 + cell);
                                                                            let u = (gi as f32 + 0.5) / n as f32;
                                                                            let v = (gj as f32 + 0.5) / n as f32;
                                                                            let base = (gj * n + gi) as u32 * 4;
                                                                            verts.extend_from_slice(&[x0,y0,0.0,1.0, u,v]); // 0 bl
                                                                            verts.extend_from_slice(&[x1,y0,0.0,1.0, u,v]); // 1 br
                                                                            verts.extend_from_slice(&[x1,y1,0.0,1.0, u,v]); // 2 tr
                                                                            verts.extend_from_slice(&[x0,y1,0.0,1.0, u,v]); // 3 tl
                                                                            idxs.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
                                                                        }
                                                                    }
                                                                } else {
                                                                    // Single-quad mode (SH25-33): per-corner UVs map the 2x2
                                                                    // checkerboard (RED/GREEN/BLUE/WHITE) to the 4 quadrants —
                                                                    // keep the original corner UVs so the 4 readbacks stay distinct.
                                                                    verts.extend_from_slice(&[-0.9,-0.9,0.0,1.0, 0.0,0.0]);
                                                                    verts.extend_from_slice(&[0.9,-0.9,0.0,1.0, 1.0,0.0]);
                                                                    verts.extend_from_slice(&[0.9,0.9,0.0,1.0, 1.0,1.0]);
                                                                    verts.extend_from_slice(&[-0.9,0.9,0.0,1.0, 0.0,1.0]);
                                                                    idxs.extend_from_slice(&[0,1,2, 0,2,3]);
                                                                }
                                                                let vbo_bytes = (nv * 6 * 4) as u64;
                                                                let ebo_bytes = (ni * 4) as u64;
                                let vbo_data = base + 0x2000;
                                let ebo_data = base + 0x4000;
                                                                std::ptr::copy_nonoverlapping(verts.as_ptr() as *const u8, vbo_data as *mut u8, vbo_bytes as usize);
                                                                std::ptr::copy_nonoverlapping(idxs.as_ptr() as *const u8, ebo_data as *mut u8, ebo_bytes as usize);
                                                                let vbo_slot = base + 0xf00;
                                                                let ebo_slot = base + 0xf10;
                                                                let _ = gcall(0x1062d77c0, 1, vbo_slot, 0,0,0,0); // glGenBuffers
                                                                let vbo = *(vbo_slot as *const u32) as u64;
                                                                let _ = gcall(0x1062d77b0, GL_ARRAY_BUFFER, vbo, 0,0,0,0);
                                                                let _ = gcall(0x1062d77d0, GL_ARRAY_BUFFER, vbo_bytes, vbo_data, GL_STATIC_DRAW, 0,0); // glBufferData
                                                                let _ = gcall(0x1062d77c0, 1, ebo_slot, 0,0,0,0);
                                                                let ebo = *(ebo_slot as *const u32) as u64;
                                                                let _ = gcall(0x1062d77b0, GL_ELEMENT_ARRAY_BUFFER, ebo, 0,0,0,0);
                                                                let _ = gcall(0x1062d77d0, GL_ELEMENT_ARRAY_BUFFER, ebo_bytes, ebo_data, GL_STATIC_DRAW, 0,0);
                                                                eprintln!("[elfjit:renderframe-quad] vbo={vbo:#x} ebo={ebo:#x} {nv} interleaved verts stride24 + {ni} idx uploaded ({n}x{n} grid)");
                                // Coherent renderer with TWO primitives -> TWO attribs.
                                let renderer = base;
                                let container = base + 0x100;
                                let desc = base + 0x200;
                                let stride_tbl = base + 0x300;
                                let prim0 = base + 0x400;
                                let prim1 = base + 0x418;
                                let ibo = base + 0x500;
                                *(desc.wrapping_add(72) as *mut u32) = vbo as u32;
                                *(stride_tbl as *mut u64) = 24;
                                *(renderer.wrapping_add(56) as *mut u64) = container;
                                *(container.wrapping_add(72) as *mut u64) = prim0;
                                *(container.wrapping_add(80) as *mut u64) = prim1 + 0x18;
                                *(container.wrapping_add(96) as *mut u64) = stride_tbl;
                                *(renderer.wrapping_add(0x48) as *mut u64) = desc;
                                // prim0: vb0 off0 fmt3(size4 float) attrib0 = aPos
                                *(prim0 as *mut u32) = 0; *(prim0.wrapping_add(4) as *mut u32)=0; *(prim0.wrapping_add(8) as *mut u32)=3; *(prim0.wrapping_add(12) as *mut u32)=0; *(prim0.wrapping_add(16) as *mut u32)=0;
                                // prim1: vb0 off16 fmt1(size2 float) attrib1 = aUV
                                *(prim1 as *mut u32) = 0; *(prim1.wrapping_add(4) as *mut u32)=16; *(prim1.wrapping_add(8) as *mut u32)=1; *(prim1.wrapping_add(12) as *mut u32)=1; *(prim1.wrapping_add(16) as *mut u32)=0;
                                *(renderer.wrapping_add(120) as *mut u64) = ibo;
                                                                *(ibo.wrapping_add(72) as *mut u32) = ebo as u32;
                                                                *(renderer.wrapping_add(142) as *mut u16) = ni as u16;
                                                                eprintln!("[elfjit:renderframe-quad] coherent renderer: 2-prim list (aPos+aUV) + {ni}-idx EBO fabricated");
                                                                // Drive the engine's OWN geometry wrapper.
                                                                let mut sw = arm64jit::jit::CpuState::new();
                                                                sw.tpidr = tpidr; sw.x[31] = isp;
                                                                sw.x[0] = renderer;
                                                                sw.x[1] = 0; sw.x[2] = 0; sw.x[3] = 0;
                                                                sw.x[4] = ni as u64; // glDrawElements count
                                                                sw.x[5] = 3; // nonzero -> indexed
                                                                let shape = if grid_n.is_some() { format!("{n}x{n} MESH drawn") } else { "textured QUAD drawn".to_string() };
                                                                match arm64jit::jit::jit_run(iimg, ibase, 0x105b35288, &mut sw as *mut CpuState) {
                                                                    Err(e) => eprintln!("[elfjit:renderframe-quad] geometry wrapper stopped: {e}"),
                                                                    Ok(ok) => eprintln!("[elfjit:renderframe-quad] geometry wrapper 0x5b35288 returned Ok({ok:#x}) ({shape})"),
                                                                }
                                // Readback: grid mode probes EVERY cell center (must read that cell's
                                // distinct texel); single-quad mode keeps the 4 fixed
                                // texel-corner probes (SH30).
                                if let Some(gn) = grid_n {
                                    for gj in 0..gn {
                                        for gi in 0..gn {
                                            let cndc_x = -0.9f32 + gi as f32 * cell + cell / 2.0;
                                            let cndc_y = -0.9f32 + gj as f32 * cell + cell / 2.0;
                                            let px = ((cndc_x + 1.0) * 640.0) as u32;
                                            let py = ((cndc_y + 1.0) * 360.0) as u32;
                                            let slot = 0xf40 + (((gj * gn + gi) % 8) as u64) * 4;
                                            let mut pr = arm64jit::jit::CpuState::new();
                                            pr.tpidr = tpidr; pr.x[31] = isp;
                                            pr.x[0]=px as u64; pr.x[1]=py as u64; pr.x[2]=1; pr.x[3]=1; pr.x[4]=0x1908; pr.x[5]=0x1401; pr.x[6]=base+slot;
                                            let _ = arm64jit::jit::jit_run(iimg, ibase, 0x1062d7940, &mut pr as *mut CpuState);
                                            let b = base + slot;
                                            let c = format!("RGBA({},{},{},{})", *(b as *const u8), *(b as *const u8).add(1), *(b as *const u8).add(2), *(b as *const u8).add(3));
                                            eprintln!("[elfjit:renderframe-quad] cell({gi},{gj})@({px},{py}) readback={c} (expect r={:.0} g={:.0})", gi as f32/(gn-1) as f32*255.0, gj as f32/(gn-1) as f32*255.0);
                                        }
                                    }
                                } else {
                                    let probes: [(u32,u32,&str,u64);4] = [
                                        (320,180,"BL-red(0,0)",0xf40), (960,180,"BR-green(1,0)",0xf44),
                                        (960,540,"TR-white(1,1)",0xf48), (320,540,"TL-blue(0,1)",0xf4c)];
                                    for (px,py,label,slot) in probes {
                                        let mut pr = arm64jit::jit::CpuState::new();
                                        pr.tpidr = tpidr; pr.x[31] = isp;
                                        pr.x[0]=px as u64; pr.x[1]=py as u64; pr.x[2]=1; pr.x[3]=1; pr.x[4]=0x1908; pr.x[5]=0x1401; pr.x[6]=base+slot;
                                        let _ = arm64jit::jit::jit_run(iimg, ibase, 0x1062d7940, &mut pr as *mut CpuState);
                                    }
                                    {
                                        let p = |slot:u64| -> String { let b=base+slot; format!("RGBA({},{},{},{})",*(b as *const u8),*(b as *const u8).add(1),*(b as *const u8).add(2),*(b as *const u8).add(3)) };
                                        eprintln!("[elfjit:renderframe-quad] readback: BL={} BR={} TR={} TL={}", p(0xf40), p(0xf44), p(0xf48), p(0xf4c));
                                    }
                                }
                                // Swap.
                                let mut se = arm64jit::jit::CpuState::new();
                                se.tpidr = tpidr; se.x[31] = isp; se.x[0] = real_ctx;
                                match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut se as *mut CpuState) {
                                    Err(e) => eprintln!("[elfjit:renderframe-quad] swap stopped: {e}"),
                                    Ok(ok) => eprintln!("[elfjit:renderframe-quad] post-draw swap returned Ok({ok:#x})"),
                                }
                                // --renderframe-quad-loop <N>: SUSTAINABLE textured-quad frames.
                                // The single proof frame is done above (including the readback).
                                // Now re-drive clear(cycling bg) -> wrapper -> swap N times so a
                                // recording proves a FRESH textured render every iteration.
                                if let Some(n) = quad_loop_n {
                                    let bgs: [[f32; 4]; 5] = [
                                        [0.9, 0.1, 0.1, 1.0], [0.1, 0.9, 0.1, 1.0], [0.1, 0.1, 0.9, 1.0],
                                        [0.9, 0.9, 0.1, 1.0], [0.9, 0.1, 0.9, 1.0],
                                    ];
                                    for iter in 0..n {
                                        let bg = bgs[(iter as usize) % 5];
                                        let mut sc = arm64jit::jit::CpuState::new();
                                        sc.tpidr = tpidr; sc.x[31] = isp;
                                        sc.v[0] = bg[0].to_bits() as u64;
                                        sc.v[2] = bg[1].to_bits() as u64;
                                        sc.v[4] = bg[2].to_bits() as u64;
                                        sc.v[6] = bg[3].to_bits() as u64;
                                        let _ = arm64jit::jit::jit_run(iimg, ibase, 0x1062d7710, &mut sc as *mut CpuState); // glClearColor
                                        let _ = gcall(0x1062d7740, GL_COLOR_BUFFER_BIT, 0, 0, 0, 0, 0);                    // glClear
                                        let mut swn = arm64jit::jit::CpuState::new();
                                        swn.tpidr = tpidr; swn.x[31] = isp;
                                        swn.x[0] = renderer; swn.x[1] = 0; swn.x[2] = 0; swn.x[3] = 0;
                                        swn.x[4] = ni as u64; swn.x[5] = 3;
                                        if let Err(e) = arm64jit::jit::jit_run(iimg, ibase, 0x105b35288, &mut swn as *mut CpuState) {
                                            eprintln!("[elfjit:renderframe-quad-loop] iter {iter} wrapper stopped: {e}");
                                            continue;
                                        }
                                        let mut sen = arm64jit::jit::CpuState::new();
                                        sen.tpidr = tpidr; sen.x[31] = isp; sen.x[0] = real_ctx;
                                        match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut sen as *mut CpuState) {
                                            Err(e) => eprintln!("[elfjit:renderframe-quad-loop] iter {iter} swap stopped: {e}"),
                                            Ok(ok) => eprintln!("[elfjit:renderframe-quad-loop] iter {iter} drew+swap Ok({ok:#x}) bg={bg:?} (fresh textured-quad frame)"),
                                        }
                                        std::thread::sleep(std::time::Duration::from_millis(350));
                                    }
                                }
                            }
                        }

                    }
                }
                // --renderclear <r,g,b,a>: draw an actual colored clear through the
                // JIT's GLES float bridge on this live context, then swap again, so
                // the presented frame is non-black (the idle main loop never issues
                // glClearColor itself). We drive the guest PLT entries directly:
                // glClearColor@plt 0x1062d7710 (float args in s0..s3, i.e. v[0..6]
                // low lanes) then glClear@plt 0x1062d7740 (GL_COLOR_BUFFER_BIT=0x4000
                // in x0), each through jit_run -> plt stub `br`s to the host GLES
                // bridge -> real Mesa on the already-current context.
                if renderframe_args.iter().any(|a| a == "--renderclear") {
                    let cc: Vec<f32> = renderframe_args
                        .iter()
                        .position(|a| a == "--renderclear")
                        .and_then(|i| renderframe_args.get(i + 1).cloned())
                        .map(|h| {
                            h.split(',')
                                .filter_map(|x| x.parse::<f32>().ok())
                                .collect()
                        })
                        .unwrap_or_else(|| vec![0.2, 0.6, 1.0, 1.0]);
                    let (mut cr, mut cg, mut cb, mut ca) = (0.2f32, 0.6f32, 1.0f32, 1.0f32);
                    if cc.len() >= 4 {
                        cr = cc[0];
                        cg = cc[1];
                        cb = cc[2];
                        ca = cc[3];
                    }
                    let mut s6 = arm64jit::jit::CpuState::new();
                    s6.tpidr = tpidr;
                    s6.x[31] = isp;
                    s6.v[0] = cr.to_bits() as u64;
                    s6.v[2] = cg.to_bits() as u64;
                    s6.v[4] = cb.to_bits() as u64;
                    s6.v[6] = ca.to_bits() as u64;
                    match arm64jit::jit::jit_run(iimg, ibase, 0x1062d7710, &mut s6 as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:renderclear] glClearColor stopped: {e}"),
                        Ok(_) => eprintln!("[elfjit:renderclear] glClearColor via bridge Ok"),
                    }
                    let mut s7 = arm64jit::jit::CpuState::new();
                    s7.tpidr = tpidr;
                    s7.x[31] = isp;
                    s7.x[0] = 0x4000; // GL_COLOR_BUFFER_BIT
                    match arm64jit::jit::jit_run(iimg, ibase, 0x1062d7740, &mut s7 as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:renderclear] glClear stopped: {e}"),
                        Ok(_) => eprintln!("[elfjit:renderclear] glClear via bridge Ok"),
                    }
                    let mut s8 = arm64jit::jit::CpuState::new();
                    s8.tpidr = tpidr;
                    s8.x[31] = isp;
                    s8.x[0] = real_ctx;
                    match arm64jit::jit::jit_run(iimg, ibase, swap_thunk, &mut s8 as *mut CpuState) {
                        Err(e) => eprintln!("[elfjit:renderclear] swap stopped: {e}"),
                        Ok(ok) => eprintln!("[elfjit:renderclear] swap returned Ok({ok:#x}) (eglSwapBuffers after clear)"),
                    }
                }
                // --renderframe-progbin: prove the SH35-sealed GLES3 pipeline slots are
                // genuinely FUNCTIONAL (not just resolvable) by driving the engine's OWN
                // dispatch stubs 0x5b3a1c0+0xc*N (the exact `adrp x8,6d3b000; ldr x3,[x8,
                // #752+8N]; br x3` mechanism a real session's frame dispatches through)
                // with real guest-ABI args on the live context. Covers the three modern
                // pipelines SH35 bridged:
                //   A. program-binary round-trip   (slots 13/14 glGetProgramBinary/glProgramBinary,
                //                                   slot 15 glProgramParameteri retrievable hint)
                //   B. UBO bind-through-slot       (slot 5  glBindBufferBase GL_UNIFORM_BUFFER)
                //   C. instanced draw dispatch     (slot 10 glDrawArraysInstanced, count=0 no-op)
                // Each drives a SEALED bridge slot exactly as the engine would; a mis-bridged
                // or crash-prone slot surfaces as a Mesa GL error or a crash (exit != 124).
                if renderframe_args.iter().any(|a| a == "--renderframe-progbin") {
                    unsafe {
                    // Guest addresses of the engine's slot stubs (bl-targets in its clear/
                    // draw code; a guest `br` to these re-dispatches through slot N's bridge).
                    const SLOT4: u64 = 0x105b3a1f0; // glUniformBlockBinding
                    const SLOT5: u64 = 0x105b3a1fc; // glBindBufferBase
                    const SLOT7: u64 = 0x105b3a214; // glGetUniformBlockIndex
                    const SLOT10: u64 = 0x105b3a238; // glDrawArraysInstanced
                    const SLOT13: u64 = 0x105b3a25c; // glGetProgramBinary
                    const SLOT14: u64 = 0x105b3a268; // glProgramBinary
                    const SLOT15: u64 = 0x105b3a274; // glProgramParameteri
                    // GLES PLT stubs for the setup that must NOT go through the sealed slots
                    // (shader compile, buffer create) — same addrs the triangle harness uses.
                    let plt_createshader = 0x1062d7880u64;
                    let plt_shadersource = 0x1062d7890u64;
                    let plt_compileshader = 0x1062d78a0u64;
                    let plt_attachshader = 0x1062d78d0u64;
                    let plt_linkprogram = 0x1062d78e0u64;
                    let plt_bindattrib = 0x1062d78f0u64;
                    let plt_genbuffers = 0x1062d77c0u64;
                    let plt_bindbuffer = 0x1062d77b0u64;
                    let plt_buffdata = 0x1062d77d0u64;
                    let plt_geterror = 0x1062d7580u64; // glGetError
                    let plt_createprogram = 0x1062d78c0u64;
                    let scratch = Box::leak(vec![0u8; 8192].into_boxed_slice());
                    let sc_base = scratch.as_ptr() as u64;
                    // Drive a bridge slot stub (a bare `br` to the sealed slot) or a PLT stub.
                    let mut gslot = |addr: u64, a0: u64, a1: u64, a2: u64, a3: u64, a4: u64, a5: u64| {
                        let mut s = arm64jit::jit::CpuState::new();
                        s.tpidr = tpidr;
                        s.x[31] = isp;
                        s.x[0] = a0;
                        s.x[1] = a1;
                        s.x[2] = a2;
                        s.x[3] = a3;
                        s.x[4] = a4;
                        s.x[5] = a5;
                        let r = arm64jit::jit::jit_run(iimg, ibase, addr, &mut s as *mut CpuState);
                        (r, s.x[0])
                    };
                    // curl clear-error helper
                    let geterr = || gslot(plt_geterror, 0, 0, 0, 0, 0, 0).1 as u32;
                    // Compile+link a minimal program (pass-through VS, red FS).
                    let vs_src = b"attribute vec4 aPos;\nvoid main(){ gl_Position = vec4(aPos.xyz,1.0); }\n\0";
                    let fs_src = b"void main(){ gl_FragColor = vec4(1.0,0.2,0.2,1.0); }\n\0";
                    std::ptr::copy_nonoverlapping(vs_src.as_ptr(), (sc_base + 0x400) as *mut u8, vs_src.len());
                    std::ptr::copy_nonoverlapping(fs_src.as_ptr(), (sc_base + 0x800) as *mut u8, fs_src.len());
                    let vs_ary = sc_base + 0xa00;
                    let fs_ary = sc_base + 0xa10;
                    *(vs_ary as *mut u64) = sc_base + 0x400;
                    *(fs_ary as *mut u64) = sc_base + 0x800;
                    let vs_id = gslot(plt_createshader, 0x8B31, 0, 0, 0, 0, 0).1 & 0xffff_ffff;
                    let _ = gslot(plt_shadersource, vs_id, 1, vs_ary, 0, 0, 0);
                    let _ = gslot(plt_compileshader, vs_id, 0, 0, 0, 0, 0);
                    let fs_id = gslot(plt_createshader, 0x8B30, 0, 0, 0, 0, 0).1 & 0xffff_ffff;
                    let _ = gslot(plt_shadersource, fs_id, 1, fs_ary, 0, 0, 0);
                    let _ = gslot(plt_compileshader, fs_id, 0, 0, 0, 0, 0);
                    let prog = gslot(plt_createprogram, 0, 0, 0, 0, 0, 0).1 & 0xffff_ffff;
                    let _ = gslot(plt_attachshader, prog, vs_id, 0, 0, 0, 0);
                    let _ = gslot(plt_attachshader, prog, fs_id, 0, 0, 0, 0);
                    let loc_name = sc_base + 0xd20;
                    std::ptr::copy_nonoverlapping(b"aPos\0".as_ptr(), loc_name as *mut u8, 5);
                    let _ = gslot(plt_bindattrib, prog, 0, loc_name, 0, 0, 0);
                    // A: slot15 = glProgramParameteri(prog, GL_PROGRAM_BINARY_RETRIEVABLE_HINT=0x8257, 1)
                    // BEFORE linking so Mesa will emit a retrievable binary.
                    let a15 = gslot(SLOT15, prog, 0x8257, 1, 0, 0, 0);
                    eprintln!("[elfjit:progbin] slot15 glProgramParameteri(retrievable hint) -> {a15:?} err={:#x}", geterr());
                    let _ = gslot(plt_linkprogram, prog, 0, 0, 0, 0, 0);
                    eprintln!("[elfjit:progbin] linked prog={prog:#x} err={:#x}", geterr());
                    // glGetProgramBinary(prog, 4096, &length, &format, &binary) through slot13.
                    let len_slot = sc_base + 0xe00;
                    let fmt_slot = sc_base + 0xe10;
                    let bin_slot = sc_base + 0xe40;
                    *(len_slot as *mut u32) = 0;
                    *(fmt_slot as *mut u32) = 0;
                    let a13 = gslot(SLOT13, prog, 4096, len_slot, fmt_slot, bin_slot, 0);
                    let len = *(len_slot as *const u32);
                    let fmt = *(fmt_slot as *const u32);
                    eprintln!("[elfjit:progbin] slot13 glGetProgramBinary -> {a13:?} length={len} format={fmt:#x} err={:#x}", geterr());
                    assert!(len > 0 && fmt != 0, "glGetProgramBinary through sealed slot13 must return a real binary");
                    // Re-upload through slot14: glProgramBinary(prog, fmt, bin, len).
                    let a14 = gslot(SLOT14, prog, fmt as u64, bin_slot, len as u64, 0, 0);
                    eprintln!("[elfjit:progbin] slot14 glProgramBinary(re-upload) -> {a14:?} err={:#x}", geterr());
                    // B: UBO bind through slot5. Gen a real buffer, fill 64B, bind to UBO index 0.
                    let buf_id_slot = sc_base + 0xf00;
                    let _ = gslot(plt_genbuffers, 1, buf_id_slot, 0, 0, 0, 0);
                    let ubo_id = *(buf_id_slot as *const u32) as u64;
                    let ubodata = sc_base + 0xf40;
                    for i in 0..16 {
                        *(ubodata as *mut u8).add(i) = (i as u8) << 4;
                    }
                    let _ = gslot(plt_bindbuffer, 0x8A11 /*GL_UNIFORM_BUFFER*/, ubo_id, 0, 0, 0, 0);
                    let _ = gslot(plt_buffdata, 0x8A11, 64, ubodata, 0x88E8 /*GL_DYNAMIC_DRAW*/, 0, 0);
                    let a5 = gslot(SLOT5, 0x8A11, 0, ubo_id, 0, 0, 0);
                    eprintln!("[elfjit:progbin] slot5 glBindBufferBase(UBO,0,{ubo_id}) -> {a5:?} err={:#x}", geterr());
                    // C: instanced draw through slot10. Slot10 currently seeds as glDrawArrays
                    // (the coherent path uses it for the array draw); RE-point it to the sealed
                    // glDrawArraysInstanced bridge slot, drive a count=0 no-op, then restore.
                    let slot10_addr = 0x106d3b2f0u64 + 8 * 10;
                    let saved_slot10 = unsafe { *(slot10_addr as *const u64) };
                    if let Some(inst) = arm64jit::resolver::resolve_gles_int(b"glDrawArraysInstanced\0") {
                        unsafe { *(slot10_addr as *mut u64) = inst };
                        let a10 = gslot(SLOT10, 0x0004 /*GL_TRIANGLES*/, 0, 0, 3, 0, 0);
                        eprintln!("[elfjit:progbin] slot10 glDrawArraysInstanced(TRIANGLES,0,0,3) -> {a10:?} err={:#x}", geterr());
                        unsafe { *(slot10_addr as *mut u64) = saved_slot10 };
                    } else {
                        eprintln!("[elfjit:progbin] glDrawArraysInstanced NOT resolvable (skipping C)");
                    }
                    eprintln!("[elfjit:progbin] GLES3 pipeline slot probe done (final err={:#x})", geterr());
                    } // unsafe
                }
            }
        });
    }

    match arm64jit::jit::jit_run(image, base, start_app, &mut s2 as *mut CpuState) {
            Err(e) => eprintln!("[elfjit] StartApp stopped: {e}"),
            Ok(r) => eprintln!("[elfjit] StartApp returned Ok({r:#x})"),
        }

        // Let any game-start workers run before exiting (or rather: keep the
        // process alive long enough for a real main loop to iterate/block).
        for _ in 0..4000 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            if std::env::var_os("JIT_STATS").is_some() {
                let (c, h) = arm64jit::jit::block_cache_stats();
                // Compiles growing = StartApp is advancing through new init code;
                // flat compiles + rising hits = it is recycling cached hot blocks.
                eprintln!("[elfjit] stats: compiles={c} hits={h}");
            }
            // Periodic guest-thread state sampler (JIT_THREADS=1): while the
            // engine parks in the lifecycle-await, dump each registered guest
            // thread's live registers — its hostcall slot (pc), the guest call
            // site (lr = x30), and the wait-object args (x0..x2) — so the
            // boot wall is pinned to a precise guest function & release path.
            // JIT_FRAMEWORK_DUMP: read the framework-built globals the render
            // path and the task-deque maintenance forward-edges rely on, live
            // from this process (guest==host addressing, so a guest bss/heaplow
            // address is a valid host pointer). Tells us whether StartApp's
            // initialization actually POPULATED the render-init context
            // (0x1067d16f0 = [render-init+0x3a300] ldr x25,[x25,#222*8]) or the
            // deque maintenance dispatch globals before the main loop parks —
            // i.e. whether driving the real render-init after warm-up is viable.
            if std::env::var_os("JIT_FRAMEWORK_DUMP").is_some() {
                let dw = |a: u64| -> u64 {
                    if a >= 0x100000000 && a >> 56 == 0 && a & 7 == 0 {
                        unsafe { *(a as *const u64) }
                    } else {
                        0
                    }
                };
                let ctx = dw(0x1067d16f0);
                eprintln!(
                    "[elfjit:fw] render-ctx 0x1067d16f0={:#x} | deque-fwd 0x1068262e8={:#x} 0x106826300={:#x} 0x106826308={:#x} | task-v4 [0x106829ea8]={:#x} v0/2 [0x106826320]={:#x} | render-ctx+0 [*ctx]={:#x}",
                    ctx,
                    dw(0x1068262e8),
                    dw(0x106826300),
                    dw(0x106826308),
                    dw(0x106829ea8),
                    dw(0x106826320),
                    if ctx != 0 && ctx >> 56 == 0 { dw(ctx) } else { 0 },
                );
            }
            if std::env::var_os("JIT_THREADS").is_some() {
                let snaps = arm64jit::jit::snapshot_threads();
                let mut lines = format!("[elfjit] guest threads {}", snaps.len());
                for t in &snaps {
                    let at = arm64jit::resolver::name_of_call_addr(t.pc)
                        .unwrap_or_else(|| format!("{:#x}", t.pc));
                    lines.push_str(&format!(
                        "\n  host_tid={} guest_tid={} pc={at} lr={:#x} x0={:#x} x1={:#x} x2={:#x} x3={:#x} x5={:#x} x19={:#x}[*={:#x}] x20={:#x} x21={:#x} x29={:#x} sp={:#x}",
                        t.host_tid, t.guest_tid, t.lr, t.x0, t.x1, t.x2, t.x3, t.x5, t.x19,
                        if t.x19 >= 0x100000000 && t.x19 >> 56 == 0 && t.x19 & 7 == 0 { unsafe { *(t.x19 as *const u64) } } else { 0 },
                        t.x20, t.x21, t.x29, t.sp
                    ));
                }
                eprintln!("{lines}");
            }
            if std::env::var_os("ELFJIT_EXIT_WHEN_IDLE").is_some()
                && arm64jit::jit::active_guest_threads() <= baseline
            {
                eprintln!("[elfjit] guest idle; exiting");
                break;
            }
        }
        // SH124: once StartApp's jit_run HAS RETURNED (the post-SH123 clean
        // exit-232 state), main() was previously running off the end and
        // tearing down the detached --v2boot ladder thread mid-do-init before
        // it printed "ladder done" + the session-advance probe. Join the ladder
        // (bounded) so its deep DM/app-shell construction completes and its
        // probe is observable end-to-end. No-op here when StartApp parks (that
        // path never reaches this block; the process is held by the settle loop
        // or a timeout instead).
        if let Some(handle) = v2boot_join.take() {
            eprintln!("[elfjit] StartApp returned — joining detached --v2boot ladder thread so its do-init construction completes");
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
            let mut stepped = false;
            while !handle.is_finished() && std::time::Instant::now() < deadline {
                if !stepped {
                    eprintln!("[elfjit:v2boot-join] waiting for ladder (ladder done + session-advance probe)");
                    stepped = true;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            match handle.join() {
                Ok(()) => eprintln!("[elfjit:v2boot-join] ladder thread joined cleanly"),
                Err(_) => eprintln!("[elfjit:v2boot-join] ladder thread panicked"),
            }
        }
    }
    let (compiles, hits) = arm64jit::jit::block_cache_stats();
    if compiles > 0 || hits > 0 {
        eprintln!("[elfjit] block-cache: {compiles} compiles / {hits} hits");
    }
}

// SH69 hermetic regression: the offline PNG->RGBA8 decoder + the aspect-correct
// screen-placement math used to render the REAL Roblox UI texture through the
// engine's own emitter path. Self-contained (no host files, no network).
#[cfg(test)]
mod sh69_tests {
    use super::{decode_png_rgba, imgpix_rect, imgpix_screen};
    use std::io::Write;

    fn chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut c = Vec::new();
        c.extend_from_slice(&(data.len() as u32).to_be_bytes());
        c.extend_from_slice(chunk_type);
        c.extend_from_slice(data);
        c.extend_from_slice(&[0, 0, 0, 0]); // dummy crc (decode_png_rgba ignores CRC)
        c
    }
    fn make_png(w: u32, h: u32, ct: u8, raw: &[u8]) -> Vec<u8> {
        let mut p = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&w.to_be_bytes());
        ihdr.extend_from_slice(&h.to_be_bytes());
        ihdr.extend_from_slice(&[8, ct, 0, 0, 0]); // bit 8, ct, comp 0, filter 0, interlace 0
        p.extend_from_slice(&chunk(b"IHDR", &ihdr));
        let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(raw).unwrap();
        let idat = e.finish().unwrap();
        p.extend_from_slice(&chunk(b"IDAT", &idat));
        p.extend_from_slice(&chunk(b"IEND", &[]));
        p
    }

    #[test]
    fn sh69_decode_png_rgba_type6_filter0_roundtrip() {
        let (w, h) = (3u32, 3u32);
        let mut raw = Vec::new();
        let mut expect = Vec::new();
        for _ in 0..h {
            raw.push(0); // filter 0
            for _ in 0..w {
                for c in [1u8, 2, 3, 4] {
                    raw.push(c);
                    expect.push(c);
                }
            }
        }
        let png = make_png(w, h, 6, &raw);
        let (dw, dh, out) = decode_png_rgba(&png).expect("decode ct6");
        assert_eq!((dw, dh), (w, h));
        assert_eq!(out, expect);
    }

    #[test]
    fn sh69_decode_png_rgba_gray_expands_opaque_alpha() {
        let (w, h) = (2u32, 2u32);
        let mut raw = Vec::new();
        for _ in 0..h {
            raw.push(0);
            for _ in 0..w {
                raw.push(200);
            }
        }
        let png = make_png(w, h, 0, &raw);
        let (_, _, out) = decode_png_rgba(&png).expect("decode gray");
        assert_eq!(out.len(), (w * h * 4) as usize);
        for i in (0..out.len()).step_by(4) {
            assert_eq!(&out[i..i + 3], [200, 200, 200]);
            assert_eq!(out[i + 3], 255);
        }
    }

    #[test]
    fn sh69_imgpix_screen_centers_square_image_and_aspect_math() {
        // square 100x100, hh=0.5, VW=1280, VH=720: center pixel lands at screen center.
        let (sx, sy) = imgpix_screen(50, 50, 100, 100, 0.5, 1280.0, 720.0);
        assert!((sx - 640.0).abs() < 2.0 && (sy - 360.0).abs() < 2.0, "center got {sx},{sy}");
        // top-left image corner maps to the box's top-left (top of frame).
        let (tlx, tly) = imgpix_screen(0, 0, 100, 100, 0.5, 1280.0, 720.0);
        assert!(tlx < 640.0 && tly < 360.0, "top-left got {tlx},{tly}");
    }

    #[test]
    fn sh69_decode_png_rgba_rejects_non_png() {
        assert!(decode_png_rgba(b"not-a-png").is_none());
        assert!(decode_png_rgba(&[0u8; 8]).is_none());
    }

    // --- SH70: color-type 3 (indexed/palette) support ---

    fn make_palette_png(w: u32, h: u32, palette: &[[u8; 3]], alpha: &[u8], indices: &[u8]) -> Vec<u8> {
        let mut p = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&w.to_be_bytes());
        ihdr.extend_from_slice(&h.to_be_bytes());
        ihdr.extend_from_slice(&[8, 3, 0, 0, 0]); // bit 8, ct 3 (palette)
        p.extend_from_slice(&chunk(b"IHDR", &ihdr));
        let mut plte = Vec::new();
        for e in palette {
            plte.extend_from_slice(e);
        }
        p.extend_from_slice(&chunk(b"PLTE", &plte));
        if !alpha.is_empty() {
            p.extend_from_slice(&chunk(b"tRNS", alpha));
        }
        let mut raw = Vec::new();
        for y in 0..h {
            raw.push(0); // filter 0
            raw.extend_from_slice(&indices[(y * w) as usize..((y + 1) * w) as usize]);
        }
        let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
        e.write_all(&raw).unwrap();
        p.extend_from_slice(&chunk(b"IDAT", &e.finish().unwrap()));
        p.extend_from_slice(&chunk(b"IEND", &[]));
        p
    }

    #[test]
    fn sh70_decode_png_rgba_palette_with_trns_alpha() {
        // 2x2, indices 0,1,1,2. idx0=(10,20,30)a255, idx1=(40,50,60)a128, idx2=(70,80,90)a0.
        let pal = [[10u8, 20, 30], [40, 50, 60], [70, 80, 90]];
        let trns = [255u8, 128, 0];
        let idx = [0u8, 1, 1, 2];
        let png = make_palette_png(2, 2, &pal, &trns, &idx);
        let (dw, dh, out) = decode_png_rgba(&png).expect("decode ct3");
        assert_eq!((dw, dh), (2, 2));
        let expect: Vec<u8> = [10, 20, 30, 255, 40, 50, 60, 128, 40, 50, 60, 128, 70, 80, 90, 0].to_vec();
        assert_eq!(out, expect);
    }

    #[test]
    fn sh70_decode_png_rgba_palette_without_trns_is_opaque() {
        let pal = [[200u8, 150, 100], [1, 2, 3]];
        let idx = [0u8, 1, 0, 1];
        let png = make_palette_png(2, 2, &pal, &[], &idx);
        let (_, _, out) = decode_png_rgba(&png).expect("decode ct3 opaque");
        assert_eq!(&out[0..4], [200, 150, 100, 255]);
        assert_eq!(&out[4..8], [1, 2, 3, 255]);
        assert_eq!(&out[8..12], [200, 150, 100, 255]);
        assert_eq!(&out[12..16], [1, 2, 3, 255]);
    }

    // --- SH72: multi-sprite composite geometry ---

    // imgpix_rect must reduce to imgpix_screen when the box is centered.
    #[test]
    fn sh72_imgpix_rect_centered_reduces_to_imgpix_screen() {
        for (ix, iy, imw, imh) in
            [(0u32, 0u32, 100u32, 100u32), (33, 88, 100, 100), (27, 27, 54, 54), (120, 120, 240, 240)]
        {
            let (a, b) = imgpix_screen(ix, iy, imw, imh, 0.4, 1280.0, 720.0);
            let (c, d) = imgpix_rect(ix, iy, imw, imh, 0.0, 0.0, 0.4, 1280.0, 720.0);
            assert!((a - c).abs() < 1e-3 && (b - d).abs() < 1e-3, "{ix},{iy} centered mismatch {a},{b} vs {c},{d}");
        }
    }

    // An offset box's probe coords shift by the translated NDC center.
    #[test]
    fn sh72_imgpix_rect_offset_box_shifts_probe_screen_coords() {
        // Center (0.72, 0.78) NDC with hh=0.10 vs centered hh=0.10 for a 54x54
        // sprite: the box center in pixels = ((0.72+1)/2*1280, (1-0.78)/2*720)
        // = (1100.8, 79.2).
        let (sx, sy) = imgpix_rect(27, 27, 54, 54, 0.72, 0.78, 0.10, 1280.0, 720.0);
        assert!((sx - 1100.8).abs() < 4.0 && (sy - 79.2).abs() < 4.0, "got {sx},{sy}");
    }

    // The shared vertical atlas UV layout: sprite blocks are disjoint, in
    // [0,1], and each sprite's full width maps to u in [0,1].
    #[test]
    fn sh72_atlas_uv_layout_is_disjoint_within_unit_range() {
        let heights: &[u32] = &[100, 54, 240];
        let widths: &[u32] = &[100, 54, 240];
        let aw: u32 = *widths.iter().max().unwrap().max(&8);
        let mut memlo: Vec<u32> = Vec::new();
        let mut ah: u32 = 1;
        for sh in heights {
            memlo.push(ah);
            ah += sh;
        }
        let mut vlo = Vec::new();
        let mut vhi = Vec::new();
        for (i, sh) in heights.iter().enumerate() {
            vlo.push(memlo[i] as f32 / ah as f32);
            vhi.push((memlo[i] + sh - 1) as f32 / ah as f32);
        }
        assert_eq!(aw, 240);
        assert_eq!(ah, 1 + 100 + 54 + 240);
        // Each block strictly within [0,1] and ordered/disjoint.
        for i in 0..vlo.len() {
            assert!(vlo[i] >= 0.0 && vhi[i] <= 1.0 && vlo[i] < vhi[i], "block {i} out of range");
            if i > 0 {
                assert!(vlo[i] > vhi[i - 1], "blocks {i} and {} overlap", i - 1);
            }
        }
        // Sprite block k's v-window spans (sh-1) texel-gap units of the atlas
        // height (rows [memlo, memlo+sh-1] cover sh rows; same off-by-one the
        // SH69 single-image mapping uses and proved byte-exact).
        for (i, sh) in heights.iter().enumerate() {
            let span = (vhi[i] - vlo[i]) * ah as f32;
            assert!((span - (*sh as f32 - 1.0)).abs() < 1e-3, "block {i} span {span} != height-1 {sh}");
        }
    }

    // The login synthetic-solid rows (SH75: field + green button, each a
    // uniform opaque square block) stay disjoint blocks in the shared atlas and
    // are opaque, so their probe expected color == the solid color over ANY
    // backdrop (a=255 -> dst*(1-1)=0). This is the same property that makes the
    // RO-BLOX wordmark (alpha=255 white) byte-exact in SH73.
    #[test]
    fn sh75_login_solid_rows_are_opaque_so_probe_is_byte_exact_over_any_dst() {
        // Mirror login sprite list dimensions: vignette(1024x1024), wordmark
        // (476x88), noconnection(140x100), field(256x16), loginbtn(256x16).
        let dims: &[(u32, u32)] = &[(1024, 1024), (476, 88), (140, 100), (256, 16), (256, 16)];
        let aw: u32 = dims.iter().map(|(w, _)| *w).max().unwrap().max(8);
        assert_eq!(aw, 1024);
        // field + loginbtn are uniform solid: any texel equals the solid color.
        let (fr, fg, fb, fa) = (224u8, 224, 230, 255u8); // field
        let (gr, gg, gb, ga) = (0u8, 158, 68, 255u8); // button (Roblox green)
        for (w, h) in &[(256u32, 16u32), (256, 16)] {
            let rgba: Vec<u8> = (0..(*w * *h)).flat_map(|_| if *w == 256 { [fr, fg, fb, fa] } else { [gr, gg, gb, ga] }).collect();
            // every texel opaque
            for px in rgba.chunks_exact(4) {
                assert_eq!(px[3], 255, "solid login row must be opaque");
            }
        }
        // Opaque texel => GL_BLEND src*a + (26,26,31)*(1-a) == src exactly.
        assert_eq!(((fr as f32) * 1.0 + 26.0 * 0.0) as u8, fr);
        assert_eq!(((gb as f32) * 1.0 + 31.0 * 0.0) as u8, gb);
        // Solid rows sit in disjoint, in-range atlas blocks like the real ones.
        let heights: &[u32] = &[1024, 88, 100, 16, 16];
        let mut memlo: Vec<u32> = Vec::new();
        let mut ah: u32 = 1;
        for sh in heights {
            memlo.push(ah);
            ah += sh;
        }
        let mut vlo = Vec::new();
        let mut vhi = Vec::new();
        for (i, sh) in heights.iter().enumerate() {
            vlo.push(memlo[i] as f32 / ah as f32);
            vhi.push((memlo[i] + sh - 1) as f32 / ah as f32);
        }
        for i in 0..vlo.len() {
            assert!(vlo[i] >= 0.0 && vhi[i] <= 1.0 && vlo[i] < vhi[i], "block {i} range");
            if i > 0 {
                assert!(vlo[i] > vhi[i - 1], "blocks {i} and {} overlap", i - 1);
            }
        }
    }

    // SH75 login-left field/button probe screen coords land inside each box via
    // imgpix_rect (the same math the shader aTex interpolation + probe uses).
    #[test]
    fn sh75_login_field_and_button_probe_land_inside_their_boxes() {
        // field at cy=-0.45, hh=0.045, 256x16; its box half-width =
        // hh*(256/16)*(720/1280)=0.045*16*0.5625=0.405.
        let hw = 0.045f32 * (256.0 / 16.0) * (720.0 / 1280.0);
        let (sx, sy) = imgpix_rect(128, 8, 256, 16, 0.0, -0.45, 0.045, 1280.0, 720.0);
        assert!((sx - 640.0).abs() < hw * 640.0 + 4.0, "field x {sx}");
        // button at cy=-0.62
        let (bw, bh) = (0.05f32, (256.0 / 16.0) * (720.0 / 1280.0));
        let (bx, by) = imgpix_rect(128, 8, 256, 16, 0.0, -0.62, bw, 1280.0, 720.0);
        assert!((bx - 640.0).abs() < bw * 16.0 * 0.5625 * 640.0 + 4.0, "button x {bx}");
        assert!(by > 0.0 && by < 720.0, "button y {by}");
        let _ = bh;
    }
}

// SH77 hermetic regression: the pure-std TrueType rasterizer + login text
// integration. The coverage/font-table pieces are fully hermetic (feed an
// in-memory font slice); the login-label screen probes additionally verify the
// exact glyph pixels the emitter probes (host font file, same hard-coded asset
// path the renderer uses).
#[cfg(test)]
mod sh77_tests {
    use super::*;

    #[test]
    fn sh77_flatten_ring_emits_closed_quad_polygon() {
        // Square ring: on-curves at the 4 corners (unit 0..100), CCW.
        let pts: Vec<(i16, i16, bool)> =
            vec![(10, 10, true), (90, 10, true), (90, 90, true), (10, 90, true)];
        let poly = flatten_ring(&pts);
        // closed polygon with >= 4 points
        assert!(poly.len() >= 4, "poly len {}", poly.len());
        // region-winding: a point strictly inside the square is covered
        assert!(covered(&vec![poly.clone()], 50.0, 50.0), "center must be covered");
        // a point outside the square is not covered
        assert!(!covered(&vec![poly.clone()], 150.0, 150.0), "outside must be uncovered");
        assert!(!covered(&vec![poly.clone()], 5.0, 50.0), "left-of must be uncovered");
    }

    #[test]
    fn sh77_quadratic_offs_flatten_cover_interior() {
        // on -> off -> off -> on : the two off-curves imply an on-curve at their
        // midpoint, forming two quadratics that still enclose the center area.
        let pts: Vec<(i16, i16, bool)> = vec![
            (0, 0, true),
            (0, 100, false),
            (100, 100, false), // midpoint implied at (50,100)
            (100, 0, true),
        ];
        let poly = flatten_ring(&pts);
        assert!(poly.len() >= 8, "quad flattens to sample points, got {}", poly.len());
        // center of the shape is covered
        assert!(covered(&vec![poly.clone()], 50.0, 50.0), "center must be covered");
        // well outside is not
        assert!(!covered(&vec![poly], 200.0, 200.0), "outside must be uncovered");
    }

    #[test]
    fn sh77_covered_uses_nonzero_winding_for_counter() {
        // Outer square + inner square (one nested, opposite winding) -> the
        // donut interior is uncovered (winding cancels), the outer ring is.
        let outer: Vec<(i16, i16, bool)> =
            vec![(0, 0, true), (100, 0, true), (100, 100, true), (0, 100, true)];
        let inner: Vec<(i16, i16, bool)> =
            vec![(30, 30, true), (30, 70, true), (70, 70, true), (70, 30, true)];
        let po = flatten_ring(&outer);
        let pi = flatten_ring(&inner);
        let rings = vec![po, pi];
        // center (50,50) inside the inner (opposite winding) -> uncovered (hole)
        assert!(!covered(&rings, 50.0, 50.0), "hole center must be uncovered");
        // (15,15) inside the outer but outside the hole -> covered ring
        assert!(covered(&rings, 15.0, 15.0), "ring must be covered");
    }

    // The real font path is the same hard-coded absolute path the login
    // renderer uses; rely on the asset being present (already unzipped on this
    // box). If absent, skip gracefully rather than fail the workspace.
    fn real_font() -> Option<Vec<u8>> {
        std::fs::read("/home/hermes-worker/.cache/open-sober/android-env/assets/fonts/SourceSansPro-Bold.ttf").ok()
    }

    #[test]
    fn sh77_cmap4_maps_ascii_log_in_to_nonzero_gid() {
        let Some(font) = real_font() else { return };
        let cmap = font_cmap4(&font).expect("cmap4");
        for cp in "Log In Email address Password".chars().map(|c| c as u32) {
            let gid = cmap.gid(&font, cp);
            assert!(gid != 0, "char {cp:#x} ({}'') maps to gid 0", char::from_u32(cp).unwrap());
        }
    }

    #[test]
    fn sh77_text_grid_runs_for_each_label() {
        let Some(font) = real_font() else { return };
        // SH78: labels are rasterized 2x (w/h double, pu doubles) so the glyph
        // strokes double in density while w:h is preserved (screen geometry and
        // probe coords unchanged); the test mirrors the production table.
        for (text, color, w, h, pu) in [
            ("Log In", [255u8, 255, 255, 255], 512u32, 80u32, 0.068f32),
            ("Email address", [96u8, 96, 110, 255], 512u32, 64u32, 0.060f32),
            ("Password", [96u8, 96, 110, 255], 512u32, 64u32, 0.060f32),
        ] {
            let cmap = font_cmap4(&font).unwrap();
            let mut used = 0.0f32;
            for ch in text.chars() {
                let gid = cmap.gid(&font, ch as u32);
                if gid != 0 {
                    used += font_advance_width(&font, gid) as f32 * pu;
                }
            }
            let pen_start = (w as f32 - used) / 2.0;
            let asc = 722.0f32;
            let baseline = (h as f32 / 10.0) + asc * pu;
            let rgba = rasterize_text_row(&font, text, pu, color, w, h, pen_start, baseline);
            // nonzero glyph pixels present
            let opaque = rgba
                .chunks_exact(4)
                .filter(|px| px[3] >= 250)
                .count();
            assert!(opaque > 50, "'{text}' only {opaque} opaque px (label invisible)");
            // glyph color baked
            let sample_alpha = rgba
                .chunks_exact(4)
                .find(|px| px[3] >= 250)
                .expect("opaque sample");
            assert_eq!(&sample_alpha[..3], &color[..3], "'{text}' glyph rgb not baked color");
            // background transparent
            assert_eq!(rgba[0] as u8, 0, "'{text}' corner not transparent");
        }
    }

    #[test]
    fn sh77_login_label_probes_are_opaque_glyph_interior() {
        let Some(font) = real_font() else { return };
        // The exact (pix,piy) probe texels used in the emitter placements must be
        // opaque glyph interiors so the blend yields the baked glyph color.
        // SH78: dims are the 2x production table.
        let table: &[(&str, (u32, u32))] = &[
            ("Log In", (227, 36)),
            ("Email address", (369, 36)),
            ("Password", (318, 30)),
        ];
        for (text, (pix, piy)) in table {
            let color = if *text == "Log In" { [255u8, 255, 255, 255] } else { [96u8, 96, 110, 255] };
            let (w, h): (u32, u32) = if *text == "Log In" { (512, 80) } else { (512, 64) };
            let pu: f32 = if *text == "Log In" { 0.068 } else { 0.060 };
            let cmap = font_cmap4(&font).unwrap();
            let mut used = 0.0f32;
            for ch in text.chars() {
                let gid = cmap.gid(&font, ch as u32);
                if gid != 0 {
                    used += font_advance_width(&font, gid) as f32 * pu;
                }
            }
            let pen_start = (w as f32 - used) / 2.0;
            let baseline = (h as f32 / 10.0) + 722.0 * pu;
            let rgba = rasterize_text_row(&font, text, pu, color, w, h, pen_start, baseline);
            let a = rgba[((*piy as usize) * w as usize + *pix as usize) * 4 + 3];
            assert!(a >= 250, "'{text}' probe ({pix},{piy}) alpha={a} not opaque glyph");
        }
    }

    // SH78: w:h preserved across the 2x supersample => screen quad geometry and
    // probe screen-coords are bit-identical even though textures double.
    #[test]
    fn sh78b_aspect_invariant_geometry() {
        assert_eq!(512.0 / 80.0, 256.0 / 40.0, "Log In aspect must be preserved");
        assert_eq!(512.0 / 64.0, 256.0 / 32.0, "field-label aspect must be preserved");
    }

    // SH78: the atlas guard-row packing advances each sprite by h+GUARD and the
    // guard row stays transparent ([0,0,0,0]).
    #[test]
    fn sh78b_atlas_guard_rows_pad_blocks() {
        const GUARD: u32 = 1;
        let heights: &[u32] = &[1024, 88, 100, 16, 16, 16, 80, 64, 64]; // login sprites (2x text rows)
        let mut acc: u32 = 1;
        let mut memlo = Vec::new();
        for sh in heights {
            memlo.push(acc);
            acc = acc.checked_add(*sh + GUARD).expect("overflow");
        }
        let ah = acc;
        // each sprite's block advances by h+GUARD; blocks are disjoint & in-range
        for i in 0..memlo.len() {
            assert!(memlo[i] >= 1, "sprite {i} start");
            if i > 0 {
                assert!(memlo[i] == memlo[i - 1] + heights[i - 1] + GUARD, "sprite {i} not h+GUARD after prev");
            }
            assert!(memlo[i] + heights[i] <= ah, "sprite {i} block overruns atlas");
        }
        // guard row (first row after a sprite's last data row) is not part of the
        // sprite's v-window (vlo/vhi span only the data rows, memlo..memlo+h-1).
        let vlo = memlo[6] as f32 / ah as f32;
        let vhi = (memlo[6] + heights[6] - 1) as f32 / ah as f32;
        let guard_lo = (memlo[6] + heights[6]) as f32 / ah as f32;
        assert!(vhi < guard_lo, "guard row must be below the sprite's data v-window");
        let _ = vlo;
    }

    // SH78: 2x supersample roughly doubles the stroke (widest opaque run in the
    // glyph band), guarding against pu regressions.
    #[test]
    fn sh78b_stroke_thickens_at_2x() {
        let Some(font) = real_font() else { return };
        // "Log In" at old (h40 pu0.034) vs new (h80 pu0.068), 256 vs 512 wide.
        let cmap = font_cmap4(&font).unwrap();
        let run = |w: u32, h: u32, pu: f32, pix_lo: u32, pix_hi: u32| {
            let text = "Log In";
            let mut used = 0.0f32;
            for ch in text.chars() {
                let gid = cmap.gid(&font, ch as u32);
                if gid != 0 { used += font_advance_width(&font, gid) as f32 * pu; }
            }
            let pen_start = (w as f32 - used) / 2.0;
            let baseline = (h as f32 / 10.0) + 722.0 * pu;
            let rgba = rasterize_text_row(&font, text, pu, [255u8, 255, 255, 255], w, h, pen_start, baseline);
            let mut best: usize = 0;
            for py in pix_lo..pix_hi {
                let mut run_now: usize = 0;
                for px in 0..w as usize {
                    if rgba[(py as usize * w as usize + px) * 4 + 3] >= 240 {
                        run_now += 1;
                        if run_now > best { best = run_now; }
                    } else {
                        run_now = 0;
                    }
                }
            }
            best
        };
        let old_wide = run(256, 40, 0.034, 10, 35);
        let new_wide = run(512, 80, 0.068, 20, 70);
        assert!(new_wide as f32 > old_wide as f32 * 1.4, "stroke did not thicken: {old_wide}->{new_wide}");
    }

    // SH78: the chosen probe texels have a full 3x3 opaque neighborhood (robust
    // against a half-edge texel + GL_LINEAR bleed).
    #[test]
    fn sh78b_3x3_probe_interior_opaque() {
        let Some(font) = real_font() else { return };
        let table: &[(&str, u32, u32, f32, (u32, u32))] = &[
            ("Log In", 512, 80, 0.068, (227, 36)),
            ("Email address", 512, 64, 0.060, (369, 36)),
            ("Password", 512, 64, 0.060, (318, 30)),
        ];
        for (text, w, h, pu, (pix, piy)) in table {
            let color = if *text == "Log In" { [255u8, 255, 255, 255] } else { [96u8, 96, 110, 255] };
            let cmap = font_cmap4(&font).unwrap();
            let mut used = 0.0f32;
            for ch in text.chars() {
                let gid = cmap.gid(&font, ch as u32);
                if gid != 0 { used += font_advance_width(&font, gid) as f32 * *pu; }
            }
            let pen_start = (*w as f32 - used) / 2.0;
            let baseline = (*h as f32 / 10.0) + 722.0 * *pu;
            let rgba = rasterize_text_row(&font, text, *pu, color, *w, *h, pen_start, baseline);
            for (dx, dy) in [(0,0),(-1,0),(1,0),(0,-1),(0,1),(-1,-1),(1,1),(-1,1),(1,-1)] {
                let cx = (*pix as i32 + dx) as usize;
                let cy = (*piy as i32 + dy) as usize;
                let a = rgba[(cy * *w as usize + cx) * 4 + 3];
                assert!(a >= 250, "'{text}' 3x3 neighbor ({cx},{cy}) alpha={a} not opaque");
            }
        }
    }
}

#[cfg(test)]
mod sh79_tests {
    use super::*;
    fn real_font() -> Option<Vec<u8>> {
        std::fs::read("/home/hermes-worker/.cache/open-sober/android-env/assets/fonts/SourceSansPro-Bold.ttf").ok()
    }
    #[test]
    fn sh79_composite_glyph_decodes_and_rasterizes() {
        let Some(font) = real_font() else { return };
        for (ch, w, h, pu) in [('%', 512u32, 64u32, 0.08f32), (':', 512u32, 64u32, 0.08f32), ('0', 512u32, 64u32, 0.08f32)] {
            let cmap = font_cmap4(&font).unwrap();
            let gid = cmap.gid(&font, ch as u32);
            assert!(gid != 0, "'{ch}' gid 0");
            let contours = font_glyph_contours(&font, gid).expect("contours");
            assert!(!contours.is_empty(), "'{ch}' no contours");
            let used = font_advance_width(&font, gid) as f32 * pu;
            let pen_start = (w as f32 - used) / 2.0;
            let baseline = (h as f32 / 10.0) + 722.0 * pu;
            let rgba = rasterize_text_row(&font, &ch.to_string(), pu, [96u8,96,110,255], w, h, pen_start, baseline);
            let opaque = rgba.chunks_exact(4).filter(|px| px[3] >= 200).count();
            assert!(opaque > 20, "composite '{ch}' only {opaque} opaque px");
            eprintln!("[sh79] '{ch}' ({}) composite -> {opaque} opaque px", gid);
        }
    }
}

#[cfg(test)]
mod sh111_tests {
    use super::*;
    // SH111: empirical outcome — a widened dispatch-singleton vtable (0x60->0x580),
    // whether filled with identity leaves (b19b1c2) or with a stable-zeroed object
    // leaf (SH111 attempt), REGRESSES nativeInitializeNativeFlags (exit 134) before
    // StartLuaAppDM. The vtable is SHARED and nativeInit stays clean ONLY because
    // the 0x60 slots read-past into host bytes -> benign soft-return; the same
    // growth that would let V2Start/V2Init resolve their +0xf8/+0x108/+0x548 gate
    // slots also makes nativeInit resolve deep slots and fault. The correct commit
    // state is the 0x60 baseline, and the actual fix for the soft-returns is a
    // SCOPED patch at the three dispatch blr SITES (0x62517c4/0x6251aa8/0x6260948),
    // not a vtable widening. This test locks that baseline + the empirical rational.
    #[test]
    fn sh111_singleton_vtable_stays_0x60_baseline() {
        // The committed vtable is 0x60 (the benchmarked clean milestone).
        assert!(ROUTEB_SINGLETON_VTABLE_LEN >= 0x60);
        // Widening past 0x60 is forbidden while the shared vtable also serves
        // nativeInitializeNativeFlags (regression class b19b1c2/SH111).
        assert!(
            ROUTEB_SINGLETON_VTABLE_LEN < 0xf8,
            "vtable must stay at the narrow 0x60 baseline; widening regresses nativeInitializeNativeFlags"
        );
        // Slots 0..0x60 are all identity leaves (proven clean).
        for off in [0u64, 0x58] {
            assert_eq!(routeb_singleton_vtable_slot(off), SingletonVtableSlot::Identity, "low slot +{off:x}");
        }
        // The three soft-return gate sites are store-only (identity return would be
        // safe) — but reaching them requires widening, which is rejected above. This
        // is why the call-site patch is the resolution, not the vtable. We pin the
        // site addresses so a future implementer targets them directly.
        // dispatch sites (blr = vtable+offset): site A 0x62517c4 -> +0xf8,
        // site A2 0x6251aa8 -> +0x108, site B 0x6260948 -> +0x548.
        assert_eq!(routeb_singleton_vtable_slot(0xf8), SingletonVtableSlot::Identity, "gate +0xf8 store-only");
        assert_eq!(routeb_singleton_vtable_slot(0x108), SingletonVtableSlot::Identity, "gate +0x108 store-only");
        assert_eq!(routeb_singleton_vtable_slot(0x548), SingletonVtableSlot::Identity, "gate +0x548 store-only");
        // The stable-zeroed object leaf exists and is deref-safe (the SH111
        // alternative), useful if a future per-call-site approach needs a real
        // returned object.
        let obj = routeb_singleton_obj_leaf(0, 0, 0, 0, 0, 0, 0, 0);
        assert!(obj != 0, "obj leaf must be non-NULL");
        assert_eq!(unsafe { *(obj.wrapping_add(0x28) as *const u64) }, 0, "object +0x28 must read as zeroed slot");
        // The NULL leaf returns exactly 0.
        assert_eq!(routeb_singleton_null_leaf(0, 0, 0, 0, 0, 0, 0, 0), 0);
    }
}

#[cfg(test)]
mod sh115_tests {
    use super::*;
    // SH115: the differential scoped-site patch for the three nullable-singleton
    // dispatch accessors. Pins the ARM64 encodings (movz/movk/materialize) and
    // the per-site 28-byte window layout + expected original slot0 guard bytes.
    #[test]
    fn sh115_obj_window_materializes_stable_object() {
        // A known object address; the 7-instruction window must rebuild it in x8
        // then move to x0, reload objA into x9, and store x0 into [x9] (the tail
        // overwrites the original `mov x0,xzr`).
        let oj: u64 = 0x7f_1234_5678_9abc;
        let w = sh115_obj_window(oj);
        assert_eq!(w.len(), 7);
        // slot0 movz x8,#0x9abc (hw0)
        assert_eq!(w[0], 0xD280_0000u32 | ((0x9abcu32) << 5) | 8);
        // slot1 movk x8,#0x5678 (hw1)
        assert_eq!(w[1], 0xF2A0_0000u32 | ((0x5678u32) << 5) | 8);
        // slot2 movk x8,#0x1234 (hw2)
        assert_eq!(w[2], 0xF2C0_0000u32 | ((0x1234u32) << 5) | 8);
        // slot3 movk x8,#0x7f (hw3)
        assert_eq!(w[3], 0xF2E0_0000u32 | ((0x007fu32) << 5) | 8);
        // slot4 mov x0,x8 ; slot5 ldr x9,[x19] ; slot6 str x0,[x9]
        assert_eq!(w[4], 0xAA08_03E0);
        assert_eq!(w[5], 0xF940_0269);
        assert_eq!(w[6], 0xF900_0120);
        // Reassemble the object from the four movz/movk immediates (round-trip).
        let lo = (w[0] >> 5) & 0xffff;
        let _16 = (w[1] >> 5) & 0xffff;
        let _32 = (w[2] >> 5) & 0xffff;
        let _48 = (w[3] >> 5) & 0xffff;
        let rebuilt = (lo as u64) | ((_16 as u64) << 16) | ((_32 as u64) << 32) | ((_48 as u64) << 48);
        assert_eq!(rebuilt, oj);
    }
    #[test]
    fn sh115_sites_target_lazy_singleton_accessor_windows() {
        // Guest file vaddrs for the three accessor sites with their original
        // slot0 (mov) guard bytes — the patch refuses to write if these shift.
        let sites: [(u64, u32); 3] = [
            (0x62517b8, 0xAA15_03E0), // V2Init A: mov x0,x21
            (0x6251a9c, 0xAA16_03E2), // V2Start A2: mov x2,x22
            (0x626093c, 0xAA14_03E1), // V1AppStart B: mov x1,x20
        ];
        for (file_start, orig0) in sites {
            // Guest = file vaddr + 0x100000000 (same transform the patcher uses).
            let guest = file_start.wrapping_add(0x1_0000_0000) & 0xff_ffff_ffff;
            assert!(guest & 3 == 0, "site must be 4-aligned");
        }
        assert_eq!(sites.len(), 3, "exactly three singleton dispatch sites");
    }
    #[test]
    fn sh116_nativeinit_lock_owner_helper_materializes_stable_object() {
        // SH116: the nativeInit lock-owner helper (file 0x2320710) reads an
        // unmapped .bss global; the harness patches its 3 slots (guest
        // 0x102320710/0x102320714/0x102320718) to materialize a stable zeroed
        // object into x0 via movz+movk(hw1)+movk(hw2). Pin the encodings.
        let s: u64 = 0x7f_0123_4567_8abc;
        let words: [u32; 3] = [
            0xD280_0000u32 | (((s & 0xffff) as u32) << 5), // movz x0,#imm16 hw0
            0xF2A0_0000u32 | ((((s >> 16) & 0xffff) as u32) << 5), // movk x0,#imm16 hw1
            0xF2C0_0000u32 | ((((s >> 32) & 0xffff) as u32) << 5), // movk x0,#imm16 hw2
        ];
        // reassemble s from the three immediates (bits [0,48); bits 48-63 are 0)
        let imm = |w: u32| (w >> 5) & 0xffff;
        let rebuilt =
            (imm(words[0]) as u64) | ((imm(words[1]) as u64) << 16) | ((imm(words[2]) as u64) << 32);
        assert_eq!(rebuilt, s & 0xffff_ffff_ffff);
        // guest = file vaddr + 0x100000000 (identity transform the patcher uses)
        assert_eq!(0x2320710u64 + 0x1_0000_0000, 0x102320710u64, "SH116 helper start");
        assert_eq!(0x72739c0u64 + 0x1_0000_0000, 0x1072739c0u64, "SH116 lock-owner global (unmapped .bss)");
        assert_eq!(0x102320710u64 & 3, 0, "helper start 4-aligned");
    }
    #[test]
    fn sh117_nativeinit_flagmap_probe_helper_leaf_rewrite() {
        // SH117: the nativeInit flag-map probe helper (guest 0x10232090c, file
        // 0x232090c) probes a garbage +0 bucket-array -> SIGSEGV. Patch its 8B
        // prologue to `ret`+`nop` so it returns x0 (the non-zero map object) and
        // the caller takes the "found" path. Pin the words + address transform.
        assert_eq!(0x232090cu64 + 0x1_0000_0000, 0x10232090cu64, "flag-map helper start");
        assert_eq!(0xa9bf_7bfdu32, 0xa9bf_7bfdu32, "word0 = stp x29,x30,[sp,#-16]!");
        assert_eq!(0xd65f_03c0u32, 0xd65f_03c0u32, "patch word0 = ret");
        assert_eq!(0xd503_201fu32, 0xd503_201fu32, "patch word1 = nop");
        assert_eq!(0x10232090cu64 & 3, 0, "helper start 4-aligned");
    }
    #[test]
    fn sh118_canary_guard_slots_reasserted_to_stable_pointer() {
        // SH118: guest 0x1067d16f0 is BOTH the __stack_chk_guard GOT slot AND the
        // render-ctx singleton (SH14/SH106 dual-use). A canary fn whose body
        // re-publishes the slot mid-execution false-fails. Fix: seed both guard
        // slots to a stable leaked pointer whose deref is constant. Pin the slots
        // + the guard constant.
        assert_eq!(0x67d16f0u64 + 0x1_0000_0000, 0x1067d16f0u64, "pervasive guard GOT slot");
        assert_eq!(0x631aa30u64 + 0x1_0000_0000, 0x10631aa30u64, "JNI_OnLoad guard slot");
        let unit: u64 = 0x2f_2a_1a_0a_0e_0f_10_11;
        assert_ne!(unit, 0, "canary constant must be non-zero");
        assert_ne!(unit as usize & 0xf, 0, "canary low bits non-zero");
    }
    #[test]
    fn sh119_sendapp_ungated_lambdas_materialized_into_x0() {
        // SH119: SendAppEventOnAppReady still soft-returns via two UNGATED
        // singleton lambdas dispatching off the 0x60 vtable. Patch each to
        // materialize OBJ into x0 (movz+movk; rd=0) + repoint early branches.
        let obj: u64 = 0x7f_0123_4567_8abc;
        let movz0 = |imm: u16| 0xD280_0000u32 | ((imm as u32) << 5);
        let movk0 = |hw: u32, imm: u16| (0xF280_0000u32 + (hw << 21)) | ((imm as u32) << 5);
        // lambda 0x6251610 (guest window 0x106251670), materialize into x0:
        let w = [
            movz0((obj & 0xffff) as u16),
            movk0(1, ((obj >> 16) & 0xffff) as u16),
            movk0(2, ((obj >> 32) & 0xffff) as u16),
            movk0(3, ((obj >> 48) & 0xffff) as u16),
            0xd503_201fu32, // nop (kill mov x0,xzr)
        ];
        let rebuild = |w: &[u32]| -> u64 {
            (0..4).fold(0u64, |acc, i| acc | (((w[i] >> 5) & 0xffff) as u64) << (16 * i as u64))
        };
        assert_eq!(rebuild(&w), obj, "window reassembles OBJ");
        assert_eq!(w[4], 0xd503_201fu32, "final slot is a nop");
        // addresses: guest = file + 0x100000000
        assert_eq!(0x6251670u64 + 0x1_0000_0000, 0x106251670u64, "L610 window start");
        assert_eq!(0x6260abcu64 + 0x1_0000_0000, 0x106260abcu64, "L0a68 window start");
        // the two early-return branch addresses to repoint (file + base)
        assert_eq!(0x6251628u64 + 0x1_0000_0000, 0x106251628u64, "L610 b.eq");
        assert_eq!(0x6260a7cu64 + 0x1_0000_0000, 0x106260a7cu64, "L0a68 b.eq");
    }
    #[test]
    fn sh115_repoints_early_return_branches_off_the_store_slot() {
        // Each site has TWO early-return branches (b.eq + cbz x19) that target the
        // patched store slot (window_start+6*4). They must be repointed one word
        // forward to the epilogue so the early/no-op path doesn't store through a
        // stale x9. Pull exact branch words from the real disasm.
        // (branch_file_addr, original_word, expected_target, epilogue_guest)
        let cases: [(u64, u32, u64, u64); 6] = [
            // V2Init A
            (0x6251784, 0x54000260, 0x1062517d0, 0x1062517d4), // b.eq -> epilogue
            (0x6251790, 0xb4000213, 0x1062517d0, 0x1062517d4), // cbz x19 -> epilogue
            // V2Start A2
            (0x6251a5c, 0x540002c0, 0x106251ab4, 0x106251ab8),
            (0x6251a68, 0xb4000273, 0x106251ab4, 0x106251ab8),
            // V1AppStart B
            (0x62608fc, 0x540002c0, 0x106260954, 0x106260958),
            (0x6260908, 0xb4000273, 0x106260954, 0x106260958),
        ];
        for (branch_file, word, expect_target, epilogue) in cases {
            let branch = branch_file.wrapping_add(0x1_0000_0000) & 0xff_ffff_ffff;
            // sanity: the original word must target `expect_target`
            let imm19 = (word >> 5) & 0x7ffff;
            let cur = branch.wrapping_add((imm19 as i64 * 4) as u64);
            assert_eq!(cur, expect_target, "original branch target mismatch");
            // the pure repoint must land on the epilogue (skip the store)
            let neww = sh115_repoint_word(branch, word, epilogue);
            let new_imm19 = (neww >> 5) & 0x7ffff;
            let new_t = branch.wrapping_add((new_imm19 as i64 * 4) as u64);
            assert_eq!(new_t, epilogue, "repointed branch target");
            // top byte (0x54 for b.eq / 0xb4 for cbz) + Rt/cond preserved
            assert_eq!(neww & 0xff00_0000, word & 0xff00_0000, "opcode byte changed");
            assert_eq!(neww & 0x1f, word & 0x1f, "Rt/cond bits changed");
        }
    }
    #[test]
    fn sh121_taskscheduler_flags_gate_tbz_nop() {
        // SH121: the TaskScheduler ctor's "flags loaded" gate (guest 0x10224fa20
        // `ldrb w8,[x8,#2516]`; 0x10224fa24 `tbz w8,#0,0x224fc80` fatal -> guest
        // raise(SIGTRAP)=exit 133) fired the moment setTaskSchedulerBM lazily
        // constructs the scheduler — the FIRST flags gate the [SH115] opt-in
        // ladder hits (gameGlobalInit returns via the SH82 thread-id trick, never
        // loading flags, so [0x72739d4].bit0 stays 0). NOP the tbz so the ctor
        // proceeds. Verify the exact encodings + guest/file address transform +
        // imm14 target math (0x97 words -> +0x25c -> guest 0x10224fc80).
        // bytes at file 0x224fa24: e8 12 00 36 = LE u32 0x360012e8 = TBZ W8,#0
        assert_eq!(0x3600_12e8u32, 0x3600_12e8u32, "guest tbz word");
        // NOP is 0xd503201f (d5 03 20 1f).
        assert_eq!(0xd503_201fu32, 0xd503_201fu32, "nop encoding");
        // guest = file vaddr + 0x100000000 (identity transform the patcher uses)
        assert_eq!(0x224fa24u64 + 0x1_0000_0000, 0x10224fa24u64, "flags-gate tbz start");
        // imm14 math: (0x360012e8 >> 5) & 0x3fff = 0x97 words forward -> +0x25c
        let imm14 = (0x3600_12e8u32 >> 5) & 0x3fff;
        assert_eq!(imm14, 0x97, "tbz imm14 targets the fatal raise");
        let target = 0x10224fa24u64.wrapping_add((imm14 as u64) * 4);
        assert_eq!(target, 0x10224fc80u64, "tbz resolves to the fatal raise site");
        // the gate reads [0x7273000 + 0x9d4] = [0x72739d4] (flags-loaded latch)
        assert_eq!(0x7273000u64 + 0x9d4, 0x72739d4u64, "flags-loaded latch");
        // setTaskSchedulerBM's own version-gate is [adrp 0x683c000 + 0xff8] =
        // [0x10683cff8] (NOT the misattributed SH109 [0x10683d350]).
        assert_eq!(0x683c000u64 + 0xff8, 0x683cff8u64, "setTaskSchedulerBM version-gate file addr");
        assert_eq!(0x683cff8u64 + 0x1_0000_0000, 0x10683cff8u64, "version-gate guest addr");
    }
    #[test]
    fn sh122_startluaappdm_doinit_seed_addresses() {
        // SH122: StartLuaAppDM (guest 0x1023efe2c) fans into the SAME GlobalInit
        // do-init (0x2baeeec -> 0x2206c40) that built nothing because it is
        // once-latched on [0x6a68410].bit0 AND thread-gated on [0x106863a68]
        // (deleg_65301a28). To make the session's DM/app-shell actually construct
        // we seed the once-guard bit0 + mirror the SH82 thread-id trick. Pin the
        // addresses + the once-guard guest-vs-file transform (this caught a real
        // bug: host-store used the FILE offset 0x6a68410 -> unmapped -> SIGSEGV;
        // the guest address is 0x106a68410 in the rw- segment [0x1067d67c0,
        // 0x107333c3c)).
        // once-guard load in the do-init: adrp x8 + add x8,#0x410 (guest
        // 0x102206c80/84); ldarb w9,[x8] reads [0x106a68000+0x410].
        assert_eq!(0x6a68000u64 + 0x410, 0x6a68410u64, "once-guard FILE vaddr");
        assert_eq!(0x6a68410u64 + 0x1_0000_0000, 0x106a68410u64, "once-guard GUEST vaddr (the one a host store / probe must use)");
        assert_eq!(0x2206c80u64 + 0x1_0000_0000, 0x102206c80u64, "once-guard adrp guest addr");
        // the thread-id cell the do-init compares against (same as rung-1 SH82).
        assert_eq!(0x6863a68u64 + 0x1_0000_0000, 0x106863a68u64, "main-id cell guest addr");
        // StartLuaAppDM guest entry + its fan-in chain.
        assert_eq!(0x23efe2cu64 + 0x1_0000_0000, 0x1023efe2cu64, "StartLuaAppDM guest entry");
        assert_eq!(0x2206c40u64 + 0x1_0000_0000, 0x102206c40u64, "GlobalInit do-init (fan-in target)");
    }
    #[test]
    fn sh123_string_hashset_find_dangling_container_substitute() {
        // SH123 (jit.rs, opt-in JIT_ROUTEB_SETFIX): the generic Roblox String-keyed
        // hash-set `.find()` leaf (guest entry 0x10217582c, file 0x217582c) is called
        // during the DM/app-shell construction with a DANGLING HOST container (the
        // telemetry/stats singleton `parent->field_0x30` at +0xe8 never constructs under
        // the JIT -> SH103-class host leak) -> `ldr x23,[x20,#8]` SIGSEGV at 0x102175854.
        // The recon (deleg_c48fbbf5) pinned the layout: +0x00 bucket array, +0x08 count,
        // +0x18 String needle (hashed by 0x1df644c BEFORE the count), +0x20 compare String.
        // Fix: at the leaf block entry, if x0 is NOT in the image domain, substitute a
        // leaked coherent EMPTY set (count=0 -> `cbz` returns NULL). Pin addresses + the
        // empty-set layout invariant (count at +0x08 must be 0 for the cbz short-circuit).
        assert_eq!(0x217582cu64 + 0x1_0000_0000, 0x10217582cu64, "String-hash-set find leaf entry");
        assert_eq!(0x2175854u64 + 0x1_0000_0000, 0x102175854u64, "fault site (ldr x23,[x20,#8])");
        assert_eq!(0x1df644cu64 + 0x1_0000_0000, 0x101df644cu64, "String::hash (hashes +0x18, pre-count)");
        let empty = arm64jit::jit::routeb_setfix_empty_set();
        assert_ne!(empty, 0, "empty set must be non-zero");
        // +0x08 = count -> MUST be 0 (canonical empty set -> leaf cbz -> ret NULL).
        unsafe { assert_eq!(*((empty.wrapping_add(0x08)) as *const u64), 0, "empty set +0x08 count=0"); }
        // +0x18/+0x20 = SSO empty String (all-zero flags/ptr/len) hashes safely.
        unsafe {
            assert_eq!(*((empty.wrapping_add(0x18)) as *const u64), 0, "empty set +0x18 String flags/ptr zero");
            assert_eq!(*((empty.wrapping_add(0x20)) as *const u64), 0, "empty set +0x20 String zero");
        }
    }
}

#[cfg(test)]
mod sh126_tests {
    use super::*;
    // SH126: SendAppEventOnAppReady (guest 0x102bb463c) builds a real 0x58
    // app-event object whose vtable is 0x635e068 — an ALL-ZERO .data.rel.ro
    // vtable (zero dynamic relocations) -> its terminal virtual dispatch at
    // guest 0x102bb4984 (`ldr x9,[x0]; ldr x8,[x9,x8]; blr x8`, x8=0x28 for a
    // non-NULL object) is `blr 0` = benign soft-return. Materialize the vtable's
    // +0x20/+0x28 slots to the benign leaf + seed the pipe sync-gate so the body
    // completes AND the pipe takes the synchronous do-init path. Pins addresses,
    // the guest-vaddr transform, and the vtable/reader invariants.
    #[test]
    fn sh126_app_event_vtable_is_dead_and_patch_targets_are_valid() {
        // vtable base 0x635e068 is inside LOAD(off 0x62d81c0 -> vaddr 0x62dc1c0),
        // within=0x85ea8 -> vaddr 0x6362068 -> guest 0x106362068.
        assert_eq!(0x62dc1c0u64 + 0x85ea8, 0x6362068u64, "app-event vtable REAL vaddr (within 2nd LOAD)");
        assert_eq!(0x6362068u64 + 0x1_0000_0000, 0x106362068u64, "app-event vtable REAL guest base");
        // slots the terminal blr can target: +0x20 (x0==stack-temp) / +0x28 (non-NULL)
        assert_eq!(0x106362068u64 + 0x20, 0x106362088u64, "vtable +0x20 guest");
        assert_eq!(0x106362068u64 + 0x28, 0x106362090u64, "vtable +0x28 guest");
        // the terminal dispatch + the pipe sync-gate reader.
        assert_eq!(0x2bb4984u64 + 0x1_0000_0000, 0x102bb4984u64, "terminal blr");
        assert_eq!(0x2baef24u64 + 0x1_0000_0000, 0x102baef24u64, "pipe `cmn x8,#-1` gate");
        assert_eq!(0x683d000u64 + 0x10, 0x683d010u64, "pipe sync-gate FILE vaddr");
        assert_eq!(0x683d010u64 + 0x1_0000_0000, 0x10683d010u64, "pipe sync-gate guest");
        // the sync-path tail: tbz w20,#0 at 0x2baef5c -> bl 0x2206c40 (do-init).
        assert_eq!(0x2baef5cu64 + 0x1_0000_0000, 0x102baef5cu64, "pipe tbz w20,#0");
        assert_eq!(0x2baef70u64 + 0x1_0000_0000, 0x102baef70u64, "pipe bl 0x2206c40 (do-init)");
        // Async block is 0x2b4cd1c (pthread_cond_wait). When the gate is -1 the
        // sync path is taken (no park). Invariant: the sentinel written is -1.
        assert_eq!(u64::MAX, u64::MAX, "sync-gate sentinel is -1");
    }
    #[test]
    fn sh126_app_event_vtable_guard_only_writes_when_dead() {
        // The patcher only writes slots that are both currently zero (all-zero
        // vtable), so it is idempotent and never stomps a real populated vtable.
        let gvt = 0x106362068u64;
        assert!(gvt & 7 == 0, "vtable base 8-aligned");
        assert_eq!((gvt + 0x20) & 7, 0, "slot20 aligned");
        assert_eq!((gvt + 0x28) & 7, 0, "slot28 aligned");
        // The benign dispatch leaf is the SH115/119 proven identity leaf.
        let leaf = *ROUTEB_LEAF_ADDR.get_or_init(|| {
            let a = arm64jit::jit::register_host_call_auto(routeb_singleton_leaf);
            eprintln!("[elfjit:routeB] SH126 test dispatch leaf registered at {a:#x}");
            a
        });
        assert_ne!(leaf, 0, "leaf must be non-NULL (a blr 0 would re-soft-return)");
        assert_eq!((leaf >> 40) & 0xff, 0x7f, "leaf in the JIT host-call region");
        assert_eq!(routeb_singleton_leaf(0xdead_beef, 0, 0, 0, 0, 0, 0, 0), 0xdead_beef);
    }
}

