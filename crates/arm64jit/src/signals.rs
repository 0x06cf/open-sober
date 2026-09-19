// SPDX-License-Identifier: MIT
//
// Guest signal handling for the arm64jit runtime. Serves the Linux signal
// contract to translated AArch64 guest code:
//
//   - `rt_sigaction` (134) records SIG_DFL / SIG_IGN / a guest handler fn.
//   - `kill` (129) / `tgkill` (131) deliver to a guest thread: a SAME-thread
//     (self) signal runs the handler synchronously; a DIFFERENT guest thread
//     gets a cooperative `pending_signal` that its own dispatcher loop picks up.
//   - A dispatched handler is entered as a guest function with the aarch64
//     signal ABI (x0 = signo, x1 = siginfo*, x2 = ucontext*), the interrupted
//     context is saved, and on the handler's `ret` (x30 == SIGRET sentinel) or
//     an explicit `rt_sigreturn` (139) the context is restored and the
//     interrupted guest resumes right after the original `svc`.
//   - Un-handled signals fall back to the POSIX default disposition (ignore /
//     terminate the process), so a guest that sends itself SIGTERM or writes to
//     a closed pipe behaves like a real Linux process instead of carrying on.

use crate::jit::CpuState;

/// Magic guest pointer a dispatched signal handler's `ret` jumps to (`x30`).
/// The dispatcher loop recognizes this pc and restores the saved context. It is
/// a deliberately invalid instruction address (nowhere near guest code or the
/// host-thunk region), so it can never be a real branch target.
pub const SIGRET: u64 = 0x5151_5253_5455_5657; // "SIGRET"

/// A guest-visible per-signal action, in the kernel's aarch64 `struct sigaction`
/// semantics: `handler` is 0 = SIG_DFL, 1 = SIG_IGN, else a guest function addr.
#[derive(Clone, Copy)]
pub struct GuestSigAction {
    pub handler: u64,
    pub flags: u64,
    pub mask: [u8; 8],
}

/// Process-wide signal action table, indexed by signal number (1..=64).
static SIG_ACTIONS: std::sync::Mutex<[GuestSigAction; 65]> = std::sync::Mutex::new(
    [GuestSigAction {
        handler: 0,
        flags: 0,
        mask: [0; 8],
    }; 65],
);

/// Saved interrupted guest context for an in-flight signal handler, per guest
/// thread (a process-local stack so nested signals unwind LIFO). The guest
/// siginfo/ucontext bytes live in the frame itself: since guest == host
/// addresses here, the handler's x1/x2 args can point straight at these
/// buffers, which stay valid until `sigreturn` pops the frame.
struct SigFrame {
    x: [u64; 32],
    v: [u64; 64],
    sp: u64,
    tpidr: u64,
    nzcv: u32,
    /// Guest address to resume at after the handler returns: the instruction
    /// right after the interrupted `svc` (the post-syscall continuation).
    pc: u64,
    si: [u8; 128],  // guest siginfo buffer (handler x1)
    uc: [u8; 1024], // guest ucontext buffer (handler x2)
}

thread_local! {
    static FRAMES: std::cell::RefCell<Vec<Box<SigFrame>>> = std::cell::RefCell::new(Vec::new());
}

/// Test/app hook: clear the process-wide action table. silences nothing — just
/// returns every signal to its default disposition.
pub fn reset_actions() {
    *SIG_ACTIONS.lock().unwrap() = [GuestSigAction {
        handler: 0,
        flags: 0,
        mask: [0; 8],
    }; 65];
}

/// Signal numbers Linux forbids blocking / catching (SIGKILL=9, SIGSTOP=19).
/// The kernel silently drops these from any set a thread tries to block, and
/// they are always actionable regardless of the mask.
pub fn unblockable_mask() -> u64 {
    (1u64 << (9 - 1)) | (1u64 << (19 - 1))
}

/// Whether `sig` is currently in this thread's blocked mask (Linux sigset: bit
/// N-1 set means signal N is blocked). SIGKILL/SIGSTOP are never blocked.
pub fn is_blocked(st: &CpuState, sig: u32) -> bool {
    if !(1..=64).contains(&sig) {
        return false;
    }
    if sig == 9 || sig == 19 {
        return false;
    }
    (st.blocked_mask >> (sig - 1)) & 1 == 1
}

/// Atomically OR `sig` into this thread's pending mask (`pending_mask`): the
/// signal has been raised but cannot be delivered yet (it is blocked). The
/// owning thread delivers it once `rt_sigprocmask` unblocks `sig`. Safe to
/// call from another host thread posting a signal to this guest thread.
pub fn mark_pending(st: &mut CpuState, sig: u32) {
    if (1..=64).contains(&sig) {
        let bit = 1u64 << (sig - 1);
        // Atomic OR so a cross-thread sender can race the owner's clear
        // without losing a newly-pending signal.
        let p = &mut st.pending_mask as *mut u64;
        unsafe {
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            std::ptr::write_volatile(p, std::ptr::read_volatile(p) | bit);
        }
    }
}

/// Try to deliver `sig` to the current guest thread (`st`). If `sig` is in
/// `st.blocked_mask`, mark it pending and do NOT dispatch (it will run once
/// unblocked). Otherwise dispatch immediately (handler / default / ignore).
/// `resume` is the interrupted guest PC (see `dispatch_current_thread`).
pub fn deliver(st: &mut CpuState, sig: u32, resume: u64) {
    if is_blocked(st, sig) {
        mark_pending(st, sig);
    } else {
        dispatch_current_thread(st, sig, resume);
    }
}

/// Drain one deliverable pending signal: if any pending signal is no longer
/// blocked, atomically clear its bit and return it; else None. Called at a
/// block boundary after `rt_sigprocmask` unblocks something, so a previously-
/// blocked signal gets dispatched as soon as it becomes deliverable.
pub fn take_deliverable_pending(st: &mut CpuState) -> Option<u32> {
    let p = &mut st.pending_mask as *mut u64;
    unsafe {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        let pend = std::ptr::read_volatile(p);
        let blocked = st.blocked_mask;
        let deliverable = pend & !blocked;
        if deliverable == 0 {
            return None;
        }
        // Lowest set bit = lowest signal number pending & unblocked (Linux
        // delivers in ascending signal-number order).
        let sig = deliverable.trailing_zeros() as u32 + 1;
        std::ptr::write_volatile(p, pend & !(1u64 << (sig - 1)));
        Some(sig)
    }
}

/// Apply `rt_sigprocmask` (syscall 135) semantics: how in
/// {SIG_BLOCK=0, SIG_UNBLOCK=1, SIG_SETMASK=2}, `set`/`oset` point at 8-byte
/// sigsets, `sigsetsize` is the sigset byte size (must be >= 8). Returns 0 on
/// success, -EINVAL for a bad `how` / `sigsetsize`.
pub fn sigprocmask(st: &mut CpuState, how: u64, set: u64, oset: u64, sigsetsize: u64) -> i64 {
    if sigsetsize < 8 || how > 2 {
        return (-libc::EINVAL) as i64;
    }
    // SIG_SETMASK with a NULL `set` is invalid (there'd be nothing to set).
    if set == 0 && how == 2 {
        return (-libc::EINVAL) as i64;
    }
    let old = st.blocked_mask;
    // Read the incoming set only if provided (may be NULL for a pure query).
    if set != 0 {
        // SAFETY: guest passed a readable 8-byte sigset (sigsetsize >= 8).
        let new = unsafe { std::ptr::read_unaligned(set as *const u64) };
        st.blocked_mask = match how {
            0 => old | new,  // SIG_BLOCK
            1 => old & !new, // SIG_UNBLOCK
            _ => new,        // SIG_SETMASK (how == 2)
        };
        // The kernel ignores attempts to block SIGKILL/SIGSTOP (and would
        // deliver SIGKILL before the syscall returns); drop their bits so a
        // blocked_mask with them can't stall a pending SIGKILL forever.
        st.blocked_mask &= !unblockable_mask();
    }
    // Report the previous mask to oset (if provided).
    if oset != 0 {
        // SAFETY: guest passed a writable 8-byte sigset (sigsetsize >= 8).
        unsafe {
            std::ptr::write_unaligned(oset as *mut u64, old);
        }
    }
    0
}

/// POSIX default disposition for an un-handled signal. Returns `true` when the
/// default action terminates the process (everything not ignore/stop/continue).
/// There is no job control in the single-process runtime, so SIGTSTP/SIGTTIN/
/// SIGTTOU/SIGSTOP/SIGCONT collapse to "ignore", as do SIGCHLD/SIGURG/SIGWINCH.
fn default_terminates(sig: u64) -> bool {
    !matches!(sig, 17 | 18 | 19 | 20 | 21 | 22 | 23 | 28)
}

/// Install or query a signal action (`rt_sigaction`, syscall 134).
///
/// Guest aarch64 kernel `struct sigaction` (all sizes in bytes):
///   sa_handler : u64 @ 0        (0 = SIG_DFL, 1 = SIG_IGN, else fn addr)
///   sa_flags   : u64 @ 8
///   sa_restorer: u64 @ 16
///   sa_mask    : 8  bytes @ 24  (kernel sigset_t, `sigsetsize` = 8)
pub fn rt_sigaction(sig: u64, act: u64, oact: u64) -> i64 {
    if sig < 1 || sig > 64 {
        return (-libc::EINVAL) as i64;
    }
    let mut table = SIG_ACTIONS.lock().unwrap();
    if oact != 0 {
        let cur = table[sig as usize];
        // SAFETY: the guest passed a writable oact of at least the 32-byte
        // (max(24+8)) aarch64 sigaction struct.
        unsafe {
            let p = oact as *mut u8;
            (p as *mut u64).write_unaligned(cur.handler);
            (p.add(8) as *mut u64).write_unaligned(cur.flags);
            (p.add(16) as *mut u64).write_unaligned(0); // we install no restorer
            std::ptr::copy_nonoverlapping(&cur.mask as *const u8, p.add(24), 8);
        }
    }
    if act != 0 {
        // SAFETY: the guest passed a readable act of >= 32 bytes.
        unsafe {
            let p = act as *const u8;
            let handler = (p as *const u64).read_unaligned();
            let flags = (p.add(8) as *const u64).read_unaligned();
            let mut mask = [0u8; 8];
            std::ptr::copy_nonoverlapping(p.add(24), &mut mask as *mut u8, 8);
            table[sig as usize] = GuestSigAction {
                handler,
                flags,
                mask,
            };
        }
    }
    drop(table);
    0
}

/// Apply the effective action for `sig` to the CURRENT guest thread (`st`):
///   - SIG_IGN / default-ignore -> nothing (signal consumed).
///   - default-terminate        -> process terminates (128 + sig), like Linux.
///   - installed guest handler   -> save context, prepare the handler (set
///                                  `st.redirect_request = handler` so the
///                                  dispatcher loop runs it next, and set
///                                  `x30 = SIGRET` so the handler's `ret` hands
///                                  back to the dispatcher's sigreturn), then
///                                  restore the context and resume at `resume`.
/// `resume` is the guest PC of the interrupted context: for a signal taken
/// during a self-delivering syscall this is `st.svc_next` (the instruction after
/// the `svc`); for a cross-thread (cooperative-pending) pickup it is the
/// thread's current `st.pc`. Runs on the owning thread of `st`.
pub fn dispatch_current_thread(st: &mut CpuState, sig: u32, resume: u64) {
    let act = SIG_ACTIONS.lock().unwrap()[sig as usize];
    if act.handler == 1 {
        return; // SIG_IGN
    }
    if act.handler == 0 {
        // Default disposition.
        if default_terminates(sig as u64) {
            // SH131 diag: before terminating on a SIGTRAP (which the engine
            // uses as its fatal — exit 133, the SH121 class at a NEW site on a
            // released clone worker), dump the current guest pc so the exact
            // raise()/brk site can be patched rather than blamed on the flood.
            if sig == 5 {
                eprintln!(
                    "[signals] SIGTRAP default-terminate on tid {} gettid={}: pc={:#x} x30={:#x} x0={:#x} (128{}={}, the fatal exit 133) — pin the guest raise site (SH121-class)",
                    std::thread::current().name().unwrap_or("?"),
                    unsafe { libc::gettid() },
                    st.pc, st.x[30], st.x[0], sig, 128 + sig
                );
            }
            // SAFETY: a SIG_DFL-terminating signal ends the whole process, as
            // on Linux (no guest settable handlers get to run for it).
            unsafe {
                libc::_exit(128 + sig as libc::c_int);
            }
        }
        return; // default-ignore
    }
    begin_handler(st, sig, act.handler, resume);
}

/// Save `st`'s context and enter a guest signal handler (`handler`) with the
/// aarch64 signal ABI. On the handler's `ret` (x30 = SIGRET) the dispatcher
/// calls `sigreturn` to restore. `resume` is the interrupted guest PC to resume
/// at after the handler returns.
fn begin_handler(st: &mut CpuState, sig: u32, handler: u64, resume: u64) {
    let mut f = Box::new(SigFrame {
        x: st.x,
        v: st.v,
        sp: st.x[31],
        tpidr: st.tpidr,
        nzcv: st.nzcv,
        pc: resume,
        si: [0u8; 128],
        uc: [0u8; 1024],
    });
    // The interrupted syscall (kill/tgkill) returns 0 to the guest after the
    // handler completes, matching Linux. The kernel would leave the syscall
    // return in x0; we mirror that by overriding the saved x0.
    f.x[0] = 0;
    // siginfo_t header: si_signo, si_errno=0, si_code=SI_USER(0).
    f.si[0..4].copy_from_slice(&(sig as i32).to_le_bytes());
    f.si[8..12].copy_from_slice(&0i32.to_le_bytes());
    let si_addr = f.si.as_ptr() as u64;
    let uc_addr = f.uc.as_ptr() as u64;
    FRAMES.with(|fr| fr.borrow_mut().push(f));
    // Configure the register file to run the handler per the aarch64 signal
    // convention: x0=signo, x1=siginfo*, x2=ucontext*, x30=SIGRET (so the
    // handler's `ret` hands back to the dispatcher's sigreturn). sp stays the
    // interrupted sp; the handler pushes its own frames beneath it. The
    // redirect_request tells the Svc arm (self-delivery) / dispatcher loop
    // (pending pickup) to run the handler instead of the interrupted code.
    st.redirect_request = handler;
    st.x[0] = sig as u64;
    st.x[1] = si_addr;
    st.x[2] = uc_addr;
    st.x[30] = SIGRET;
}

/// Restore the interrupted context (`rt_sigreturn`, syscall 139, or the SIGRET
/// handler-`ret` path). Returns false if there was no saved frame for this
/// thread — a stray sigreturn, which the guest should never produce.
pub fn sigreturn(st: &mut CpuState) -> bool {
    let popped = FRAMES.with(|fr| fr.borrow_mut().pop());
    match popped {
        Some(f) => {
            st.x = f.x;
            st.v = f.v;
            st.x[31] = f.sp;
            st.tpidr = f.tpidr;
            st.nzcv = f.nzcv;
            st.pc = f.pc;
            true
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jit::CpuState;

    /// A serialization lock for the SIG_ACTIONS table + FRAMES TLS so parallel
    /// hermetics never race a shared signal-action store (the table is
    /// process-global; reset_actions + install/query must be mutually owned).
    static SIG_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn sigprocmask_block_unblock_setmask_drops_unblockable_and_osets() {
        let mut st = CpuState::new();
        let _g = SIG_TEST_LOCK.lock();
        reset_actions();

        // An 8-byte set buffer we write into, then hand to sigprocmask.
        let mut buf = vec![0u8; 8];
        // Block SIGUSR1(10) + SIGUSR2(12): bits 9, 11.
        let block = (1u64 << 9) | (1u64 << 11);
        unsafe { std::ptr::write_unaligned(buf.as_mut_ptr() as *mut u64, block) };

        // SIG_BLOCK from an empty mask.
        let r = sigprocmask(&mut st, 0, buf.as_mut_ptr() as u64, 0, 8);
        assert_eq!(r, 0);
        assert_eq!(st.blocked_mask, block);
        assert!(is_blocked(&st, 10));
        assert!(is_blocked(&st, 12));
        assert!(!is_blocked(&st, 11));

        // oset reports the just-set mask.
        let mut o = vec![0u8; 8];
        let r = sigprocmask(
            &mut st,
            1,
            buf.as_mut_ptr() as u64,
            o.as_mut_ptr() as u64,
            8,
        );
        assert_eq!(r, 0);
        let oset = unsafe { std::ptr::read_unaligned(o.as_ptr() as *const u64) };
        assert_eq!(oset, block, "oset carries the previous mask");

        // SIG_UNBLOCK cleared both.
        assert_eq!(st.blocked_mask, 0, "SIG_UNBLOCK of the whole set clears it");
        assert!(!is_blocked(&st, 10));

        // Attempting to BLOCK SIGKILL(9)+SIGSTOP(19) must be silently dropped.
        let killstop = (1u64 << 8) | (1u64 << 18);
        unsafe { std::ptr::write_unaligned(buf.as_mut_ptr() as *mut u64, block | killstop) };
        let r = sigprocmask(&mut st, 0, buf.as_mut_ptr() as u64, 0, 8);
        assert_eq!(r, 0);
        // Only the blockable bits survive; kill/stop never blocked.
        assert_eq!(st.blocked_mask, block);
        assert!(!is_blocked(&st, 9));
        assert!(!is_blocked(&st, 19));

        // SIG_SETMASK replaces wholesale.
        let only12 = 1u64 << 11;
        unsafe { std::ptr::write_unaligned(buf.as_mut_ptr() as *mut u64, only12) };
        let r = sigprocmask(&mut st, 2, buf.as_mut_ptr() as u64, 0, 8);
        assert_eq!(r, 0);
        assert_eq!(st.blocked_mask, only12);
    }

    #[test]
    fn sigprocmask_einval_paths() {
        let mut st = CpuState::new();
        // sigsetsize < 8 is invalid.
        assert_eq!(sigprocmask(&mut st, 0, 0, 0, 4), -(libc::EINVAL) as i64);
        // how > 2 is invalid.
        assert_eq!(sigprocmask(&mut st, 3, 0, 0, 8), -(libc::EINVAL) as i64);
        // SIG_SETMASK (2) with a NULL set is invalid.
        assert_eq!(sigprocmask(&mut st, 2, 0, 0, 8), -(libc::EINVAL) as i64);
        // A NULL set + NULL oset with SIG_BLOCK is a no-op query that still
        // returns 0 (Linux permits a pure query with both NULL).
        assert_eq!(sigprocmask(&mut st, 0, 0, 0, 8), 0);
    }

    #[test]
    fn pending_deliverable_ordering_and_blocked_hold() {
        let mut st = CpuState::new();
        let _g = SIG_TEST_LOCK.lock();
        reset_actions();

        // mark 12 then 10 pending; both unblocked.
        mark_pending(&mut st, 12);
        mark_pending(&mut st, 10);
        // Lowest set bit = ascending signal number: 10 before 12.
        assert_eq!(take_deliverable_pending(&mut st), Some(10));
        assert_eq!(take_deliverable_pending(&mut st), Some(12));
        assert_eq!(take_deliverable_pending(&mut st), None, "drained");

        // A blocked signal stays pending and is not delivered.
        st.blocked_mask = 1u64 << 11; // block 12
        mark_pending(&mut st, 12);
        mark_pending(&mut st, 10);
        assert_eq!(
            take_deliverable_pending(&mut st),
            Some(10),
            "unblocked 10 delivered"
        );
        assert_eq!(
            take_deliverable_pending(&mut st),
            None,
            "12 held blocked in pending_mask"
        );
        assert!(st.pending_mask & (1u64 << 11) != 0, "12 still pending");
        // Once unblocked, it becomes deliverable.
        st.blocked_mask = 0;
        assert_eq!(
            take_deliverable_pending(&mut st),
            Some(12),
            "12 now delivered"
        );
    }

    #[test]
    fn sigaction_install_query_roundtrip_and_einval() {
        let _g = SIG_TEST_LOCK.lock();
        reset_actions();

        // Out-of-range signal is EINVAL.
        assert_eq!(rt_sigaction(0, 0, 0), -(libc::EINVAL) as i64);
        assert_eq!(rt_sigaction(65, 0, 0), -(libc::EINVAL) as i64);

        // Install handler 0x1234 with flags on SIGUSR1(10).
        let mut act = vec![0u8; 32];
        unsafe {
            (act.as_mut_ptr() as *mut u64).write_unaligned(0x1234u64); // handler
            (act.as_mut_ptr().add(8) as *mut u64).write_unaligned(0x4000u64); // flags
                                                                              // mask @ 24 (8 bytes): block SIGUSR2.
            std::ptr::copy_nonoverlapping(
                (1u64 << 11).to_le_bytes().as_ptr(),
                act.as_mut_ptr().add(24),
                8,
            );
        }
        let r = rt_sigaction(10, act.as_mut_ptr() as u64, 0);
        assert_eq!(r, 0);

        // Query it back via oact.
        let mut oact = vec![0u8; 32];
        let r = rt_sigaction(10, 0, oact.as_mut_ptr() as u64);
        assert_eq!(r, 0);
        let handler = unsafe { (oact.as_ptr() as *const u64).read_unaligned() };
        let flags = unsafe { (oact.as_ptr().add(8) as *const u64).read_unaligned() };
        let mut mask = [0u8; 8];
        unsafe { std::ptr::copy_nonoverlapping(oact.as_ptr().add(24), &mut mask as *mut u8, 8) };
        assert_eq!(handler, 0x1234);
        assert_eq!(flags, 0x4000);
        assert_eq!(mask, (1u64 << 11).to_le_bytes());
    }

    #[test]
    fn handler_frame_sigreturn_restores_context() {
        let mut st = CpuState::new();
        let _g = SIG_TEST_LOCK.lock();
        reset_actions();

        // Seed an interrupted context that begin_handler must save + restore.
        for i in 0..32 {
            st.x[i] = 0x1000 + i as u64;
        }
        st.x[31] = 0x7000_0000; // sp
        st.x[0] = 0xDEAD; // will be overridden by the syscall-return 0
        st.pc = 0x402000; // interrupted pc (post-svc continuation)
        st.tpidr = 0x1234_5678;
        st.nzcv = 0xABCD;

        // Install a real handler so dispatch_current_thread enters begin_handler
        // (no default-ignore / default-terminate branch).
        let mut act = vec![0u8; 32];
        unsafe {
            (act.as_mut_ptr() as *mut u64).write_unaligned(0x1234u64);
        }
        let r = rt_sigaction(10, act.as_mut_ptr() as u64, 0);
        assert_eq!(r, 0);

        dispatch_current_thread(&mut st, 10, 0x402004);

        // Handler ABI: x0=signo, x1=siginfo*, x2=ucontext*, x30=SIGRET, pc
        // unchanged (redirect_request drives the handler run), redirect set.
        assert_eq!(st.redirect_request, 0x1234, "handler queued to run");
        assert_eq!(st.x[0], 10, "x0 = signo");
        assert!(
            st.x[1] != 0 && st.x[2] != 0,
            "siginfo/ucontext pointers set ({:#x},{:#x})",
            st.x[1],
            st.x[2]
        );
        assert_eq!(st.x[30], SIGRET, "x30 = SIGRET");
        assert!((st.x[1] & 15) == 0, "siginfo pointer naturally aligned");

        // siginfo header: si_signo=10, si_errno=0 (si_code omitted; offset 0/8).
        let si = unsafe { std::ptr::read_unaligned(st.x[1] as *const i32) };
        assert_eq!(si, 10, "siginfo.si_signo");

        // sigreturn restores the interrupted context wholesale.
        let ok = sigreturn(&mut st);
        assert!(ok, "sigreturn finds a saved frame");
        assert_eq!(st.pc, 0x402004, "resumes right after the interrupted svc");
        assert_eq!(st.tpidr, 0x1234_5678);
        assert_eq!(st.nzcv, 0xABCD);
        assert_eq!(st.x[31], 0x7000_0000, "sp restored");
        for i in 1..31 {
            assert_eq!(
                st.x[i],
                0x1000 + i as u64,
                "x{i} restored (except x0 overridden)"
            );
        }
        // x0 was overridden to 0 (syscall return), not restored to 0xDEAD.
        assert_eq!(st.x[0], 0, "x0 forced to syscall-return 0");

        // A second sigreturn (no saved frame) is a stray — returns false.
        assert!(!sigreturn(&mut st), "stray sigreturn detected");
    }
}
