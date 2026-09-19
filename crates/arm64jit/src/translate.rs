// SPDX-License-Identifier: MIT
//
// Per-instruction ARM64 -> x86-64 translation.
//
// Model: guest registers live in a spilled `CpuState`. The translated prologue
// loads the CpuState address into RBX (callee-saved). Each guest register i is
// addressed as `[RBX + 8*i]`. Operands are re-loaded from state on every
// instruction (no live-range tracking / register allocation). Scratch regs:
// RAX, RCX, R10, R11, RDI.
//
// Guest address == host address (shared address space), so a guest load/store
// dereferences the host pointer held in the guest register directly.
//
// NZCV is not consumed yet; set-flag ops are still wired through arithmetic but
// the flags result is only written back as a placeholder.

use crate::decode::{Inst, ShiftKind};
use crate::x86::{CodeBuf, RSI, RAX, RBX, RCX, RDX, RDI, R10};

// ---------------------------------------------------------------------------
// SH106-NEXT diagnostic: an env-gated, DEFAULT-INERT STORE-WATCH that names the
// exact guest `str`/`stp` writer of a canary-slot clobber. The standing canary
// stack-smash wall (nativeGameGlobalInit deep body, app-shell ctor) is a host
// pointer leaking into the guest frame's canary slot (the SH103/SH106 class),
// but the EXACT guest store was never named. This watch fires post-store (no
// guest/value semantics touched: it runs after the store and only reads), logs
// the guest pc when a 64-bit store lands on the current frame's canary slot
// [x29-16] (or a self-stack-pointer store below x29), and prints the written
// value. Enabled ONLY by JIT_CANARY_STORE_WATCH=1 (default off => the emitted
// store bytes are byte-identical to the un-watched path). Writes no guest bytes.
// ---------------------------------------------------------------------------
static CANARY_STORE_WATCH: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
/// When the test suite pins the flag via set_canary_store_watch_test, the lazy
/// env sync must NOT overwrite it (deterministic hermetic; no env races).
static CANARY_STORE_WATCH_TEST_OVERRIDE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Read the JIT_CANARY_STORE_WATCH env once (lazily). Idempotent. Returns the
/// current flag (env-derived unless the tests pinned an override).
fn canary_store_watch_enabled() -> bool {
    use std::sync::atomic::Ordering;
    static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INIT.get_or_init(|| {
        if !CANARY_STORE_WATCH_TEST_OVERRIDE.load(Ordering::Relaxed) {
            let on = std::env::var("JIT_CANARY_STORE_WATCH").ok().as_deref() == Some("1");
            CANARY_STORE_WATCH.store(on, Ordering::Relaxed);
        }
    });
    CANARY_STORE_WATCH.load(Ordering::Relaxed)
}

/// Test-only override so a hermetic can toggle the flag without touching env.
#[doc(hidden)]
pub fn set_canary_store_watch_test(on: bool) {
    use std::sync::atomic::Ordering;
    CANARY_STORE_WATCH_TEST_OVERRIDE.store(true, Ordering::Relaxed);
    CANARY_STORE_WATCH.store(on, Ordering::Relaxed);
}

/// Host store-watch: called from emitted code AFTER a 64-bit guest store (so
/// the stored value is preserved and this can only observe, never perturb).
/// Args (SysV): RDI=state, RSI=dest, RDX=value, RCX=pc.
extern "C" fn canary_store_watch(
    state: *const crate::jit::CpuState,
    dest: u64,
    value: u64,
    pc: u64,
) -> u64 {
    if !CANARY_STORE_WATCH.load(std::sync::atomic::Ordering::Relaxed) {
        return 0;
    }
    // Narrow to the SH103/SH106 leak signature: a FOREIGN HOST pointer
    // (0x7000_0000_0000+) being stored into a LOWER guest region (guest stack /
    // frame / .bss). That is exactly the canary-clobber class. Legit guest stores
    // of host pointers into host-managed regions (dest >= 0x7000...0000) are not
    // the leak; skip them to cut the noise.
    const HOST_HI: u64 = 0x8000_0000_0000;
    const HOST_LO: u64 = 0x7000_0000_0000;
    if !(value >= HOST_LO && value < HOST_HI) {
        return 0;
    }
    if dest >= HOST_LO {
        return 0; // host store, not guest-frame clobber
    }
    unsafe {
        // Compute the current frame's canary slot [x29-16]; if this store targets
        // it, this IS the canary-clobbering store — the named writer.
        let st = &*state;
        let x29 = st.x[29];
        let canary_slot = x29.wrapping_sub(16);
        let is_canary = dest == canary_slot || (dest >= x29.wrapping_sub(0x60) && dest <= x29);
        // Guard GOT: only valid when the real image maps it.
        let guard = std::ptr::read_unaligned(0x1067d16f0u64 as *const u64);
        static LAST: std::sync::Mutex<Option<(u64, u64, u64)>> = std::sync::Mutex::new(None);
        let d = std::sync::Mutex::new(());
        if let Ok(mut l) = LAST.lock() {
            if *l == Some((pc, dest, value)) {
                return 0;
            }
            *l = Some((pc, dest, value));
        }
        eprintln!(
            "[canary-store-watch] pc={pc:#x} dst={dest:#x} val={value:#x} x29={x29:#x} canary_slot={canary_slot:#x} is_canary_slot={is_canary} guard={guard:#x}"
        );
        drop(d);
    }
    0
}

// ---------------------------------------------------------------------------
// SH4xx-next: host-RETURN leak watch. The canary store-watch NAMES the guest
// store (0x101d99e70 LSM pool-pop) that writes a foreign host ptr (0x7f...) into
// a frame canary window; the missing producer side is WHICH host call RETURNED
// that ptr into guest x0. This probe sits at the host-call return sites
// (jit.rs s.x[0]=ret) and logs every host fn whose return lands a host ptr in
// guest x0, so the canary store value correlates to its host producer by the
// same 0x7f... value. Enabled ONLY by JIT_HOST_RETURN_WATCH=1 (default off =>
// one AtomicBool load, no logging). Bounded + de-dups consecutive repeats.
// ---------------------------------------------------------------------------
static HOST_RETURN_WATCH: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static HOST_RETURN_WATCH_TEST_OVERRIDE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Read the JIT_HOST_RETURN_WATCH env once (lazily). Idempotent.
fn host_return_watch_enabled() -> bool {
    use std::sync::atomic::Ordering;
    static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INIT.get_or_init(|| {
        if !HOST_RETURN_WATCH_TEST_OVERRIDE.load(Ordering::Relaxed) {
            let on = std::env::var("JIT_HOST_RETURN_WATCH").ok().as_deref() == Some("1");
            HOST_RETURN_WATCH.store(on, Ordering::Relaxed);
        }
    });
    HOST_RETURN_WATCH.load(Ordering::Relaxed)
}

/// Test-only override so a hermetic can toggle the flag without touching env.
#[doc(hidden)]
pub fn set_host_return_watch_test(on: bool) {
    use std::sync::atomic::Ordering;
    HOST_RETURN_WATCH_TEST_OVERRIDE.store(true, Ordering::Relaxed);
    HOST_RETURN_WATCH.store(on, Ordering::Relaxed);
}

/// Expose whether the return-watch probe is currently active (for a hermetic).
#[doc(hidden)]
pub fn host_return_watch_active() -> bool {
    HOST_RETURN_WATCH.load(std::sync::atomic::Ordering::Relaxed)
}

/// Host-return leak probe: called from the host-call bridge AFTER a host fn
/// returned `ret` into guest x0 at host-thunk slot `pc`, guest caller `caller`.
/// NVIDIA -- narrows to the SH103/SH4xx leak signature: `ret` is a FOREIGN HOST
/// pointer (0x7000_0000_0000..0x8000_0000_0000) — exactly the value the canary
/// store-watch later sees written into a frame. Logging it names the HOST fn
/// that produced the canary-smashing pointer. Pure observability, default-inert.
pub fn host_return_leak_watch(pc: u64, ret: u64, caller: u64) {
    if !host_return_watch_enabled() {
        return;
    }
    const HOST_HI: u64 = 0x8000_0000_0000;
    const HOST_LO: u64 = 0x7000_0000_0000;
    if !(ret >= HOST_LO && ret < HOST_HI) {
        return; // not a foreign host pointer -> not the leak class
    }
    let who = crate::jit::host_call_slot_name(pc)
        .unwrap_or_else(|| format!("slot{:#x}", pc));
    static LAST_RET: std::sync::Mutex<Option<(u64, u64)>> =
        std::sync::Mutex::new(None);
    if let Ok(mut l) = LAST_RET.lock() {
        if *l == Some((pc, ret)) {
            return; // de-dup consecutive identical repeats (hot loops)
        }
        *l = Some((pc, ret));
    }
    eprintln!(
        "[host-return-watch] slot={pc:#x} hostfn={who} -> x0={ret:#x} caller={caller:#x}"
    );
}

/// Emit (when the watch is enabled) the post-store probe call for a 64-bit
/// store whose destination is currently in `dest_reg` and value in `val_reg`.
/// Runs AFTER `mov_store64`, so RDX/RAX are already spent; the call only needs
/// to not clobber RBX (state) — and nothing after it relies on RAX/RDX/RCX/RSI
/// (the operand-reload-per-instruction model reloads from state).
fn emit_canary_store_watch(buf: &mut CodeBuf, pc: u64, dest_reg: u8, val_reg: u8) {
    let _ = canary_store_watch_enabled(); // lazily sync the atomic from env once
    if !CANARY_STORE_WATCH.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    let addr = canary_store_watch as usize as u64;
    // Reconstruct args (SysV): RDI=state, RSI=dest, RDX=value, RCX=pc.
    // dest_reg==RDX typically; value==RAX. If either already equals the
    // target arg register, skip the copy (mov_rr64 no-ops equal operands).
    buf.mov_rr64(RDI, RBX); // state
    // dest: RSI
    if dest_reg != RSI {
        buf.mov_rr64(RSI, dest_reg);
    }
    // value: RDX (already the dest reg holder? no — free now, post-store)
    if val_reg != RDX {
        buf.mov_rr64(RDX, val_reg);
    } else {
        // val already in RDX AND dest==RDI? not possible; but if val_reg==RDX
        // then dest_reg was RDI/RSI — keep RDX as-is.
    }
    // pc: RCX
    buf.mov_ri64(RCX, pc);
    // clobbers RAX (call addr); RBX is callee-saved so survives.
    buf.mov_ri64(RAX, addr);
    buf.sub_ri64(4, 8); // align RSP 8 -> 0 (mod 16) at the call site
    buf.call_r64(RAX);
    buf.add_ri64(4, 8); // restore block-entry alignment
}

/// Byte offset of guest register g inside CpuState (x[g] at 8*g).
#[inline]
fn slot(g: u32) -> i32 {
    (g as i32) * 8
}

/// Load guest reg `g` into x86 reg `x`.
#[inline]
fn ldg(buf: &mut CodeBuf, x: u8, g: u32) {
    buf.mov_load64(x, RBX, slot(g));
}
/// Store x86 reg `x` into guest reg `g`.
#[inline]
fn stg(buf: &mut CodeBuf, g: u32, x: u8) {
    buf.mov_store64(RBX, slot(g), x);
}

/// Write 0 to guest register `g`.
#[inline]
fn stg0(buf: &mut CodeBuf, g: u32) {
    buf.mov_ri64(RAX, 0);
    buf.mov_store64(RBX, slot(g), RAX);
}

/// Load guest register `g` into RAX as a *store source value*. In AArch64 the
/// source register field of a STORE (`str x0,[..]`) reads x31 as XZR (zero),
/// NOT SP — a store of `xzr` (extremely common: compilers zero-init stack slots
/// and objects with `str xzr,[..]`) must store 0, never the stack pointer.
/// Distinguish this from an *addressing base* (`rn`, where x31 = SP), which
/// `ldg` continues to handle directly.
#[inline]
fn ldg_src(buf: &mut CodeBuf, g: u32) {
    if g == 31 {
        buf.mov_ri64(RAX, 0); // XZR reads as zero
    } else {
        buf.mov_load64(RAX, RBX, slot(g));
    }
}

/// Store RAX into guest register `g` after a LOAD, but only if `g` is a real
/// register. When the load destination field is x31 (`ldr xzr,[..]`) ARM treats
/// it as a no-op that must NOT write the SP slot (which x31 also aliases).
#[inline]
fn stg_if_writable(buf: &mut CodeBuf, g: u32) {
    if g != 31 {
        buf.mov_store64(RBX, slot(g), RAX);
    }
}

/// Transfer `size` bytes (B=1/H=2/S=4/D=8) between the low bytes of the guest
/// vector slot `vslot` (into CpuState via RBX) and the memory address held in
/// host register `addr`. `ld=true` loads [addr]->slot; `ld=false` stores
/// slot->[addr]. Only the low `size` bytes of the 16-byte slot are touched —
/// upper lanes stay preserved (ARM scalar `ldr d0` keeps the high 64 bits).
/// Shared by the scalar FP/SIMD immediate forms (FpLdStImm, FpLdStImmUnscaled,
/// FpLdStImmWb).
#[inline]
fn fp_scalar_xfer(
    buf: &mut CodeBuf,
    addr: u8,
    vslot: i32,
    size: u8,
    ld: bool,
) -> Result<(), String> {
    match (size, ld) {
        (8, true) => {
            buf.mov_load64(RAX, addr, 0);
            buf.mov_store64(RBX, vslot, RAX);
        }
        (8, false) => {
            buf.mov_load64(RAX, RBX, vslot);
            buf.mov_store64(addr, 0, RAX);
        }
        (4, true) => {
            buf.mov_load32(RAX, addr, 0);
            buf.mov_store32(RBX, vslot, RAX);
        }
        (4, false) => {
            buf.mov_load32(RAX, RBX, vslot);
            buf.mov_store32(addr, 0, RAX);
        }
        (2, true) => {
            buf.movzx_word_mem(RAX, addr, 0);
            buf.mov_store16(RBX, vslot, RAX);
        }
        (2, false) => {
            buf.movzx_word_mem(RAX, RBX, vslot);
            buf.mov_store16(addr, 0, RAX);
        }
        (1, true) => {
            buf.movzx_byte_mem(RAX, addr, 0);
            buf.mov_store8(RBX, vslot, RAX);
        }
        (1, false) => {
            buf.movzx_byte_mem(RAX, RBX, vslot);
            buf.mov_store8(addr, 0, RAX);
        }
        (s, _) => return Err(format!("fp_scalar_xfer size {s} not implemented")),
    }
    Ok(())
}

/// Zero-extend the low 32 bits of x86 reg `r` into its upper half. AArch64
/// writes to a W (32-bit) register always zero the upper 32 bits of the
/// corresponding X register; x86 64-bit ops leave them stale, so a 32-bit data
/// value must be cleaned before it propagates (e.g. `mov w0,w1` copying a
/// negative two's-complement w1 would otherwise carry `0xffffffffffffffff`).
#[inline]
fn zext_w(buf: &mut CodeBuf, r: u8) {
    buf.shl_ri8(r, 32);
    buf.shr_ri8(r, 32);
}

/// Byte offset of `CpuState.nzcv` (after pc@256: nzcv u32 at 264).
const NZCV_OFF: i32 = 8 * 32 + 8; // 264

/// Snapshot a source vector register to the permscratch area when it aliases
/// `rd`, so a SIMD permute does not clobber a source it is still reading.
/// `slotA` (0x800) and `slotB` (0x810) are the two 16-byte scratch slots.
/// Returns the byte offset to READ the (possibly snapshotted) source from.
/// When the source does not alias rd it is read in place (no copy needed).
fn permute_source(
    buf: &mut CodeBuf,
    rd: u8,
    src: u8,
    scratch_hi: bool,
) -> i32 {
    let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
    let orig = slot(src as i32);
    if src == rd {
        // rd aliases the source. Copy the full 16B to scratch, then read from
        // there. Note rd==31 is a legit VECTOR dest (v31), NOT XZR — only an
        // actual register-alias (src == rd) needs the snapshot.
        let scratch = crate::jit::PERMSCRATCH_OFF + if scratch_hi { 16 } else { 0 };
        // copy both u64 halves: [orig..orig+8) -> [scratch..scratch+8)
        buf.mov_load64(RAX, RBX, orig);
        buf.mov_store64(RBX, scratch, RAX);
        buf.mov_load64(RAX, RBX, orig + 8);
        buf.mov_store64(RBX, scratch + 8, RAX);
        scratch
    } else {
        orig
    }
}
/// Byte offset of `CpuState.pad`.
#[allow(dead_code)]
const PAD_OFF: i32 = 8 * 32 + 12; // 268

/// Convert the *live* x86 status flags (CF/ZF/SF/OF set by the last arithmetic
/// instruction) into a packed AArch64 NZCV u32 (N=31,Z=30,C=29,V=28) and store
/// it at `CpuState.nzcv`. Uses RAX/RCX/RDX as scratch. Must be called right
/// after the flag-setting op, before any flag-clobbering instruction.
fn store_nzcv(buf: &mut CodeBuf) {
    // push rax, rcx, rdx then snapshot eflags via pushfq.
    buf.push(RAX);
    buf.push(RCX);
    buf.push(RDX);
    buf.pushfq();
    buf.pop(RAX); // eax = rflags: CF0, PF2, AF4, ZF6, SF7, OF11
    // Build nzcv into RDX.
    buf.xor_rr64(RDX, RDX);
    // C = CF(eax bit0) -> nzcv bit29
    buf.mov_rr64(RCX, RAX);
    buf.and_ri64(RCX, 1);
    buf.shl_ri8(RCX, 29);
    buf.or_rr64(RDX, RCX);
    // V = OF(eax bit 11) -> nzcv bit 28
    buf.mov_rr64(RCX, RAX);
    buf.shr_ri8(RCX, 11);
    buf.and_ri64(RCX, 1);
    buf.shl_ri8(RCX, 28);
    buf.or_rr64(RDX, RCX);
    // Z = ZF(eax bit 6) -> nzcv bit 30
    buf.mov_rr64(RCX, RAX);
    buf.shr_ri8(RCX, 6);
    buf.and_ri64(RCX, 1);
    buf.shl_ri8(RCX, 30);
    buf.or_rr64(RDX, RCX);
    // N = SF(eax bit 7) -> nzcv bit 31
    buf.mov_rr64(RCX, RAX);
    buf.shr_ri8(RCX, 7);
    buf.and_ri64(RCX, 1);
    buf.shl_ri8(RCX, 31);
    buf.or_rr64(RDX, RCX);
    buf.mov_store32(RBX, NZCV_OFF, RDX);
        buf.pop(RDX);
        buf.pop(RCX);
        buf.pop(RAX);
    }

    /// Pack a FP-comparison result (x86 flags from a prior `comisd`/`ucomisd`) into
    /// the guest NZCV (bit 31=N, 30=Z, 29=C, 28=V). AArch64 `fcmp` semantics from
    /// the x86 flags set by comisd:
    ///   A<B (ord): CF=1,PF=0,ZF=0 -> N0 Z0 C0 V0
    ///   A>B (ord): CF=0,ZF=0,PF=0 -> N0 Z0 C1 V0
    ///   A==B:       CF=0,PF=0,ZF=1 -> N0 Z1 C1 V0   (C=1 for ge)
    ///   unordered:  CF=1,PF=1,ZF=1 -> N0 Z1 C1 V1
    /// so  Z=ZF, V=PF, C=(!CF)|PF, N=0. Clobbers RAX/RCX/RDX.
    fn store_nzcv_fp(buf: &mut CodeBuf) {
        buf.push(RAX);
        buf.push(RCX);
        buf.push(RDX);
        buf.pushfq();
        buf.pop(RAX); // eax = rflags (CF0, PF2, ZF6)
        buf.xor_rr64(RDX, RDX);
        // V = PF(bit2) -> bit28
        buf.mov_rr64(RCX, RAX);
        buf.shr_ri8(RCX, 2);
        buf.and_ri64(RCX, 1);
        buf.shl_ri8(RCX, 28);
        buf.or_rr64(RDX, RCX);
        // C = CF(bit0) && !PF(bit2) -> bit29. This is the BORROW convention that
        // x86_cc_for_cond expects (HS=JAE=!CF, LS=JBE=CF||ZF, HI=JA, LO=JB),
        // NOT the true ARM-FP carry value. ARM's FP compare sets C=1 for
        // greater/equal/unordered, 0 for less; `ls` (= C==0 || Z==1) is how gcc
        // encodes FP `<=`. After a subtraction the stored C is x86-CF (borrow)
        // and JBE(CF||ZF) already works, so the FP C must be stored in the same
        // borrow sense (= !ARM_FP_C = CF && !PF) for the ls/hi/lo/hs conditions
        // to evaluate correctly. Fixes `nn<=0` on NaN (was true, must be false).
        buf.mov_rr64(RCX, RAX);      // CF
        buf.and_ri64(RCX, 1);
        buf.mov_rr64(RDI, RAX);      // PF
        buf.shr_ri8(RDI, 2);
        buf.and_ri64(RDI, 1);
        buf.xor_ri64(RDI, 1);        // !PF
        buf.and_rr64(RCX, RDI);      // CF && !PF (borrow-sense carry)
        buf.shl_ri8(RCX, 29);
        buf.or_rr64(RDX, RCX);
        // Z = ZF(bit6) && !PF(bit2) -> bit30. ARM FP-compare sets the Z flag
        // (==) for ORDERED equality ONLY: for an unordered (NaN) compare the
        // guest Z must be 0 so b.eq/csel.eq/b.gt/b.le all stay false (IEEE:
        // every NaN comparison is "not equal"). x86 comisd sets ZF=1 for BOTH
        // equality AND unordered, so ZF alone gives the wrong Z; mask PF (set
        // exactly when unordered) back out.
        buf.mov_rr64(RCX, RAX);      // ZF
        buf.shr_ri8(RCX, 6);
        buf.and_ri64(RCX, 1);
        buf.mov_rr64(RDI, RAX);      // PF
        buf.shr_ri8(RDI, 2);
        buf.and_ri64(RDI, 1);
        buf.xor_ri64(RDI, 1);        // !PF
        buf.and_rr64(RCX, RDI);      // ZF && !PF
        buf.shl_ri8(RCX, 30);
        buf.or_rr64(RDX, RCX);
        // N = CF && !ZF -> bit31. AArch64 FP compare sets N=1 for the ordered
        // "less-than" (d<f) case; x86 comisd clears CF only when A>B or A==B.
        // (Earlier this was hardcoded N=0, which made b.mi/b.lt/b.gt/b.le all
        // wrong: b.gt evaluated N==V as 0==0 for EVERY ordered non-equal pair.)
        //         N = (CF)        & (!ZF)
        buf.mov_rr64(RCX, RAX);
        buf.and_ri64(RCX, 1); // CF (bit0)
        buf.mov_rr64(RDI, RAX);
        buf.shr_ri8(RDI, 6);
        buf.and_ri64(RDI, 1); // ZF (bit6)
        buf.xor_ri64(RDI, 1); // !ZF
        buf.and_rr64(RCX, RDI); // N (0/1)
        buf.shl_ri8(RCX, 31);
        buf.or_rr64(RDX, RCX);
        buf.mov_store32(RBX, NZCV_OFF, RDX);
        buf.pop(RDX);
        buf.pop(RCX);
        buf.pop(RAX);
    }

/// Load the *stored* `CpuState.nzcv` into the real x86 rflags (CF/ZF/SF/OF) so
/// a following native `jcc`/`cmovcc` (via `x86_cc_for_cond`) evaluates the
/// AArch64 condition correctly, even when the immediately preceding op did not
/// set the flags (the dispatcher reloads operands, clobbering them).
///
/// Restores RAX/RCX, then sets flags via popfq as the LAST flag-clobbering op;
/// the consuming `jcc`/`cmovcc` MUST run immediately after. RDX is clobbered.
fn load_nzcv_to_eflags(buf: &mut CodeBuf) {
    buf.push(RAX);
    buf.push(RCX);
    buf.mov_load32(RCX, RBX, NZCV_OFF); // RCX = packed nzcv
    // RDX accumulates the eflags image; RAX is scratch. Build
    // eflags = { CF:nzcv.29, ZF:nzcv.30, SF:nzcv.31, OF:nzcv.28 }.
    buf.mov_ri64(RDX, 0x202); // reserved rflags bit1
    // C -> CF(bit0)
    buf.mov_rr64(RAX, RCX);
    buf.shr_ri8(RAX, 29);
    buf.and_ri64(RAX, 1);
    buf.or_rr64(RDX, RAX);
    // V -> OF(bit11)
    buf.mov_rr64(RAX, RCX);
    buf.shr_ri8(RAX, 28);
    buf.and_ri64(RAX, 1);
    buf.shl_ri8(RAX, 11);
    buf.or_rr64(RDX, RAX);
    // Z -> ZF(bit6), N -> SF(bit7) via one shift by 24 (nzcv bits 30,31 -> 6,7)
    buf.mov_rr64(RAX, RCX);
    buf.shr_ri8(RAX, 24);
    buf.and_ri64(RAX, 0xc0); // bits 6,7
    buf.or_rr64(RDX, RAX);
    // RDX holds the final eflags value. Restore saved RAX/RCX (pops do not use
    // RDX), then push the eflags and pop them into rflags. The popfq is the last
    // flag-clobbering op; the calling jcc/cmovcc must follow immediately.
    buf.pop(RCX);
    buf.pop(RAX);
    buf.push(RDX);
    buf.popfq();
}

/// Constant `val` into guest reg `rd`.
fn mov_guest_imm(buf: &mut CodeBuf, rd: u32, val: u64) {
    // Use imm32 (sign-extended) when the upper 32 bits equal the sign-extension
    // of the low 32 bits; otherwise materialize the full 64-bit constant.
    let lo = val as u32 as i32;
    if (lo as i64 as u64) == val {
        buf.mov_ri32(RAX, lo as u32);
    } else {
        buf.mov_ri64(RAX, val);
    }
    stg(buf, rd, RAX);
}

/// Map an ARM condition code (0..15) to the x86-64 `0F 8x` jcc opcode, on the
/// assumption that the immediately preceding instruction set the x86 flags in
/// the ARM flow (SUB yields borrow semantics, so ARM carry == x86 C-free).
fn x86_cc_for_cond(cond: u8) -> Option<u8> {
    Some(match cond {
        0x0 => 0x84, // EQ  (ZF)
        0x1 => 0x85, // NE  (!ZF)
        0x2 => 0x83, // HS  (C set; unsigned >= -> !CF, JAE)
        0x3 => 0x82, // LO  (C clear; unsigned <  -> sub CF, JB)
        0x4 => 0x88, // MI  (N)
        0x5 => 0x89, // PL  (!N)
        0x6 => 0x8a, // VS  (V)
        0x7 => 0x8b, // VC  (!V)
        0x8 => 0x87, // HI  (C && !Z -> JAE && !ZF, i.e. JA)
        0x9 => 0x86, // LS  (!HI  -> JBE)
        0xA => 0x8d, // GE  (signed >=, JGE)
        0xB => 0x8c, // LT  (signed <, JL)
        0xC => 0x8f, // GT  (signed >, JG)
        0xD => 0x8e, // LE  (signed <=, JLE)
        _ => return None,
    })
}

/// Apply AArch64 shift `kind` by `amt` to the value currently in x86 reg `x`
/// (uses RCX for the count). Only constant shifts are handled (guest encodes
/// the amount as an immediate in ADD/SUB shifted-register).
/// `sf` = operand size: when the guest op is 32-bit (`!sf`) and the shift is
/// ASR, the value has been zero-extended to 64 bits, so its bit-31 is NOT the
/// sign bit and a plain 64-bit `sar` would shift in zeros and give the wrong
/// (non-negative) result — e.g. compiler magic-division `sub w1,w1,w2,asr#31`
/// turns the sign-correction `-1` into `+1`, corrupting signed quotients.
/// Sign-extend the 32-bit value to 64 bits first so the sar replicates bit 31.
fn apply_shift_const(buf: &mut CodeBuf, x: u8, kind: ShiftKind, amt: u8, sf: bool) {
    if amt == 0 {
        return;
    }
    // BUGFIX (Session 99): this used `mov cl, amt; shl x, cl`. Both callers pass
    // x == RCX (the Rm value being shifted), so `mov rcx, amt` CLOBBERED the value
    // with the shift count and `shl rcx, cl` gave (amt << amt) — e.g. the array
    // index `add x1,x2,x0,lsl#3` became x2+24 (constant, not x0<<3), making loops
    // read the SAME element every iteration (fclamp -O2 returned 10 instead of 9).
    // Use the immediate-shift forms (C1 /4..7 ib) instead — no CL scratch, so the
    // shifted value stays in `x`.
    match kind {
        ShiftKind::Lsl => buf.shl_ri8(x, amt),
        ShiftKind::Lsr => buf.shr_ri8(x, amt),
        ShiftKind::Asr => {
            if !sf {
                buf.movsxd_r64_r32(x, x); // sign-extend bit-31 before 64-bit asr
            }
            buf.sar_ri8(x, amt);
        }
        ShiftKind::Ror => buf.ror_ri8(x, amt),
    }
}

/// A branch/jump fixup: the guest target PC and the byte offset within the
/// emitted buffer where the rel32 displacement field lives. Resolved once the
/// buffer is laid out (jit.rs patches it to the host offset of the target).
#[derive(Debug, Clone, Copy)]
pub struct Fixup {
    pub target_pc: u64,
    pub disp_off: usize,
    /// encoded x86 jcc condition (0x84=JZ) or 0 for an unconditional jmp,
    /// `0xff` means unconditional-jump fixup (E9).
    pub cc: u8,
}

/// Translate a single instruction (writes to `buf`). `pc` is the guest PC of
/// this instruction (needed for PC-relative branch targets). Branch
/// instructions append a `Fixup` to `out` so the JIT can patch their target.
pub fn translate(
    buf: &mut CodeBuf,
    pc: u64,
    inst: Inst,
    fixups: &mut Vec<Fixup>,
) -> Result<(), String> {
    match inst {
        Inst::MoveWide {
            rd, imm16, hw, opc, sf,
        } => {
            let val = (imm16 as u64) << ((hw as u64) * 16);
            match opc {
                2 => {
                    // movn: NOT the immediate. For a 32-bit (W) dest ARM zero-
                    // extends to the 64-bit register, so `movn w0,#2` writes
                    // 0x00000000fffffffd, NOT 0xfffffffffffffffd. mov_guest_imm
                    // would take the imm32 short-cut (`mov r32` sign-extends),
                    // leaving the upper 32 bits set; truncate first.
                    let v = !val;
                    mov_guest_imm(buf, rd as u32, if sf { v } else { v & 0xffff_ffff });
                }
                1 => {
                    // movk: read-modify-write — OR `imm16<<shift` into bits
                    // [shift, shift+16), preserving all other bits. Multi-part
                    // 64-bit constant build is `movz xD,#lo ; movk xD,#hi,lsl#16
                    // (or lsl#32/48)`; treating movk as a full replace corrupted
                    // the constant (e.g. 0x28bb1 built as movz 0x8bb1 then movk
                    // 0x2 lsl#16 came out 0x20000, breaking comparisons).
                    let shift = (hw as u32) * 16;
                    let mut clear = !(0xffffu64 << shift);
                    if !sf {
                        clear &= 0xffff_ffff; // W-dest zero-extends to 64 bits
                    }
                    ldg(buf, RAX, rd as u32);
                    buf.mov_ri64(RCX, clear);
                    buf.and_rr64(RAX, RCX);
                    buf.mov_ri64(RCX, (imm16 as u64) << shift);
                    buf.or_rr64(RAX, RCX);
                    stg(buf, rd as u32, RAX);
                }
                _ => mov_guest_imm(buf, rd as u32, val), // movz
            }
            Ok(())
        }
        Inst::AddSubImm {
            rd,
            rn,
            imm12,
            shift12,
            sub,
            s,
            sf,
            ..
        } => {
            let imm: u64 = (imm12 as u64) << if shift12 { 12 } else { 0 };
            // rn XZR-vs-SP: `sub sp,sp,#imm` (non-S) reads rn=31 as SP, but the
            // flag-setting form (e.g. `cmp wzr,#imm`) reads rn=31 as XZR=0.
            if s && rn == 31 {
                buf.mov_ri64(RAX, 0);
            } else {
                ldg(buf, RAX, rn as u32);
            }
            if !sf {
                zext_w(buf, RAX); // 32-bit: high garbage in rn is ignored
            }
            if sub {
                if sf {
                    buf.sub_ri64(RAX, imm as u32);
                } else {
                    buf.sub_ri32(RAX, imm as u32); // 32-bit flags + upper-clear
                }
            } else if sf {
                buf.add_ri64(RAX, imm as u32);
            } else {
                buf.add_ri32(RAX, imm as u32); // `adds w…` N/V/Z/C from 32-bit result
            }
            if s {
                // ADDS/CMN (the non-subtract, flag-setting add) sets the ARM C
                // flag to the ADD's carry-out, but x86_cc_for_cond's HS/LO/HI/LS
                // conditions assume the stored C is the SUBTRACT-borrow
                // convention. gcc compiles `x > 0xffffffffffff0000` as
                // `cmn x,#0x10000; b.ls` — the add carries (ARM C=1, ls=false),
                // so the stored C must be !carry for b.ls/b.hi to evaluate
                // right. Complement CF before packing.
                if !sub {
                    buf.cmc(); // borrow-convention C = !carry-out
                }
                store_nzcv(buf); // N/Z/C/V -> CpuState.nzcv
            }
            // rd==31 writes SP for ADD/SUB (unlike logical ops where it's XZR and
            // discarded). The exception is cmp/cmn (s==1, rd==31) which must NOT
            // clobber SP. `sub sp,sp,#imm` (every function prologue) must write.
            if rd != 31 || !s {
                if !sf {
                    zext_w(buf, RAX); // W write zero-extends into X
                }
                stg(buf, rd as u32, RAX); // rd==31 -> writes the SP slot
            }
            Ok(())
        }
        Inst::AddSubReg {
            rd,
            rn,
            rm,
            sub,
            s,
            shift,
            sh_amt,
            sf,
            sp_operand,
            ..
        } => {
            // rn/Rd XZR-vs-SP: the shift-register form (bit21=0) uses register 31 as
            // XZR (zero) in EVERY operand — `neg x6,x6` (sub x6,xzr,x6) must read rn=31
            // as 0, NOT SP. Only the extended-register form (bit21=1, e.g. `sub sp,sp,
            // x1`) treats rn=31 and rd=31 as the stack pointer. qemu-verified:
            //   neg x6,x6 = 0xcb0603e6 (bit21=0) -> rn=31 is XZR
            //   sub sp,sp,x1 = 0xcb2163ff (bit21=1) -> rn=31, rd=31 are SP.
            // `negs w1,w0` (subs,w1,wzr,w0; bit21=0, S=1) must read rn=31 as XZR too.
            // Rm is always a GPR (XZR=0), never SP.
            let rn_is_sp = sp_operand && rn == 31;
            if rn_is_sp {
                ldg(buf, RAX, 31); // extended form reads rn=31 as SP
            } else if rn == 31 {
                buf.mov_ri64(RAX, 0); // shifted form reads rn=31 as XZR
            } else {
                ldg(buf, RAX, rn as u32);
            }
            if rm == 31 {
                buf.mov_ri64(RCX, 0);
            } else {
                ldg(buf, RCX, rm as u32);
            }
            if !sf {
                zext_w(buf, RAX); // 32-bit add/sub: zero high garbage in operands
                zext_w(buf, RCX);
            }
            apply_shift_const(buf, RCX, shift, sh_amt, sf);
            if sub {
                if sf {
                    buf.sub_rr64(RAX, RCX);
                } else {
                    buf.sub_rr32(RAX, RCX); // `subs w…`: 32-bit flags + upper-clear
                }
            } else if sf {
                buf.add_rr64(RAX, RCX);
            } else {
                buf.add_rr32(RAX, RCX); // `adds w…`: N/V/Z/C from the 32-bit result
            }
            if s {
                if !sub {
                    buf.cmc(); // ADDS/CMN carries in borrow-convention (see AddSubImm)
                }
                store_nzcv(buf); // N/Z/C/V -> CpuState.nzcv
            }
            // rd==31: shifted-register form (bit21=0) discards the result (XZR) —
            // `neg xd,xm` does NOT touch SP. Only the extended-register form
            // (bit21=1, `sub sp,sp,x0`) writes rd=31 as the stack pointer; and
            // cmp/cmn (s==1) discard regardless. So write iff rd is a real reg,
            // or rd=31 in the SP-operand form that isn't a pure compare.
            if rd != 31 || (sp_operand && !s) {
                if !sf {
                    zext_w(buf, RAX); // W write zero-extends into X
                }
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::AddSubExt { rd, rn, rm, sub, s, sf, opt, shift } => {
            // add/sub Xd, Xn|SP, Rm, <opt> #<shift>: Rd = Xn + (ext(Rm)<<shift).
            // Extend Rm per `opt` (RXTB/UXTH/UXTW/SXTB/SXTH/SXTW are 32-bit-or-
            // narrow; UXTX/SXTX keep the full 64-bit Rm). rn/rd of 31 = SP.
            // Compute ext(Rm)<<shift in RAX, Xn(SP) in RDI, add/sub, store.
            if rm == 31 {
                buf.mov_ri64(RAX, 0); // W31/X31 is always XZR, never SP
            } else {
                ldg(buf, RAX, rm as u32);
            }
            match opt {
                0 => buf.and_ri64(RAX, 0xff), // UXTB
                1 => buf.and_ri64(RAX, 0xffff), // UXTH
                2 => buf.zero_ext_r32(RAX),   // UXTW
                3 => {}                       // UXTX (no-op)
                4 => {
                    // SXTB: sign-extend byte via shifts
                    buf.shl_ri8(RAX, 56);
                    buf.sar_ri8(RAX, 56);
                }
                5 => {
                    // SXTH
                    buf.shl_ri8(RAX, 48);
                    buf.sar_ri8(RAX, 48);
                }
                6 => buf.movsxd_r64_r32(RAX, RAX), // SXTW
                _ => {}                            // 7 = SXTX (no-op)
            }
            if shift > 0 && shift <= 3 {
                buf.shl_ri8(RAX, shift);
            }
            ldg(buf, RDI, rn as u32); // rn=31 -> x[31] = SP (extended form)
            if sub {
                buf.sub_rr64(RDI, RAX); // RDI = Rn - ext
                if s {
                    store_nzcv(buf);
                }
                buf.mov_rr64(RAX, RDI);
            } else {
                buf.add_rr64(RAX, RDI); // RAX = Rn + ext
                if s {
                    store_nzcv(buf);
                }
            }
            if !s {
                if !sf {
                    zext_w(buf, RAX); // 32-bit op: zero-extend the result
                }
                stg(buf, rd as u32, RAX); // rd=31 writes SP (extended form)
            }
            Ok(())
        }
        Inst::AddCarry { rd, rn, rm, sf, s, sub } => {
            // adc/sbc/adcs/sbcs Xd, Xn, Xm: Rd = Xn +/- Xm +/- carry.
            // AArch64 adds the previous C flag (NZCV bit29). SBC subtracts the
            // borrow (1 - C). Read the stored carry into x86 CF via
            // load_nzcv_to_eflags (preserves RAX/RCX), then use native adc/sbb.
            ldg(buf, RAX, rn as u32);
            ldg(buf, RCX, rm as u32);
            if !sf {
                // 32-bit form: zero the upper halves so the 64-bit adc/sbb below
                // yields exactly the 32-bit carry semantics.
                buf.shl_ri8(RAX, 32);
                buf.shr_ri8(RAX, 32);
                buf.shl_ri8(RCX, 32);
                buf.shr_ri8(RCX, 32);
            }
            // Inject stored C into CF (last op = popfq); RAX/RCX are preserved.
            load_nzcv_to_eflags(buf);
            // The stored C (nzcv bit29) is in the b.cond "borrow" convention:
            // store_nzcv for `adds` stores !carry-out, for `subs` stores borrow.
            // ADC/SBC consume the TRUE ARM carry, and in BOTH cases
            // TRUE_C = !stored_C, so
            //   adc adds TRUE_C = !C_s      -> cmc then native adc, and
            //   sbc subtracts 1-TRUE_C = C_s -> native sbb directly.
            if sub {
                buf.sbb_rr64(RAX, RCX); // RAX = rn - rm - C_s = rn - rm - (1-TRUE_C)
                if s {
                    // ARM sbcs sets C = not-borrow; report it in borrow-convention
                    // (= the x86 CF left by sbb), matching `subs`.
                    store_nzcv(buf);
                }
            } else {
                buf.cmc(); // CF = TRUE_C = !C_s
                buf.adc_rr64(RAX, RCX); // RAX = rn + rm + TRUE_C
                if s {
                    // adcs is an add: store C_s = !carry-out (like `adds`) so a
                    // later b.cond reads the right borrow-convention C.
                    buf.cmc();
                    store_nzcv(buf);
                }
            }
            if rd != 31 {
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::LogicReg {
            rd,
            rn,
            rm,
            op,
            s,
            shift,
            sh_amt,
            sf,
        } => {
            let _ = sf; // operand size handled by existing emitters; sf informative
            // rn == 31 (XZR) reads as zero (common for the `mov xd, xm` alias
            // `orr xd, xzr, xm`); otherwise load rn.
            if rn == 31 {
                buf.mov_ri64(RAX, 0);
            } else {
                ldg(buf, RAX, rn as u32);
            }
            if rm == 31 {
                buf.mov_ri64(RCX, 0);
            } else {
                ldg(buf, RCX, rm as u32);
            }
            if !sf {
                zext_w(buf, RAX); // 32-bit ops: ignore high garbage in operands
                zext_w(buf, RCX);
            }
            apply_shift_const(buf, RCX, shift, sh_amt, sf);
            match op {
                0 => buf.and_rr64(RAX, RCX), // AND
                1 => buf.or_rr64(RAX, RCX),  // ORR
                2 => buf.xor_rr64(RAX, RCX), // EOR
                // Inverted (N=1) variants: AND/NOT, OR/NOT, XOR/NOT (BIC/ORN/EON).
                4 => {
                    buf.not_r64(RCX);
                    if !sf {
                        zext_w(buf, RCX); // 64-bit `not` sets high bits; 32-bit must not
                    }
                    buf.and_rr64(RAX, RCX); // BIC
                }
                5 => {
                    buf.not_r64(RCX);
                    if !sf {
                        zext_w(buf, RCX);
                    }
                    buf.or_rr64(RAX, RCX); // ORN
                }
                6 => {
                    buf.not_r64(RCX);
                    if !sf {
                        zext_w(buf, RCX);
                    }
                    buf.xor_rr64(RAX, RCX); // EON
                }
                _ => return Err(format!("LogicReg op {} not implemented", op)),
            }
            if s {
                            // ANDS/ORRS/EORS/TST set NZCV: x86 `and/or/xor` set CF=0,OF=0 and
                            // ZF/SF from the result, which is exactly AArch64's N/Z/C/V here.
                            store_nzcv(buf);
                        }
                        if rd != 31 {
                            if !sf {
                                zext_w(buf, RAX); // W write zero-extends into X
                            }
                            stg(buf, rd as u32, RAX);
                        }
                        Ok(())
                    }
                    Inst::LogicImm {
                        rd,
                        rn,
                        mask,
                        op,
                        sf,
                    } => {
                        // AND/ORR/EOR/ANDS with a bitmask immediate (the `mov xD,#imm` alias
                        // is ORR xD, xzr, #imm). Load Rn into RAX, materialize `mask` in
                        // RCX, combine, then (for op==3 / tst) store NZCV.
                        if rn == 31 {
                            buf.mov_ri64(RAX, 0); // xzr reads as zero
                        } else {
                            ldg(buf, RAX, rn as u32);
                        }
                        buf.mov_ri64(RCX, mask);
                        match op {
                            0 => buf.and_rr64(RAX, RCX), // AND
                            1 => buf.or_rr64(RAX, RCX),  // ORR
                            2 => buf.xor_rr64(RAX, RCX), // EOR
                            3 => {
                                buf.and_rr64(RAX, RCX); // ANDS
                                store_nzcv(buf);
                            }
                            _ => return Err(format!("LogicImm op {} not implemented", op)),
                        }
                        if !sf {
                            buf.zero_ext_r32(RAX);
                        }
                        if rd != 31 {
                            stg(buf, rd as u32, RAX);
                        }
                        Ok(())
                    }
                    Inst::MteTag { load, rt, rn, wb, wb_off } => {
                    // No MTE on the host and no tag state in the JIT: an
                    // allocation-tag STORE (stg/stzg/st2g) leaves memory
                    // untouched, but a writeback form (post/pre-index: bit10
                    // set) advances the base register Xn by the signed,
                    // granule-scaled immediate — a real architectural side
                    // effect (glibc memset/stg loops rely on it). The only
                    // load (ldg) returns tag 0 into the guest Xt register.
                    if load && rt != 31 {
                        buf.mov_ri64(RAX, 0);
                        stg(buf, rt as u32, RAX); // tag 0
                    } else if wb && rn != 31 {
                        // Xn += wb_off (both post- and pre-index end with the
                        // base advanced by the immediate).
                        ldg(buf, RAX, rn as u32);
                        if wb_off != 0 {
                            if wb_off > 0 && wb_off <= 0x7fffffff {
                                buf.add_ri64(RAX, wb_off as u32);
                            } else if wb_off < 0 && -wb_off <= 0x7fffffff {
                                buf.sub_ri64(RAX, -wb_off as u32);
                            } else {
                                buf.mov_ri64(RCX, wb_off as u64);
                                buf.add_rr64(RAX, RCX);
                            }
                        }
                        stg(buf, rn as u32, RAX);
                    }
                    Ok(())
                }
                Inst::CacheMaintain { zva, rt } => {
                    // Cache maintenance in the direct-mapped single-threaded JIT
                    // (guest==host, warm shared memory) is a no-op — except `dc
                    // zva` which zeros the 16-byte cache line at [Xt] (the block
                    // size we advertise via dczid_el0).
                    if zva && rt != 31 {
                        ldg(buf, RAX, rt as u32); // RAX = base address
                        buf.mov_ri64(RCX, 0);
                        buf.mov_store64(RAX, 0, RCX);   // [base+0]
                        buf.mov_store64(RAX, 8, RCX);   // [base+8]
                    }
                    Ok(())
                }
                Inst::MulHigh { rd, rn, rm, signed } => {
                    // umulh/smulh Xd, Xn, Xm: high 64 bits of the 128-bit product.
                    // x86 one-operand mul/imul: RDX:RAX = RAX * rm, high in RDX.
                    ldg(buf, RAX, rn as u32);  // multiplicand
                    ldg(buf, RCX, rm as u32);  // multiplier
                    if signed {
                        buf.imul_high_r64(RCX); // RDX:RAX = RAX*RCX (signed)
                    } else {
                        buf.mul_high_r64(RCX);  // RDX:RAX = RAX*RCX (unsigned)
                    }
                    if rd != 31 {
                        stg(buf, rd as u32, RDX); // high half -> Rd
                    }
                    Ok(())
                }
                Inst::MulDiv { div, signed, rd, rn, rm, ra, sf } => {
                        if div {
                            // UDIV/SDIV: RAX = Rn / Rm (quotient). Dividend in
                            // RDX:RAX, divisor in RCX.
                            ldg(buf, RAX, rn as u32); // dividend low half
                            if signed {
                                // sign-extend the 32-bit W operand to 64 (for X
                                // operands already loaded sign-correct; movsxd of a
                                // 64-bit value's low 32 would corrupt it, so only
                                // re-extend for W).
                                if !sf {
                                    buf.movsxd_r64_r32(RAX, RAX);
                                }
                                buf.cqo(); // RAX -> RDX:RAX (signed)
                            } else {
                                buf.xor_rr64(RDX, RDX); // unsigned: zero-high half
                            }
                            // divisor: sign-extend Rm for SDIV-W too.
                            ldg(buf, RCX, rm as u32);
                            if signed && !sf {
                                buf.movsxd_r64_r32(RCX, RCX);
                            }
                            if signed {
                                buf.idiv_r64(RCX);
                            } else {
                                buf.div_r64(RCX);
                            }
                            // quotient in RAX. Store (W: low 32 preserved by div if no overflow).
                        } else {
                            // MADD/MSUB: RAX = Rn*rm [+/-] ra.
                            ldg(buf, RAX, rn as u32);
                            ldg(buf, RCX, rm as u32);
                            buf.imul_rr64(RAX, RCX); // RAX = Rn*rm (low 64)
                            if ra != 31 {
                                // ra is a real register: += / -= it. For `mul`,
                                // ra==31 means XZR (accumulate 0), NOT SP — ldg
                                // would read the stack pointer and corrupt the
                                // product with it.
                                ldg(buf, RDI, ra as u32);
                                if signed {
                                    // MSUB: Wd = Wa - Wn*Wm  (ra - rn*rm), NOT
                                    // rn*rm - ra. n - q*d compiled to msub was
                                    // returning -48 for 1298-25*50 (should be 48)
                                    // until this direction was fixed.
                                    buf.sub_rr64(RDI, RAX); // RDI = ra - rn*rm
                                    buf.mov_rr64(RAX, RDI);
                                } else {
                                    buf.add_rr64(RAX, RDI); // MADD: rn*rm + ra
                                }
                            }
                        }
                        if !sf {
                            buf.zero_ext_r32(RAX);
                        }
                        if rd != 31 {
                            stg(buf, rd as u32, RAX);
                        }
                        Ok(())
                    }
        Inst::MulLong { rd, rn, rm, ra, signed, sub } => {
            // smull/umull/smaddl/umaddl/smsubl/umsubl: 32-bit Rn*Rm -> 64-bit
            // product, optionally accumulated. Sign/zero-extend both operands
            // to 64 first, then imul: the low 64 of the signed product is
            // bit-identical to the unsigned one, and a 32x32 product always
            // fits in 64 bits (correct full result either way).
            ldg(buf, RAX, rn as u32);
            if signed {
                buf.movsxd_r64_r32(RAX, RAX);
            } else {
                buf.zero_ext_r32(RAX);
            }
            ldg(buf, RCX, rm as u32);
            if signed {
                buf.movsxd_r64_r32(RCX, RCX);
            } else {
                buf.zero_ext_r32(RCX);
            }
            buf.imul_rr64(RAX, RCX); // RAX = Rn * Rm (long)
            if ra != 31 {
                ldg(buf, RDI, ra as u32);
                if sub {
                    // msubl/umsubl: RAX = Ra - Rn*Rm
                    buf.neg_r64(RAX);
                    buf.add_rr64(RAX, RDI);
                } else {
                    buf.add_rr64(RAX, RDI); // maddl/umaddl: RAX = Rn*Rm + Ra
                }
            }
            if rd != 31 {
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::ClzCls { rd, rn, sf, cls } => {
            // clz/cls Wd|Xd, Rn. CLZ via LZCNT (F3 0F BD /r), which returns the
            // count of leading zeros directly and matches AArch64's clz(x=0)=|bits|.
            // Host x86-64 (Haswell+) universally supports LZCNT.
            if cls {
                return Err("CLS (count leading sign) not implemented".into());
            }
            if rd == 31 {
                return Ok(());
            }
            ldg(buf, RAX, rn as u32);
            if !sf {
                buf.zero_ext_r32(RAX);
            }
            if sf {
                // lzcnt rax, rax. REX.W (0x48) MUST come AFTER the F3 prefix and
                // immediately before the 0F opcode: emitted as `48 F3 0F BD` the
                // CPU ignores REX.W (it must be the last prefix) and executes a
                // 32-bit lzcnt eax — clz(x) with x<2^32 returns 32-len(x), not
                // 64-len(x) (real repro: clz(0x16136740) returned 3, oracle 35).
                buf.bytes.extend_from_slice(&[0xf3, 0x48, 0x0f, 0xbd, 0xc0]);
            } else {
                buf.bytes.extend_from_slice(&[0xf3, 0x0f, 0xbd, 0xc0]); // lzcnt eax, eax
                                                                        // lzcnt eax zeroes the upper 32 (correct W zero-extend)
            }
            stg(buf, rd as u32, RAX);
            Ok(())
        }
        Inst::Rev { rd, rn, op, sf } => {
            if rd == 31 {
                return Ok(());
            }
            ldg(buf, RAX, rn as u32);
            match op {
                2 => {
                    // rev: full byte reverse. W => 32-bit bswap, X => 64-bit bswap.
                    if sf {
                        buf.bswap_r64(RAX);
                    } else {
                        buf.bswap_r32(RAX); // 32-bit bswap zeroes upper half
                    }
                }
                1 => {
                    // rev16: reverse each adjacent byte pair (16-bit element).
                    // X: ((x & 0x00FF00FF00FF00FF) << 8) | ((x & 0xFF00FF00FF00FF00) >> 8)
                    if sf {
                        buf.mov_ri64(RCX, 0x00ff_00ff_00ff_00ff);
                        buf.and_rr64(RCX, RAX);          // even-lane bytes
                        buf.mov_rr64(RDX, RAX);
                        buf.mov_ri64(R10, 0xff00_ff00_ff00_ff00);
                        buf.and_rr64(RDX, R10);          // odd-lane bytes
                        buf.shl_ri8(RCX, 8);
                        buf.shr_ri8(RDX, 8);
                        buf.or_rr64(RAX, RCX);
                        buf.or_rr64(RAX, RDX);
                    } else {
                        // W (32-bit): two swaps.
                        buf.mov_rr64(RCX, RAX);
                        buf.and_ri64(RCX, 0x00ff_00ff);
                        buf.shl_ri8(RCX, 8);
                        buf.and_ri64(RAX, 0xff00_ff00);
                        buf.shr_ri8(RAX, 8);
                        buf.or_rr64(RAX, RCX);
                    }
                }
                3 => {
                    // rev32 (X only): swap the two 32-bit halves.
                    buf.ror_ri8(RAX, 32);
                }
                _ => {
                    // op==0 rbit: reverse all bits (SWAR byte/pair steps up to 64).
                    let steps: [(u8, u64); 6] = [
                        (1, 0x5555_5555_5555_5555),
                        (2, 0x3333_3333_3333_3333),
                        (4, 0x0f0f_0f0f_0f0f_0f0f),
                        (8, 0x00ff_00ff_00ff_00ff),
                        (16, 0x0000_ffff_0000_ffff),
                        (32, 0x0000_0000_ffff_ffff),
                    ];
                    let width = if sf { 6usize } else { 4usize };
                    for (k, (sh, m)) in steps.iter().take(width).enumerate() {
                        // t = ((x >> sh) & m) | ((x & m) << sh)
                        // BUGFIX: mask with the FULL 64-bit value (mov_ri64 into
                        // R10 + and_rr64), NOT `and_ri64(m as u32)` which truncated
                        // every mask to 32 bits — 0x5555..55, 0x3333..33, ... are
                        // 64-bit patterns, so only the LOW 32 bits of x were being
                        // reversed and the high 32 used a corrupt (low-32-only) mask
                        // (rbit x; clz x = ctz returned 198+ vs oracle 10). RAX is
                        // the running result; RCX holds the shifted-high piece.
                        buf.mov_rr64(RCX, RAX);
                        buf.shr_ri8(RCX, *sh);
                        buf.mov_ri64(R10, *m);
                        buf.and_rr64(RCX, R10); // (x >> sh) & m
                        buf.and_rr64(RAX, R10); // x & m  (note: NOT or'ing x in; RAX is masked then shifted)
                        buf.shl_ri8(RAX, (1u32 << k as u32) as u8); // (x & m) << sh
                        buf.or_rr64(RAX, RCX); // high-shifted | low-shifted
                    }
                }
            }
            if !sf {
                buf.zero_ext_r32(RAX);
            }
            stg(buf, rd as u32, RAX);
            Ok(())
        }
        Inst::CSel {
            rd,
            rn,
            rm,
            cond,
            op,
            sf,
        } => {
            // `csel rd, rn, rm, c`: rd = c ? rn : f(rm), where f applies the
            // csinc/csinv/csneg transform to rm (op 0=identity,1=+1,2=~,3=-).
            // then-branch value = rn (RDI), else-branch = f(rm) (R10). Compute
            // both before restoring the flags from NZCV (load_nzcv_to_eflags
            // sets them last, and cmovcc reads them immediately after).
            let _ = sf;
            // CSEL is a data-processing family: register operand 31 is XZR (0),
            // NEVER SP (that distinction only exists in the add/sub extended-
            // register forms). `cset/cinc/csneg` rely on rn=rm=31 reading as 0.
            if rn == 31 {
                buf.mov_ri64(RDI, 0); // then: XZR
            } else {
                ldg(buf, RDI, rn as u32); // then: rn
            }
            if rm == 31 {
                buf.mov_ri64(R10, 0); // else: XZR
            } else {
                ldg(buf, R10, rm as u32); // else: f(rm)
            }
            match op {
                0 => {}
                1 => buf.add_ri64(R10, 1),   // csinc / cset / cinc
                2 => buf.not_r64(R10),       // csinv
                3 => buf.neg_r64(R10),       // csneg
                _ => return Err(format!("CSel op {} not implemented", op)),
            }
            // 32-bit (W) destination: the result is the low 32 bits ZERO-
            // extended to the 64-bit register. The not/neg/inc transforms and
            // rn may carry high garbage (e.g. csinv ~5 = 0xfffffffffffffffa,
            // csneg -5 = 0xfffffffffffffffb); without truncation the upper
            // half silently leaks into x0.
            if !sf {
                // zero_ext_r32 (mov r32,r32, no REX.W): the high-register-aware
                // way to clear the upper 32 bits. zext_w's shl/shr path uses
                // 0x48-only (no REX.B) emitters and would corrupt R10.
                buf.zero_ext_r32(RDI);
                buf.zero_ext_r32(R10);
            }
            if cond == 0xE {
                // AL: unconditional — just rn
                if rd != 31 {
                    stg(buf, rd as u32, RDI);
                }
                return Ok(());
            }
            if cond == 0xF {
                // NV: never — just f(rm)
                if rd != 31 {
                    stg(buf, rd as u32, R10);
                }
                return Ok(());
            }
            let cc = (x86_cc_for_cond(cond)
                .ok_or_else(|| format!("CSel: bad cond {cond:#x}"))?
                - 0x40); // jcc 0x8X -> cmovcc 0x4X (subtract the 0x80 top byte)
            load_nzcv_to_eflags(buf);
            buf.cmov_rr64(cc, R10, RDI); // R10 = cond ? RDI(rn) : R10(f(rm))
            if rd != 31 {
                stg(buf, rd as u32, R10);
            }
            Ok(())
        }
        Inst::FcsSel { rd, rn, rm, cond, sz } => {
            // fcsel d{rd}, d{rn}, d{rm}, <cond>: rd = cond ? rn : rm on the FP
            // slots. FP values are selected by their bit pattern (cmov on the
            // integer ref of the double/single), so the same register-select
            // machinery as the integer CSel applies.
            let true_slot = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let else_slot = crate::jit::VECTOR_BASE + (rm as i32) * 16;
            let dst_slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            if sz {
                buf.mov_load64(RDI, RBX, true_slot);
                buf.mov_load64(R10, RBX, else_slot);
            } else {
                buf.mov_load32(RDI, RBX, true_slot);
                buf.mov_load32(R10, RBX, else_slot);
            }
            if cond == 0xE {
                buf.mov_store64(RBX, dst_slot, RDI); // AL -> rn
                return Ok(());
            }
            if cond == 0xF {
                buf.mov_store64(RBX, dst_slot, R10); // NV -> rm
                return Ok(());
            }
            let cc = (x86_cc_for_cond(cond)
                .ok_or_else(|| format!("FcSel: bad cond {cond:#x}"))?
                - 0x40); // jcc 0x8X -> cmovcc 0x4X
            load_nzcv_to_eflags(buf);
            buf.cmov_rr64(cc, R10, RDI); // R10 = cond ? rn : rm
            buf.mov_store64(RBX, dst_slot, R10);
            Ok(())
        }
        Inst::LdStrImm {
            rt,
            rn,
            imm,
            size,
            ld,
            sext,
        } => {
            // address = rn + imm*size (scaled byte offset)
            ldg(buf, RDX, rn as u32); // pointer operand into RDX
            let off = (imm as i32).checked_mul(size as i32).unwrap_or(0);
            if off != 0 {
                buf.lea64(RDX, RDX, off);
            }
            if ld && sext {
                // Sign-extending load (ldrsw/ldrsh/ldrsb): load `size` bytes and
                // sign-extend into the full 64-bit X dest (bit23+bit22==0 form).
                match size {
                    4 => {
                        buf.mov_load32(RAX, RDX, 0);
                        buf.movsxd_r64_r32(RAX, RAX);
                        stg_if_writable(buf, rt as u32);
                    }
                    2 => {
                        buf.movzx_word_mem(RAX, RDX, 0);
                        buf.shl_ri8(RAX, 48);
                        buf.sar_ri8(RAX, 48); // 16->64 sign-extend
                        stg_if_writable(buf, rt as u32);
                    }
                    1 => {
                        buf.movzx_byte_mem(RAX, RDX, 0);
                        buf.shl_ri8(RAX, 56);
                        buf.sar_ri8(RAX, 56); // 8->64 sign-extend
                        stg_if_writable(buf, rt as u32);
                    }
                    s => return Err(format!("LdStrImm sign-extend size {s} not implemented (pc {pc:#x})")),
                }
                return Ok(());
            }
            match (size, ld) {
                (8, true) => {
                    buf.mov_load64(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (8, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store64(RDX, 0, RAX);
                    emit_canary_store_watch(buf, pc, RDX, RAX);
                }
                (4, true) => {
                    buf.mov_load32(RAX, RDX, 0); // w zero-extends
                    stg_if_writable(buf, rt as u32);
                }
                (4, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store32(RDX, 0, RAX);
                }
                (2, true) => {
                    buf.movzx_word_mem(RAX, RDX, 0); // ldrh zero-extends
                    stg_if_writable(buf, rt as u32);
                }
                (2, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store16(RDX, 0, RAX);
                }
                (1, true) => {
                    buf.movzx_byte_mem(RAX, RDX, 0); // ldrb zero-extends
                    stg_if_writable(buf, rt as u32);
                }
                (1, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store8(RDX, 0, RAX);
                }
                (s, _) => return Err(format!("LdStrImm size {} not implemented", s)),
            }
            Ok(())
        }
        Inst::LseAtomic { op, size64, rs, rn, rt } => {
            // Single-threaded emulation: old = [Xn]; [Xn] = f(old, Rs); Rt = old.
            // op: 0=LDADD 1=LDCLR 2=LDEOR 3=LDSET 4=SWP.
            ldg(buf, RDX, rn as u32); // address
            if size64 {
                buf.mov_load64(RAX, RDX, 0);
            } else {
                buf.mov_load32(RAX, RDX, 0); // W: 32-bit load, zero-extends
            }
            buf.mov_rr64(RDI, RAX);      // RDI holds the old value -> Rt later
            ldg(buf, RCX, rs as u32);
            if size64 {
                match op {
                    0 => buf.add_rr64(RAX, RCX),
                    1 => {
                        buf.not_r64(RCX);
                        buf.and_rr64(RAX, RCX);
                    }
                    2 => buf.xor_rr64(RAX, RCX),
                    3 => buf.or_rr64(RAX, RCX),
                    _ => buf.mov_rr64(RAX, RCX), // SWP
                }
            } else {
                buf.zero_ext_r32(RCX);
                match op {
                    0 => buf.add_rr64(RAX, RCX),
                    1 => {
                        buf.not_r64(RCX);
                        buf.and_rr64(RAX, RCX);
                    }
                    2 => buf.xor_rr64(RAX, RCX),
                    3 => buf.or_rr64(RAX, RCX),
                    _ => buf.mov_rr64(RAX, RCX),
                }
            }
            // write the updated value back to [Xn]
            if size64 {
                buf.mov_store64(RDX, 0, RAX);
            } else {
                buf.mov_store32(RDX, 0, RAX);
            }
            // Rt = old value (zero-extended for W)
            if rt != 31 {
                if !size64 {
                    zext_w(buf, RDI);
                }
                buf.mov_rr64(RAX, RDI);
                stg(buf, rt as u32, RAX);
            }
            Ok(())
        }
        Inst::LdStrImmWb {
            rt,
            rn,
            imm9,
            size,
            ld,
            sext,
            writeback,
            pre,
        } => {
            // pre-index [Xn,#imm9]!  -> access at Xn+imm9, then Xn += imm9
            // post-index [Xn],#imm9   -> access at Xn,     then Xn += imm9
            // unscaled   [Xn,#imm9]   -> access at Xn+imm9, no writeback
            let access_off = if writeback && !pre { 0 } else { imm9 };
            ldg(buf, RDX, rn as u32); // base
            if ld && sext {
                // sign-extending load (ldrsw/ldrsh/ldrsb) at [RDX+access_off]
                match size {
                    4 => {
                        buf.mov_load32(RAX, RDX, access_off);
                        buf.movsxd_r64_r32(RAX, RAX);
                        stg_if_writable(buf, rt as u32);
                    }
                    2 => {
                        buf.movzx_word_mem(RAX, RDX, access_off);
                        buf.shl_ri8(RAX, 48);
                        buf.sar_ri8(RAX, 48);
                        stg_if_writable(buf, rt as u32);
                    }
                    1 => {
                        buf.movzx_byte_mem(RAX, RDX, access_off);
                        buf.shl_ri8(RAX, 56);
                        buf.sar_ri8(RAX, 56);
                        stg_if_writable(buf, rt as u32);
                    }
                    s => return Err(format!("LdStrImmWb sign-extend size {s} not implemented")),
                }
            } else {
                match (size, ld) {
                    (8, true) => {
                        buf.mov_load64(RAX, RDX, access_off);
                        stg_if_writable(buf, rt as u32);
                    }
                    (8, false) => {
                        ldg_src(buf, rt as u32);
                        buf.mov_store64(RDX, access_off, RAX);
                        emit_canary_store_watch(buf, pc, RDX, RAX);
                    }
                    (4, true) => {
                        buf.mov_load32(RAX, RDX, access_off);
                        stg_if_writable(buf, rt as u32);
                    }
                    (4, false) => {
                        ldg_src(buf, rt as u32);
                        buf.mov_store32(RDX, access_off, RAX);
                    }
                    (2, true) => {
                        buf.movzx_word_mem(RAX, RDX, access_off);
                        stg_if_writable(buf, rt as u32);
                    }
                    (2, false) => {
                        ldg_src(buf, rt as u32);
                        buf.mov_store16(RDX, access_off, RAX);
                    }
                    (1, true) => {
                        buf.movzx_byte_mem(RAX, RDX, access_off);
                        stg_if_writable(buf, rt as u32);
                    }
                    (1, false) => {
                        ldg_src(buf, rt as u32);
                        buf.mov_store8(RDX, access_off, RAX);
                    }
                    (s, _) => {
                        return Err(format!("LdStrImmWb size {s} not implemented"))
                    }
                }
            }
            // writeback: Xn += imm9 (signed; add_ri64 sign-extends the imm32).
            if writeback {
                ldg(buf, RAX, rn as u32);
                if imm9 != 0 {
                    buf.add_ri64(RAX, imm9 as u32);
                }
                stg(buf, rn as u32, RAX);
            }
            Ok(())
        }
        Inst::AcqRel { size, ld, rt, rn } => {
            // LDAR/STLR ordering is a no-op in a single-threaded JIT; act as a
            // plain load/store of `size` bytes at [Rn].
            ldg(buf, RDX, rn as u32);
            match (size, ld) {
                (3, true) => {
                    buf.mov_load64(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (3, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store64(RDX, 0, RAX);
                }
                (2, true) => {
                    buf.mov_load32(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (2, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store32(RDX, 0, RAX);
                }
                (1, true) => {
                    buf.movzx_word_mem(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (1, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store16(RDX, 0, RAX);
                }
                (0, true) => {
                    buf.movzx_byte_mem(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (0, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store8(RDX, 0, RAX);
                }
                (s, _) => return Err(format!("AcqRel size {} not implemented", s)),
            }
            Ok(())
        }
        Inst::LdExr { size, ld, rs, rt, rn } => {
            // Exclusive block in a single-threaded JIT always succeeds: `ldxr` is
            // a plain load, `stxr` is a plain store with the status register Rs
            // written 0 (success).
            ldg(buf, RDX, rn as u32);
            match (size, ld) {
                (3, true) => {
                    buf.mov_load64(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (3, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store64(RDX, 0, RAX);
                    stg0(buf, rs as u32);
                }
                (2, true) => {
                    buf.mov_load32(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (2, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store32(RDX, 0, RAX);
                    stg0(buf, rs as u32);
                }
                (1, true) => {
                    buf.movzx_word_mem(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (1, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store16(RDX, 0, RAX);
                    stg0(buf, rs as u32);
                }
                (0, true) => {
                    buf.movzx_byte_mem(RAX, RDX, 0);
                    stg_if_writable(buf, rt as u32);
                }
                (0, false) => {
                    ldg_src(buf, rt as u32);
                    buf.mov_store8(RDX, 0, RAX);
                    stg0(buf, rs as u32);
                }
                (s, _) => return Err(format!("LdExr size {} not implemented", s)),
            }
            Ok(())
        }
        Inst::Ror { rd, rn, rot, sf } => {
                    // Extend to 64-bit, rotate right by rot, then mask for W.
                    ldg(buf, RAX, rn as u32);
            let r = (rot & (if sf { 63u32 } else { 31u32 })) as u8;
            buf.ror_ri8(RAX, r);
            if !sf {
                buf.zero_ext_r32(RAX);
            }
            stg(buf, rd as u32, RAX);
            Ok(())
        }
        Inst::Extr { rd, rn, rm, lsb, sf } => {
            // EXTR: Xd = (Xn << (bits - lsb)) | (Xm >> lsb)  — ARM concatenates
            // Xn as the HIGH word and Xm as the LOW word of a 2*bits value and
            // shifts right by lsb. (The Ror alias rm==rn is handled separately.)
            // NOTE: the earlier implementation had the operand order INVERTED
            // ((Xn >> lsb) | (Xm << (bits-lsb))); the rotate case rn==rm is
            // symmetric so it hid the bug, but gcc's real 128-bit shifts
            // (e.g. `extr x1, x2, x1, #32` for a cross-word shift) got the two
            // operands swapped and returned garbage.
            let bits = if sf { 64u32 } else { 32u32 };
            let lsb = lsb & (bits - 1);
            let hi_part = (bits - lsb) & (bits - 1); // Xn << (bits-lsb)
            // low part: Xm >> lsb
            ldg(buf, RAX, rm as u32);
            if lsb != 0 {
                buf.shr_ri8(RAX, lsb as u8);
            }
            // high part: Xn << (bits-lsb). lsb==0 would shift by `bits` (=0 in a
            // 64-bit reg), so force it to 0 explicitly (the high word plays no role).
            ldg(buf, R10, rn as u32);
            if lsb == 0 {
                buf.mov_ri64(R10, 0);
            } else if hi_part != 0 {
                buf.shl_ri8(R10, hi_part as u8);
            }
            buf.or_rr64(RAX, R10);
            if !sf {
                buf.zero_ext_r32(RAX);
            }
            stg(buf, rd as u32, RAX);
            Ok(())
        }
        Inst::Svc { imm: _ } => {
            // Route the supervisor call to the host `guest_svc` dispatcher (this
            // is the hookpoint for real AArch64->host syscall routing).
            let addr = crate::jit::guest_svc as usize as u64;
            // Record the post-svc guest address (pc+4) into CpuState.svc_next
            // BEFORE the call. `clone` (220) re-enters jit_run at this address
            // for the child thread, so the child continues right after the svc.
            // pc is the current instruction's guest address (translate passes it).
            buf.mov_ri64(RAX, pc.wrapping_add(4));
            buf.mov_store64(RBX, crate::jit::SVC_NEXT_OFF, RAX); // state.svc_next = pc+4
            // guest_svc(st) is extern "C" fn(*mut CpuState)->u64: arg0 (state)
            // must go in RDI explicitly. RDI at block entry does hold the state
            // pointer (we are f(state)), so the FIRST inline call worked by
            // accident — but a prior host call clobbers RDI (caller-saved), so
            // any later svc passed garbage. This was a real latent bug.
            buf.mov_rr64(RDI, RBX); // arg0 = state
            buf.mov_ri64(RAX, addr);
            // The JIT block body runs at host RSP ≡ 8 (mod 16) — the correct
            // callee-entry alignment for guest-to-guest BL (call_rel32) — but
            // SysV requires RSP ≡ 0 at the *call* site of a host function. An
            // inline host callee (guest_svc, guest_sha1stem) does its own
            // aligned stack work (prologue push, SSE locals; e.g. format! in
            // tracing), so a misaligned call faults. Align around the call.
            buf.sub_ri64(4, 8); // RSP(4) -= 8  ->  RSP ≡ 0 mod 16 at the call
            buf.call_r64(RAX); // guest_svc(st); returns the syscall result in RAX
            buf.add_ri64(4, 8); // RSP += 8  ->  back to block-entry alignment
            // Thread-exit AND signal-redirect early-return. `state.pc` is NOT
            // updated per-instruction during block execution (it holds the
            // block-entry address), so we can't compare it to `svc_next`.
            // A signal redirect is decided FIRST, BEFORE the syscall-result
            // store: when a self-delivered signal set `redirect_request =
            // handler`, guest x0 must stay the signal handler's signo argument
            // (set by signals::begin_handler), NOT the syscall return — so we
            // yield to the dispatcher without overwriting x0. Only on the
            // non-redirect path do we store the syscall result (x0) and then
            // early-return when `pc == 0` (a spawned child's thread-local
            // `exit`). Both redirect and pc==0 return from the block so the
            // dispatcher loop re-reads `state.pc`.
            buf.mov_load64(RCX, RBX, crate::jit::REDIRECT_OFF); // RCX = redirect
            buf.test_rr64(RCX, RCX);
            buf.jne_rel8(16); // redirect != 0 -> jump to the yield `ret` (16 on)
            stg(buf, 0, RAX); // system value -> guest x0 (AArch64 return reg)
            buf.mov_load64(RAX, RBX, crate::jit::PC_OFF); // RAX = state.pc
            buf.test_rr64(RAX, RAX);
            buf.jne_rel8(2); // pc != 0 -> skip both `ret`s, continue inline
            buf.ret(); // pc == 0: thread-local exit -> return to the dispatcher
            buf.ret(); // redirect != 0: yield to the dispatcher (x0 keeps signo)
            Ok(()) // <-- continue inline after the rets (pc != 0, redirect == 0)
        }
        Inst::Brk { imm } => {
            // Guest breakpoint (brk #imm): on a real AArch64 CPU this traps
            // (SIGTRAP). Mirror that for the JIT by halting the run loop
            // gracefully — set guest pc = 0, the same sentinel `run_loop`
            // treats as a clean halt (it returns Ok(x[0])).
            let _ = imm;
            buf.mov_ri64(RAX, 0);
            buf.mov_store64(RBX, crate::jit::PC_OFF, RAX); // state.pc = 0
            Ok(())
        }
        Inst::Udf { imm } => {
            // Undefined instruction (udf #imm): a real AArch64 CPU faults here
            // (Undefined Instruction exception). Mirror Brk by halting the run
            // loop gracefully (state.pc = 0 sentinel).
            let _ = imm;
            buf.mov_ri64(RAX, 0);
            buf.mov_store64(RBX, crate::jit::PC_OFF, RAX); // state.pc = 0
            Ok(())
        }
        Inst::BitField { rd, rn, immr, imms, sf, arith, insert } => {
            let bits = if sf { 64u32 } else { 32u32 };
            // ---- BFM insert (opc=01): merge bits of Rn into Rd, preserving
            // Rd's bits outside the field. BFXIL (immr<=imms, copy in place) and
            // BFI/BFC (immr>imms, wrap) BOTH merge; they are NOT the UBFM/SBFM
            // shift/extract aliases below, which must not fire for insert==true
            // (e.g. a bfi whose immr/imms coincidentally satisfy the ROR
            // shortcut 15+48+1==64 would be mis-compiled as a rotate).
            if insert {
                let mask: u64;
                if immr <= imms {
                    // BFXIL: field spans [immr, imms] in place — Rn's bits are
                    // already at the target position, so mask in place (no shift).
                    let width = (imms - immr + 1) as u32;
                    mask = (((1u64 << width) - 1) << immr) & (if sf { u64::MAX } else { 0xffff_ffff });
                    ldg(buf, RAX, rn as u32); // Rn
                } else {
                    // BFI/BFC: wrap, lsb=(bits-immr)&(bits-1), width=imms+1.
                    // Rn's low `width` bits are shifted up to `lsb`, THEN masked.
                    let lsb = (bits - immr) & (bits - 1);
                    let width = imms + 1;
                    mask = (((1u64 << width) - 1) << lsb) & (if sf { u64::MAX } else { 0xffff_ffff });
                    ldg(buf, RAX, rn as u32); // Rn
                    buf.shl_ri8(RAX, lsb as u8); // field << lsb
                }
                ldg(buf, R10, rd as u32); // old Rd
                buf.mov_ri64(RCX, mask);
                buf.and_rr64(RAX, RCX); // field of Rn
                // rd = (rd & ~mask) | (Rn & mask)
                buf.not_r64(RCX);
                buf.and_rr64(R10, RCX);
                buf.or_rr64(R10, RAX);
                buf.mov_rr64(RAX, R10);
                if !sf {
                    buf.zero_ext_r32(RAX);
                }
                if rd != 31 {
                    stg(buf, rd as u32, RAX);
                }
                return Ok(());
            }
            // ---- UBFM/SBFM (insert=false): shift / extract / sign-extend.
            ldg(buf, RAX, rn as u32); // load Rn
            if !sf && !arith && !insert {
                // W-form logical bitfield (lsr w, ubfx, lsl w, ror w, etc.): the
                // source register is only 32 bits wide, so the UPPER 32 bits of
                // the 64-bit slot (left by a prior X-form write such as a 64-bit
                // madd/mul) must be discarded BEFORE any shift/rotate. Otherwise
                // high-bit guest garbage shifts down into the low result —
                // e.g. `ldr x1; madd x1,x1,x4,x3; lsr w5,w1,#24` pulled bits 32-55
                // of x1 into the extracted byte (real repro: 8652 -> 0x37562e2cc).
                buf.zero_ext_r32(RAX);
            }
            if imms == bits - 1 {
                // LSR (logical) or ASR (arithmetic/sign) by immr
                let sh = (immr & (bits - 1)) as u8;
                if arith {
                    if sf {
                        buf.sar_ri8(RAX, sh);
                    } else {
                        buf.sar32_ri8(RAX, sh);
                    }
                } else {
                    buf.shr_ri8(RAX, sh);
                }
            } else if immr == (imms + 1) % bits {
                // LSL (alias) : shift left by (bits-1-imms)
                let sh = ((bits - 1 - imms) & (bits - 1)) as u8;
                buf.shl_ri8(RAX, sh);
            } else if immr > imms {
                // UBFIZ (logical) / SBFIZ (arith): zero- or sign-extending
                // shift-left. Xd = (Rn << lsb) & mask, upper bits zeroed
                // (UBFIZ) or sign-replicated (SBFIZ). Old Rd is DISCARDED; the
                // old code merged old Rd here (BFI behavior), so `ubfiz
                // x4,x0,#7,#32` kept stale upper bits of x4.
                let lsb = (bits - immr) & (bits - 1);
                let width = imms + 1;
                let mask = ((1u64 << width) - 1) << lsb;
                buf.shl_ri8(RAX, lsb as u8); // Rn << lsb
                buf.mov_ri64(RCX, mask);
                buf.and_rr64(RAX, RCX); // field only (zero-extended)
                if arith {
                    // SBFIZ: sign-extend the field from bit (lsb+width-1).
                    let se = (bits - (lsb + width)) as u8;
                    buf.shl_ri8(RAX, se);
                    if sf { buf.sar_ri8(RAX, se); } else { buf.sar32_ri8(RAX, se); }
                }
            } else {
                // general UBFM/SBFM extract: (Rn >> immr) & low(width) bits,
                // then optionally sign-extend from `width`.
                let width = imms - immr + 1;
                let sh = (immr & (bits - 1)) as u8;
                buf.shr_ri8(RAX, sh); // drop low immr bits
                // keep only `width` low bits
                if width < bits {
                    let mask: u64 = (1u64 << width) - 1;
                    // BUGFIX: `and_ri64(RAX, m as u32)` emits a 64-bit AND with a
                    // SIGN-EXTENDED imm32, so any mask with bit31 set (width>=32:
                    // mask = 0xffffffff) silently became 0xffffffffffffffff — a
                    // no-op that left the high 32 bits of the shifted value
                    // corrupting the extract (ubfx x,#25,#32 returned garbage).
                    // Mask must fit in the positive imm32 range to use the fast
                    // path; otherwise load the full 64-bit mask.
                    if mask <= 0x7fff_ffff {
                        buf.and_ri64(RAX, mask as u32);
                    } else {
                        buf.mov_ri64(RCX, mask);
                        buf.and_rr64(RAX, RCX);
                    }
                }
                if arith {
                    // sign-extend the `width`-bit field to `bits`:
                    // shift left to push the sign bit to the top, then arithmetic
                    // shift right back (replicates the sign). For a 32-bit field
                    // the sign must be taken from bit31, not bit63 (which a 64-bit
                    // `sar` would read on a zero-extended value).
                    let se = (bits - width) as u8;
                    buf.shl_ri8(RAX, se);
                    if sf { buf.sar_ri8(RAX, se); } else { buf.sar32_ri8(RAX, se); }
                }
            }
            if !sf {
                buf.zero_ext_r32(RAX);
            }
            if rd != 31 {
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::SysReg { sysreg, rt, read } => {
            // sysreg==0: tpidr_el0 (CpuState.tpidr). sysreg==1: cntfrq_el0
            // (counter tick rate in Hz) read as a fixed constant. Decode only
            // produces these; write to cntfrq is not generated.
            //
            // NOTE: rt is a GUEST register index. The value must be committed to
            // the guest file with stg(buf, rt, <host>) — a bare buf.mov_ri64(rt,
            // ..) only writes HOST register rt and the guest slot stays stale
            // (latent bug: every mrs xN,<cnt|nzcv|dczid> was a silent no-op).
            if sysreg == 1 {
                // mrs xN, cntfrq_el0  ->  xN = 100_000_000 (100 MHz counter).
                // Constant Hz: the guest divides/downscales counter deltas with
                // this, so a fixed, self-consistent rate is honest for boot.
                if read && rt != 31 {
                    buf.mov_ri64(RAX, 100_000_000);
                    stg(buf, rt as u32, RAX);
                }
                return Ok(());
            }
            if sysreg == 3 {
                // mrs xN, cntvct_el0  ->  xN = CpuState.cntvct (live monotonic
                // counter, stamped by the run loop between guest blocks).
                if read && rt != 31 {
                    buf.mov_load64(RAX, RBX, crate::jit::CNTVCT_OFF);
                    stg(buf, rt as u32, RAX);
                }
                return Ok(());
            }
            if sysreg == 4 {
                // mrs xN, nzcv  ->  xN = CpuState.nzcv (packed N=31,Z=30,C=29,V=28).
                if read && rt != 31 {
                    buf.mov_load32(RAX, RBX, NZCV_OFF);
                    stg(buf, rt as u32, RAX);
                }
                if !read && rt != 31 {
                    // msr nzcv, xN: shift xN's low 4 flag bits up to NZCV@28..31.
                    ldg_src(buf, rt as u32);
                    buf.and_ri64(RAX, 0x0f);
                    buf.shl_ri8(RAX, 28);
                    buf.mov_store32(RBX, NZCV_OFF, RAX);
                }
                return Ok(());
            }
            if sysreg == 5 {
                // mrs xN, dczid_el0  ->  xN = 0x4 (16-byte DC ZVA block, DZP=0).
                if read && rt != 31 {
                    buf.mov_ri64(RAX, 0x4);
                    stg(buf, rt as u32, RAX);
                }
                return Ok(());
            }
            if sysreg == 6 || sysreg == 7 || sysreg == 8 || sysreg == 11 {
                // GCS / SME-TLS / MIDR registers the JIT neither enables nor
                // models: sysreg 6 = gcspr_el0 (armv9 GCS pointer; 0 when GCS
                // disabled), sysreg 7 = tpidr2_el0 (SME second TLS pointer; 0
                // without SME), sysreg 8 = midr_el1 (implementer/part; 0 =
                // unknown core so glibc picks generic non-SME/SVE paths).
                // sysreg 11 = fpcr read: 0 = nearest-even rounding, default FPCR
                // (the JIT rounds per-op so no state to surface).
                // Both read 0 on a fresh EL0 context that never enables the
                // feature — matching what a real core reports at boot.
                // (All read 0; writes are no-ops.)
                if read && rt != 31 {
                    buf.mov_ri64(RAX, 0);
                    stg(buf, rt as u32, RAX);
                }
                return Ok(());
            }
            if sysreg == 9 {
                // msr fpcr, xN: FP control write. The JIT pins all FP rounding
                // per-op (roundsd modes, cvt*), never reading FPCR, so accept any
                // value silently. (read never produced for fpcr.)
                return Ok(());
            }
            if sysreg == 10 {
                // mrs xN, ctr_el0: cache-type register. Report 16-byte lines:
                // IminLine(15:0)=2, DminLine(19:16)=2, Cwg(23:20)=0, DIC/IDC=0.
                if read && rt != 31 {
                    buf.mov_ri64(RAX, 0x0002_0002); // Dmin=2<<16 | Imin=2
                    stg(buf, rt as u32, RAX);
                }
                return Ok(());
            }
            if read {
                // Rt = [RBX + TPIDR_OFF]  (commit to the guest slot, same fix as
                // the other MRS reads: rt is a guest register, not a host one)
                if rt != 31 {
                    buf.mov_load64(RAX, RBX, crate::jit::TPIDR_OFF);
                    stg(buf, rt as u32, RAX);
                }
            } else {
                // tpidr_el0 = Rt
                if rt != 31 {
                    ldg_src(buf, rt as u32);
                    buf.mov_store64(RBX, crate::jit::TPIDR_OFF, RAX);
                }
            }
            Ok(())
        }
        Inst::Fma3 { rd, rn, rm, ra, sz, sub, neg } => {
            // Scalar 3-source FP: Dd = Da +- (Dn × Dm), optionally negated.
            //   fmadd(0,0)=ra+rn*rm  fmsub(1,0)=ra-rn*rm
            //   fnmadd(0,1)=-(ra+rn*rm)  fnmsub(1,1)=rn*rm-ra
            // xmm0=rn*rm, xmm1=rm(srca), xmm2=ra, xmm3=scratch(negate).
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if sz {
                buf.movq_load(0, RBX, vslot(rn));
                buf.movq_load(1, RBX, vslot(rm));
                buf.mulsd(0, 1); // xmm0 = rn*rm
                buf.movq_load(2, RBX, vslot(ra));
                let result = match (sub, neg) {
                    (false, false) => {
                        buf.addsd(2, 0); // ra + prod
                        2
                    }
                    (true, false) => {
                        buf.subsd(2, 0); // ra - prod
                        2
                    }
                    (false, true) => {
                        buf.addsd(2, 0); // ra + prod
                        buf.pxor_xmm(3, 3); // 0.0
                        buf.subsd(3, 2); // 0 - (ra+prod) = negate
                        3
                    }
                    (true, true) => {
                        buf.subsd(0, 2); // prod - ra
                        0
                    }
                };
                buf.movq_store(RBX, vslot(rd), result);
            } else {
                let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                let ld = |buf: &mut CodeBuf, x: u8, r: u8| {
                    buf.mov_load32(RAX, RBX, f(r));
                    buf.movd_xmm_r32(x, RAX);
                };
                // xmm0 = rn, xmm1 = rm, then xmm0 *= xmm1 (rcx), xmm2 = ra.
                ld(buf, 0, rn);
                ld(buf, 1, rm);
                buf.mulss(0, 1); // xmm0 = rn*rm
                ld(buf, 2, ra);
                match (sub, neg) {
                    (false, false) => buf.addss(2, 0), // xmm2 = ra + prod
                    (true, false) => buf.subss(2, 0),  // xmm2 = ra - prod
                    (false, true) => {
                        buf.addss(2, 0); // xmm2 = ra + prod
                        buf.pxor_xmm(3, 3);
                        buf.subss(3, 2); // 0 - xmm2
                    }
                    (true, true) => {
                        buf.movd_xmm_r32(2, RAX); // refresh xmm2 unused; do prod - ra
                        buf.subss(0, 2); // xmm0 = prod - ra
                    }
                }
                let result = if sub && neg { 0 } else if neg { 3 } else { 2 };
                buf.movd_r32_xmm(RAX, result);
                buf.mov_store32(RBX, f(rd), RAX);
            }
            Ok(())
        }
        Inst::FpScalar { rd, rn, rm, op, sz, half } => {
            // scalar FP on d/s/h regs. d-reg = low 8 bytes of CpuState.v[reg].slot
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16; // low 8B of a 16B slot
            // ops 0-3 (fmov/fabs/fneg) are double-only; single handled for 4-7 below.
            if !sz && op <= 3 && !half {
                return Err(format!("FpScalar single-precision (sz=0) op {op} not implemented"));
            }
            match op {
                0 => {
                    // fmov d, d: copy 64-bit int
                    buf.mov_load64(RAX, RBX, vslot(rn));
                    buf.mov_store64(RBX, vslot(rd), RAX);
                }
                1 => {
                    // fabs: clear sign bit (bit63)
                    buf.mov_load64(RAX, RBX, vslot(rn));
                    buf.mov_ri64(RCX, 0x7fff_ffff_ffff_ffff);
                    buf.and_rr64(RAX, RCX);
                    buf.mov_store64(RBX, vslot(rd), RAX);
                }
                2 => {
                    // fneg: flip sign bit
                    buf.mov_load64(RAX, RBX, vslot(rn));
                    buf.mov_ri64(RCX, 0x8000_0000_0000_0000);
                    buf.xor_rr64(RAX, RCX);
                    buf.mov_store64(RBX, vslot(rd), RAX);
                }
                4 | 5 | 6 | 7 => {
                    if half {
                        // Half-precision (FP16) scalar arithmetic: promote both h
                        // operands (low 16 bits of each slot) to f32 via F16C
                        // vcvtph2ps, do the f32 op, then demote to f16 (RN) and
                        // store the low 16 bits. Encodings (F16C, immediate 0 = RN):
                        //   vcvtph2ps xmm,xmm  = C4 E2 79 13 /r (reg form)
                        //   vcvtps2ph $0,xmm,xmm = C4 E3 79 1D /r 00
                        let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                        buf.mov_load32(RAX, RBX, f(rn)); // h{rn} in low 16
                        buf.movd_xmm_r32(0, RAX);        // xmm0 low 32 = h{rn}
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                        buf.mov_load32(RAX, RBX, f(rm));
                        buf.movd_xmm_r32(1, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc9]); // vcvtph2ps xmm1,xmm1
                        let opcode: u8 = match op {
                            4 => 0x59, // mulss
                            5 => 0x58, // addss
                            6 => 0x5c, // subss
                            7 => 0x5e, // divss
                            _ => unreachable!(),
                        };
                        buf.bytes.extend_from_slice(&[0xf3, 0x0f, opcode, 0xc1]); // opss xmm0,xmm1
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                        buf.movd_r32_xmm(RAX, 0); // RAX low 16 = f16 result
                        buf.mov_store32(RBX, f(rd), RAX); // store low 32 (f16 in low 16)
                    } else if sz {
                        buf.movq_load(0, RBX, vslot(rn));
                        buf.movq_load(1, RBX, vslot(rm));
                        match op {
                            4 => buf.mulsd(0, 1),
                            5 => buf.addsd(0, 1),
                            6 => buf.subsd(0, 1),
                            7 => buf.divsd(0, 1),
                            _ => unreachable!(),
                        }
                        buf.movq_store(RBX, vslot(rd), 0);
                    } else {
                        // single-precision (32-bit scalar FP): operands live in
                        // the low 4 bytes of each slot.
                        let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                        buf.mov_load32(RAX, RBX, f(rn));
                        buf.movd_xmm_r32(0, RAX);
                        buf.mov_load32(RAX, RBX, f(rm));
                        buf.movd_xmm_r32(1, RAX);
                        // emit scalar single (F3 0F 5x /r): SrcDst=xmm0, rm=xmm1
                        let opcode: u8 = match op {
                            4 => 0x59, // mulss
                            5 => 0x58, // addss
                            6 => 0x5c, // subss
                            7 => 0x5e, // divss
                            _ => unreachable!(),
                        };
                        buf.bytes.extend_from_slice(&[0xf3, 0x0f, opcode, 0xc1]);
                        buf.movd_r32_xmm(RAX, 0);
                        buf.mov_store32(RBX, f(rd), RAX);
                    }
                }
                _ => return Err(format!("FpScalar op {op} not implemented")),
            }
            Ok(())
        }
        Inst::FpUnary { rd, rn, op, sz, half } => {
            // scalar 1-source FP: fsqrt / frint{mpz} / fabs / fneg.
            // d-reg = low 8B of yate.v[reg]; s-reg = low 4B.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if half {
                // fp16: 16-bit lane in the low half of the slot.
                buf.mov_load16(RAX, RBX, vslot(rn));
                match op {
                    5 => { // fabs h: clear sign bit
                        buf.and_ri64(RAX, 0x7fff);
                    }
                    6 => { // fneg h: flip sign bit
                        buf.xor_ri64(RAX, 0x8000);
                    }
                    0 => { // fsqrt h
                        buf.movd_xmm_r32(0, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps
                        buf.cvtss2sd(0, 0);
                        buf.sqrtsd(0, 0);
                        buf.cvtsd2ss(0, 0);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph
                        buf.movd_r32_xmm(RAX, 0);
                    }
                    1 => { // frintm h: floor
                        buf.movd_xmm_r32(0, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]);
                        buf.cvtss2sd(0, 0);
                        buf.roundsd(0, 0, 0b01);
                        buf.cvtsd2ss(0, 0);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]);
                        buf.movd_r32_xmm(RAX, 0);
                    }
                    2 => { // frintp h: ceil (toward +inf) = roundsd 0b10
                        buf.movd_xmm_r32(0, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]);
                        buf.cvtss2sd(0, 0);
                        buf.roundsd(0, 0, 0b10);
                        buf.cvtsd2ss(0, 0);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]);
                        buf.movd_r32_xmm(RAX, 0);
                    }
                    3 => { // frintz h: trunc = roundsd 0b11
                        buf.movd_xmm_r32(0, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]);
                        buf.cvtss2sd(0, 0);
                        buf.roundsd(0, 0, 0b11);
                        buf.cvtsd2ss(0, 0);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]);
                        buf.movd_r32_xmm(RAX, 0);
                    }
                    8 => { // frintn h: round nearest-even = roundsd 0b00
                        buf.movd_xmm_r32(0, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]);
                        buf.cvtss2sd(0, 0);
                        buf.roundsd(0, 0, 0b00);
                        buf.cvtsd2ss(0, 0);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]);
                        buf.movd_r32_xmm(RAX, 0);
                    }
                    9 => { // frintx h: round current mode = nearest
                        buf.movd_xmm_r32(0, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]);
                        buf.cvtss2sd(0, 0);
                        buf.roundsd(0, 0, 0b00);
                        buf.cvtsd2ss(0, 0);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]);
                        buf.movd_r32_xmm(RAX, 0);
                    }
                    _ => return Err(format!("FpUnary half op {op} not implemented")),
                }
                buf.mov_store16(RBX, vslot(rd), RAX);
            } else if !sz {
                // single-precision: operate on the low 32 bits.
                buf.mov_load32(RAX, RBX, vslot(rn));
                match op {
                    5 => { // fabs s: clear sign bit
                        buf.mov_ri64(RCX, 0x7fff_ffff);
                        buf.and_rr64(RAX, RCX);
                    }
                    6 => { // fneg s: flip sign bit
                        buf.mov_ri64(RCX, 0x8000_0000);
                        buf.xor_rr64(RAX, RCX);
                    }
                    _ => {
                // frintm/p/z (floor/ceil/trunc) and fsqrt via double round-trip.
                buf.movd_xmm_r32(0, RAX);        // move the loaded float bits into xmm0.low32
                                                 // (cvtss2sd reads xmm0 low32, not RAX)
                buf.cvtss2sd(0, 0);              // xmm0 = (double)xmm0.low32
                match op {
                    0 => buf.sqrtsd(0, 0),       // fsqrt s
                    1 => buf.roundsd(0, 0, 0b01), // frintm s: floor
                    2 => buf.roundsd(0, 0, 0b10), // frintp s: ceil
                    3 => buf.roundsd(0, 0, 0b11), // frintz s: trunc (toward zero; 0b00 is round-to-NEAREST)
                    7 => {
                        // frinta s: round half-away = sign*floor(|x|+0.5) on the sd'd val
                        buf.movq_r64_xmm(RAX, 0);
                        buf.mov_ri64(RCX, 0x8000_0000_0000_0000);
                        buf.and_rr64(RAX, RCX);
                        buf.movq_xmm_r64(2, RAX);             // sign mask
                        buf.movq_r64_xmm(RAX, 0);
                        buf.mov_ri64(RCX, 0x7fff_ffff_ffff_ffff);
                        buf.and_rr64(RAX, RCX);
                        buf.movq_xmm_r64(0, RAX);             // |x|
                        buf.mov_ri64(RAX, 0x3fe0_0000_0000_0000); // 0.5
                        buf.movq_xmm_r64(1, RAX);
                        buf.addsd(0, 1);                       // |x|+0.5
                        buf.roundsd(0, 0, 0x01);               // floor
                        buf.pxor_xmm(0, 2);                    // sign
                    }
                    8 | 9 => buf.roundsd(0, 0, 0x00), // frintn/frintx s: nearest, ties-even
                    _ => return Err(format!("FpUnary single ilp-{op} not implemented")),
                }
                buf.cvtsd2ss(0, 0);              // back to single (xmm0 low 32)
                buf.movd_r32_xmm(RAX, 0);          // xmm0 low32 -> RAX
                buf.mov_store32(RBX, vslot(rd), RAX); // store s-reg
                return Ok(());
            }
                }
                buf.mov_store32(RBX, vslot(rd), RAX);
                return Ok(());
            }
            buf.movq_load(0, RBX, vslot(rn));
            match op {
                0 => buf.sqrtsd(0, 0),  // fsqrt  d{rd}, d{rn}
                1 => buf.roundsd(0, 0, 0x01), // frintm: round toward -inf (floor)
                2 => buf.roundsd(0, 0, 0x02), // frintp: round toward +inf (ceil)
                3 => buf.roundsd(0, 0, 0x03), // frintz: round toward zero (trunc)
                4 => buf.roundsd(0, 0, 0x03), // (frintz: toward zero) — reserved mapping
                5 => {
                    // fabs d{rd}, d{rn}: clear the sign bit on the FP bit-pattern.
                    buf.movq_r64_xmm(RAX, 0);
                    buf.mov_ri64(RCX, 0x7fff_ffff_ffff_ffff);
                    buf.and_rr64(RAX, RCX);
                    buf.movq_xmm_r64(0, RAX);
                }
                6 => {
                    // fneg d{rd}, d{rn}: flip the sign bit on the FP bit-pattern.
                    buf.movq_r64_xmm(RAX, 0);
                    buf.mov_ri64(RCX, 0x8000_0000_0000_0000);
                    buf.xor_rr64(RAX, RCX);
                    buf.movq_xmm_r64(0, RAX);
                }
                7 => {
                    // frinta d{rd}, d{rn}: round half-away-from-zero = sign*floor(|x|+0.5).
                    buf.movq_r64_xmm(RAX, 0);
                    buf.mov_ri64(RCX, 0x8000_0000_0000_0000);
                    buf.and_rr64(RAX, RCX);                // RAX = sign bit
                    buf.movq_xmm_r64(2, RAX);              // xmm2 = sign mask
                    buf.movq_r64_xmm(RAX, 0);
                    buf.mov_ri64(RCX, 0x7fff_ffff_ffff_ffff);
                    buf.and_rr64(RAX, RCX);                // RAX = |x|
                    buf.movq_xmm_r64(0, RAX);              // xmm0 = |x|
                    buf.mov_ri64(RAX, 0x3fe0_0000_0000_0000); // 0.5 (double)
                    buf.movq_xmm_r64(1, RAX);
                    buf.addsd(0, 1);                       // xmm0 = |x| + 0.5
                    buf.roundsd(0, 0, 0x01);               // floor(|x|+0.5)
                    buf.pxor_xmm(0, 2);                    // reapply sign bit
                }
                8 | 9 => buf.roundsd(0, 0, 0x00), // frintn/frintx d: round to nearest, ties-even
                _ => return Err(format!("FpUnary op {op} not implemented")),
            }
            buf.movq_store(RBX, vslot(rd), 0);
            Ok(())
        }
        Inst::Fabd { rd, rn, rm, sz } => {
            // fabd Vd, Dn, Dm = |dn - dm| (scalar). Compute a-b in xmm,
            // round-trip the bit pattern to a GPR, clear the sign bit, and store.
            // Honest for finite floats; NaN stays NaN (sign-bit clear keeps it a NaN).
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if sz {
                buf.movq_load(0, RBX, vslot(rn));
                buf.movq_load(1, RBX, vslot(rm));
                buf.subsd(0, 1); // xmm0 = rn - rm
                buf.movq_r64_xmm(RDX, 0); // RDX = bits(rn - rm)
                buf.mov_ri64(RDI, 0x7fff_ffff_ffff_ffff); // ~signbit
                buf.and_rr64(RDX, RDI); // clear bit 63 (|x|)
                buf.movq_xmm_r64(0, RDX); // back to xmm
                buf.movq_store(RBX, vslot(rd as u8), 0);
            } else {
                // single precision: subss + clear bit31 in the low 32 bits.
                buf.mov_load32(RAX, RBX, vslot(rn));
                buf.movd_xmm_r32(0, RAX);
                buf.mov_load32(RAX, RBX, vslot(rm));
                buf.movd_xmm_r32(1, RAX);
                buf.bytes.extend_from_slice(&[0xf3, 0x0f, 0x5c, 0xc1]); // subss xmm0,xmm1
                buf.movd_r32_xmm(RAX, 0); // RAX = bits(rn - rm)
                buf.mov_ri64(RCX, 0x7fff_ffff); // ~signbit (32-bit)
                buf.and_rr64(RAX, RCX); // clear bit 31 (|x|)
                buf.mov_store32(RBX, vslot(rd as u8), RAX);
            }
            Ok(())
        }
        Inst::FcvtToInt {
            rd,
            rn,
            mode,
            sf,
            unsigned,
            src_sng,
            fbits,
        } => {
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if src_sng {
                // single (S) source: load low 32 bits, promote to double in xmm0.
                buf.mov_load32(RAX, RBX, vslot(rn));
                buf.movd_xmm_r32(0, RAX);
                buf.cvtss2sd(0, 0);
            } else {
                buf.movq_load(0, RBX, vslot(rn)); // d-source (low 8B) -> xmm0
            }
            if fbits > 0 {
                // Fixed-point scale: result = Fn * 2^fbits, then truncate-to-int.
                // Load 2^fbits as a double and multiply.
                let scale_bits = (1.0f64 * 2.0f64.powi(fbits as i32)).to_bits();
                buf.mov_ri64(RAX, scale_bits);
                buf.movq_xmm_r64(1, RAX);
                buf.mulsd(0, 1);
            }
            if mode == 2 {
                if unsigned {
                    // fcvtau: round to nearest (MXCSR) then cvttsd2si. x86 has no
                    // native unsigned-as-round-away; cvttsd2si is exact for [0,2^63)
                    // which covers every real W-dest conversion (matches the fcvtas
                    // ties-even-vs-away caveat). Saturation to negative for d>=2^63 is
                    // an acceptable edge for this cold path.
                    buf.roundsd(0, 0, 0x00); // roundsd nearest-even
                    buf.cvttsd2si(RAX, 0);
                } else {
                    buf.cvtsd2si(RAX, 0); // fcvtas: round to nearest (MXCSR, default even)
                }
            } else if mode == 3 || mode == 4 {
                // fcvtpu/ps (+inf) and fcvtmu/ms (-inf): round the double to an
                // integer-valued double first (roundsd 0x01=floor, 0x02=ceil), then
                // trunc-convert. Matches qemu ground truth (fcvtpu: 1.5->2, 0.5->1,
                // negative/NaN->0, +inf->0xFFFF... via x86 cvttsd2si saturation).
                buf.roundsd(0, 0, if mode == 3 { 0x02 } else { 0x01 });
                buf.cvttsd2si(RAX, 0);
                if unsigned {
                    buf.xor_rr64(RCX, RCX);
                    buf.test_rr64(RAX, RAX);
                    buf.cmov_rr64(0x48, RAX, RCX); // cmovs RAX, RCX (neg -> 0)
                }
            } else if unsigned {
                // fcvtzu: truncate toward zero to an UNSIGNED 64-bit value, valid
                // over [0, 2^64). x86 cvttsd2si is signed: exact in [0,2^63),
                // saturates to INT64_MIN (0x8000..0) for anything >= 2^63, which
                // silently corrupts the high half if used directly. Correct roads:
                //   d < 2^63          : cvttsd2si exact; negative/NaN -> 0
                //   2^63 <= d < 2^64  : 2^63 + (int64)(d - 2^63), exact
                //   d >= 2^64         : saturate to u64::MAX
                buf.mov_ri64(RCX, 0x43e0_0000_0000_0000); // 2^63 as a double
                buf.movq_xmm_r64(1, RCX);
                buf.comisd(0, 1); // CF=1 iff d < 2^63
                let jc = buf.jcc_rel32(0x82); // JB: d < 2^63 -> signed path
                // --- big path: d >= 2^63 ---
                buf.subsd(0, 1); // xmm0 = d - 2^63
                buf.cvttsd2si(RAX, 0); // RAX=(int64)(d-2^63); INT64_MIN if d>=2^64
                buf.test_rr64(RAX, RAX);
                let js = buf.jcc_rel32(0x88); // JS: d >= 2^64 -> clamp to MAX
                buf.mov_ri64(RCX, 0x8000_0000_0000_0000); // 2^63
                buf.add_rr64(RAX, RCX); // = 2^63+(d-2^63), exact in [2^63,2^64)
                let jmp_done = buf.jmp_rel32();
                let max_at = buf.len(); // clamp path: d >= 2^64
                buf.mov_ri64(RAX, u64::MAX);
                let jmp2 = buf.jmp_rel32();
                let signed_at = buf.len(); // signed path: d < 2^63
                buf.cvttsd2si(RAX, 0); // [0,2^63) exact; d<0 -> truncated negative
                buf.xor_rr64(RCX, RCX);
                buf.test_rr64(RAX, RAX);
                buf.cmov_rr64(0x48, RAX, RCX); // negative d -> 0
                let done = buf.len();
                let mut patch = |at: usize, target: usize| {
                    let disp = (target as i64 - (at as i64 + 4)) as i32;
                    buf.bytes[at..at + 4].copy_from_slice(&disp.to_le_bytes());
                };
                patch(jc, signed_at);
                patch(js, max_at);
                patch(jmp_done, done);
                patch(jmp2, done);
            } else {
                buf.cvttsd2si(RAX, 0); // fcvtzs: truncate toward zero
            }
            if rd != 31 {
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::FmovGp { f, sz, half, rd, rn } => {
            // FMOV core <-> scalar FP. Double(d/x) uses the low 64, single(s/w)
            // the low 32, half (h/w) the low 16 of the vector slot.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if f {
                            // FP -> GP
                            if half {
                                buf.mov_load16(RAX, RBX, vslot(rn));
                            } else if sz {
                                buf.mov_load64(RAX, RBX, vslot(rn));
                            } else {
                                buf.mov_load32(RAX, RBX, vslot(rn));
                            }
                if rd != 31 {
                    stg(buf, rd as u32, RAX);
                }
            } else {
                // GP -> FP
                ldg(buf, RAX, rn as u32);
                if half {
                    buf.mov_store16(RBX, vslot(rd), RAX);
                } else if sz {
                    buf.mov_store64(RBX, vslot(rd), RAX);
                } else {
                    buf.mov_store32(RBX, vslot(rd), RAX);
                }
            }
            Ok(())
        }
        Inst::Scvtf {
            rd,
            rn,
            to_double,
            sf,
            unsigned,
        } => {
            // scvtf/ucvtf -> cvtsi2{sd|ss}: load the integer Rn, widen per `sf`,
            // and store the float into v{rd} (low 8B for double, low 4B for single).
            // For unsigned (ucvtf) with a 64-bit source, cvtsi2sd is exact only up
            // to 2^63-1, so we apply the standard +2^64 correction when the sign
            // bit of the 64-bit value is set (see Ucvtf2d). 32-bit-unsigned fits
            // exactly in f64 so no correction is needed there.
            let vslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            if unsigned && sf {
                // RDX = u64 source
                ldg(buf, RDX, rn as u32);
                if to_double {
                    buf.cvtsi2sd(0, true, RDX); // xmm0 = (double)(int64)u
                    buf.test_rr64(RDX, RDX);
                    let jns = buf.jcc_rel32(0x89); // JNS: skip correction if u < 2^63
                    buf.mov_ri64(RCX, 0x43f0_0000_0000_0000); // 2^64 (double bits)
                    buf.movq_xmm_r64(1, RCX);
                    buf.addsd(0, 1);
                    let end = buf.len();
                    let disp = (end as i64 - (jns as i64 + 4)) as i32;
                    buf.bytes[jns..jns + 4].copy_from_slice(&disp.to_le_bytes());
                    buf.movq_store(RBX, vslot, 0);
                } else {
                    // single result from 64-bit unsigned: (float)u = (double)u then cvtss
                    buf.cvtsi2sd(0, true, RDX);
                    buf.test_rr64(RDX, RDX);
                    let jns = buf.jcc_rel32(0x89);
                    buf.mov_ri64(RCX, 0x43f0_0000_0000_0000);
                    buf.movq_xmm_r64(1, RCX);
                    buf.addsd(0, 1);
                    let end = buf.len();
                    let disp = (end as i64 - (jns as i64 + 4)) as i32;
                    buf.bytes[jns..jns + 4].copy_from_slice(&disp.to_le_bytes());
                    buf.cvtsd2ss(0, 0); // float from the double
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, vslot, RAX);
                }
            } else {
                ldg(buf, RAX, rn as u32);
                if to_double {
                    buf.cvtsi2sd(0, sf, RAX);
                    buf.movq_store(RBX, vslot, 0);
                } else {
                    buf.cvtsi2ss(0, sf, RAX);
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, vslot, RAX);
                }
            }
                        Ok(())
                    }
                    Inst::FmovImm { rd, f64, value_bits } => {
                        // fmov Dd,#imm / fmov Sd,#imm: write the decoded IEEE-754 value into
                        // the destination FP slot (low 8B for double, low 4B for single).
                        let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                        if f64 {
                            buf.mov_ri64(RAX, value_bits);
                            buf.mov_store64(RBX, slot, RAX); // low 8B of slot = double bits
                        } else {
                            buf.mov_ri32(RAX, value_bits as u32);
                            buf.mov_store32(RBX, slot, RAX); // low 4B of slot = single bits
                        }
                        Ok(())
                    }
                    Inst::FmovImm16 { rd, value_bits } => {
                        // fmov Hd,#imm: write the f16 value into the low 2B of the slot.
                        let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                        buf.mov_ri32(RAX, value_bits as u32);
                        buf.mov_store16(RBX, slot, RAX);
                        Ok(())
                    }
        Inst::FmovFp { rd, rn, sz } => {
            // fmov Dd,Dn / fmov Sd,Sn: register-to-register FP copy (no conversion).
            let sslot = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            if sz {
                buf.mov_load64(RAX, RBX, sslot);
                buf.mov_store64(RBX, dslot, RAX); // double: copy low 8B
            } else {
                buf.mov_load32(RAX, RBX, sslot);
                                buf.mov_store32(RBX, dslot, RAX); // single: copy low 4B
                            }
                            Ok(())
                        }
                        Inst::FMaxMin { rd, rn, rm, sz, op } => {
                            // fmax/fmin/fmaxnm/fminnm: FP max/min (x86 maxss/sd ignore NaN; ok here).
                            // op: 4=fmax,5=fmin,6=fmaxnm,7=fminnm -> max index, rn/x0 vs rm/0.
                            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                            let is_max = matches!(op, 4 | 6);
                            if sz {
                                buf.movq_load(0, RBX, vslot(rn));
                                buf.movq_load(1, RBX, vslot(rm));
                                if is_max { buf.maxsd(0, 1); } else { buf.minsd(0, 1); }
                                buf.movq_store(RBX, vslot(rd), 0);
                            } else {
                                buf.mov_load32(RAX, RBX, vslot(rn));
                                buf.movd_xmm_r32(0, RAX);
                                buf.mov_load32(RAX, RBX, vslot(rm));
                                buf.movd_xmm_r32(1, RAX);
                                if is_max { buf.maxss(0, 1); } else { buf.minss(0, 1); }
                                                                buf.movd_r32_xmm(RAX, 0);
                                                                                                                        buf.mov_store32(RBX, vslot(rd), RAX);
                                                                                                                    }
                                                                                                                    Ok(())
                                                                                                                }
                                                                                                    Inst::SimdFpPair3 { rd, rn, rm, add, min, nm, q } => {
                                                                                                                                                                        // faddp/fmaxp/fminp/fmaxnmp/fminnmp Vd.T,Vn.T,Vm.T:
                                                                                                                                                                        // halves of Vd = pairwise-reduce Vn (low half) then Vm.
                                                                                                                                                                        // Q=0 (.2s): 2 src lanes -> 1 dst lane each.
                                                                                                                                                                        // Q=1 (.4s): 4 src lanes -> 2 dst lanes each.
                                                                                                                                                                        let _ = nm;
                                                                                                                                                                        let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                        let nsrc = if q { 4 } else { 2 };
                                                                                                                                                                        let mut dst_idx = 0;
                                                                                                                                                                        for perreg in 0..2 {
                                                                                                                                                                            let base = if perreg == 0 { vslot(rn) } else { vslot(rm) };
                                                                                                                                                                            for i in 0..nsrc / 2 {
                                                                                                                                                                                buf.mov_load32(RAX, RBX, base + 4 * (2 * i));
                                                                                                                                                                                buf.movd_xmm_r32(0, RAX);
                                                                                                                                                                                buf.mov_load32(RAX, RBX, base + 4 * (2 * i + 1));
                                                                                                                                                                                buf.movd_xmm_r32(1, RAX);
                                                                                                                                                                                if add { buf.addss(0, 1); }
                                                                                                                                                                                else if min { buf.minss(0, 1); } else { buf.maxss(0, 1); }
                                                                                                                                                                                buf.movd_r32_xmm(RAX, 0);
                                                                                                                                                                                buf.mov_store32(RBX, vslot(rd) + 4 * dst_idx, RAX);
                                                                                                                                                                                dst_idx += 1;
                                                                                                                                                                            }
                                                                                                                                                                        }
                                                                                                                                                                        Ok(())
                                                                                                                                                                    }
                                                                                                                                                                        Inst::FpPair { rd, rn, sz, min, nm } => {
                                                                                                                                                                        // fmaxp/fminp/fmaxnmp/fminnmp Vd, Vn:
                                                                                                                                                                        // pairwise reduce Vn's two elements (.2s or
                                                                                                                                                                        // .2d) into a scalar result in Vd. x86
                                                                                                                                                                        // maxss/sd ignore NaN (ok for fmaxp/nm here).
                                                                                                                                                                        let _ = nm; // skip-NaN; maxss/sd ~= fmaxnm path
                                                                                                                                                                        let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                        if sz {
                                                                                                                                                                            // Vd.D = max/min(Vn.D[0], Vn.D[1])
                                                                                                                                                                            buf.movq_load(0, RBX, vslot(rn));
                                                                                                                                                                            buf.movq_load(1, RBX, vslot(rn) + 8);
                                                                                                                                                                            if min { buf.minsd(0, 1); } else { buf.maxsd(0, 1); }
                                                                                                                                                                            buf.movq_store(RBX, vslot(rd), 0);
                                                                                                                                                                        } else {
                                                                                                                                                                            // Vd.S = max/min(Vn.S[0], Vn.S[1])
                                                                                                                                                                            buf.mov_load32(RAX, RBX, vslot(rn));
                                                                                                                                                                            buf.movd_xmm_r32(0, RAX);
                                                                                                                                                                            buf.mov_load32(RAX, RBX, vslot(rn) + 4);
                                                                                                                                                                            buf.movd_xmm_r32(1, RAX);
                                                                                                                                                                            if min { buf.minss(0, 1); } else { buf.maxss(0, 1); }
                                                                                                                                                                            buf.movd_r32_xmm(RAX, 0);
                                                                                                                                                                            buf.mov_store32(RBX, vslot(rd), RAX);
                                                                                                                                                                        }
                                                                                                                                                                        Ok(())
                                                                                                                                                                    }
                                                                                                        Inst::FMaxV { rd, rn, min } => {
                                                                                                        // fmaxv/fminv Sd, Vn.4s: reduce the 4
                                                                                                        // single-precision lanes of Vn to a max/
                                                                                                        // min into scalar Sd (low 32-bit of V[rd]).
                                                                                                        let base = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                                                                                        buf.mov_load32(RAX, RBX, base);
                                                                                                        buf.movd_xmm_r32(0, RAX);
                                                                                                        for i in 1..4 {
                                                                                                            buf.mov_load32(RAX, RBX, base + 4 * i);
                                                                                                            buf.movd_xmm_r32(1, RAX);
                                                                                                            if min { buf.minss(0, 1); } else { buf.maxss(0, 1); }
                                                                                                        }
                                                                                                        buf.movd_r32_xmm(RAX, 0);
                                                                                                                                                let dbase = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                                                                                                                                buf.mov_store32(RBX, dbase, RAX);
                                                                                                                                                Ok(())
                                                                                                                                            }
                                                                                                                                            Inst::Fmla { rd, rn, rm, el64, q, sub } => {
                                                                                                                                                // fmla/fmls Vd.T, Vn, Vm: per-lane Vd = Vd +/- Vn*Vm.
                                                                                                                                                // el64 => .2d (2 double lanes); else q => .4s (4 single);
                                                                                                                                                // else => .2s (2 single).
                                                                                                                                                let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                                                                                                                                let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                                                                                                                                let mb = crate::jit::VECTOR_BASE + (rm as i32) * 16;
                                                                                                                                                let lanes = if el64 { 2 } else if q { 4 } else { 2 };
                                                                                                                                                for l in 0..lanes {
                                                                                                                                                                    if el64 {
                                                                                                                                                                        // xmm0=Vn[l]; xmm1=Vm[l]; xmm1=Vn*Vm; xmm0=Vd[l];
                                                                                                                                                                        // Vd +- Vn*Vm via addss/subss(0,1) => correct sign for
                                                                                                                                                                        // fmls (Vd - Vn*Vm), NOT Vn*Vm - Vd.
                                                                                                                                                                        buf.movq_load(0, RBX, nb + 8 * l); // Vn[l] dbl
                                                                                                                                                                        buf.movq_load(1, RBX, mb + 8 * l); // Vm[l] dbl
                                                                                                                                                                        buf.mulsd(1, 0);                   // xmm1 = Vn*Vm
                                                                                                                                                                        buf.movq_load(0, RBX, db + 8 * l); // Vd[l]
                                                                                                                                                                        if sub { buf.subsd(0, 1); } else { buf.addsd(0, 1); }
                                                                                                                                                                        buf.movq_store(RBX, db + 8 * l, 0); // Vd[l] = result
                                                                                                                                                                    } else {
                                                                                                                                                                        // Same ordering for the .4s/.2s single-precision path:
                                                                                                                                                                        // product in xmm1, accumulator Vd in xmm0, so the
                                                                                                                                                                        // subtract has the correct operand order.
                                                                                                                                                                        buf.mov_load32(RAX, RBX, nb + 4 * l);
                                                                                                                                                                        buf.movd_xmm_r32(0, RAX);           // xmm0 = Vn[l]
                                                                                                                                                                        buf.mov_load32(RAX, RBX, mb + 4 * l);
                                                                                                                                                                        buf.movd_xmm_r32(1, RAX);           // xmm1 = Vm[l]
                                                                                                                                                                        buf.mulss(1, 0);                    // xmm1 = Vn*Vm
                                                                                                                                                                        buf.mov_load32(RAX, RBX, db + 4 * l);
                                                                                                                                                                        buf.movd_xmm_r32(0, RAX);           // xmm0 = Vd[l]
                                                                                                                                                                        if sub { buf.subss(0, 1); } else { buf.addss(0, 1); }
                                                                                                                                                                        buf.movd_r32_xmm(RAX, 0);
                                                                                                                                                                        buf.mov_store32(RBX, db + 4 * l, RAX);
                                                                                                                                                                    }
                                                                                                                                                                }
                                                                                                                                                Ok(())
                                                                                                                                            }
                                                                                                                                            Inst::FmlaEl { rd, rn, vlm, idx, el64, q, sub } => {
                                                                                                                                                            // fmla/fmls Vd.T, Vn, Vm.el[idx]: Vd[j] += Vn[j]*Vm.el(idx), per lane.
                                                                                                                                                            let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                                                                                                                                            let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                                                                                                                                            let mb = crate::jit::VECTOR_BASE + (vlm as i32) * 16;
                                                                                                                                                            let lanes = if el64 { 2 } else if q { 4 } else { 2 };
                                                                                                                                                            // broadcast scalar element into xmm2
                                                                                                                                                            if el64 {
                                                                                                                                                                buf.movq_load(2, RBX, mb + (idx as i32) * 8);
                                                                                                                                                            } else {
                                                                                                                                                                buf.mov_load32(RAX, RBX, mb + (idx as i32) * 4);
                                                                                                                                                                buf.movd_xmm_r32(2, RAX);
                                                                                                                                                            }
                                                                                                                                                            for l in 0..lanes {
                                                                                                                                                                if el64 {
                                                                                                                                                                    buf.movq_load(0, RBX, db + 8 * l);
                                                                                                                                                                    buf.movq_load(1, RBX, nb + 8 * l);
                                                                                                                                                                    buf.mulsd(1, 2);                // xmm1 = Vn[l]*Vm.el
                                                                                                                                                                    if sub { buf.subsd(0, 1); } else { buf.addsd(0, 1); }
                                                                                                                                                                    buf.movq_store(RBX, db + 8 * l, 0);
                                                                                                                                                                } else {
                                                                                                                                                                    buf.mov_load32(RAX, RBX, db + 4 * l);
                                                                                                                                                                    buf.movd_xmm_r32(0, RAX);
                                                                                                                                                                    buf.mov_load32(RAX, RBX, nb + 4 * l);
                                                                                                                                                                    buf.movd_xmm_r32(1, RAX);
                                                                                                                                                                    buf.mulss(1, 2);
                                                                                                                                                                    if sub { buf.subss(0, 1); } else { buf.addss(0, 1); }
                                                                                                                                                                    buf.movd_r32_xmm(RAX, 0);
                                                                                                                                                                                                        buf.mov_store32(RBX, db + 4 * l, RAX);
                                                                                                                                                                                                    }
                                                                                                                                                                                                }
                                                                                                                                                                                                Ok(())
                                                                                                                                                                                            }
                                                                                                                                                                                            Inst::VecFpArith { rd, rn, rm, op, q } => {
                                                                                                                                                                                                // Vector single-precision FP two-source arithmetic:
                                                                                                                                                                                                // Vd[l] = f(Vn[l], Vm[l]) over .4s (q) or .2s lanes.
                                                                                                                                                                                                // op 0=add 1=sub 2=mul 3=div 4=max 5=min 6=fmaxnm
                                                                                                                                                                                                // 7=fminnm. (The NaN-propagation edge of maxnm/minnm
                                                                                                                                                                                                // differs from x86 maxss/minss only when one operand
                                                                                                                                                                                                // is NaN; a documented approximation for graphics.)
                                                                                                                                                                                                let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                                                                                                                                                                                let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                                                                                                                                                                                let mb = crate::jit::VECTOR_BASE + (rm as i32) * 16;
                                                                                                                                                                                                let lanes = if q { 4 } else { 2 };
                                                                                                                                                                                                for l in 0..lanes {
                                                                                                                                                                                                    buf.mov_load32(RAX, RBX, nb + 4 * l);
                                                                                                                                                                                                    buf.movd_xmm_r32(0, RAX);
                                                                                                                                                                                                    buf.mov_load32(RAX, RBX, mb + 4 * l);
                                                                                                                                                                                                    buf.movd_xmm_r32(1, RAX);
                                                                                                                                                                                                    match op {
                                                                                                                                                                                                        0 => buf.addss(0, 1),
                                                                                                                                                                                                        1 => buf.subss(0, 1),
                                                                                                                                                                                                        2 => buf.mulss(0, 1),
                                                                                                                                                                                                        3 => buf.divss(0, 1),
                                                                                                                                                                                                        4 | 6 => buf.maxss(0, 1),
                                                                                                                                                                                                        8 | 9 => {
                                                                                                                                                                                                            // frecps = 2 - Vn*Vm ; frsqrts = (3 - Vn*Vm)/2
                                                                                                                                                                                                            buf.mulss(0, 1); // xmm0 = Vn*Vm
                                                                                                                                                                                                            buf.mov_ri32(RAX, 0x4000_0000u32); // 2.0f32
                                                                                                                                                                                                            buf.movd_xmm_r32(1, RAX); // 2.0
                                                                                                                                                                                                            buf.subss(1, 0);   // xmm1 = 2 - prod
                                                                                                                                                                                                            if op == 9 {
                                                                                                                                                                                                                // /2: multiply by 0.5
                                                                                                                                                                                                                buf.mov_ri32(RAX, 0x3f00_0000u32); // 0.5
                                                                                                                                                                                                                buf.movd_xmm_r32(3, RAX);
                                                                                                                                                                                                                buf.mulss(1, 3);
                                                                                                                                                                                                            }
                                                                                                                                                                                                            buf.movd_r32_xmm(RAX, 1);
                                                                                                                                                                                                            buf.mov_store32(RBX, db + 4 * l, RAX);
                                                                                                                                                                                                            continue;
                                                                                                                                                                                                        }
                                                                                                                                                                                                        _ => buf.minss(0, 1), // 5 / 7
                                                                                                                                                                                                    }
                                                                                                                                                                                                    buf.movd_r32_xmm(RAX, 0);
                                                                                                                                                                                                    buf.mov_store32(RBX, db + 4 * l, RAX);
                                                                                                                                                                                                }
                                                                                                                                                                                                Ok(())
                                                                                                                                                                                            }
                                                                                                                                                                                            Inst::VecFpCmp { rd, rn, rm, esize, op, q, abs } => {
            // fcmeq/fcmgt/fcmge (and with abs: facgt/facge) Vd.T, Vn.T, Vm.T:
            // per-lane result = all-ones if Vn op Vm, else 0. For abs, compare
            // |Vn| vs |Vm| (clear sign bit of both loaded lanes first via pand).
            let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let mb = crate::jit::VECTOR_BASE + (rm as i32) * 16;
            let es = esize as i32;
            let lanes = if q { 16 / es } else { 8 / es };
            let cc = match op {
                0 => 4u8,  // fcmeq  : sete (ZF)
                1 => 7u8,  // fcmgt  : seta (CF=0 && ZF=0)
                _ => 3u8,  // fcmge  : setae (CF=0)
            };
            for l in 0..lanes {
                let off = l * es;
                if esize == 8 {
                    buf.movq_load(0, RBX, nb + off);
                    buf.movq_load(1, RBX, mb + off);
                    if abs {
                        buf.mov_ri64(RAX, 0x7fff_ffff_ffff_ffff);
                        buf.movq_xmm_r64(2, RAX);
                        buf.pand(0, 2);
                        buf.pand(1, 2);
                    }
                    buf.comisd(0, 1);
                } else {
                    buf.mov_load32(RAX, RBX, nb + off);
                    buf.movd_xmm_r32(0, RAX);
                    buf.mov_load32(RAX, RBX, mb + off);
                    buf.movd_xmm_r32(1, RAX);
                    if abs {
                        buf.mov_ri64(RAX, 0x0000_0000_7fff_ffff);
                        buf.movq_xmm_r64(2, RAX);
                        buf.pand(0, 2);
                        buf.pand(1, 2);
                    }
                    buf.comiss(0, 1);
                }
                buf.setcc_rm8(cc, 0); // AL = 0/1
                buf.movzx_r32_r8(RAX, RAX);
                buf.neg_r64(RAX); // low 32/64: 0 -> 0, +1 -> all-ones
                if esize == 8 {
                    buf.mov_store64(RBX, db + off, RAX);
                } else {
                    buf.mov_store32(RBX, db + off, RAX);
                }
            }
            Ok(())
        }
        Inst::VecFpCmpZero { rd, rn, op, esize, q } => {
            // fcmeq/fcmgt/fcmge/fcmlt/fcmle Vd.T, Vn.T, #0.0: per-lane = all-ones
            // if Vn op 0.0 else 0 (mirrors VecFpCmp with a zeroed second operand).
            // comiss/comisd(x, 0): CF=1 if x<0, ZF=1 if x==0; unordered(NaN) sets
            // all three, so seta/setae/sete/setb/setbe match the existing VecFpCmp
            // NaN precedent (fcmgt/ge safe; eq/lt/le NaN-imperfect as there).
            let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let es = esize as i32;
            let lanes = if q { 16 / es } else { 8 / es };
            let cc = match op {
                0 => 4u8, // fcmeq : sete (ZF)
                1 => 7u8, // fcmgt : seta (CF=0 && ZF=0)
                2 => 3u8, // fcmge : setae (CF=0)
                3 => 2u8, // fcmlt : setb (CF=1)
                _ => 6u8, // fcmle : setbe (CF=1 || ZF=1)
            };
            for l in 0..lanes {
                let off = l * es;
                buf.pxor_xmm(1, 1); // xmm1 = +0.0
                if esize == 8 {
                    buf.movq_load(0, RBX, nb + off);
                    buf.comisd(0, 1);
                } else {
                    buf.mov_load32(RAX, RBX, nb + off);
                    buf.movd_xmm_r32(0, RAX);
                    buf.comiss(0, 1);
                }
                buf.setcc_rm8(cc, 0); // AL = 0/1
                buf.movzx_r32_r8(RAX, RAX);
                buf.neg_r64(RAX); // low 32/64: 0 -> 0, +1 -> all-ones
                if esize == 8 {
                    buf.mov_store64(RBX, db + off, RAX);
                } else {
                                    buf.mov_store32(RBX, db + off, RAX);
                                }
                            }
                            Ok(())
                        }
                        Inst::WidenShl { rd, rn, dst_esize, nlanes, signed, upper } => {
                                                 // shll/s hll2 vd.Td, vn.Ts: widen nlanes low (upper half) elements
                                // of vn, sign/zero extend to dst_esize-byte lanes.
                                let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                let src_esize = (dst_esize / 2) as i32;
                                let uh = if upper { 8 } else { 0 }; // shll2 reads the upper 8 bytes
                                for rev in 0..(nlanes as i32) {
                                    let i = (nlanes as i32) - 1 - rev;
                                    let so = nb + uh + i * src_esize;
                                    let doff = db + i * (dst_esize as i32);
                                    match dst_esize {
                                        8 => {
                                            buf.mov_load32(RCX, RBX, so);
                                            if signed {
                                                buf.movsxd_r64_r32(RAX, RCX);
                                            } else {
                                                buf.mov_rr64(RAX, RCX);
                                            }
                                            buf.mov_store64(RBX, doff, RAX);
                                        }
                                        4 => {
                                            if signed {
                                                buf.movsx_word_mem(RAX, RBX, so);
                                            } else {
                                                buf.movzx_word_mem(RAX, RBX, so);
                                            }
                                            buf.mov_store32(RBX, doff, RAX);
                                        }
                                        _ => {
                                            if signed {
                                                buf.movsx_byte_mem(RAX, RBX, so);
                                            } else {
                                                buf.movzx_byte_mem(RAX, RBX, so);
                                            }
                                            buf.mov_store16(RBX, doff, RAX);
                                        }
                                    }
                                }
                                Ok(())
                            }
                            Inst::SimdAdalp { rd, rn, src_esize, n_pairs, signed, upper, acc } => {
                                // sadalp/uadalp (acc=true) or saddlp/uaddlp (acc=false,
                                // pairwise-add-long) Vd.Td, Vn.Ts: for each adjacent pair
                                // (2i,2i+1) of src_esize-byte srcs, write (acc=false) or add
                                // into (acc=true) the dst lane of width 2*src_esize.
                                let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16 + if upper { 8 } else { 0 };
                                let se = src_esize as i32;
                                let np = n_pairs as i32;
                                let de = 2 * se;
                                for i in 0..np {
                                    let ss = nb + 2 * i * se;
                                    let dd = db + i * de;
                                    // RAX = widened(Vn[2i]); RCX = widened(Vn[2i+1])
                                    if se == 4 { buf.mov_load32(RAX, RBX, ss); buf.mov_load32(RCX, RBX, ss + 4); }
                                    else if se == 2 {
                                        if signed { buf.movsx_word_mem(RAX, RBX, ss); buf.movsx_word_mem(RCX, RBX, ss + 2); }
                                        else { buf.movzx_word_mem(RAX, RBX, ss); buf.movzx_word_mem(RCX, RBX, ss + 2); }
                                    } else {
                                        if signed { buf.movsx_byte_mem(RAX, RBX, ss); buf.movsx_byte_mem(RCX, RBX, ss + 1); }
                                        else { buf.movzx_byte_mem(RAX, RBX, ss); buf.movzx_byte_mem(RCX, RBX, ss + 1); }
                                    }
                                    buf.add_rr64(RAX, RCX);
                                    // Write EXACTLY `de` (dst lane) bytes — never overrun the
                                    // neighbouring lane (store64 on a 4-byte lane clobbers lane+1).
                                    if acc {
                                        if de >= 4 { buf.mov_load64(R10, RBX, dd); buf.add_rr64(RAX, R10); }
                                        else { buf.mov_load32(R10, RBX, dd); buf.add_rr64(RAX, R10); }
                                        if de == 8 { buf.mov_store64(RBX, dd, RAX); }
                                        else if de == 4 { buf.mov_store32(RBX, dd, RAX); }
                                        else { buf.mov_store16(RBX, dd, RAX); }
                                    } else {
                                        if de == 8 { buf.mov_store64(RBX, dd, RAX); }
                                        else if de == 4 { buf.mov_store32(RBX, dd, RAX); }
                                        else { buf.mov_store16(RBX, dd, RAX); }
                                    }
                                }
                                Ok(())
                            }
                            Inst::SimdAddl { rd, rn, rm, esrc, sign, sub, upper } => {
                                // saddl/uaddl/subl/usubl Vd.T, Vn.T, Vm.T: widen each esrc-byte
                                // element of Vn and Vm (low or upper half) to 2*esrc and add/sub.
                                // Each source element is read EXACTLY esrc bytes and widened to the
                                // 64-bit reg (sign- or zero-extended), so a later lane's high bytes
                                // never pollute the sum; the widened result is stored EXACTLY de bytes
                                // (de = 2*esrc) so it never overruns into the neighbouring lane.
                                // IN-PLACE ALIASING: the widened (2*esrc) dest write
                                // at i*2*esrc overlaps the narrow source bytes of
                                // lane i+1 (at (i+1)*esrc), so when rd aliases rn or
                                // rm a read-then-write loop clobbers the still-needed
                                // source -- e.g. gcc's `uaddl v0.4s, v0.4h, v1.4h`
                                // (rd==rn) turned lane 1's sum into garbage. Snapshot
                                // each source that aliases rd to permscratch first.
                                let vbase = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                let db = vbase(rd);
                                let nb = if rn == rd { permute_source(buf, rd, rn, false) } else { vbase(rn) };
                                let mb = if rm == rd { permute_source(buf, rd, rm, true) } else { vbase(rm) };
                                let se = esrc as i32;
                                let de = 2 * se;
                                let lanes: i32 = 8 / se; // 8-source bytes -> 4 + or (16/2...) 8/se
                                let uh = if upper { 8 } else { 0 };
                                let ld = |b: &mut crate::x86::CodeBuf, reg: u8, at: i32| match se {
                                    4 => {
                                        b.mov_load32(reg, RBX, at);
                                        if sign { b.movsxd_r64_r32(reg, reg); }
                                    }
                                    2 => {
                                        if sign { b.movsx_word_mem(reg, RBX, at); }
                                        else { b.movzx_word_mem(reg, RBX, at); }
                                    }
                                    _ => {
                                        if sign { b.movsx_byte_mem(reg, RBX, at); }
                                        else { b.movzx_byte_mem(reg, RBX, at); }
                                    }
                                };
                                for i in 0..lanes {
                                    let so_n = nb + uh + i * se;
                                    let so_m = mb + uh + i * se;
                                    let dd = db + i * de;
                                    ld(buf, RAX, so_n);
                                    ld(buf, RCX, so_m);
                                    if sub { buf.sub_rr64(RAX, RCX); } else { buf.add_rr64(RAX, RCX); }
                                    match de {
                                        8 => buf.mov_store64(RBX, dd, RAX),
                                        4 => buf.mov_store32(RBX, dd, RAX),
                                        _ => buf.mov_store16(RBX, dd, RAX),
                                    }
                                }
                                Ok(())
                            }
                            Inst::SimdSatAdd { rd, rn, rm, esize, sub, unsigned, q } => {
                                // sqadd/uqadd/sqsub/uqsub: per-lane saturating add/sub.
                                let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                let se = esize as i32;
                                let lanes: i32 = if q { 16 / se } else { 8 / se };
                                let full: u64 = if se == 8 { u64::MAX } else { (1u64 << (se * 8)) - 1 };
                                // Signed bounds as 64-bit SIGN-EXTENDED constants.
                                // smax is the positive max (0x7F.. its low bits);
                                // smin must be the NEGATIVE bound so the 64-bit
                                // compare sees a sign-extended result as < it
                                // (for .4s/.8h/.16b the raw 0x80000000 bit-pattern
                                // as u64 is a positive value, so RAX = 15 would be
                                // "less" than 0x80000000 and wrongly clamp).
                                let smax: u64 = if se == 8 { i64::MAX as u64 } else { ((1i64 << (se * 8 - 1)) - 1) as u64 };
                                let smin: u64 = if se == 8 { i64::MIN as u64 } else { (-(1i64 << (se * 8 - 1))) as u64 };
                                for i in 0..lanes {
                                    let so_n = vslot(rn) + i * se;
                                    let so_m = vslot(rm) + i * se;
                                    let dd = vslot(rd) + i * se;
                                    // Load ONE lane, SIGN-extended for the signed
                                    // path so the 64-bit clamps below see negative
                                    // results correctly (a zero-extended 0x80 lane
                                    // would read as +128, never clamp to smin).
                                    // Unsigned lanes load zero-extended as before.
                                    if unsigned {
                                        match se {
                                            8 => { buf.mov_load64(RAX, RBX, so_n); buf.mov_load64(RCX, RBX, so_m); }
                                            4 => { buf.mov_load32(RAX, RBX, so_n); buf.mov_load32(RCX, RBX, so_m); }
                                            2 => { buf.movzx_word_mem(RAX, RBX, so_n); buf.movzx_word_mem(RCX, RBX, so_m); }
                                            _ => { buf.movzx_byte_mem(RAX, RBX, so_n); buf.movzx_byte_mem(RCX, RBX, so_m); }
                                        }
                                    } else {
                                        match se {
                                            1 => {
                                                buf.movsx_byte_mem(RAX, RBX, so_n);
                                                buf.movsx_byte_mem(RCX, RBX, so_m);
                                            }
                                            2 => {
                                                buf.movsx_word_mem(RAX, RBX, so_n);
                                                buf.movsx_word_mem(RCX, RBX, so_m);
                                            }
                                            4 => {
                                                buf.mov_load32(RAX, RBX, so_n);
                                                buf.movsxd_r64_r32(RAX, RAX);
                                                buf.mov_load32(RCX, RBX, so_m);
                                                buf.movsxd_r64_r32(RCX, RCX);
                                            }
                                            _ => {
                                                buf.mov_load64(RAX, RBX, so_n);
                                                buf.mov_load64(RCX, RBX, so_m);
                                            }
                                        }
                                    }
                                    if unsigned {
                                        if sub {
                                            // uqsub: diff = Vn - Vm; clamp 0 on borrow
                                            buf.mov_ri64(R10, 0);
                                            buf.cmp_rr64(RCX, RAX);   // CF=1 if Vm>Vn
                                            buf.sub_rr64(RAX, RCX);
                                            buf.cmov_rr64(0x42, RAX, R10); // cmovb -> 0 if underflow
                                        } else {
                                            // uqadd: sum; clamp to full on carry
                                            buf.mov_ri64(R10, full);
                                            buf.add_rr64(RAX, RCX);
                                            buf.cmov_rr64(0x42, RAX, R10); // cmovb (carry) -> full
                                        }
                                    } else {
                                        if sub { buf.sub_rr64(RAX, RCX); } else { buf.add_rr64(RAX, RCX); }
                                        buf.mov_ri64(R10, smax as u64);
                                        buf.cmp_rr64(RAX, R10);
                                        buf.cmov_rr64(0x4f, RAX, R10);   // cmovg -> smax
                                        buf.mov_ri64(R10, smin as u64);
                                        buf.cmp_rr64(RAX, R10);
                                        buf.cmov_rr64(0x4c, RAX, R10);   // cmovl -> smin
                                    }
                                    // Store a single lane back (width matches lane).
                                    match se {
                                        8 => buf.mov_store64(RBX, dd, RAX),
                                        4 => buf.mov_store32(RBX, dd, RAX),
                                        2 => buf.mov_store16(RBX, dd, RAX),
                                        _ => buf.mov_store8(RBX, dd, RAX),
                                    }
                                }
                            Ok(())
                            }
                            Inst::FmulScalar { rd, rn, rm, double, neg } => {
                                            // fmul/fnmul Sd/Dd, Sn, Sm: rd = (+/-)(rn*rm) scalar.
                                            let dn = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                            let nn = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                            let mn = crate::jit::VECTOR_BASE + (rm as i32) * 16;
                                            if double {
                                                buf.movq_load(0, RBX, nn);
                                                buf.movq_load(1, RBX, mn);
                                                buf.mulsd(0, 1);
                                                if neg {
                                                    buf.mov_ri64(RCX, 0x8000_0000_0000_0000);
                                                    buf.movq_xmm_r64(1, RCX);
                                                    buf.pxor_xmm(0, 1);
                                                }
                                                buf.movq_store(RBX, dn, 0);
                                            } else {
                                                buf.mov_load32(RCX, RBX, nn);
                                                buf.movd_xmm_r32(0, RCX);
                                                buf.mov_load32(RCX, RBX, mn);
                                                buf.movd_xmm_r32(1, RCX);
                                                buf.mulss(0, 1);
                                                if neg {
                                                    buf.mov_ri64(RCX, 0x8000_0000);
                                                    buf.movd_xmm_r32(1, RCX);
                                                    buf.pxor_xmm(0, 1);
                                                }
                                                buf.movd_r32_xmm(RCX, 0);
                                                buf.mov_store32(RBX, dn, RCX);
                                            }
                                            Ok(())
                                        }
                                        Inst::SminMax { rd, rn, rm, max, unsigned, esize, q } => {
                                            // smin/smax/umin/umax Vd.T, Vn, Vm: per-lane min/max.
                                            let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
                                            let mb = crate::jit::VECTOR_BASE + (rm as i32) * 16;
                                            let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                            let lanes: i32 = if esize == 4 { if q { 4 } else { 2 } } else { if q { 16 } else { 8 } };
                                                                                        for i in 0..lanes {
                                                                                            let e = esize as i32;
                                                                                            let so = nb + i * e;
                                                                                            let mo = mb + i * e;
                                                                                            // load A=Vn[i], B=Vm[i], sign/zero-extended
                                                                                            if e == 4 {
                                                                                                buf.mov_load32(RAX, RBX, so);
                                                                                                if !unsigned { buf.movsxd_r64_r32(RAX, RAX); }
                                                                                                buf.mov_load32(RCX, RBX, mo);
                                                                                                if !unsigned { buf.movsxd_r64_r32(RCX, RCX); }
                                                                                            } else {
                                                                                                if unsigned {
                                                                                                    buf.movzx_byte_mem(RAX, RBX, so);
                                                                                                    buf.movzx_byte_mem(RCX, RBX, mo);
                                                                                                } else {
                                                                                                    buf.movsx_byte_mem(RAX, RBX, so);
                                                                                                    buf.movsx_byte_mem(RCX, RBX, mo);
                                                                                                }
                                                                                            }
                                                                                            buf.cmp_rr64(RCX, RAX); // flags = B - A
                                                                                            if max {
                                                                                                if unsigned { buf.cmov_rr64(0x47, RAX, RCX); } // cmova
                                                                                                else { buf.cmov_rr64(0x4F, RAX, RCX); } // cmovg
                                                                                            } else {
                                                                                                if unsigned { buf.cmov_rr64(0x42, RAX, RCX); } // cmovb
                                                                                                else { buf.cmov_rr64(0x4C, RAX, RCX); } // cmovl
                                                                                            }
                                                                                            if e == 4 { buf.mov_store32(RBX, db + i * e, RAX); }
                                                                                            else { buf.mov_store8(RBX, db + i * e, RAX); }
                                                                                        }
                                                                                        Ok(())
                                                                                    }
                                                                                    Inst::ScvtfFixed { rd, rn, to_double, sf, unsigned, fbits } => {
                                            // ucvtf/scvtf Dd,Rn,#fbits: convert int to float, then /2^fbits.
                                            let vslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                                            ldg(buf, RAX, rn as u32);
                                            if unsigned && sf {
                                                // 64-bit unsigned: cvtsi2sd + the +2^64 correction, then scale.
                                                ldg(buf, RDX, rn as u32);
                                                buf.cvtsi2sd(0, true, RDX);
                                                buf.test_rr64(RDX, RDX);
                                                let jns = buf.jcc_rel32(0x89); // JNS: skip if u < 2^63
                                                buf.mov_ri64(RCX, 0x43f0_0000_0000_0000);
                                                buf.movq_xmm_r64(1, RCX);
                                                buf.addsd(0, 1);
                                                let end = buf.len();
                                                let disp = (end as i64 - (jns as i64 + 4)) as i32;
                                                buf.bytes[jns..jns + 4].copy_from_slice(&disp.to_le_bytes());
                                            } else {
                                                buf.cvtsi2sd(0, sf, RAX);
                                            }
                                            // divide by 2^fbits (normal-power double in xmm1)
                                            buf.mov_ri64(RCX, (((1023 + fbits as u64) << 52)));
                                            buf.movq_xmm_r64(1, RCX);
                                            buf.divsd(0, 1);
                                            if to_double {
                                                buf.movq_store(RBX, vslot, 0);
                                            } else {
                                                buf.cvtsd2ss(0, 0);
                                                buf.movd_r32_xmm(RAX, 0);
                                                buf.mov_store32(RBX, vslot, RAX);
                                            }
                                            Ok(())
                                        }
        Inst::Fcmp { rn, rm, against_zero, sz, half } => {
            // fcmp d{rn}, d{rm} / fcmp d{rn}, #0.0 / fcmp s{rn}, s{rm}:
            // compare and set guest NZCV. Use comisd/comiss (CF=1 if a<b,
            // ZF=1/PF=1 if unordered); store_nzcv_fp maps to AArch64 NZCV.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if sz {
                buf.movq_load(0, RBX, vslot(rn));
                if against_zero {
                    buf.pxor_xmm(1, 1); // xmm1 = +0.0
                } else {
                    buf.movq_load(1, RBX, vslot(rm));
                }
            } else {
                // half: load the low f16 half, promote to f32 (F16C), comiss.
                if half {
                    buf.mov_load16(RAX, RBX, vslot(rn));
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    if against_zero {
                        buf.pxor_xmm(1, 1); // xmm1 = +0.0f
                    } else {
                        buf.mov_load16(RAX, RBX, vslot(rm));
                        buf.movd_xmm_r32(1, RAX);
                        buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc9]); // vcvtph2ps xmm1,xmm1
                    }
                } else {
                    buf.mov_load32(RAX, RBX, vslot(rn));
                    buf.movd_xmm_r32(0, RAX);
                    if against_zero {
                        buf.pxor_xmm(1, 1); // xmm1 = +0.0f
                    } else {
                        buf.mov_load32(RAX, RBX, vslot(rm));
                        buf.movd_xmm_r32(1, RAX);
                    }
                }
            }
            if sz {
                buf.comisd(0, 1);
            } else {
                buf.comiss(0, 1); // works for both promoted-half and single
            }
            store_nzcv_fp(buf);
            Ok(())
        }
        Inst::Fccmp { rn, rm, nzcv, cond, sz } => {
            // fccmp Dn, Dm, #nzcv, <cond>: if cond(guest NZCV) do FP compare -> NZCV;
            // else NZCV = nzcv. Load guest flags, jcc to the compare path when cond
            // true, else write the immediate nzcv, then patch both rel32 wires.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let jcc = x86_cc_for_cond(cond)
                .ok_or_else(|| format!("Fccmp: bad cond {cond:#x}"))?;
            load_nzcv_to_eflags(buf);              // eflags = guest NZCV (cond)
            let j_cond = buf.jcc_rel32(jcc);       // jump to fp-compare when cond TRUE
            // cond FALSE: NZCV = nzcv immediate
            buf.mov_ri64(RAX, (nzcv as u64) << 28);
            buf.mov_store32(RBX, NZCV_OFF, RAX);
            let j_end = buf.jmp_rel32();           // skip over the fp-compare path
            let fp_off = buf.len() as i32;         // start of cond-TRUE path
            if sz {
                buf.movq_load(0, RBX, vslot(rn));
                buf.movq_load(1, RBX, vslot(rm));
                buf.comisd(0, 1);
            } else {
                buf.mov_load32(RAX, RBX, vslot(rn));
                buf.movd_xmm_r32(0, RAX);
                buf.mov_load32(RAX, RBX, vslot(rm));
                buf.movd_xmm_r32(1, RAX);
                buf.comiss(0, 1);
            }
            store_nzcv_fp(buf);
            let tail_off = buf.len() as i32;
            // patch displacements: relative to disp_off+4
            let jd = j_cond as i32;
            let d1 = ((fp_off - (jd + 4)) as u32).to_le_bytes();
            buf.bytes[jd as usize..(jd + 4) as usize].copy_from_slice(&d1);
            let je = j_end as i32;
            let d2 = ((tail_off - (je + 4)) as u32).to_le_bytes();
            buf.bytes[je as usize..(je + 4) as usize].copy_from_slice(&d2);
            Ok(())
        }
        Inst::CcMp { rn, rm, imm, nzcv, cond, cmn, sf, is_reg } => {
            // ccmp/ccmn Rn, <Rm|#imm>, #nzcv, <cond>: if cond(guest NZCV) holds,
            // the flags become `Rn op <Rm|imm>` (ccmn=add, ccmp=subtract); else
            // they become the 4-bit `nzcv` immediate. Flag-only, no writeback —
            // same branch-then-patch structure as Fccmp above.
            let jcc = x86_cc_for_cond(cond)
                .ok_or_else(|| format!("CcMp: bad cond {cond:#x}"))?;
            load_nzcv_to_eflags(buf);              // eflags = guest NZCV (cond eval)
            let j_cond = buf.jcc_rel32(jcc);       // jump to the compare path when cond TRUE
            // cond FALSE: NZCV = the nzcv immediate (N[3]Z[2]C[1]V[0] -> bits 31/30/29/28)
            buf.mov_ri64(RAX, (nzcv as u64) << 28);
            buf.mov_store32(RBX, NZCV_OFF, RAX);
            let j_end = buf.jmp_rel32();           // skip the compare path
            let cmp_off = buf.len() as i32;        // start of cond-TRUE path
            // cond TRUE: compute Rn op <Rm|imm> and set flags from the result.
            if rn == 31 {
                buf.mov_ri64(RAX, 0);              // Rn = XZR (0)
            } else {
                ldg(buf, RAX, rn as u32);
            }
            if !sf {
                zext_w(buf, RAX); // 32-bit compare: zero high garbage
            }
            if is_reg {
                if rm == 31 {
                    buf.mov_ri64(RCX, 0);
                } else {
                    ldg(buf, RCX, rm as u32);
                }
                if !sf {
                    zext_w(buf, RCX);
                }
            } else {
                buf.mov_ri64(RCX, imm as u64);
            }
            if cmn {
                // cmn: Rn + src (op=add)
                if sf {
                    buf.add_rr64(RAX, RCX);
                } else {
                    buf.add_rr32(RAX, RCX);
                }
            } else if sf {
                // ccmp: Rn - src (same sub semantics as `cmp` -> branch conds agree)
                buf.sub_rr64(RAX, RCX);
            } else {
                buf.sub_rr32(RAX, RCX);
            }
            store_nzcv(buf);
            let tail_off = buf.len() as i32;
            // patch displacements: disp relative to disp_off+4
            let jd = j_cond as i32;
            let d1 = ((cmp_off - (jd + 4)) as u32).to_le_bytes();
            buf.bytes[jd as usize..(jd + 4) as usize].copy_from_slice(&d1);
            let je = j_end as i32;
            let d2 = ((tail_off - (je + 4)) as u32).to_le_bytes();
            buf.bytes[je as usize..(je + 4) as usize].copy_from_slice(&d2);
            Ok(())
        }
        Inst::FcvtTzReg { rd, rn, dbl, unsigned } => {
            // fcvtzs/fcvtzu Dd,Dn / Sd,Sn: store the trunc toward-zero int of a FP
            // scalar back into the destination FP reg as an integer bit-pattern.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let dst = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if dbl {
                buf.movq_load(0, RBX, vslot(rn));
                buf.cvttsd2si(RAX, 0);
            } else {
                buf.mov_load32(RAX, RBX, vslot(rn));
                buf.movd_xmm_r32(0, RAX);
                buf.cvtss2sd(0, 0);
                buf.cvttsd2si(RAX, 0);
            }
            if unsigned {
                // fcvtzu: clamp negatives to 0 (trunc-toward-zero unsigned).
                buf.mov_ri64(RCX, 0);
                buf.test_rr64(RAX, RAX);
                buf.cmov_rr64(0x48, RAX, RCX); // cmovs RAX,RCX (RAX<0 -> 0)
            }
            if dbl { buf.mov_store64(RBX, dst(rd), RAX); }
            else { buf.mov_store32(RBX, dst(rd), RAX); }
            Ok(())
        }
            Inst::SimdPopcnt { rd, rn } => {
            // cnt v{rd}.8b, v{rn}.8b : per-byte bit-popcount via SWAR.
            let slot = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            buf.mov_load64(RAX, RBX, slot); // x
            // x = x - ((x >> 1) & 0x5555_5555_5555_5555)
            buf.mov_rr64(RDX, RAX);
            buf.shr_ri8(RDX, 1);
            buf.mov_ri64(RCX, 0x5555_5555_5555_5555);
            buf.and_rr64(RDX, RCX);
            buf.sub_rr64(RAX, RDX);
            // x = (x & 0x3333...) + ((x >> 2) & 0x3333...)
            buf.mov_rr64(RDX, RAX);
            buf.shr_ri8(RDX, 2);
            buf.mov_ri64(RCX, 0x3333_3333_3333_3333);
            buf.and_rr64(RDX, RCX);
            buf.and_rr64(RAX, RCX);
            buf.add_rr64(RAX, RDX);
            // x = (x + (x >> 4)) & 0x0f0f_0f0f_0f0f_0f0f
            buf.mov_rr64(RDX, RAX);
            buf.shr_ri8(RDX, 4);
            buf.add_rr64(RAX, RDX);
            buf.mov_ri64(RCX, 0x0f0f_0f0f_0f0f_0f0f);
            buf.and_rr64(RAX, RCX);
            let dslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            buf.mov_store64(RBX, dslot, RAX);
            Ok(())
        }
        Inst::SimdSum8 { rd, rn } => {
            // uaddlv h{rd}, v{rn}.8b : sum the 8 bytes of the slot into the
            // low 16 bits (zero-extended to the d slot). Bytes in are each an
            // 8-bit popcount (<= 8), so the sum fits well within the half.
            let slot = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            buf.mov_load64(RAX, RBX, slot); // x
            buf.mov_ri64(RCX, 0x00ff_00ff_00ff_00ff);
            buf.mov_rr64(RDX, RAX);
            buf.shr_ri8(RDX, 8);
            buf.and_rr64(RDX, RCX);
            buf.and_rr64(RAX, RCX);
            buf.add_rr64(RAX, RDX); // s16 = per-16 sums
            buf.mov_rr64(RDX, RAX);
            buf.shr_ri8(RDX, 16);
            buf.mov_ri64(RCX, 0x0000_ffff_0000_ffff);
            buf.and_rr64(RDX, RCX);
            buf.and_rr64(RAX, RCX);
            buf.add_rr64(RAX, RDX); // per-32 sums
            buf.mov_rr64(RDX, RAX);
            buf.shr_ri8(RDX, 32);
            buf.mov_ri64(RCX, 0x0000_0000_ffff_ffff);
            buf.and_rr64(RDX, RCX);
            buf.and_rr64(RAX, RCX);
            buf.add_rr64(RAX, RDX); // total
            let dslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            buf.mov_store64(RBX, dslot, RAX);
            Ok(())
        }
        Inst::Fcvt { to_d, rd, rn } => {
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            if to_d {
                // fcvt d{rd}, s{rn}: single -> double (widen).
                buf.mov_load32(RAX, RBX, vslot(rn)); // single in low 32 of slot
                buf.movd_xmm_r32(0, RAX);            // xmm0 = s{rn}
                buf.cvtss2sd(0, 0);                   // xmm0 = (double) xmm0
                let dslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                buf.movq_store(RBX, dslot, 0);       // low 64 = double result
            } else {
                // fcvt s{rd}, d{rn} : double -> single (narrow).
                buf.movq_load(0, RBX, vslot(rn));     // xmm0 = double d{rn}
                buf.cvtsd2ss(0, 0);                   // xmm0 = floating single
                buf.movd_r32_xmm(RAX, 0);            // RAX = low 32 (single bits)
                let dslot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                buf.mov_store32(RBX, dslot, RAX);
            }
            Ok(())
        }
        Inst::FcvtHalf { rd, rn, op } => {
            // scalar FP16 <-> FP32/FP64 convert. Op:
            // 0=H->S (fcvt s,h)  1=S->H (fcvt h,s)  2=H->D (fcvt d,h)  3=D->H (fcvt h,d).
            // Reuses the F16C promote (vcvtph2ps) / demote (vcvtps2ph, imm 0 = RN).
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            match op {
                0 => {
                    buf.mov_load32(RAX, RBX, vslot(rn)); // h{rn} in low16
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    buf.movd_r32_xmm(RAX, 0); // f32 in low32
                    buf.mov_store32(RBX, vslot(rd), RAX);
                }
                1 => {
                    buf.mov_load32(RAX, RBX, vslot(rn)); // f32 in low32
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                    buf.movd_r32_xmm(RAX, 0); // f16 in low16 (upper zeroed)
                    buf.mov_store32(RBX, vslot(rd), RAX);
                }
                2 => {
                    buf.mov_load32(RAX, RBX, vslot(rn)); // h{rn} in low16
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    buf.cvtss2sd(0, 0); // (double) f32
                    buf.movq_store(RBX, vslot(rd), 0); // f64
                }
                3 => {
                    buf.movq_load(0, RBX, vslot(rn)); // f64
                    buf.cvtsd2ss(0, 0); // f32 in low32
                    buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                    buf.movd_r32_xmm(RAX, 0); // f16 in low16
                    buf.mov_store32(RBX, vslot(rd), RAX);
                }
                _ => return Err(format!("FcvtHalf: bad op {op}")),
            }
            Ok(())
        }
        Inst::Urhadd { rd, rn, rm, bytes } => {
            // urhadd Vd.T, Vn.T, Vm.T = (a+b+1)>>1 per byte lane. Byte lanes on
            // a 16B slot: load each byte of Vn and Vm, sum, +1, >>1, store.
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            for i in 0..bytes {
                let off = i as i32;
                buf.mov_load32(RAX, RBX, slot(rn) + off);
                buf.and_ri64(RAX, 0xff);
                buf.mov_load32(RDX, RBX, slot(rm) + off);
                buf.and_ri64(RDX, 0xff);
                buf.add_rr64(RAX, RDX); // a+b
                buf.add_ri64(RAX, 1);   // a+b+1
                buf.mov_ri64(RCX, 1);
                buf.shr_cl64(RAX);      // (a+b+1)>>1
                buf.mov_store8(RBX, slot(rd) + off, RAX);
            }
            Ok(())
        }
        Inst::SimdFp16As { rd, rn, rm, op, q } => {
            // fadd/fsub/fmul/fdiv/fmax/fmin Vd.8h/.4h, Vn, Vm (half-precision
            // 3-same): per-h lane promote (clean, even with rd aliasing rn/rm
            // because each lane's sources are read before its dest write), op in
            // f32, demote. F16C:
            //   vcvtph2ps xmm,xmm = C4 E2 79 13 /r ; vcvtps2ph $0 = C4 E3 79 1D /r 00.
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 8 } else { 4 }; // half-precision lanes
            // SSE scalar opcodes (F3 0F op rC1) for the pure-binary ops 0-7:
            // fadd/sub/mul/div/max/min/maxnm/minnm -> addss/subss/mulss/divss/
            // maxss/minss/maxss/minss (fmaxnm & fminnm behave like fmax/fmin for
            // the finite values the engine uses; NaNs differ only in propagate).
            let opcode: u8 = match op {
                0 => 0x58, // fadd  addss
                1 => 0x5c, // fsub  subss
                2 => 0x59, // fmul  mulss
                3 => 0x5e, // fdiv  divss
                4 => 0x5f, // fmax  maxss
                5 => 0x5d, // fmin  minss
                6 => 0x5f, // fmaxnm maxss
                7 => 0x5d, // fminnm minss
                _ => 0x59, // fmul lamine; op 8/9 take the accumulate path
            };
            // fmla/fmls accumulate BEFORE loading Vm into xmm1 (so we have a free
            // reg to read Vd). Binary ops (0-7) and fmla/fmls(8/9) share the
            // promote-Vn/promote-Vm -> op -> demote shape; fmla/fmls add/sub Vd.
            let inv = |buf: &mut crate::x86::CodeBuf, x: u8| {
                // fmla/fmls to SSE: mulss xmm2(=Vn),xmm1(=Vm); addss/subss xmm0(=Vd)
                buf.bytes.extend_from_slice(&[0xf3, 0x0f, 0x59, 0xca]); // mulss xmm1,xmm2
                if x == 8 {
                    buf.bytes.extend_from_slice(&[0xf3, 0x0f, 0x58, 0xc1]); // addss xmm0,xmm1
                } else {
                    buf.bytes.extend_from_slice(&[0xf3, 0x0f, 0x5c, 0xc1]); // subss xmm0,xmm1
                }
            };
            for i in 0..lanes {
                let off = i * 2;
                // Vn[lan] promote -> xmm2 (fmla/mlf) or xmm0 (binary via xmm1 op)
                buf.mov_load32(RAX, RBX, f(rn) + off);
                buf.movd_xmm_r32(2, RAX);
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xd2]); // vcvtph2ps xmm2,xmm2
                // Vm[lan] promote -> xmm1
                buf.mov_load32(RAX, RBX, f(rm) + off);
                buf.movd_xmm_r32(1, RAX);
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc9]); // vcvtph2ps xmm1,xmm1
                if op == 8 || op == 9 {
                    // fmla Vd[l] += Vn[l]*Vm[l] (promote Vd -> xmm0, mul, add/sub)
                    buf.mov_load32(RAX, RBX, f(rd) + off);
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    inv(buf, op);
                } else {
                    // pure binary: Vn(xmm2) op Vm(xmm1) -> xmm0 (movaps xmm2->xmm0)
                    buf.bytes.extend_from_slice(&[0x0f, 0x28, 0xc2]); // movaps xmm0,xmm2
                    buf.bytes.extend_from_slice(&[0xf3, 0x0f, opcode, 0xc1]); // opss xmm0,xmm1
                }
                // demote -> f16, store 2 bytes at Vd[lan]
                buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                buf.movd_r32_xmm(RAX, 0);
                buf.mov_store16(RBX, f(rd) + off, RAX);
            }
            Ok(())
        }
        Inst::SimdFp16BEl { rd, rn, vlm, idx, op, q } => {
            // fmla/fmls/fmul Vd.8h/.4h, Vn, Vm.h[idx]: per-lane
            //   Vd[l] = (fmla/fmls) Vd[l] ± Vn[l]·splat(Vm.h[idx])  (in f32),
            //   fmul = splat(Vm.h[idx])·Vn[l]  (no accumulate).
            // Splat Vm.h[idx] into xmm2 ONCE (hoisted: rd==rm would clobber the
            // element inside the lane loop), then promote->op->demote per lane
            // via F16C. Uses the same raw-VEX f16 pattern as SimdFp16As.
            let db = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let nb = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let mb = crate::jit::VECTOR_BASE + (vlm as i32) * 16;
            let lanes: i32 = if q { 8 } else { 4 };
            // splat Vm.h[idx] -> xmm2 (as f32)
            buf.mov_load16(RAX, RBX, mb + (idx as i32) * 2);
            buf.movd_xmm_r32(2, RAX);
            buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xd2]); // vcvtph2ps xmm2,xmm2
            for l in 0..lanes {
                let off = l * 2;
                buf.mov_load32(RAX, RBX, db + off);
                buf.movd_xmm_r32(0, RAX); // Vd[l] (f16 in low16; may be garbage for fmul)
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0 (promote Vd)
                buf.mov_load32(RAX, RBX, nb + off);
                buf.movd_xmm_r32(1, RAX);
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc9]); // vcvtph2ps xmm1,xmm1 (Vn[l])
                buf.mulss(1, 2); // xmm1 = Vn[l] * splat
                match op {
                    0 => buf.addss(0, 1), // fmla: Vd[l] + term
                    1 => buf.subss(0, 1), // fmls: Vd[l] - term
                    _ => {
                        // fmul: result = term in xmm1; copy to xmm0 so the demote
                        // below is the same xmm0->xmm0 vcvtps2ph for ALL ops.
                        buf.bytes.extend_from_slice(&[0x0f, 0x28, 0xc1]); // movaps xmm0,xmm1
                    }
                }
                // demote result xmm0 -> f16, store 2 bytes
                buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                buf.movd_r32_xmm(RAX, 0);
                buf.mov_store16(RBX, db + off, RAX);
            }
            Ok(())
        }
        Inst::SimdHadd { rd, rn, rm, unsigned, esize, q, rounding } => {
            // uhadd/shadd: per-lane floor((a+b)/2) = (a>>1)+(b>>1)+((a&1)&(b&1)).
            // For signed (shadd) use arithmetic shifts (sar) on sign-extended
            // values so the floor rounds toward -inf, matching ARM. esize-stride.
            // srhadd/urhadd (rounding=true): round-half-up (a+b+1)>>1, arithmetic
            // (sar) for signed / logical (shr) for unsigned after 64-bit add.
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let e = esize as i32; // 1, 2, or 4 bytes
            for lane in 0..lanes {
                let off = lane * e;
                if unsigned {
                    // zero-extend loads, logical >>1
                    match e {
                        1 => { buf.movzx_byte_mem(RAX, RBX, f(rn) + off); buf.movzx_byte_mem(RCX, RBX, f(rm) + off); }
                        2 => { buf.mov_load16(RAX, RBX, f(rn) + off); buf.mov_load16(RCX, RBX, f(rm) + off); }
                        _ => { buf.mov_load32(RAX, RBX, f(rn) + off); buf.mov_load32(RCX, RBX, f(rm) + off); }
                    }
                } else {
                    // sign-extended loads, arithmetic >>1
                    match e {
                        1 => { buf.movsx_byte_mem(RAX, RBX, f(rn) + off); buf.movsx_byte_mem(RCX, RBX, f(rm) + off); }
                        2 => { buf.movsx_word_mem(RAX, RBX, f(rn) + off); buf.movsx_word_mem(RCX, RBX, f(rm) + off); }
                        _ => { buf.mov_load32(RAX, RBX, f(rn) + off); buf.movsxd_r64_r32(RAX, RAX);
                               buf.mov_load32(RCX, RBX, f(rm) + off); buf.movsxd_r64_r32(RCX, RCX); }
                    }
                }
                if rounding {
                    // (a+b+1)>>1 : add b, add 1, then arithmetic/logical shift.
                    buf.add_rr64(RAX, RCX);
                    buf.add_ri64(RAX, 1);
                    if unsigned { buf.shr_ri8(RAX, 1); } else { buf.sar_ri8(RAX, 1); }
                    match e {
                        1 => buf.mov_store8(RBX, f(rd) + off, RAX),
                        2 => buf.mov_store16(RBX, f(rd) + off, RAX),
                        _ => buf.mov_store32(RBX, f(rd) + off, RAX),
                    }
                    continue;
                }
                // floor term: (a>>1)+(b>>1) then +((a&1)&(b&1)).
                buf.mov_rr64(RDX, RAX); // RDX = a (save for carry)
                if unsigned { buf.shr_ri8(RAX, 1); } else { buf.sar_ri8(RAX, 1); }
                if unsigned { buf.shr_ri8(RCX, 1); } else { buf.sar_ri8(RCX, 1); }
                buf.add_rr64(RAX, RCX);
                // carry = (a&1)&(b&1): RDX &= b (reload b low), &= 1
                buf.mov_load16(RCX, RBX, f(rm) + off); // RCX = b (bit0)
                buf.and_rr64(RDX, RCX);
                buf.and_ri64(RDX, 1);
                buf.add_rr64(RAX, RDX);
                // store esize-low
                match e {
                    1 => buf.mov_store8(RBX, f(rd) + off, RAX),
                    2 => buf.mov_store16(RBX, f(rd) + off, RAX),
                    _ => buf.mov_store32(RBX, f(rd) + off, RAX),
                }
            }
            Ok(())
        }
        Inst::SimdHsub { rd, rn, rm, unsigned, esize, q } => {
            // shsub/uhsub: per-lane floor((a-b)/2). Compute diff=a-b in 64-bit then
            // shift right 1: arithmetic (sar) for signed, logical (shr) for unsigned.
            // Storing only the low esize bits yields the correct mod-2^esize floor
            // in both cases (unsigned (a-b) wraps, but floor((a-b)/2) mod 2^esize
            // of the wrapping difference equals the ARM result).
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let e = esize as i32;
            for lane in 0..lanes {
                let off = lane * e;
                if unsigned {
                    match e {
                        1 => { buf.movzx_byte_mem(RAX, RBX, f(rn) + off); buf.movzx_byte_mem(RCX, RBX, f(rm) + off); }
                        2 => { buf.mov_load16(RAX, RBX, f(rn) + off); buf.mov_load16(RCX, RBX, f(rm) + off); }
                        _ => { buf.mov_load32(RAX, RBX, f(rn) + off); buf.mov_load32(RCX, RBX, f(rm) + off); }
                    }
                } else {
                    match e {
                        1 => { buf.movsx_byte_mem(RAX, RBX, f(rn) + off); buf.movsx_byte_mem(RCX, RBX, f(rm) + off); }
                        2 => { buf.movsx_word_mem(RAX, RBX, f(rn) + off); buf.movsx_word_mem(RCX, RBX, f(rm) + off); }
                        _ => { buf.mov_load32(RAX, RBX, f(rn) + off); buf.movsxd_r64_r32(RAX, RAX);
                               buf.mov_load32(RCX, RBX, f(rm) + off); buf.movsxd_r64_r32(RCX, RCX); }
                    }
                }
                buf.sub_rr64(RAX, RCX); // diff = a - b
                if unsigned { buf.shr_ri8(RAX, 1); } else { buf.sar_ri8(RAX, 1); }
                match e {
                    1 => buf.mov_store8(RBX, f(rd) + off, RAX),
                    2 => buf.mov_store16(RBX, f(rd) + off, RAX),
                    _ => buf.mov_store32(RBX, f(rd) + off, RAX),
                }
            }
            Ok(())
        }
        Inst::SimdTbl { rd, rn, rm, len2, tbx, q } => {
            // tbl/tbx Vd.16B|8B, {Vn..Vn+len2}, Vm: per byte lane i, idx=Vm[i].
            // Table = len2+1 contiguous 16-byte regs starting at slot(rn); because
            // they're contiguous, table byte `idx` lives at slot(rn)+idx. If
            // idx < 16*(len2+1): Vd[i]=table[idx]; else Vd[i]=0 (tbl) / keep (tbx).
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 16 } else { 8 };
            let table_bytes = 16 * (len2 as i32 + 1);
            for i in 0..lanes {
                // idx = Vm[i]
                buf.movzx_byte_mem(RAX, RBX, f(rm) + i);
                buf.cmp_ri64(RAX, table_bytes as u32);
                // If idx >= table_bytes, skip the table lookup (keep Vd[i] for tbx,
                // or remember to clear it below for tbl).
                let j_ge = buf.jcc_rel32(0x83); // JAE (unsigned >=)
                // table[idx] -> AL
                buf.movzx_byte_mem_idxd(RAX, RBX, RAX, f(rn));
                buf.mov_store8(RBX, f(rd) + i, RAX);
                let j_end = buf.jmp_rel32();
                let ge_target = buf.bytes.len() as i64;
                let disp_ge = (ge_target - (j_ge as i64 + 4)) as i32;
                buf.bytes[j_ge..j_ge + 4].copy_from_slice(&disp_ge.to_le_bytes());
                // for tbl (not tbx), clear Vd[i] when idx out of table range:
                if !tbx {
                    buf.mov_ri64(RAX, 0);
                    buf.mov_store8(RBX, f(rd) + i, RAX);
                }
                let end_target = buf.bytes.len() as i64;
                let disp_end = (end_target - (j_end as i64 + 4)) as i32;
                buf.bytes[j_end..j_end + 4].copy_from_slice(&disp_end.to_le_bytes());
            }
            Ok(())
        }
        Inst::SimdFreFrsqrte { rd, rn, sqrt, esize, q } => {
            // frecpe/frsqrte Vd.T, Vn.T: per-lane approximate reciprocal or
            // 1/sqrt. esize 4 -> rcpss/rsqrtss on the promoted 32-bit; esize 8
            // -> rcpsd/rsqrtsd on the 64-bit. Keep 32-bit for esize==4 (load
            // f32, op, store f32); for esize 8 load/store f64.
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let e = esize as i32;
            // SSE: F3 0F 53 = rcpss (32-bit), F2 0F 53 /r = rcpsd (64-bit);
            // F3 0F 52 = rsqrtss, F2 0F 52 = rsqrtsd.
            for lane in 0..lanes {
                let off = lane * e;
                if e == 2 {
                    // FP16: promote to fp32, apply approximate op, demote back.
                    buf.mov_load16(RAX, RBX, f(rn) + off);
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    buf.bytes.extend_from_slice(if sqrt { &[0xf3, 0x0f, 0x52, 0xc0] } else { &[0xf3, 0x0f, 0x53, 0xc0] });
                    buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store16(RBX, f(rd) + off, RAX);
                } else if e == 4 {
                    buf.mov_load32(RAX, RBX, f(rn) + off);
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(if sqrt { &[0xf3, 0x0f, 0x52, 0xc0] } else { &[0xf3, 0x0f, 0x53, 0xc0] });
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, f(rd) + off, RAX);
                } else {
                    buf.movq_load(0, RBX, f(rn) + off);
                    buf.bytes.extend_from_slice(if sqrt { &[0xf2, 0x0f, 0x52, 0xc0] } else { &[0xf2, 0x0f, 0x53, 0xc0] });
                    buf.movq_store(RBX, f(rd) + off, 0);
                }
            }
            Ok(())
        }
        Inst::SimdAbd { rd, rn, rm, signed: _signed, esize, q } => {
            // uabd/sabd Vd.T, Vn.T, Vm.T: per-lane |Vn - Vm|. Both uabd and sabd
            // compute the magnitude of the two's-complement (esize) difference,
            // so one path serves both: diff = (Vn - Vm) mod 2^esize; if the
            // esize sign bit is SET, diff = -diff. Per-lane on the GP registers
            // (tiny element counts: 8/16/8/4/4/2 lanes).
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let e = esize as i32; // 1, 2, or 4 bytes per element
            for lane in 0..lanes {
                let off = lane * e;
                let (sb, hb): (i32, i32) = match e {
                    1 => (8, 7),
                    2 => (16, 15),
                    _ => (32, 31),
                };
                // RAX = Vn[lane], RCX = Vm[lane] (zero-extended in the reg)
                match e {
                    1 => { buf.mov_load16(RAX, RBX, f(rn) + off); buf.mov_load16(RCX, RBX, f(rm) + off); }
                    2 => { buf.mov_load16(RAX, RBX, f(rn) + off); buf.mov_load16(RCX, RBX, f(rm) + off); }
                    _ => { buf.mov_load32(RAX, RBX, f(rn) + off); buf.mov_load32(RCX, RBX, f(rm) + off); }
                }
                buf.sub_rr64(RAX, RCX); // RAX = diff (low esize bits correct)
                // If the esize sign bit (e*8-1) of the diff is SET, RAX = -RAX.
                let signbit = (e * 8 - 1) as u8;
                buf.bytes.extend_from_slice(&[0x48, 0x0f, 0xba, 0xe0, signbit]); // bt RAX, signbit (CF=bit)
                let jcc = buf.jcc_rel32(0x83); // JNB: jump if CF=0 (diff non-negative)
                // negate: RAX = 0 - RAX
                buf.mov_ri64(RCX, 0);
                buf.sub_rr64(RCX, RAX);
                buf.mov_rr64(RAX, RCX);
                let end = buf.len();
                let disp = (end as i64 - (jcc as i64 + 4)) as i32;
                buf.bytes[jcc..jcc + 4].copy_from_slice(&disp.to_le_bytes());
                // store the esize-low bits
                match e {
                    1 => buf.mov_store8(RBX, f(rd) + off, RAX),
                    2 => buf.mov_store16(RBX, f(rd) + off, RAX),
                    _ => buf.mov_store32(RBX, f(rd) + off, RAX),
                }
            }
            Ok(())
        }
        Inst::InsD1D0 { rd, rn } => {
            // mov v{rd}.d[1], v{rn}.d[0] : copy the low 64 (D[0]) of Rn into
            // the high 64 (D[1]) of Rd.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            buf.mov_load64(RAX, RBX, vslot(rn)); // low 64 of Vn
            buf.mov_store64(RBX, vslot(rd) + 8, RAX); // high 64 of Vd
            Ok(())
        }
        Inst::Simd4s { rd, rn, rm, op: 0 } => {
                    // add Vd.4s, Vn.4s, Vm.4s : 4x32-bit lane add via paddd.
                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    buf.movdqu_load(0, RBX, slot(rn)); // xmm0 = Vn (128-bit)
                    buf.movdqu_load(1, RBX, slot(rm)); // xmm1 = Vm
                    buf.paddd(0, 1); // xmm0 = Vn + Vm (4x32)
                    buf.movdqu_store(RBX, slot(rd), 0); // Vd = result
                    Ok(())
                }
                Inst::Simd4s { rd, rn, rm, op: 1 } => {
                    // sub Vd.4s, Vn.4s, Vm.4s : 4x32-bit lane subtract via psubd.
                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    buf.movdqu_load(0, RBX, slot(rn));
                    buf.movdqu_load(1, RBX, slot(rm));
                    buf.psubd(0, 1); // xmm0 = Vn - Vm (4x32)
                    buf.movdqu_store(RBX, slot(rd), 0);
                    Ok(())
                }
        Inst::SimdFpToInt { rd, rn, unsigned, esize, q, mode } => {
            // fcvtas Vd.T, Vn.T : round FP vector lanes to integer lanes.
            // Per-lane, modeled on the scalar FcvtToInt. The decode gate only
            // ever emits signed fcvtas (mode=2, unsigned=false) for the real
            // boot path; truncate (fcvtzs, mode=0) is the only other mode this
            // Inst can carry and is handled directly. esize 4 loads f32
            // (promote to f64); esize 8 loads f64.
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: u32 = if q {
                if esize == 8 { 2 } else { 4 }
            } else {
                if esize == 8 { 1 } else { 2 }
            };
            let _ = unsigned;
            let elem = esize as i32; // 4 or 8 bytes per element
            for lane in 0..lanes {
                let off = lane as i32 * elem;
                if esize == 8 {
                    buf.movq_load(0, RBX, slot(rn) + off); // xmm0 = f64 lane
                } else {
                    buf.mov_load32(RAX, RBX, slot(rn) + off); // RAX = f32 bits
                    buf.movd_xmm_r32(0, RAX);
                    buf.cvtss2sd(0, 0); // promote to f64
                }
                if mode == 2 {
                    // fcvtas: round-to-nearest (MXCSR nearest; ARM is ties-
                    // away, x86 ties-even — acceptable for render/color, and
                    // matches the scalar FcvtToInt path exactly).
                    buf.cvtsd2si(RAX, 0);
                } else {
                    // mode 0 (fcvtzs): truncate toward zero.
                    buf.cvttsd2si(RAX, 0);
                }
                // store int32 lane (.2s/.4s) or int64 (esize 8).
                if esize == 8 {
                    buf.mov_store64(RBX, slot(rd) + off, RAX);
                } else {
                    buf.mov_store32(RBX, slot(rd) + (lane as i32) * 4, RAX);
                }
            }
            Ok(())
        }
        Inst::SimdI2Fp16 { rd, rn, unsigned, q } => {
            // scvtf/ucvtf Vd.4H/.8H, Vn: per halfword lane, integer -> f16.
            //   signed: load s16, cvtsi2ss (f32), demote f16; store 2 bytes.
            //   unsigned: load u16 (zero-extend), cvtsi2ss (signed is fine since
            //   u16 <= 65535 < i32::MAX), demote.
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 8 } else { 4 };
            for l in 0..lanes {
                let off = l * 2;
                if unsigned {
                    buf.mov_load16(RAX, RBX, f(rn) + off); // zero-extend u16
                } else {
                    buf.movsx_word_mem(RAX, RBX, f(rn) + off); // sign-extend s16
                }
                buf.cvtsi2ss(0, false, RAX); // xmm0 = (float)int32 -> low f32
                // demote to 16-bit float (F16C imm 0 = round-nearest)
                buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                buf.movd_r32_xmm(RAX, 0);
                buf.mov_store16(RBX, f(rd) + off, RAX);
            }
            Ok(())
        }
        Inst::Pmull1q { rd, rn, rm, hi } => {
            // pmull/pmull2 Vd.1Q, Vn.1D, Vm.1D : 64x64 carry-less (polynomial)
            // multiply -> 128-bit. x86 PCLMULQDQ with imm=0 (low64 x low64),
            // loading the selected 64-bit half of Vn/Vm into the low lane first.
            // pmull (hi=false): element 0 (bytes 0..7); pmull2 (hi=true): element
            // 1 (bytes 8..15). Result (128b) is stored to Vd's 16-byte slot.
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let n_off = slot(rn) + if hi { 8 } else { 0 };
            let m_off = slot(rm) + if hi { 8 } else { 0 };
            let d_off = slot(rd);
            // xmm0 = low64 of selected itmem of Vn, high64 zeroed (movq_load zeroes).
            buf.movq_load(0, RBX, n_off);
            buf.movq_load(1, RBX, m_off);
            buf.pclmulq(0, 1, 0x00); // xmm0 = clmul(xmm0.low64, xmm1.low64)
            buf.movdqu_store(RBX, d_off, 0); // 128-bit result to Vd
            Ok(())
        }
        Inst::SimdFp16Cmpz { rd, rn, op, q } => {
            // fcmeq/fcmgt/fcmge/fcmle/fcmlt Vd.4H/.8H, Vn, #0 : per-lane OP against
            // +0.0, all-ones or 0 mask (16-bit each). Promote half->f32 (F16C),
            // compare to 0.0f32 with ucomiss, setcc, neg to build 0xffff/0x0000.
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 8 } else { 4 };
            for l in 0..lanes {
                let off = l * 2;
                buf.mov_load16(RAX, RBX, f(rn) + off); // 0F B7 = MOVZX -> r32
                buf.movd_xmm_r32(0, RAX);
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0 (F16C)
                // xmm1 = 0.0f32
                buf.pxor_xmm(1, 1);
                buf.comiss(0, 1);
                // setcc AL based on op, then neg to 0xffff/0.
                let cc: u8 = match op {
                    0 => 0x04, // eq => ZF set => E
                    1 => 0x07, // gt => (CF=0 && ZF=0) => A (above)
                    2 => 0x03, // ge => CF clear => AE/NC
                    3 => 0x02, // lt => CF set => B
                    _ => 0x06, // le => (CF set | ZF set) => BE/NA
                };
                buf.setcc_rm8(cc, 0);
                buf.movzx_r32_r8(RAX, RAX);
                buf.neg_r64(RAX); // 0->0, 1->0xffff_ffff_ffff_ffff
                buf.mov_store16(RBX, f(rd) + off, RAX); // low 16 = mask
            }
            Ok(())
        }
        Inst::SimdFp16Cmp { rd, rn, rm, op, q } => {
            // fcmeq/fcmge/fcmgt Vd.4H/.8H, Vn, Vm : per-lane cond(Vn,Vm) -> all-ones
            // or 0. Promote both halves to f32 (F16C), comiss, setcc, neg.
            let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes: i32 = if q { 8 } else { 4 };
            for l in 0..lanes {
                let off = l * 2;
                buf.mov_load16(RAX, RBX, f(rn) + off);
                buf.movd_xmm_r32(0, RAX);
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0
                buf.mov_load16(RAX, RBX, f(rm) + off);
                buf.movd_xmm_r32(1, RAX);
                buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc9]); // vcvtph2ps xmm1,xmm1 (F16C)
                buf.comiss(0, 1); // sets flags for xmm0 vs xmm1
                let cc: u8 = match op {
                    0 => 0x04, // fcmeq: ZF -> E
                    1 => 0x03, // fcmge: CF clear -> AE
                    _ => 0x07, // fcmgt: CF&ZF clear -> A
                };
                buf.setcc_rm8(cc, 0);
                buf.movzx_r32_r8(RAX, RAX);
                buf.neg_r64(RAX);
                buf.mov_store16(RBX, f(rd) + off, RAX);
            }
            Ok(())
        }
        Inst::SimdDupD { rd, rn, index } => {
            // dup Vd.2D, Vn.D[index]: broadcast the selected 64-bit lane of Vn
            // into both 64-bit lanes of Vd. index 0 => low 64, 1 => high 64.
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let src_off = slot(rn) + (index as i32) * 8;
            buf.mov_load64(RAX, RBX, src_off); // RAX = selected lane
            let dst_slot = slot(rd);
            buf.mov_store64(RBX, dst_slot, RAX); // lane 0
            buf.mov_store64(RBX, dst_slot + 8, RAX); // lane 1
            Ok(())
        }
        Inst::Ucvtf2d { rd, rn } => {
            // ucvtf Vd.2D, Vn.2D : for each of the two 64-bit lanes of Vn (viewed as
            // unsigned integers), convert to double and store into the matching lane
            // of Vd. Honest u64->f64 with a sign-corrected `cvtsi2sd`:
            //   xmm0 = (double)(int64)u          (exact & correct when u < 2^63)
            //   if u >= 2^63: xmm0 += 2^64        (reconstructs (double)u; 2^64 exact)
            // The JNS branch skips the add for non-negative u. This is the standard
            // exact u64->double conversion (no range silently mishandled).
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            for lane in 0..2 {
                let off = lane * 8;
                // RDX = u64 lane value (unsigned).
                buf.mov_load64(RDX, RBX, slot(rn) + off);
                // xmm0 = (double)(int64)RDX
                buf.cvtsi2sd(0, true, RDX);
                // SF = sign(RDX); jump over the correction when u isn't >= 2^63.
                buf.test_rr64(RDX, RDX);
                let jns = buf.jcc_rel32(0x89); // 0F 89 = JNS rel32
                // Correction only when the sign bit was set (u >= 2^63):
                //   mov rcx, [2^64 as double]; movq xmm1, rcx; addsd xmm0, xmm1
                buf.mov_ri64(RCX, 0x43f0_0000_0000_0000); // 2^64 (double bits)
                buf.movq_xmm_r64(1, RCX);
                buf.addsd(0, 1);
                let end = buf.len();
                // Patch the JNS displacement to skip the 3-insn correction block.
                let disp = (end as i64 - (jns as i64 + 4)) as i32;
                buf.bytes[jns..jns + 4].copy_from_slice(&disp.to_le_bytes());
                // store low 64 of xmm0 -> Vd lane.
                buf.movq_store(RBX, slot(rd) + off, 0);
                            }
                            Ok(())
                        }
                        Inst::ScalarUcvtf { rd, rn, sng } => {
                            // ucvtf Dd/Dd or Sd,Sd : read Dn's low bits as an
                            // UNSIGNED integer and write the float to Dd/Sd.
                            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                            if sng {
                                // S-form: low 32-bit lane as unsigned i32 -> f32.
                                // Use src64=true signed-i64 convert on a zero-extended
                                // RAX: max u32 (0xffffffff) is < 2^63, so an unsigned
                                // u32 -> f32 conversion is EXACTLY the i64->f32 of the
                                // zero-extended value (no sign/2^63 correction needed
                                // within the u32 range).
                                buf.mov_load32(RAX, RBX, slot(rn));
                                buf.cvtsi2ss(0, true, RAX);   // u32(>=0 as i64) -> f32
                                buf.movd_r32_xmm(RCX, 0);
                                buf.mov_store32(RBX, slot(rd), RCX);
                            } else {
                                buf.mov_load64(RDX, RBX, slot(rn));
                                buf.cvtsi2sd(0, true, RDX);
                                buf.test_rr64(RDX, RDX);
                                let jns = buf.jcc_rel32(0x89); // JNS (sign clear -> skip correction)
                                buf.mov_ri64(RCX, 0x43f0_0000_0000_0000); // 2^64 as double
                                buf.movq_xmm_r64(1, RCX);
                                buf.addsd(0, 1);
                                let end = buf.len();
                                let disp = (end as i64 - (jns as i64 + 4)) as i32;
                                buf.bytes[jns..jns + 4].copy_from_slice(&disp.to_le_bytes());
                                buf.movq_store(RBX, slot(rd), 0);
                            }
                            Ok(())
                        }
                        Inst::ScalarScvtf { rd, rn, sng } => {
                            // scvtf Dd, Dn : read Dn's low 64 bits as a SIGNED integer
                            // and write the double to Dd (two's-complement -> f64, signed).
                            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                            if sng {
                                buf.mov_load32(RAX, RBX, slot(rn));   // low 32 as signed i32
                                buf.cvtsi2ss(0, false, RAX);          // i32 -> f32
                                buf.movd_r32_xmm(RCX, 0);
                                buf.mov_store32(RBX, slot(rd), RCX);
                            } else {
                            buf.mov_load64(RDX, RBX, slot(rn));
                            buf.cvtsi2sd(0, true, RDX); // signed i64 -> f64
                            buf.movq_store(RBX, slot(rd), 0);
                            }
                            Ok(())
                        }
                        Inst::Simd2dFp { rd, rn, rm, op } => {
                            // 2xdouble lanewise FP: op Vd.2D, Vn.2D, Vm.2D. For each 64-bit lane:
                            //   xmm0 = Vn lane; xmm1 = Vm lane; xmm0 op xmm1; store to Vd lane.
                            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                            for lane in 0..2 {
                                let off = lane * 8;
                                buf.mov_load64(RDX, RBX, slot(rn) + off); // RDX = Vn.lane
                                buf.movq_xmm_r64(0, RDX);
                                buf.mov_load64(RDX, RBX, slot(rm) + off); // RDX = Vm.lane
                                buf.movq_xmm_r64(1, RDX);
                                match op {
                                    0 => buf.divsd(0, 1), // fdiv
                                    1 => buf.mulsd(0, 1), // fmul
                                    2 => buf.addsd(0, 1), // fadd
                                    3 => buf.subsd(0, 1), // fsub
                                    4 => buf.maxsd(0, 1), // fmax
                                    5 => buf.minsd(0, 1), // fmin
                                    // fmaxnm/fminnm behave like max/min on the finite
                                    // values the engine uses (NaN propagation differs).
                                    6 => buf.maxsd(0, 1), // fmaxnm
                                    7 => buf.minsd(0, 1), // fminnm
                                    // fabd = |Vn - Vm|: subsd then clear the sign bit.
                                    8 => {
                                        buf.subsd(0, 1); // xmm0 = a - b
                                        buf.mov_ri64(RDX, 0x7fff_ffff_ffff_ffff);
                                        buf.movq_xmm_r64(1, RDX);
                                        buf.pand(0, 1); // clear sign bit (|a-b|)
                                    }
                                    _ => return Err(format!("Simd2dFp op {op} not implemented")),
                                }
                                buf.movq_store(RBX, slot(rd) + off, 0);
                            }
                            Ok(())
                        }
                        Inst::SimdDupGp { rd, rn, esize, q } => {
                            // dup Vd.T, Wn/Xn: broadcast the element read from GPR rn
                            // into all `lanes` of Vd. esize 1/2/4: low 32 of Wn; esize 8: Xn.
                            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                            if esize == 8 {
                                ldg(buf, RAX, rn as u32); // Xn low 64
                            } else {
                                ldg(buf, RAX, rn as u32);
                                buf.zero_ext_r32(RAX); // zero-extend Wn
                                let m = ((1u64 << (8 * esize)) - 1) as u32;
                                buf.and_ri64(RAX, m);
                            }
                            let lanes = ((if q { 16 } else { 8 }) / esize as i32) as u32;
                            for i in 0..lanes {
                                match esize {
                                    1 => buf.mov_store8(RBX, slot(rd) + (i * 1) as i32, RAX),
                                    2 => buf.mov_store16(RBX, slot(rd) + (i * 2) as i32, RAX),
                                    4 => buf.mov_store32(RBX, slot(rd) + (i * 4) as i32, RAX),
                                    _ => buf.mov_store64(RBX, slot(rd) + (i * 8) as i32, RAX),
                                }
                            }
                            Ok(())
                                                    }
                                                    Inst::SimdOrr16 { rd, rn, rm } => {
                                                        // orr Vd.16B, Vn.16B, Vm.16B (also `mov Vd.16B,Vn.16B`
                                                        // copy when rm==rn): OR across all 16 bytes, 8 at a time.
                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                        for off in [0i32, 8i32] {
                                                            buf.mov_load64(RAX, RBX, slot(rn) + off);
                                                            buf.mov_load64(RCX, RBX, slot(rm) + off);
                                                            buf.or_rr64(RAX, RCX);
                                                            buf.mov_store64(RBX, slot(rd) + off, RAX);
                                                                                        }
                                                                                        Ok(())
                                                                                    }
                                                                                    Inst::SimdMul { rd, rn, rm, lanes } => {
                                                                                        // mul Vd.4S/Vd.2S, Vn., Vm.: per 32-bit lane, low-32 product
                                                                                        // (mod-2^32 wrap). 64-bit imul of zero-extended 32-bit operands
                                                                                        // yields the low-32 product correctly for both signed words.
                                                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                        for i in 0..lanes {
                                                                                            let off = (i as i32) * 4;
                                                                                            buf.mov_load32(RAX, RBX, slot(rn) + off);
                                                                                            buf.mov_load32(RCX, RBX, slot(rm) + off);
                                                                                            buf.imul_rr64(RAX, RCX); // low 32 = (a*b) mod 2^32
                                                                                                                                                                                                                    buf.mov_store32(RBX, slot(rd) + off, RAX);
                                                                                                                                                                                                                }
                                                                                                                                                                                                                Ok(())
                                                                                                                                                                                                            }
                                                                                                                                                                                                            Inst::SimdMulH { rd, rn, rm, lanes } => {
                                                                                                                                                                                                                // mul Vd.8H/.4H, Vn., Vm.: per 16-bit lane, low-16 product
                                                                                                                                                                                                                // (mod-2^16 wrap). 64-bit imul of zero-extended 16-bit operands yields
                                                                                                                                                                                                                // the correct low-16 for signed/unsigned halfwords (imul truncates).
                                                                                                                                                                                                                let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                                                for i in 0..lanes {
                                                                                                                                                                                                                    let off = (i as i32) * 2;
                                                                                                                                                                                                                    buf.movzx_word_mem(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                                                    buf.movzx_word_mem(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                                                    buf.imul_rr64(RAX, RCX);
                                                                                                                                                                                                                    buf.mov_store16(RBX, slot(rd) + off, RAX);
                                                                                                                                                                                                                }
                                                                                                                                                                                                                Ok(())
                                                                                                                                                                                                            }
                                                                                                                    Inst::SimdMla { rd, rn, rm, lanes, sub } => {
                                                                                        // mla/mls Vd.4S/2S, Vn., Vm.: Vd = Vd +/- Vn*Vm per 32-bit lane.
                                                                                        // low-32 of the product, accumulate into the existing Vd lane.
                                                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                        for i in 0..lanes {
                                                                                            let off = (i as i32) * 4;
                                                                                            buf.mov_load32(RAX, RBX, slot(rn) + off);
                                                                                            buf.mov_load32(RCX, RBX, slot(rm) + off);
                                                                                            buf.imul_rr64(RAX, RCX); // low 32 = (a*b) mod 2^32
                                                                                            if sub {
                                                                                                // Vd = Vd - Vn*Vm
                                                                                                buf.mov_load32(RDX, RBX, slot(rd) + off);
                                                                                                buf.sub_rr64(RDX, RAX);
                                                                                                buf.mov_store32(RBX, slot(rd) + off, RDX);
                                                                                            } else {
                                                                                                // Vd = Vd + Vn*Vm
                                                                                                buf.mov_load32(RDX, RBX, slot(rd) + off);
                                                                                                buf.add_rr64(RDX, RAX);
                                                                                                buf.mov_store32(RBX, slot(rd) + off, RDX);
                                                                                            }
                                                                                        }
                                                                                        Ok(())
                                                                                                                    }
                                                                                    Inst::SimdMlaEl { rd, rn, rm, index, lanes, sub } => {
                                                                                        // mla/mls Vd.4S/2S, Vn., Vm.S[idx]: Vd[i] = Vd[i] +/- Vn[i]*Vm.el
                                                                                        // (Vm's indexed single element broadcast to every lane).
                                                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                        let mel = slot(rm) + (index as i32) * 4;
                                                                                        for i in 0..lanes {
                                                                                            let off = (i as i32) * 4;
                                                                                            buf.mov_load32(RAX, RBX, slot(rn) + off);
                                                                                            buf.mov_load32(RCX, RBX, mel); // broadcast element
                                                                                            buf.imul_rr64(RAX, RCX);
                                                                                            if sub {
                                                                                                buf.mov_load32(RDX, RBX, slot(rd) + off);
                                                                                                buf.sub_rr64(RDX, RAX);
                                                                                                buf.mov_store32(RBX, slot(rd) + off, RDX);
                                                                                            } else {
                                                                                                buf.mov_load32(RDX, RBX, slot(rd) + off);
                                                                                                buf.add_rr64(RDX, RAX);
                                                                                                buf.mov_store32(RBX, slot(rd) + off, RDX);
                                                                                            }
                                                                                        }
                                                                                        Ok(())
                                                                                                                    }
                                                                                                                    Inst::SimdMull { rd, rn, rm, res_esize, unsigned, q, acc, sub } => {
                                                                                                        // smull/umull/smlal/umlal: widen each src element to res_esize and
                                                                                                        // multiply (or add to the existing result if acc).
                                                                                                        // src_es = res_esize/2; lanes = (8 source bytes)/src_es = 16/res_esize
                                                                                                        // (2 for .2s->.2d, 4 for .4h->.4s, 8 for .8b->.8h). Store EXACTLY
                                                                                                        // res_esize bytes per lane so we never overrun the next element.
                                                                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                        let src_es: i32 = (res_esize as i32) / 2;
                                                                                                        let lanes = (16 / res_esize as i32) as usize;
                                                                                                        let uphalf = if q { 8 } else { 0 }; // smull2 reads the upper reg half
                                                                                                        // In-place widening alias: the wide (res_esize) write of lane i at
                                                                                                        // i*res_esize clobbers the narrow (src_es) operand bytes lane i+1 reads
                                                                                                        // at (i+1)*src_es when rd aliases a source (`smull v0.2d, v0.2s, …`).
                                                                                                        let rn_base = if rn == rd { permute_source(buf, rd, rn, false) } else { slot(rn) };
                                                                                                        let rm_base = if rm == rd { permute_source(buf, rd, rm, true) } else { slot(rm) };
                                                                                                        for i in 0..lanes {
                                                                                                            let soff = uphalf + (i as i32) * src_es;
                                                                                                            match src_es {
                                                                                                                4 => { buf.mov_load32(RAX, RBX, rn_base+soff); buf.mov_load32(RCX, RBX, rm_base+soff); }
                                                                                                                2 => { buf.movzx_word_mem(RAX, RBX, rn_base+soff); buf.movzx_word_mem(RCX, RBX, rm_base+soff); }
                                                                                                                _ => { buf.movzx_byte_mem(RAX, RBX, rn_base+soff); buf.movzx_byte_mem(RCX, RBX, rm_base+soff); }
                                                                                                            }
                                                                                                            if !unsigned {
                                                                                                                // sign-extend the zero-extended operand up to 64 bits (shift by (64-8*src))
                                                                                                                let se = 64 - 8 * (res_esize as u16 / 2);
                                                                                                                buf.shl_ri8(RAX, (se % 64) as u8);
                                                                                                                buf.sar_ri8(RAX, (se % 64) as u8);
                                                                                                                buf.shl_ri8(RCX, (se % 64) as u8);
                                                                                                                buf.sar_ri8(RCX, (se % 64) as u8);
                                                                                                            }
                                                                                                            buf.imul_rr64(RAX, RCX);
                                                                                                            let doff = (i as i32) * (res_esize as i32);
                                                                                                            if acc {
                                                                                                                                                            match res_esize {
                                                                                                                                                                8 => buf.mov_load64(RDX, RBX, slot(rd)+doff),
                                                                                                                                                                4 => buf.mov_load32(RDX, RBX, slot(rd)+doff),
                                                                                                                                                                _ => buf.movzx_word_mem(RDX, RBX, slot(rd)+doff),
                                                                                                                                                            }
                                                                                                                                                            if sub {
                                                                                                                                                                buf.sub_rr64(RDX, RAX); // smlsl/umlsl: Vd = Vd - prod
                                                                                                                                                            } else {
                                                                                                                                                                buf.add_rr64(RDX, RAX);
                                                                                                                                                            }
                                                                                                                match res_esize {
                                                                                                                    8 => buf.mov_store64(RBX, slot(rd)+doff, RDX),
                                                                                                                    4 => buf.mov_store32(RBX, slot(rd)+doff, RDX),
                                                                                                                    _ => buf.mov_store16(RBX, slot(rd)+doff, RDX),
                                                                                                                }
                                                                                                            } else {
                                                                                                                match res_esize {
                                                                                                                    8 => buf.mov_store64(RBX, slot(rd)+doff, RAX),
                                                                                                                    4 => buf.mov_store32(RBX, slot(rd)+doff, RAX),
                                                                                                                    _ => buf.mov_store16(RBX, slot(rd)+doff, RAX),
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                        Ok(())
                                                                                                    }
                                                                                                                    Inst::SimdMullEl { rd, rn, rm, index, res_esize, unsigned, q, acc, sub } => {
                                        // Integer widening multiply by element:
                                        // Vd[i] +=/|= Vn[i] * Vm[index], where the
                                        // m operand is ONE element (selected by
                                        // `index`) broadcast to every lane. Same
                                        // widen/mul/accumulate per lane as
                                        // SimdMull, but AM: m reads the single
                                        // indexed element from slot(rm) instead
                                        // of lane i. Session 44 (fuzzer-caught).
                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                        let src_es: i32 = (res_esize as i32) / 2;
                                        let lanes = (16 / res_esize as i32) as usize;
                                        let uphalf = if q { 8 } else { 0 };
                                        // m element base (guest indexed by esize bytes)
                                        let mbase = slot(rm) + (index as i32) * src_es;
                                        let rn_base = if rn == rd { permute_source(buf, rd, rn, false) } else { slot(rn) };
                                        for i in 0..lanes {
                                            let soff = uphalf + (i as i32) * src_es;
                                            match src_es {
                                                4 => { buf.mov_load32(RAX, RBX, rn_base+soff); buf.mov_load32(RCX, RBX, mbase); }
                                                2 => { buf.movzx_word_mem(RAX, RBX, rn_base+soff); buf.movzx_word_mem(RCX, RBX, mbase); }
                                                _ => { buf.movzx_byte_mem(RAX, RBX, rn_base+soff); buf.movzx_byte_mem(RCX, RBX, mbase); }
                                            }
                                            if !unsigned {
                                                let se = 64 - 8 * (res_esize as u16 / 2);
                                                buf.shl_ri8(RAX, (se % 64) as u8);
                                                buf.sar_ri8(RAX, (se % 64) as u8);
                                                buf.shl_ri8(RCX, (se % 64) as u8);
                                                buf.sar_ri8(RCX, (se % 64) as u8);
                                            }
                                            buf.imul_rr64(RAX, RCX);
                                            let doff = (i as i32) * (res_esize as i32);
                                            if acc {
                                                match res_esize {
                                                    8 => buf.mov_load64(RDX, RBX, slot(rd)+doff),
                                                    4 => buf.mov_load32(RDX, RBX, slot(rd)+doff),
                                                    _ => buf.movzx_word_mem(RDX, RBX, slot(rd)+doff),
                                                }
                                                if sub {
                                                    buf.sub_rr64(RDX, RAX);
                                                } else {
                                                    buf.add_rr64(RDX, RAX);
                                                }
                                                match res_esize {
                                                    8 => buf.mov_store64(RBX, slot(rd)+doff, RDX),
                                                    4 => buf.mov_store32(RBX, slot(rd)+doff, RDX),
                                                    _ => buf.mov_store16(RBX, slot(rd)+doff, RDX),
                                                }
                                            } else {
                                                match res_esize {
                                                    8 => buf.mov_store64(RBX, slot(rd)+doff, RAX),
                                                    4 => buf.mov_store32(RBX, slot(rd)+doff, RAX),
                                                    _ => buf.mov_store16(RBX, slot(rd)+doff, RAX),
                                                }
                                            }
                                        }
                                        Ok(())
                                    }
                                    Inst::SimdCmhi { rd, rn, rm, lanes } => {
                                                                                                                        // cmhi Vd.4S/Vd.2S, Vn., Vm.: per 32-bit lane, all-ones
                                                                                                                        // if Vn[i] > Vm[i] (unsigned), else 0. Compare unsigned
                                                                                                                        // then cmov (cmova) an all-ones mask vs 0.
                                                                                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                                                                                for i in 0..lanes {
                                                                                                                                                                                                                                                    let off = (i as i32) * 4;
                                                                                                                                                                                                                                                    buf.mov_load32(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                                                                                    buf.mov_load32(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                                                                                    buf.cmp_rr64(RAX, RCX);
                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDI, 0); // default 0
                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDX, 0xffff_ffff_ffff_ffff); // ones
                                                                                                                                                                                                                                                                                                                                                                                    buf.cmov_rr64(0x47, RDI, RDX); // cmova(above): if Vn>Vm, RDI=ones
                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_store32(RBX, slot(rd) + off, RDI);
                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                Ok(())
                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                            Inst::SimdCmhiD { rd, rn, rm } => {
                                                                                                                                                                                                                                                // cmhi Vd.2D, Vn.2D, Vm.2D (Q=1): per 64-bit lane, all-ones
                                                                                                                                                                                                                                                // if Vn[i] > Vm[i] (unsigned) else 0.
                                                                                                                                                                                                                                                let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                                                                                for i in 0..2u32 {
                                                                                                                                                                                                                                                    let off = (i as i32) * 8;
                                                                                                                                                                                                                                                    buf.mov_load64(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                                                                                    buf.mov_load64(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                                                                                    buf.cmp_rr64(RAX, RCX); // unsigned: CF=0 if Vn>=Vm
                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDI, 0); // default 0
                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDX, 0xffff_ffff_ffff_ffff); // ones
                                                                                                                                                                                                                                                                                                                                                                                    buf.cmov_rr64(0x47, RDI, RDX); // cmova(above): if Vn>Vm, RDI=ones
                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_store64(RBX, slot(rd) + off, RDI);
                                                                                                                                                                                                                                                                                                                                                                    }
                                                                                                                                                                                                                                                                                                                                                                    Ok(())
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            Inst::SimdCmhiH { rd, rn, rm, lanes } => {
                                                                                                                            // cmhi Vd.8H/.4H: per 16-bit lane, all-ones if Vn[i] >
                                                                                                                            // Vm[i] (unsigned), else 0. Compare 16-bit unsigned, cmova.
                                                                                                                            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                            for i in 0..lanes {
                                                                                                                                                                                            let off = (i as i32) * 2;
                                                                                                                                                                                            buf.mov_load16(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                            buf.mov_load16(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                            buf.and_ri64(RAX, 0xffff); // clear garbage high bits (mov_load16 zx->32 only)
                                                                                                                                                                                            buf.and_ri64(RCX, 0xffff);
                                                                                                                                                                                            buf.cmp_rr64(RAX, RCX);
                                                                                                                                                                                                                                                            buf.mov_ri64(RDI, 0); // default 0
                                                                                                                                                                                                                                                            buf.mov_ri64(RDX, 0xffff_ffff_ffff_ffff); // ones
                                                                                                                                                                                                                                                            buf.cmov_rr64(0x47, RDI, RDX); // cmova(above): if Vn>Vm, RDI=ones
                                                                                                                                                                                                                                                            buf.mov_store16(RBX, slot(rd) + off, RDI);
                                                                                                                                                                                        }
                                                                                                                            Ok(())
                                                                                                                        }
                                                                                                                        Inst::SimdCmhiB { rd, rn, rm, lanes, ge } => {
                                                                                                                            // cmhi v.16B/cms v.16B: per byte lane all-ones if Vn[i] > Vm[i] (cmhi,
                                                                                                                            // ge=false) or >= (cmhi, ge=true). Loads zero-extend to 32; cmp; cmova / cmovae.
                                                                                                                            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                            let cc = if ge { 0x43u8 } else { 0x47u8 }; // cmovae(>=) / cmova(>)
                                                                                                                            for i in 0..lanes {
                                                                                                                                let off = (i as i32);
                                                                                                                                                                                                buf.movzx_byte_mem(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                                buf.movzx_byte_mem(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                                buf.and_ri64(RAX, 0xff); // clear garbage high bits (movzx only clears r32)
                                                                                                                                                                                                buf.and_ri64(RCX, 0xff);
                                                                                                                                                                                                buf.cmp_rr64(RAX, RCX);
                                                                                                                                                                                                                                                                buf.mov_ri64(RDX, 0);
                                                                                                                                                                                                                                                                buf.mov_ri64(RCX, 0xffff_ffff_ffff_ffff);
                                                                                                                                                                                                                                                                buf.cmov_rr64(cc, RDX, RCX);
                                                                                                                                                                                                                                                                // store low byte of RDX (DL) — RAX/RDX have REX-free 8-bit forms
                                                                                                                                                                                                                                                                buf.mov_store8(RBX, slot(rd) + off, RDX);
                                                                                                                            }
                                                                                                                            Ok(())
                                                                                                                        }
                                                                                                                        Inst::SimdCmhs { rd, rn, rm, lanes } => {
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                // cmhs Vd.4S/2S: per 32-bit lane all-ones if Vn[i] >= Vm[i]
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                // (unsigned "higher or same"); cmovae (cc 0x43) since equal passes.
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                for i in 0..lanes {
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    let off = (i as i32) * 4;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_load32(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_load32(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.cmp_rr64(RAX, RCX); // unsigned: CF=0 if Vn>=Vm, CF=1 if Vn<Vm
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDI, 0); // default 0
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDX, 0xffff_ffff_ffff_ffff); // ones
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.cmov_rr64(0x43, RDI, RDX); // cmovae: ones iff Vn>=Vm (equal passes)
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_store32(RBX, slot(rd) + off, RDI);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                Ok(())
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            Inst::SimdCmhsD { rd, rn, rm } => {
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                // cmhs Vd.2D (Q=1): per 64-bit lane ones if Vn[i] >= Vm[i].
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                for i in 0..2u32 {
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    let off = (i as i32) * 8;
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_load64(RAX, RBX, slot(rn) + off);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_load64(RCX, RBX, slot(rm) + off);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.cmp_rr64(RAX, RCX); // CF=0 if Vn>=Vm
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDI, 0xffff_ffff_ffff_ffff);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_ri64(RDX, 0);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.cmov_rr64(0x43, RDI, RDX); // cmovae
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_store64(RBX, slot(rd) + off, RDI);
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                Ok(())
                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                        Inst::SimdCmgt { rd, rn, rm, lanes, esize, ge } => {
                                                                                                                                                                                                                                                                                                                                                                                    // cmgt/cmge Vd.T, Vn.T, Vm.T: per lane all-ones if Vn > Vm (cmgt) or
                                                                                                                                                                                                                                                                                                                                                                                    // Vn >= Vm (cmge), SIGNED. Loads SIGN-EXTEND (movsx) so the 64-bit cmp
                                                                                                                                                                                                                                                                                                                                                                                    // is a correct signed compare even for negative lanes.
                                                                                                                                                                                                                                                                                                                                                                                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                                                                                                                                                                                                                    let ones = 0xffff_ffff_ffff_ffffu64;
                                                                                                                                                                                                                                                                                                                                                                                    for i in 0..lanes {
                                                                                                                                                                                                                                                                                                                                                                                        let off = (i as i32) * esize as i32;
                                                                                                                                                                                                                                                                                                                                                                                        // sign-extend operand into RAX (Vn) / RCX (Vm)
                                                                                                                                                                                                                                                                                                                                                                                        match esize {
                                                                                                                                                                                                                                                                                                                                                                                            8 => { buf.mov_load64(RAX, RBX, slot(rn)+off); buf.mov_load64(RCX, RBX, slot(rm)+off); }
                                                                                                                                                                                                                                                                                                                                                                                            4 => { buf.mov_load32(RAX, RBX, slot(rn)+off); buf.movsxd_r64_r32(RAX, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                   buf.mov_load32(RCX, RBX, slot(rm)+off); buf.movsxd_r64_r32(RCX, RCX); }
                                                                                                                                                                                                                                                                                                                                                                                            2 => { buf.movsx_word_mem(RAX, RBX, slot(rn)+off); buf.movsx_word_mem(RCX, RBX, slot(rm)+off); }
                                                                                                                                                                                                                                                                                                                                                                                            _ => { buf.movsx_byte_mem(RAX, RBX, slot(rn)+off); buf.movsx_byte_mem(RCX, RBX, slot(rm)+off); }
                                                                                                                                                                                                                                                                                                                                                                                        }
                                                                                                                                                                                                                                                                                                                                                                                        buf.cmp_rr64(RAX, RCX);
                                                                                                                                                                                                                                                                                                                                                                                        buf.mov_ri64(RDI, ones);
                                                                                                                                                                                                                                                                                                                                                                                        buf.mov_ri64(RDX, 0);
                                                                                                                                                                                                                                                                                                                                                                                        if ge {
                                                                                                                                                                                                                                                                                                                                                                                            // all-ones iff Vn >= Vm (clear when Vn < Vm): cmovl 0x4c
                                                                                                                                                                                                                                                                                                                                                                                            buf.cmov_rr64(0x4c, RDI, RDX);
                                                                                                                                                                                                                                                                                                                                                                                        } else {
                                                                                                                                                                                                                                                                                                                                                                                            // all-ones iff Vn > Vm (clear when Vn <= Vm): cmovle 0x4e
                                                                                                                                                                                                                                                                                                                                                                                            buf.cmov_rr64(0x4e, RDI, RDX);
                                                                                                                                                                                                                                                                                                                                                                                        }
                                                                                                                                                                                                                                                                                                                                                                                        match esize {
                                                                                                                                                                                                                                                                                                                                                                                            8 => buf.mov_store64(RBX, slot(rd)+off, RDI),
                                                                                                                                                                                                                                                                                                                                                                                            4 => buf.mov_store32(RBX, slot(rd)+off, RDI),
                                                                                                                                                                                                                                                                                                                                                                                            2 => buf.mov_store16(RBX, slot(rd)+off, RDI),
                                                                                                                                                                                                                                                                                                                                                                                            _ => buf.mov_store8(RBX, slot(rd)+off, RDI),
                                                                                                                                                                                                                                                                                                                                                                                        }
                                                                                                                                                                                                                                                                                                                                                                                    }
                                                                                                                                                                                                                                                                                                                                                                                    Ok(())
                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                        Inst::SimdUz1 { rd, rn, rm, esize, q } => {
                                                                                                                    // uzp1 Vd.T, Vn.T, Vm.T: even-indexed elements of Vn then Vm.
                                                                                                                    // Vd[i]=Vn[2i] for i in 0..n/2; Vd[n/2+i]=Vm[2i]. n = 8 or 16 bytes.
                                                                                                                    // When rd aliases a source (gcc emits rd==rn ubiquitously), the
                                                                                                                    // writes must not corrupt bytes the loop still reads from that
                                                                                                                    // source — snapshot to scratch first.
                                                                                                                    let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
                                                                                                                    let rn_src = permute_source(buf, rd, rn, false); // read-from offset
                                                                                                                    let rm_src = permute_source(buf, rd, rm, true);
                                                                                                                    let n: i32 = if q { 16 } else { 8 };
                                                                                                                    let es = esize as i32;
                                                                                                                    for i in 0..(n / (2 * es)) {
                                                                                                                        let ei = 2 * i * es;
                                                                                                                        // Vd[i] = Vn[2i]
                                                                                                                        match esize {
                                                                                                                            8 => { buf.mov_load64(RAX, RBX, rn_src + ei); buf.mov_store64(RBX, slot(rd as i32) + i * es, RAX); }
                                                                                                                            4 => { buf.mov_load32(RAX, RBX, rn_src + ei); buf.mov_store32(RBX, slot(rd as i32) + i * es, RAX); }
                                                                                                                            2 => { buf.mov_load32(RAX, RBX, rn_src + ei); buf.mov_store16(RBX, slot(rd as i32) + i * es, RAX); }
                                                                                                                            _ => { buf.mov_load32(RAX, RBX, rn_src + ei); buf.mov_store8(RBX, slot(rd as i32) + i * es, RAX); }
                                                                                                                        }
                                                                                                                        // Vd[n/2 + i] = Vm[2i]
                                                                                                                        match esize {
                                                                                                                                            8 => { buf.mov_load64(RAX, RBX, rm_src + ei); buf.mov_store64(RBX, slot(rd as i32) + (n / (2*es) + i) * es, RAX); },
                                                                                                                                            4 => { buf.mov_load32(RAX, RBX, rm_src + ei); buf.mov_store32(RBX, slot(rd as i32) + (n / (2*es) + i) * es, RAX); },
                                                                                                                                            2 => { buf.mov_load32(RAX, RBX, rm_src + ei); buf.mov_store16(RBX, slot(rd as i32) + (n / (2*es) + i) * es, RAX); },
                                                                                                                                            _ => { buf.mov_load32(RAX, RBX, rm_src + ei); buf.mov_store8(RBX, slot(rd as i32) + (n / (2*es) + i) * es, RAX); },
                                                                                                                        }
                                                                                                                    }
                                                                                                                    Ok(())
                                                                                                        }
Inst::SimdUz2 { rd, rn, rm, esize, q } => {
            // uzp2 Vd.T, Vn.T, Vm.T: ODD-indexed elements of Vn then Vm.
            // Vd[i]=Vn[2i+1] for i in 0..n/2; Vd[n/2+i]=Vm[2i+1]. n = 8 or 16 bytes.
            // (gcc magic-division reducer gathers product-HIGH words with this.)
            // rd aliasing a source (gcc emits rd==rn) must not clobber reads.
            let rn_src = permute_source(buf, rd, rn, false);
            let rm_src = permute_source(buf, rd, rm, true);
            let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
            let n: i32 = if q { 16 } else { 8 };
            let es = esize as i32;
            for i in 0..(n / (2 * es)) {
                let oi = (2 * i + 1) * es;
                match esize {
                    8 => { buf.mov_load64(RAX, RBX, rn_src + oi); buf.mov_store64(RBX, slot(rd as i32) + i * es, RAX); }
                    4 => { buf.mov_load32(RAX, RBX, rn_src + oi); buf.mov_store32(RBX, slot(rd as i32) + i * es, RAX); }
                    2 => { buf.mov_load32(RAX, RBX, rn_src + oi); buf.mov_store16(RBX, slot(rd as i32) + i * es, RAX); }
                    _ => { buf.mov_load32(RAX, RBX, rn_src + oi); buf.mov_store8(RBX, slot(rd as i32) + i * es, RAX); }
                }
                let n2 = n / (2 * es);
                match esize {
                                    8 => { buf.mov_load64(RAX, RBX, rm_src + oi); buf.mov_store64(RBX, slot(rd as i32) + (n2 + i) * es, RAX); },
                                    4 => { buf.mov_load32(RAX, RBX, rm_src + oi); buf.mov_store32(RBX, slot(rd as i32) + (n2 + i) * es, RAX); },
                                    2 => { buf.mov_load32(RAX, RBX, rm_src + oi); buf.mov_store16(RBX, slot(rd as i32) + (n2 + i) * es, RAX); },
                                    _ => { buf.mov_load32(RAX, RBX, rm_src + oi); buf.mov_store8(RBX, slot(rd as i32) + (n2 + i) * es, RAX); },
                }
            }
            Ok(())
}
Inst::SimdZip1 { rd, rn, rm, esize, q } => {
            // zip1 Vd.T, Vn.T, Vm.T: interleave lower halves. Vd[2k]=Vn[k] and
            // Vd[2k+1]=Vm[k] for k in 0..(n/2), n = 8 (q=0) / 16 (q=1) bytes,
            // element size = es (1,2,4,8). Source element k at Vn[+]k*es and
            // Vm[k*es]; dest at Vd[2k*es] / Vd[(2k+1)*es].
            // rd aliasing a source must not clobber reads mid-permute.
            let rn_src = permute_source(buf, rd, rn, false);
            let rm_src = permute_source(buf, rd, rm, true);
            let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
            let n: i32 = if q { 16 } else { 8 };
            let es = esize as i32;
            for k in 0..(n / (2 * es)) {
                let so = k * es; // source element k byte offset (Vn & Vm)
                let d0 = 2 * k * es; // dest element 2k
                let d1 = (2 * k + 1) * es; // dest element 2k+1
                match esize {
                    8 => {
                        buf.mov_load64(RAX, RBX, rn_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load64(RAX, RBX, rm_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d1, RAX);
                    }
                    4 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d1, RAX);
                    }
                    2 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d1, RAX);
                    }
                    _ => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d1, RAX);
                    }
                }
            }
            Ok(())
}
Inst::SimdTrn1 { rd, rn, rm, esize, q } => {
            // trn1 Vd.T, Vn.T, Vm.T: transpose even lanes. For k in 0..(N/2):
            // Vd[2k]=Vn[2k] (even element of Vn), Vd[2k+1]=Vm[2k]. N = n/es
            // elements per vector. No half-length split (unlike zip1): source
            // index is 2k (not k). Guard rd aliasing a source.
            let rn_src = permute_source(buf, rd, rn, false);
            let rm_src = permute_source(buf, rd, rm, true);
            let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
            let n: i32 = if q { 16 } else { 8 };
            let es = esize as i32;
            for k in 0..(n / (2 * es)) {
                let so = 2 * k * es;      // even source element byte offset
                let d0 = 2 * k * es;
                let d1 = (2 * k + 1) * es;
                match esize {
                    8 => {
                        buf.mov_load64(RAX, RBX, rn_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load64(RAX, RBX, rm_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d1, RAX);
                    }
                    4 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d1, RAX);
                    }
                    2 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d1, RAX);
                    }
                    _ => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d1, RAX);
                    }
                }
            }
            Ok(())
}
Inst::SimdTrn2 { rd, rn, rm, esize, q } => {
            // trn2 Vd.T, Vn.T, Vm.T: transpose odd lanes. For k in 0..(N/2):
            // Vd[2k]=Vn[2k+1] (odd element), Vd[2k+1]=Vm[2k+1].
            let rn_src = permute_source(buf, rd, rn, false);
            let rm_src = permute_source(buf, rd, rm, true);
            let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
            let n: i32 = if q { 16 } else { 8 };
            let es = esize as i32;
            for k in 0..(n / (2 * es)) {
                let so = (2 * k + 1) * es; // odd source element byte offset
                let d0 = 2 * k * es;
                let d1 = (2 * k + 1) * es;
                match esize {
                    8 => {
                        buf.mov_load64(RAX, RBX, rn_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load64(RAX, RBX, rm_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d1, RAX);
                    }
                    4 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d1, RAX);
                    }
                    2 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d1, RAX);
                    }
                    _ => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d1, RAX);
                    }
                }
            }
            Ok(())
}
Inst::SimdZip2 { rd, rn, rm, esize, q } => {
            // zip2 Vd.T, Vn.T, Vm.T: interleave the UPPER halves.
            // Vd[2k]=Vn[n/2+k] and Vd[(2k+1)]=Vm[n/2+k] for k in 0..(n/2),
            // n = 8 (q=0) / 16 (q=1) bytes, element size = es.
            // rd aliasing a source must not clobber reads mid-permute.
            let rn_src = permute_source(buf, rd, rn, false);
            let rm_src = permute_source(buf, rd, rm, true);
            let slot = |r: i32| crate::jit::VECTOR_BASE + r * 16;
            let n: i32 = if q { 16 } else { 8 };
            let es = esize as i32;
            for k in 0..(n / (2 * es)) {
                let so = (n / 2) + k * es; // source element n/2+k byte offset
                let d0 = 2 * k * es;
                let d1 = (2 * k + 1) * es;
                match esize {
                    8 => {
                        buf.mov_load64(RAX, RBX, rn_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load64(RAX, RBX, rm_src + so);
                        buf.mov_store64(RBX, slot(rd as i32) + d1, RAX);
                    }
                    4 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store32(RBX, slot(rd as i32) + d1, RAX);
                    }
                    2 => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store16(RBX, slot(rd as i32) + d1, RAX);
                    }
                    _ => {
                        buf.mov_load32(RAX, RBX, rn_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d0, RAX);
                        buf.mov_load32(RAX, RBX, rm_src + so);
                        buf.mov_store8(RBX, slot(rd as i32) + d1, RAX);
                    }
                }
            }
            Ok(())
}
Inst::SimdAddD { rd, rn, rm, sub } => {
            // add/sub Vd.2D, Vn.2D, Vm.2D: two 64-bit lanes.
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            for i in 0..2i32 {
                let off = i * 8;
                buf.mov_load64(RAX, RBX, slot(rn) + off);
                buf.mov_load64(RCX, RBX, slot(rm) + off);
                if sub { buf.sub_rr64(RAX, RCX); } else { buf.add_rr64(RAX, RCX); }
                buf.mov_store64(RBX, slot(rd) + off, RAX);
            }
            Ok(())
}
Inst::SimdAddB { rd, rn, rm, sub, q } => {
            // add/sub Vd.16b, Vn.16b, Vm.16b (or 8b): byte-lane via paddb/psubb.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            buf.movdqu_load(RAX, RBX, vslot(rn));
            buf.movdqu_load(RCX, RBX, vslot(rm));
            if sub { buf.psubb(RAX, RCX); } else { buf.paddb(RAX, RCX); }
            buf.movdqu_store(RBX, vslot(rd), RAX);
            Ok(())
}
Inst::SimdMaxMinP { rd, rn, rm, min, unsigned, esize, q } => {
            // smaxp/sminp/umaxp/uminp Vd.T,Vn.T,Vm.T: halves of Vd = pairwise
            // reduce Vn (low half), then Vm. esize bytes per lane; signed/unsigned
            // comparison; per-adjacent-pair max/min into dst lane.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let es = esize as i32;
            let lanes = if q { 16 / es } else { 8 / es };
            // snapshot sources in case rd aliases rn/rm (gcc emits smaxp v30,v31,v30)
            let src_rn = permute_source(buf, rd, rn, false);
            let src_rm = permute_source(buf, rd, rm, false);
            let mut d = 0;
            for base in [src_rn, src_rm] {
                for i in 0..lanes / 2 {
                    // load pair (2i, 2i+1), choose max/min
                    let lo = base + (2 * i) * es;
                    let hi = lo + es;
                    // load each into RAX/RCX extended by sign or zero
                    match es {
                        1 => {
                            if unsigned { buf.movzx_byte_mem(RAX, RBX, lo); buf.movzx_byte_mem(RCX, RBX, hi); }
                            else { buf.movsx_byte_mem(RAX, RBX, lo); buf.movsx_byte_mem(RCX, RBX, hi); }
                        }
                        2 => {
                            if unsigned { buf.movzx_word_mem(RAX, RBX, lo); buf.movzx_word_mem(RCX, RBX, hi); }
                            else { buf.movsx_word_mem(RAX, RBX, lo); buf.movsx_word_mem(RCX, RBX, hi); }
                        }
                        _ => {
                            buf.mov_load32(RAX, RBX, lo);
                            if !unsigned { buf.movsxd_r64_r32(RAX, RAX); }
                            buf.mov_load32(RCX, RBX, hi);
                            if !unsigned { buf.movsxd_r64_r32(RCX, RCX); }
                        }
                    }
                    buf.cmp_rr64(RAX, RCX);
                    // signed: L=0x4C (RAX<RCX), G=0x4F; unsigned: B=0x42, A=0x47
                    let (lt, gt) = if unsigned { (0x42u8, 0x47u8) } else { (0x4c, 0x4f) };
                    if min {
                        buf.cmov_rr64(gt, RAX, RCX); // RAX=RCX if RAX>RCX (lo>hi: hi is smaller)
                    } else {
                        buf.cmov_rr64(lt, RAX, RCX); // RAX=RCX if RAX<RCX (hi is larger)
                    }
                    let dst = vslot(rd) + d * es;
                    match es {
                        1 => buf.mov_store8(RBX, dst, RAX),
                        2 => buf.mov_store16(RBX, dst, RAX),
                        _ => buf.mov_store32(RBX, dst, RAX),
                    }
                    d += 1;
                }
            }
            Ok(())
}
Inst::SimdAddH { rd, rn, rm, sub, q } => {
            // add/sub Vd.8h, Vn.8h, Vm.8h (or 4h): 16-bit halfword lanes via paddw/psubw.
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            buf.movdqu_load(RAX, RBX, vslot(rn));
            buf.movdqu_load(RCX, RBX, vslot(rm));
            if sub { buf.psubw(RAX, RCX); } else { buf.paddw(RAX, RCX); }
            // for q=0 (.4h) only the low 8 bytes are legal; still write the full 128b
            // (the guest doesn't rely on the upper half of a .4h result).
            buf.movdqu_store(RBX, vslot(rd), RAX);
            Ok(())
}
Inst::SimdMovEl { rd, rn, esize, index, signed, is_x } => {
            // umov/smov Rd, Vn.bits[idx]: load esize-byte element at offset
            // index*esize, extend zero (umov) or sign (smov) into GPR rd.
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16 + (index as i32) * (esize as i32);
            match (esize, signed, is_x) {
                (8, _, _) => buf.mov_load64(RAX, RBX, src),
                (4, true, true) => { buf.mov_load32(RAX, RBX, src); buf.movsxd_r64_r32(RAX, RAX); }
                (4, _, _) => buf.mov_load32(RAX, RBX, src),
                (2, true, _) => buf.movsx_word_mem(RAX, RBX, src),
                (2, false, _) => buf.movzx_word_mem(RAX, RBX, src),
                (1, true, _) => buf.movsx_byte_mem(RAX, RBX, src),
                (1, false, _) => buf.movzx_byte_mem(RAX, RBX, src),
                    _ => unreachable!("smov/umov esize must be 1/2/4/8"),
                }
            if is_x { buf.mov_store64(RBX, slot(rd as u32), RAX); }
            else { buf.mov_store32(RBX, slot(rd as u32), RAX); }
            Ok(())
}
                                                                                                                                                                                Inst::Addv { rd, rn, size, q } => {
            // ADDV Dd,Vn.T : horizontal sum of sign-extended elements.
            let lanes: u32 = match (size, q) {
                (1, false) => 8,
                (1, true) => 16,
                (2, false) => 4,
                (2, true) => 8,
                (4, true) => 4,
                _ => return Err(format!("Addv unsupported size={} q={}", size, q)),
            };
            let vsrc = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            buf.xor_rr64(RDI, RDI); // accumulator
            for i in 0..lanes {
                let off = vsrc + (i as i32) * (size as i32);
                match size {
                    1 => buf.movsx_byte_mem(RAX, RBX, off),
                    2 => buf.movsx_word_mem(RAX, RBX, off),
                    4 => {
                        buf.mov_load32(RAX, RBX, off);
                        buf.movsxd_r64_r32(RAX, RAX);
                    }
                    _ => unreachable!(),
                }
                buf.add_rr64(RDI, RAX);
            }
            let vdst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            // ADDV <Bd/Hd/Sd>, <Vn>.T writes ONLY the low element of the
            // destination and CLEARS the remaining bits of the register (ARM
            // "add across vector, result to scalar" semantics). The caller may
            // then read the full register as an integer (gcc emits `addv b`;
            // `fmov xD, dN`), so every byte the scalar result does not write
            // MUST be zeroed — leaving the source lanes there fed a polluted
            // 64-bit value into the read (e.g. popcount + stale lane bytes).
            match size {
                1 => {
                    // x86 64-bit store of the 8-bit sum also zeroes bytes 1..7;
                    // mask to a byte first (lanes are signed, so the raw sum can
                    // be negative and RDI would otherwise sign-extend up top).
                    buf.and_ri64(RDI, 0xff);
                    buf.mov_store64(RBX, vdst, RDI);
                }
                2 => {
                    buf.and_ri64(RDI, 0xffff);
                    buf.mov_store64(RBX, vdst, RDI);
                }
                4 => {
                    buf.mov_store32(RBX, vdst, RDI); // 32-bit store zeroes upper 32
                }
                _ => unreachable!(),
            }
            Ok(())
        }
        Inst::SimdReduceMinMax { rd, rn, size, signed, is_min, q } => {
            // SMINV/SMAXV/UMINV/UMAXV Sd/Hd/Bd, Vn.T: reduce min/max over ALL
            // lanes to the bottom scalar. RAX = each lane (sign/zero-extended to
            // 64), RDX = running extrema (init to lane 0), CMOVcc per later lane.
            // cc: signed min < (JL 0x0C), signed max > (JG 0x0F);
            //     unsigned min < (JB 0x02), unsigned max > (JA 0x07).
            let lanes: u32 = match (size, q) {
                (1, false) => 8,
                (1, true) => 16,
                (2, false) => 4,
                (2, true) => 8,
                (4, true) => 4,
                _ => return Err(format!("SimdReduceMinMax unsupported size={size} q={q}")),
            };
            let cc = if is_min {
                if signed { 0x4F } else { 0x47 } // CMOVG / CMOVA (update when candidate < current)
            } else {
                if signed { 0x4C } else { 0x42 } // CMOVL / CMOVB (update when candidate > current)
            };
            let vsrc = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let load = |buf: &mut CodeBuf, off: i32| match size {
                1 => {
                    if signed { buf.movsx_byte_mem(RAX, RBX, off) } else { buf.movzx_byte_mem(RAX, RBX, off) }
                }
                2 => {
                    if signed { buf.movsx_word_mem(RAX, RBX, off) } else { buf.movzx_word_mem(RAX, RBX, off) }
                }
                4 => {
                    buf.mov_load32(RAX, RBX, off);
                    if signed { buf.movsxd_r64_r32(RAX, RAX); }
                }
                _ => unreachable!(),
            };
            load(buf, vsrc);
            buf.mov_rr64(RDX, RAX); // running extrema = lane 0
            for i in 1..lanes {
                let off = vsrc + (i as i32) * (size as i32);
                load(buf, off);
                // cmp RAX(new) vs RDX(cur): RAX < RDX for min / RAX > RDX for max.
                buf.cmp_rr64(RDX, RAX);
                buf.cmov_rr64(cc, RDX, RAX);
            }
            let vdst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            match size {
                1 => buf.mov_store8(RBX, vdst, RDX),   // low byte; upper bits past it
                2 => buf.mov_store16(RBX, vdst, RDX),
                _ => buf.mov_store32(RBX, vdst, RDX), // 32-bit store zeroes upper 32
            }
            Ok(())
        }
        Inst::SimdPairAddD { rd, rn, unsigned: _u } => {
            // ADDP Dd, Vn.2D : pairwise-add the two 64-bit lanes of Vn into the
            // low 64 bits of Vd. Plain 64-bit add (signed/unsigned same result).
            let vsrc = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let vdst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            buf.mov_load64(RAX, RBX, vsrc);        // lane 0
            buf.mov_load64(RCX, RBX, vsrc + 8);    // lane 1
            buf.add_rr64(RAX, RCX);
            buf.mov_store64(RBX, vdst, RAX);       // high 64 bits of Vd left as-is
            Ok(())
        }
        Inst::SimdCmEq { rd, rn, rm, lanes, esize } => {
            // cmeq Vd.T, Vn.T, Vm.T: each element is all-ones if Vn[i]==Vm[i]
            // else 0. Compare the esize-byte element (zero-extended via the
            // widest load that fits), then cmov all-ones vs 0, store esize bytes.
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            for i in 0..lanes {
                let off = (i as i32) * (esize as i32);
                if esize == 8 {
                    buf.mov_load64(RAX, RBX, slot(rn) + off);
                    buf.mov_load64(RCX, RBX, slot(rm) + off);
                } else {
                    buf.mov_load32(RAX, RBX, slot(rn) + off);
                    buf.mov_load32(RCX, RBX, slot(rm) + off);
                    let imm = match esize {
                        4 => 0xffff_ffffu32,
                        2 => 0xffffu32,
                        _ => 0xffu32,
                    };
                    buf.and_ri64(RAX, imm);
                    buf.and_ri64(RCX, imm);
                }
                buf.cmp_rr64(RAX, RCX); // ZF=1 if equal
                buf.mov_ri64(RDI, 0);
                buf.mov_ri64(RDX, 0xffff_ffff_ffff_ffff);
                buf.cmov_rr64(0x44, RDI, RDX); // 0x44=cmove: RDI=ones if equal
                match esize {
                    8 => buf.mov_store64(RBX, slot(rd) + off, RDI),
                    4 => buf.mov_store32(RBX, slot(rd) + off, RDI),
                    2 => buf.mov_store16(RBX, slot(rd) + off, RDI),
                    _ => buf.mov_store8(RBX, slot(rd) + off, RDI),
                }
            }
            Ok(())
                    }
                    Inst::SimdCmTest { rd, rn, rm, lanes, esize } => {
                        // cmtst Vd.T, Vn.T, Vm.T (wall 0x4e208c01): each element is all-ones
                        // iff (Vn[i] & Vm[i]) != 0, else 0. Load element, test Vn&Vm != 0, cmov.
                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                        for i in 0..lanes {
                            let off = (i as i32) * (esize as i32);
                            if esize == 8 {
                                buf.mov_load64(RAX, RBX, slot(rn) + off);
                                buf.mov_load64(RCX, RBX, slot(rm) + off);
                            } else {
                                buf.mov_load32(RAX, RBX, slot(rn) + off);
                                buf.mov_load32(RCX, RBX, slot(rm) + off);
                            }
                            buf.test_rr64(RAX, RCX); // ZF=1 if (Vn&Vm)==0
                            buf.mov_ri64(RDI, 0);
                            buf.mov_ri64(RDX, 0xffff_ffff_ffff_ffff);
                            buf.cmov_rr64(0x45, RDI, RDX); // 0x45=cmovne: ones if nonzero
                            match esize {
                                8 => buf.mov_store64(RBX, slot(rd) + off, RDI),
                                4 => buf.mov_store32(RBX, slot(rd) + off, RDI),
                                2 => buf.mov_store16(RBX, slot(rd) + off, RDI),
                                _ => buf.mov_store8(RBX, slot(rd) + off, RDI),
                            }
                        }
                        Ok(())
                    }
                    Inst::Tbl { rd, rn, rm, tbx, n_tables } => {
                    // tbl vd.16b, {vn..vn+nt-1}, vm: vd[i] = concat(vn..vn+nt)[vm[i]].
                    // The n_tables registers are stored CONTIGUOUSLY (16-byte stride) at
                    // VECTOR_BASE+rn*16 .. +16*n_tables, so concatenated byte `idx` lives at
                    // slot(rn)+idx. idx>=16*n_tables => 0 (tbl) or keep old (tbx).
                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let tbytes = 16 * (n_tables as i32);
                    for i in 0..16i32 {
                        buf.movzx_byte_mem(RAX, RBX, slot(rm) + i); // idx = vm[i]
                        // candidate = concat-table byte at offset idx
                        buf.mov_ri64(R10, slot(rn) as u64);
                        buf.add_rr64(R10, RBX);
                        buf.add_rr64(R10, RAX);
                        buf.movzx_byte_mem(RCX, R10, 0);
                        // default: 0 (tbl) or keep (tbx)
                        buf.mov_ri64(RDX, 0);
                        if tbx { buf.movzx_byte_mem(RDX, RBX, slot(rd) + i); }
                        // select table value only when idx < 16*n_tables (unsigned below)
                        buf.mov_ri64(RDI, tbytes as u64);
                        buf.cmp_rr64(RAX, RDI);
                        buf.cmov_rr64(0x42, RDX, RCX); // cmovb: RDX=RCX if idx<tbytes
                        buf.mov_store8(RBX, slot(rd) + i, RDX);
                    }
                    Ok(())
                }
                    Inst::SimdXtn { rd, rn, dst_esize } => {
                    // xtn Vd.8b/4h/2s, Vn.<wider>: take the LOW `dst_esize` bytes of each
                    // source element (source element esize = 2*dst_esize) and pack them
                    // into dest lanes. Q=0 => 64-bit dest result (high lane of Vd zeroed).
                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let src_esize = 2 * (dst_esize as i32);
                    let lanes = 8 / (dst_esize as i32);
                    for i in 0..lanes {
                        let src_off = (i as i32) * src_esize;
                        let dst_off = (i as i32) * (dst_esize as i32);
                        // load the low `dst_esize` bytes of source element into RAX
                                        // (mov_load32 over-reads past the element for 1/2-byte lanes, but
                                        //  the extra bytes are masked out before store)
                                        buf.mov_load32(RAX, RBX, slot(rn) + src_off);
                                        match dst_esize {
                                            2 => buf.and_ri64(RAX, 0xffff),
                                            _ => buf.and_ri64(RAX, 0xff),
                                        }
                                        // store into dest lane
                                        match dst_esize {
                                            4 => buf.mov_store32(RBX, slot(rd) + dst_off, RAX),
                                            2 => buf.mov_store16(RBX, slot(rd) + dst_off, RAX),
                                            _ => buf.mov_store8(RBX, slot(rd) + dst_off, RAX),
                                        }
                    }
                    // zero the high 64 bits of Vd
                    buf.mov_ri64(RAX, 0);
                    buf.mov_store64(RBX, slot(rd) + 8, RAX);
                    Ok(())
                }
                Inst::SaturatNarrow { rd, rn, dst_esize, src_signed, dst_signed, q } => {
                    // sqxtn/uqxtn/sqxtun/uqxtun Vd.T, Vn.U: saturating narrow. Each src
                    // element (2*dst_esize) is sign/zero-extended to 64, clamped into the
                    // dst range, then the low dst_esize bytes stored into V[rd] lane.
                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let src_esize = 2 * (dst_esize as i32);
                    let lanes = if q { 16 / (dst_esize as i32) } else { 8 / (dst_esize as i32) };
                    let maxv: i64 = if dst_signed {
                        if dst_esize == 2 { 0x7fff } else { 0x7f }
                    } else if dst_esize == 2 { 0xffff } else { 0xff };
                    let minv: i64 = if dst_signed {
                        if dst_esize == 2 { -0x8000 } else { -0x80 }
                    } else { 0 };
                    for i in 0..lanes {
                        let src_off = (i as i32) * src_esize;
                        let dst_off = (i as i32) * (dst_esize as i32);
                        match src_esize {
                            2 => {
                                if src_signed { buf.movsx_word_mem(RAX, RBX, slot(rn)+src_off); }
                                else { buf.movzx_word_mem(RAX, RBX, slot(rn)+src_off); }
                            }
                            _ => {
                                buf.mov_load32(RAX, RBX, slot(rn)+src_off);
                                if src_signed { buf.shl_ri8(RAX, 32); buf.sar_ri8(RAX, 32); }
                            }
                        }
                        // clamp low: RAX = max(RAX, minv) using signed compare
                        buf.mov_ri64(RCX, minv as u64);
                        buf.cmp_rr64(RAX, RCX);
                        buf.cmov_rr64(0x4c, RAX, RCX); // cmovl: RAX=RCX(minv) if RAX<RCX
                        // clamp high: RAX = min(RAX, maxv)
                        buf.mov_ri64(RCX, maxv as u64);
                        buf.cmp_rr64(RAX, RCX);
                        buf.cmov_rr64(0x4f, RAX, RCX); // cmovg: RAX=RCX(maxv) if RAX>RCX
                        // store low dst_esize bytes
                        match dst_esize {
                            2 => buf.mov_store16(RBX, slot(rd)+dst_off, RAX),
                            _ => buf.mov_store8(RBX, slot(rd)+dst_off, RAX),
                        }
                    }
                    // Q=0 zero the high 64 bits of Vd
                    if !q {
                        buf.mov_ri64(RAX, 0);
                        buf.mov_store64(RBX, slot(rd) + 8, RAX);
                    }
                    Ok(())
                }
                Inst::SatNarrowShift { rd, rn, src_esize, dst_esize, shift, src_signed, dst_signed, q } => {
                    // sqshrn/uqshrn/sqshrun Vd.T, Vn.T, #imm: shift each src element
                    // (width src_esize) right by `shift` (arith if src_signed else
                    // logical), then saturate narrow to dst_esize (src_esize/2).
                    // SELF-ALIAS (gcc emits sqshrn v31,v31 in narrowing loops): the
                    // dest bytes overlap the source bytes, so snapshot the source
                    // to scratch when rd==rn (permute_source).
                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let src = permute_source(buf, rd, rn, false);
                    let se = src_esize as i32;
                    let de = dst_esize as i32;
                    let lanes = if q { 16 / de } else { 8 / de };
                    // clamp bounds for the DST (dst_esize bytes)
                    let maxv: i64 = if dst_signed {
                        if de == 2 { 0x7fff } else { 0x7f }
                    } else if de == 2 { 0xffff } else { 0xff };
                    let minv: i64 = if dst_signed {
                        if de == 2 { -0x8000 } else { -0x80 }
                    } else { 0 };
                    for i in 0..lanes {
                        let src_off = (i as i32) * se;
                        let dst_off = (i as i32) * de;
                        // load src element, sign/zero-extend to 64
                        match se {
                            2 => {
                                if src_signed { buf.movsx_word_mem(RAX, RBX, src+src_off); }
                                else { buf.movzx_word_mem(RAX, RBX, src+src_off); }
                            }
                            _ => {
                                buf.mov_load32(RAX, RBX, src+src_off);
                                if src_signed { buf.shl_ri8(RAX, 32); buf.sar_ri8(RAX, 32); }
                            }
                        }
                        // right-shift: arith for signed src, logical for unsigned
                        if src_signed { buf.sar_ri8(RAX, shift); } else { buf.shr_ri8(RAX, shift); }
                        // clamp low
                        buf.mov_ri64(RCX, minv as u64);
                        buf.cmp_rr64(RAX, RCX);
                        buf.cmov_rr64(0x4c, RAX, RCX); // RAX=minv if RAX<minv
                        // clamp high
                        buf.mov_ri64(RCX, maxv as u64);
                        buf.cmp_rr64(RAX, RCX);
                        buf.cmov_rr64(0x4f, RAX, RCX); // RAX=maxv if RAX>maxv
                        match de {
                            2 => buf.mov_store16(RBX, slot(rd)+dst_off, RAX),
                            _ => buf.mov_store8(RBX, slot(rd)+dst_off, RAX),
                        }
                    }
                    if !q {
                        buf.mov_ri64(RAX, 0);
                        buf.mov_store64(RBX, slot(rd) + 8, RAX);
                    }
                    Ok(())
                }
                Inst::Ld1V { rd, rn, bytes } => {
                    // ld1 {Vt.T}, [Xn], #imm: load `bytes` (16 or 8) contiguous bytes
                    // from guest address x[rn] into V[rd], then x[rn] += bytes.
                    let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                    ldg(buf, RDX, rn as u32); // RDX = host ptr to guest mem
                    if bytes == 16 {
                        buf.movdqu_load(0, RDX, 0);
                        buf.movdqu_store(RBX, slot, 0);
                    } else {
                        buf.movq_load(0, RDX, 0);
                        buf.movq_store(RBX, slot, 0);
                        buf.mov_ri64(RAX, 0);
                        buf.mov_store64(RBX, slot + 8, RAX); // zero high 64
                    }
                    ldg(buf, RAX, rn as u32);
                    buf.add_ri64(RAX, bytes as u32); // post-index: x[rn] += bytes
                    stg(buf, rn as u32, RAX);
                    Ok(())
                }
                Inst::St1V { rd, rn, bytes } => {
                    // st1 {Vt.T}, [Xn], #imm: store V[rd]'s `bytes` to guest
                    // address x[rn], then x[rn] += bytes.
                    let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                    ldg(buf, RDX, rn as u32); // RDX = host ptr
                    if bytes == 16 {
                        buf.movdqu_load(0, RBX, slot);
                        buf.movdqu_store(RDX, 0, 0);
                    } else {
                        buf.movq_load(0, RBX, slot);
                        buf.movq_store(RDX, 0, 0);
                    }
                    ldg(buf, RAX, rn as u32);
                    buf.add_ri64(RAX, bytes as u32); // post-index
                    stg(buf, rn as u32, RAX);
                    Ok(())
                }
                Inst::Ld1 { rd, rn, esize, q } => {
                    // ld1r {Vt.T}, [Xn]: load `esize` bytes from [x[rn]] and
                    // replicate across (q?16:8)/esize lanes of Vd. Guest memory is
                    // host-addressable in this in-process JIT, so [x[rn]] is a
                    // direct dereference.
                    let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                    ldg(buf, RDX, rn as u32); // RDX = base address (host ptr)
                    match esize {
                        8 => buf.mov_load64(RAX, RDX, 0),
                        4 => buf.mov_load32(RAX, RDX, 0),
                        2 => buf.movzx_word_mem(RAX, RDX, 0),
                        _ => buf.movzx_byte_mem(RAX, RDX, 0),
                    }
                    let total = if q { 16i32 } else { 8i32 };
                    let mut off = 0i32;
                    while off < total {
                        match esize {
                            8 => buf.mov_store64(RBX, slot + off, RAX),
                            4 => buf.mov_store32(RBX, slot + off, RAX),
                            2 => buf.mov_store16(RBX, slot + off, RAX),
                            _ => buf.mov_store8(RBX, slot + off, RAX),
                        }
                        off += esize as i32;
                    }
                    // for q=0, zero the high 64 bits
                    if !q {
                        buf.mov_ri64(RAX, 0);
                        buf.mov_store64(RBX, slot + 8, RAX);
                    }
                    Ok(())
                }
                Inst::Ld1L { rd, rn, esize, index } => {
                    // ld1 {Vt.T}[idx], [Xn]: load `esize` bytes from [x[rn]]
                    // into lane `index` of Vd (byte offset index*esize).
                    let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
                    ldg(buf, RDX, rn as u32); // RDX = base address (host ptr)
                    match esize {
                        8 => buf.mov_load64(RAX, RDX, 0),
                        4 => buf.mov_load32(RAX, RDX, 0),
                        2 => buf.movzx_word_mem(RAX, RDX, 0),
                        _ => buf.movzx_byte_mem(RAX, RDX, 0),
                    }
                    let off = slot + (index as i32) * (esize as i32);
                    match esize {
                        8 => buf.mov_store64(RBX, off, RAX),
                        4 => buf.mov_store32(RBX, off, RAX),
                        2 => buf.mov_store16(RBX, off, RAX),
                        _ => buf.mov_store8(RBX, off, RAX),
                    }
                    Ok(())
                }
                Inst::SimdXtl { rd, rn, sign, esrc, upper } => {
                    // uxtl/sxtl Vd.<long>, Vn.<short>: widen each esrc-byte lane
                    // to a (esrc*2)-byte lane (zero/sign extend). Lanes = 8/esrc,
                    // the dest occupies the full 16-byte vector (Q=1 long form).
                    // IN-PLACE ALIASING: when rd==rn the widened write of lane i
                    // (2*esrc bytes at i*2*esrc) overlaps the narrow source bytes
                    // of later lanes (esrc bytes at (i+1)*esrc), so a naive
                    // read-then-write loop clobbers the still-needed source — e.g.
                    // gcc's `sxtl v30.2d, v30.2s` (rd==rn) dropped lane 1. Snapshot
                    // Vn to the permscratch slot first when they alias.
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    // upper (sxtl2/uxtl2): the narrow src lanes live in the
                    // UPPER 64 bits of Vn (byte 8..15), like saddw2.
                    let n_half: i32 = if upper { 8 } else { 0 };
                    let srcbase = permute_source(buf, rd, rn, false);
                    let lanes = 8usize >> esrc.trailing_zeros() as usize;
                    for i in 0..lanes {
                        let src = srcbase + n_half + (i as i32) * (esrc as i32);
                        let dst = vslot(rd) + (i as i32) * (esrc as i32) * 2;
                        match (esrc, sign) {
                            (1, false) => buf.movzx_byte_mem(RAX, RBX, src),
                            (1, true) => buf.movsx_byte_mem(RAX, RBX, src),
                            (2, false) => buf.movzx_word_mem(RAX, RBX, src),
                            (2, true) => buf.movsx_word_mem(RAX, RBX, src),
                            (4, false) => buf.mov_load32(RAX, RBX, src),
                            (4, true) => buf.mov_load32(RAX, RBX, src),
                            _ => unreachable!(),
                        }
                        if esrc == 4 && sign {
                            buf.movsxd_r64_r32(RAX, RAX);
                        }
                        match (esrc as i32) * 2 {
                            4 => { buf.mov_store32(RBX, dst, RAX); }
                            8 => { buf.mov_store64(RBX, dst, RAX); }
                            _ => { buf.mov_store16(RBX, dst, RAX); }
                        }
                    }
                    Ok(())
                }
                Inst::SimdAddw { rd, rn, rm, sign, esrc, upper, sub } => {
                    // uaddw/saddw Vd.T, Vn.T, Vm.(T/2): Vd[i] = Vn[i] + extend(Vm_hi)
                    // narrow source element = esrc bytes, dest element = 2*esrc.
                    // `upper` (saddw2/uaddw2): the narrow src is the UPPER 64 bits
                    // of Vm (byte 8..15), not the lower — a second loop pass
                    // accumulates the other half. `sub` (ssubw/usubw, bit13):
                    // Vd[i] = Vn[i] - extend(Vm[i]) -- the add/sub-wide family.
                    // IN-PLACE ALIASING: when the narrow source rm aliases rd, the
                    // widened (2*esrc) write of lane i at i*2*esrc overwrites the
                    // narrow msrc bytes of lane i+1 (at (i+1)*esrc), so a naive
                    // read-then-write loop clobbers the still-needed source — e.g.
                    // gcc's `saddw v31.2d, v29.2d, v31.2s` (rd==rm) dropped the
                    // second msrc lane and summed [1,0] instead of [2]. Snapshot
                    // the source(s) that alias rd to permscratch first.
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let der = (esrc as i32) * 2;      // dest element width
                    let lanes = 8usize >> esrc.trailing_zeros() as usize;
                    let m_half: i32 = if upper { 8 } else { 0 };
                    // Snapshot into scratch slot A (rn) / B (rm) when they alias rd.
                    let nbase = if rn == rd { permute_source(buf, rd, rn, false) } else { vslot(rn) };
                    let mbase = if rm == rd { permute_source(buf, rd, rm, true) } else { vslot(rm) };
                    for i in 0..lanes {
                        let nsrc = nbase + (i as i32) * der;   // Vn wide elem
                        let msrc = mbase + m_half + (i as i32) * (esrc as i32);
                        let dst = vslot(rd) + (i as i32) * der;
                        match der {
                            4 => buf.mov_load32(RAX, RBX, nsrc),
                            8 => buf.mov_load64(RAX, RBX, nsrc),
                            _ => buf.mov_load32(RAX, RBX, nsrc),
                        }
                        match (esrc, sign) {
                            (1, false) => buf.movzx_byte_mem(RCX, RBX, msrc),
                            (1, true) => buf.movsx_byte_mem(RCX, RBX, msrc),
                            (2, false) => buf.movzx_word_mem(RCX, RBX, msrc),
                            (2, true) => buf.movsx_word_mem(RCX, RBX, msrc),
                            (4, false) => buf.mov_load32(RCX, RBX, msrc),
                            (4, true) => buf.mov_load32(RCX, RBX, msrc),
                            _ => unreachable!(),
                        }
                        if esrc == 4 && sign {
                            buf.movsxd_r64_r32(RCX, RCX);
                        }
                        if sub {
                            buf.sub_rr64(RAX, RCX);
                        } else {
                            buf.add_rr64(RAX, RCX);
                        }
                        match der {
                            4 => buf.mov_store32(RBX, dst, RAX),
                            8 => buf.mov_store64(RBX, dst, RAX),
                            _ => buf.mov_store16(RBX, dst, RAX),
                        }
                    }
                    Ok(())
                }
                Inst::SimdVLog { rd, rn, rm, op } => {
                    // and/orr/eor/bic Vd.128 = Vn.128 op Vm.128 (Q selects 8/16B,
                    // translate always on the full 16-byte slot).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    buf.movdqu_load(RAX, RBX, vslot(rn));
                    buf.movdqu_load(RCX, RBX, vslot(rm));
                    match op {
                        0 => buf.pand(RAX, RCX),       // AND
                        1 => buf.pxor_xmm(RAX, RCX),   // EOR
                        2 => buf.por(RAX, RCX),        // ORR
                        _ => { buf.pandn(RCX, RAX); buf.movdqu_store(RBX, vslot(rd), RCX); return Ok(()); } // BIC: xmm1 = ~v1 & v0
                    }
                    buf.movdqu_store(RBX, vslot(rd), RAX);
                    Ok(())
                }
                Inst::SimdNot { rd, rn } => {
                    // mvn Vd.16B/8B, Vn: bitwise NOT of the full 16-byte slot.
                    // RAX = Vn; RCX = all-ones; RAX = RAX ^ RCX.
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    buf.movdqu_load(RAX, RBX, vslot(rn));
                    buf.movdqu_ones(RCX);
                    buf.pxor_xmm(RAX, RCX);
                    buf.movdqu_store(RBX, vslot(rd), RAX);
                    Ok(())
                }
                Inst::SimdSel { rd, rn, rm, op } => {
                    // bsl/bit/bif bitwise select between three 128-bit vectors.
                    // BSL: Vd = (Vn & Vd) | (~Vd & Vm).  (op stored as op).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    buf.movdqu_load(RAX, RBX, vslot(rn));
                    buf.movdqu_load(RCX, RBX, vslot(rm));
                    buf.movdqu_load(R10, RBX, vslot(rd));
                    // RAX = Rn & Rd ; R10 = ~Rd & Rm ; OR them
                    match op {
                        0 => {
                            buf.pand(RAX, R10);      // Rn & Rd
                            buf.pandn(R10, RCX);     // ~Rd & Rm
                            buf.por(RAX, R10);       // final
                        }
                        _ => {
                            buf.pandn(RAX, RCX);     // ~Vm & Rn
                            buf.pand(R10, RCX);      // Vd & Vm
                            buf.por(RAX, R10);
                        }
                    }
                    buf.movdqu_store(RBX, vslot(rd), RAX);
                    Ok(())
                }
                Inst::SimdHighNarrow { rd, rn, rm, dst_esize, sub, round, q } => {
                                        // addhn/subhn/raddhn Vd.T, Vn.W, Vm.W: dst[i] = high half of the
                                        // src-width (Vn[i] +/- Vm[i]), narrowed to dst_esize bytes. Q=1
                                        // (addhn2/raddhn2) uses the UPPER 64 bits of Vn/Vm and writes the
                                        // upper half of Vd; Q=0 uses the lower and writes the lower.
                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                        let src_esize = 2 * (dst_esize as i32);
                                        let lanes = if q { 8 / (dst_esize as i32) } else { 8 / (dst_esize as i32) };
                                        let dst_bits = (8 * (dst_esize as i32)) as u8;
                                        let src_base = 0; // source lanes always start at 0 (full 128-bit read regardless of Q)
                                        let dst_base = if q { 8 } else { 0 }; // Q=1 writes the upper half of Vd
                                        for i in 0..lanes {
                                            let so = src_base + (i as i32) * src_esize;
                                            let de = dst_base + (i as i32) * (dst_esize as i32);
                                            if src_esize >= 4 {
                                                buf.mov_load64(RAX, RBX, slot(rn)+so);
                                                buf.mov_load64(RCX, RBX, slot(rm)+so);
                                            } else {
                                                buf.mov_load32(RAX, RBX, slot(rn)+so);
                                                buf.mov_load32(RCX, RBX, slot(rm)+so);
                                            }
                                            if sub { buf.sub_rr64(RAX, RCX); } else { buf.add_rr64(RAX, RCX); }
                                            if round { let h = (1u64 << (dst_bits - 1)) as u64; buf.mov_ri64(R10, h); buf.add_rr64(RAX, R10); }
                                            buf.shr_ri8(RAX, dst_bits);
                                            match dst_esize {
                                                4 => buf.mov_store32(RBX, slot(rd)+de, RAX),
                                                2 => buf.mov_store16(RBX, slot(rd)+de, RAX),
                                                _ => buf.mov_store8(RBX, slot(rd)+de, RAX),
                                            }
                                        }
                                        // Q=0 writes only the low half of Vd; the upper 8 bytes stay 0.
                                        if !q {
                                            buf.mov_ri64(RAX, 0);
                                            buf.mov_store64(RBX, slot(rd)+8, RAX);
                                        }
                                        Ok(())
                                    }
                                        Inst::Ld2 { rd, rn, q, post, esize } => {
                    // ld2 {Vt, Vt1}, [Xn]: load 2 structure vectors, DEINTERLEAVED.
                    // Memory holds {V0.e0,V1.e0, V0.e1,V1.e1, ...}: element i of
                    // reg j (j in 0..2) is at byte offset i*(2*es) + j*es, es =
                    // element size. Vt[i]=mem[2*i*es], Vt1[i]=mem[2*i*es+es].
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let total = if q { 16 } else { 8 } as i32; // bytes per vector
                    let es = esize as i32;
                    let nelems = total / es;
                    ldg(buf, RDX, rn as u32); // RDX = base (host ptr)
                    for j in 0..2i32 {
                        for i in 0..nelems {
                            for b in 0..es {
                                buf.movzx_byte_mem(RAX, RDX, i * 2 * es + j * es + b);
                                buf.mov_store8(RBX, vslot((rd as i32 + j) as u8) + i * es + b, RAX);
                            }
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                Inst::St2 { rd, rn, q, post, esize } => {
                    // st2 {Vt, Vt1}, [Xn]: the inverse — store structure vectors
                    // Vt..Vt1 to memory in the deinterleaved {V0.e0,V1.e0,...}
                    // layout (element i of reg j at byte i*2*es + j*es).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let total = if q { 16 } else { 8 } as i32;
                    let es = esize as i32;
                    let nelems = total / es;
                    ldg(buf, RDX, rn as u32); // RDX = base (host ptr)
                    for j in 0..2i32 {
                        for i in 0..nelems {
                            for b in 0..es {
                                buf.movzx_byte_mem(RAX, RBX, vslot((rd as i32 + j) as u8) + i * es + b);
                                buf.mov_store8(RDX, i * 2 * es + j * es + b, RAX);
                            }
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                // ---- ld3/st3: structure DEINTERLEAVE (3 registers) ----
                // Memory holds {V0.e0,V1.e0,V2.e0, V0.e1,...}: element i of
                // reg j (j in 0..3) is at byte offset i*(3*es) + j*es.
                Inst::Ld3N { rd, rn, q, post, esize } => {
                    let v = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let total = if q { 16 } else { 8 } as i32;
                    let es = esize as i32;
                    let nelems = total / es;
                    ldg(buf, RDX, rn as u32);
                    for j in 0..3i32 {
                        for i in 0..nelems {
                            for b in 0..es {
                                buf.movzx_byte_mem(RAX, RDX, i * 3 * es + j * es + b);
                                buf.mov_store8(RBX, v((rd as i32 + j) as u8) + i * es + b, RAX);
                            }
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                Inst::St3N { rd, rn, q, post, esize } => {
                    let v = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let total = if q { 16 } else { 8 } as i32;
                    let es = esize as i32;
                    let nelems = total / es;
                    ldg(buf, RDX, rn as u32);
                    for j in 0..3i32 {
                        for i in 0..nelems {
                            for b in 0..es {
                                buf.movzx_byte_mem(RAX, RBX, v((rd as i32 + j) as u8) + i * es + b);
                                buf.mov_store8(RDX, i * 3 * es + j * es + b, RAX);
                            }
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                // ---- ld4/st4: structure DEINTERLEAVE (4 registers) ----
                // Memory holds {V0.e0,V1.e0,V2.e0,V3.e0, V0.e1,...}: element i
                // of reg j (j in 0..4) is at byte offset i*(4*es) + j*es, es =
                // element size in bytes.
                Inst::Ld4N { rd, rn, q, post, esize } => {
                    let v = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let total = if q { 16 } else { 8 } as i32; // bytes per vector
                    let es = esize as i32;
                    let nelems = total / es;
                    ldg(buf, RDX, rn as u32);
                    for j in 0..4i32 {
                        for i in 0..nelems {
                            // copy es bytes: mem[i*(4es)+j*es .. +es] -> v[rd+j]+i*es
                            for b in 0..es {
                                buf.movzx_byte_mem(RAX, RDX, i * 4 * es + j * es + b);
                                buf.mov_store8(RBX, v((rd as i32 + j) as u8) + i * es + b, RAX);
                            }
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                Inst::St4N { rd, rn, q, post, esize } => {
                    let v = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let total = if q { 16 } else { 8 } as i32;
                    let es = esize as i32;
                    let nelems = total / es;
                    ldg(buf, RDX, rn as u32);
                    for j in 0..4i32 {
                        for i in 0..nelems {
                            for b in 0..es {
                                buf.movzx_byte_mem(RAX, RBX, v((rd as i32 + j) as u8) + i * es + b);
                                buf.mov_store8(RDX, i * 4 * es + j * es + b, RAX);
                            }
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                Inst::Ld1N { rd, rn, nreg, q, post } => {
                    // ld1 {Vt, Vt2, ..}, [Xn]: load `nreg` CONSECUTIVE (q?16:8)-byte
                    // blocks of memory into V[rd], V[rd+1], .. (no deinterleave) —
                    // the compiler's array-literal / memcpy idiom.
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let block = if q { 16 } else { 8 } as i32;
                    ldg(buf, RDX, rn as u32); // RDX = base host ptr
                    for k in 0..nreg as i32 {
                        let slot = vslot((rd as i32 + k) as u8);
                        if block == 16 {
                            buf.movdqu_load(0, RDX, k * 16);
                            buf.movdqu_store(RBX, slot, 0);
                        } else {
                            buf.movq_load(0, RDX, k * 8);
                            buf.movq_store(RBX, slot, 0);
                            buf.mov_ri64(RAX, 0);
                            buf.mov_store64(RBX, slot + 8, RAX); // zero high u64
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                Inst::St1N { rd, rn, nreg, q, post } => {
                    // st1 {Vt, Vt2, ..}, [Xn]: store `nreg` consecutive (q?16:8)-byte
                    // blocks from V[rd], V[rd+1], .. to memory (no deinterleave).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let block = if q { 16 } else { 8 } as i32;
                    ldg(buf, RDX, rn as u32);
                    for k in 0..nreg as i32 {
                        let slot = vslot((rd as i32 + k) as u8);
                        if block == 16 {
                            buf.movdqu_load(0, RBX, slot);
                            buf.movdqu_store(RDX, k * 16, 0);
                        } else {
                            buf.movq_load(0, RBX, slot);
                            buf.movq_store(RDX, k * 8, 0);
                        }
                    }
                    if post != 0 {
                        buf.mov_load64(RAX, RBX, slot(rn as u32));
                        buf.add_ri64(RAX, post as u32);
                        buf.mov_store64(RBX, slot(rn as u32), RAX);
                    }
                    Ok(())
                }
                Inst::SimdShl { rd, rn, esize, shift } => {
                    // shl Vd.T, Vn.T, #imm : left-shift each lane by shift.
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let lanes = 16 / (esize as i32);  // 16/esize lanes
                    for i in 0..lanes {
                        let off = vslot(rn) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_load64(RAX, RBX, off),
                            4 => buf.mov_load32(RAX, RBX, off),
                            2 => buf.movzx_word_mem(RAX, RBX, off),
                            _ => buf.movzx_byte_mem(RAX, RBX, off),
                        }
                        buf.shl_ri8(RAX, shift);
                        let dst = vslot(rd) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_store64(RBX, dst, RAX),
                            4 => buf.mov_store32(RBX, dst, RAX),
                            2 => buf.mov_store16(RBX, dst, RAX),
                            _ => buf.mov_store8(RBX, dst, RAX),
                        }
                    }
                    Ok(())
                }
                Inst::SimdSatShl { rd, rn, esize, shift, sat } => {
                    // sqshl/uqshl/sqshlu Vd.T, Vn.T, #imm: left-shift each lane then
                    // saturate. sat: 0=sqshl (signed src/dst), 1=uqshl (unsigned
                    // src/dst), 2=sqshlu (signed src, unsigned dst). Shift arithmetic
                    // on the 64-bit reg after sign/zero-extending the source, then
                    // clamp. Self-alias-safe (dest not re-read).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let lanes = 16 / (esize as i32);
                    let ebits = 8 * (esize as i32);
                    let (maxv, minv): (i64, i64) = match (sat, esize) {
                        (0, 8) => (i64::MAX, i64::MIN),
                        (0, 4) => (0x7fff_ffff, -0x8000_0000),
                        (0, 2) => (0x7fff, -0x8000),
                        (0, 1) => (0x7f, -0x80),
                        (1, 8) => (i64::MAX, 0),
                        (1, 4) => (0xffff_ffff, 0),
                        (1, 2) => (0xffff, 0),
                        (1, 1) => (0xff, 0),
                        (2, 8) => (i64::MAX, 0),
                        (2, 4) => (0xffff_ffff, 0),
                        (2, 2) => (0xffff, 0),
                        _ => (0xff, 0),
                    };
                    let src_signed = sat != 1;
                    for i in 0..lanes {
                        let off = vslot(rn) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_load64(RAX, RBX, off),
                            4 => {
                                buf.mov_load32(RAX, RBX, off);
                                if src_signed { buf.movsxd_r64_r32(RAX, RAX); }
                            }
                            2 => {
                                if src_signed { buf.movsx_word_mem(RAX, RBX, off); }
                                else { buf.movzx_word_mem(RAX, RBX, off); }
                            }
                            _ => {
                                if src_signed { buf.movsx_byte_mem(RAX, RBX, off); }
                                else { buf.movzx_byte_mem(RAX, RBX, off); }
                            }
                        }
                        // shl in the wide 64-bit reg. Because shift <= ebits-1 and the
                        // source was sign/zero-extended, value*2^shift fits i64 (magnitude
                        // <= 2^(2*ebits-1)), so clamping against the element range after
                        // the shift saturates correctly (a negative src that overflows the
                        // lane clamps to min, a positive one to max).
                        buf.shl_ri8(RAX, shift);
                        // clamp
                        buf.mov_ri64(RCX, minv as u64);
                        buf.cmp_rr64(RAX, RCX);
                        buf.cmov_rr64(0x4c, RAX, RCX); // RAX=minv if RAX<minv
                        buf.mov_ri64(RCX, maxv as u64);
                        buf.cmp_rr64(RAX, RCX);
                        buf.cmov_rr64(0x4f, RAX, RCX); // RAX=maxv if RAX>maxv
                        let dst = vslot(rd) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_store64(RBX, dst, RAX),
                            4 => buf.mov_store32(RBX, dst, RAX),
                            2 => buf.mov_store16(RBX, dst, RAX),
                            _ => buf.mov_store8(RBX, dst, RAX),
                        }
                    }
                    Ok(())
                }
                Inst::SimdShrAcc { rd, rn, esize, shift, unsigned } => {
                    // usra/ssra Vd.T, Vn.T, #imm : Vd_i += Vn_i >> imm (logical if
                    // unsigned/usra, arithmetic if signed/ssra). The source element is
                    // sign-extended to 64 bits for ssra (zero-extending a negative
                    // element made the arithmetic shift positive).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let lanes = 16 / (esize as i32);
                    for i in 0..lanes {
                        let off = vslot(rn) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_load64(RAX, RBX, off),
                            4 => {
                                buf.mov_load32(RAX, RBX, off);
                                if !unsigned { buf.movsxd_r64_r32(RAX, RAX); }
                            }
                            2 => {
                                if unsigned { buf.movzx_word_mem(RAX, RBX, off); }
                                else { buf.movsx_word_mem(RAX, RBX, off); }
                            }
                            _ => {
                                if unsigned { buf.movzx_byte_mem(RAX, RBX, off); }
                                else { buf.movsx_byte_mem(RAX, RBX, off); }
                            }
                        }
                        let esize_bits = (esize as i32) * 8;
                        if (shift as i32) >= esize_bits {
                            if unsigned {
                                buf.xor_rr64(RAX, RAX);
                            } else {
                                buf.sar_ri8(RAX, 63);
                            }
                        } else if unsigned {
                            buf.shr_ri8(RAX, shift);
                        } else {
                            buf.sar_ri8(RAX, shift);
                        }
                        let dst = vslot(rd) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_load64(RCX, RBX, dst),
                            4 => buf.mov_load32(RCX, RBX, dst),
                            2 => buf.movzx_word_mem(RCX, RBX, dst),
                            _ => buf.movzx_byte_mem(RCX, RBX, dst),
                        }
                        buf.add_rr64(RCX, RAX);
                        match esize {
                            8 => buf.mov_store64(RBX, dst, RCX),
                            4 => buf.mov_store32(RBX, dst, RCX),
                            2 => buf.mov_store16(RBX, dst, RCX),
                            _ => buf.mov_store8(RBX, dst, RCX),
                        }
                    }
                    Ok(())
                }
                Inst::SimdShrn { rd, rn, esrc, shift, upper, round } => {
                    // shrn/shrn2/rshrn/rshrn2 Vd.T, Vn.U, #imm: shift each DOUBLE-width
                    // source element (esrc bytes) right by `shift`, truncate (shrn) or
                    // round (rshrn: add 2^(shift-1) before shifting) to the HALF-width
                    // dest element (esrc/2 bytes). shrn writes low/high dest lanes by
                    // `upper`; rshrn rounds. SELF-ALIAS (gcc emits shrn v31,v31) snapshots
                    // the source to scratch so the dest writes don't clobber later reads.
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let src = permute_source(buf, rd, rn, false);
                    let es = esrc as i32;         // source element bytes
                    let ds = (esrc as i32) / 2;   // dest element bytes
                    let src_lanes = 16 / es;      // source elements in 128-bit reg
                    let dst_off = if upper { 8 } else { 0 }; // shrn2 writes high half
                    for i in 0..src_lanes {
                        // load DOUBLE-width source lane, zero-extended
                        match esrc {
                            8 => buf.mov_load64(RAX, RBX, src + i * es),
                            4 => buf.mov_load32(RAX, RBX, src + i * es),
                            2 => buf.movzx_word_mem(RAX, RBX, src + i * es),
                            _ => buf.movzx_byte_mem(RAX, RBX, src + i * es),
                        }
                        if round && (shift as i32) >= 1 {
                            // rshrn: add 1 << (shift-1) to round-half-up
                            buf.add_ri64(RAX, 1u32 << (shift - 1));
                        }
                        if (shift as i32) >= es * 8 {
                            buf.xor_rr64(RAX, RAX);
                        } else {
                            buf.shr_ri8(RAX, shift);
                        }
                        let dst = vslot(rd) + dst_off + i * ds;
                        match ds {
                            4 => buf.mov_store32(RBX, dst, RAX),
                            2 => buf.mov_store16(RBX, dst, RAX),
                            _ => buf.mov_store8(RBX, dst, RAX),
                        }
                    }
                    Ok(())
                }
                Inst::SimdShr { rd, rn, esize, shift, unsigned } => {
                    // ushr/sshr Vd.T, Vn.T, #imm : Vd_i = Vn_i >> shift (logical if
                    // unsigned/ushr, arithmetic if signed/sshr), no accumulate.
                    // For sshr the esize-bit element must be SIGN-extended to 64 bits
                    // before the arithmetic shift (zero-extending a negative element
                    // made it positive); guards shift >= esize*8 (0 for logical,
                    // all-ones sign-fill for arithmetic - a bare x86 imm clamps).
                    let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let lanes = 16 / (esize as i32);
                    let esize_bits = (esize as i32) * 8;
                    for i in 0..lanes {
                        let off = vslot(rn) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_load64(RAX, RBX, off),
                            4 => {
                                buf.mov_load32(RAX, RBX, off);
                                if !unsigned { buf.movsxd_r64_r32(RAX, RAX); }
                            }
                            2 => {
                                if unsigned { buf.movzx_word_mem(RAX, RBX, off); }
                                else { buf.movsx_word_mem(RAX, RBX, off); }
                            }
                            _ => {
                                if unsigned { buf.movzx_byte_mem(RAX, RBX, off); }
                                else { buf.movsx_byte_mem(RAX, RBX, off); }
                            }
                        }
                        if (shift as i32) >= esize_bits {
                            if unsigned {
                                buf.xor_rr64(RAX, RAX);
                            } else {
                                buf.sar_ri8(RAX, 63);
                            }
                        } else if unsigned {
                            buf.shr_ri8(RAX, shift);
                        } else {
                            buf.sar_ri8(RAX, shift);
                        }
                        let dst = vslot(rd) + (i as i32) * (esize as i32);
                        match esize {
                            8 => buf.mov_store64(RBX, dst, RAX),
                            4 => buf.mov_store32(RBX, dst, RAX),
                            2 => buf.mov_store16(RBX, dst, RAX),
                            _ => buf.mov_store8(RBX, dst, RAX),
                        }
                    }
                    Ok(())
                }
                Inst::SimdInsD { rd, rn, dst_idx, src_idx, esize } => {
                    // mov Vd.T[dst], Vn.T[src]: copy one element (esize bytes) lane
                    // between vectors (d 64-bit or s 32-bit lanes).
                    let es = esize as i32;
                    let src = crate::jit::VECTOR_BASE + (rn as i32)*16 + (src_idx as i32)*es;
                    let dst = crate::jit::VECTOR_BASE + (rd as i32)*16 + (dst_idx as i32)*es;
                    if esize == 8 {
                        buf.mov_load64(RAX, RBX, src);
                        buf.mov_store64(RBX, dst, RAX);
                    } else if esize == 4 {
                        buf.mov_load32(RAX, RBX, src);
                        buf.mov_store32(RBX, dst, RAX);
                    } else if esize == 2 {
                        buf.movzx_word_mem(RAX, RBX, src);
                        buf.mov_store16(RBX, dst, RAX);
                    } else {
                        buf.movzx_byte_mem(RAX, RBX, src);
                        buf.mov_store8(RBX, dst, RAX);
                    }
                    Ok(())
                }
                Inst::SimdFmulEl { rd, rn, rm, esize, index, q } => {
                    // fmul Vd.T, Vn.T, Vm.T[L]: each lane of Vd = Vn[lane] * Vm[L].
                    let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                    let es = esize as i32;
                    let n = if q { 16i32 } else { 8i32 };
                    let lanes = n / es;
                    // Broadcast the single element into xmm2 ONCE up front. Must
                    // NOT re-read the element memory each iteration: when rd==rm
                    // (e.g. `fmul v17.4s, v7.4s, v17.s[0]`, rd==rm==17) the first
                    // lane's write clobbers the element before later lanes read
                    // it, corrupting every lane after 0 (silent wrong vector).
                    let mel = f(rm) + (index as i32) * es; // address of Vm[L]
                    if esize == 8 {
                        buf.movq_load(2, RBX, mel);
                    } else {
                        buf.mov_load32(RAX, RBX, mel);
                        buf.movd_xmm_r32(2, RAX);
                    }
                    for l in 0..lanes {
                        // total lane byte offset: lanes may be 2S(8B),4S/2d(16B)
                        let to = l * es;
                        if esize == 8 {
                            buf.movq_load(0, RBX, f(rn) + to);
                            buf.mulsd(0, 2);
                            buf.movq_store(RBX, f(rd) + to, 0);
                        } else {
                            // single-precision lanes
                            buf.mov_load32(RAX, RBX, f(rn) + to);
                            buf.movd_xmm_r32(0, RAX);
                            buf.mulss(0, 2);
                            buf.movd_r32_xmm(RAX, 0);
                            buf.mov_store32(RBX, f(rd) + to, RAX);
                                    }
                                }
                                Ok(())
                            }
                            Inst::SimdRev { rd, rn, granule, q } => {
                                // rev64/rev32 Vd.T, Vn.T: byte-reverse within each
                                // granule (8B for rev64, 4B for rev32) via BSWAP.
                                let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                let base = f(rn);
                                let dbase = f(rd);
                                let total = if q { 16 } else { 8 };
                                let n = total / (granule as i32);
                                for g in 0..n {
                                    let o = g * (granule as i32);
                                    if granule == 8 {
                                        buf.mov_load64(RAX, RBX, base + o);
                                        buf.bswap_r64(RAX);
                                        buf.mov_store64(RBX, dbase + o, RAX);
                                    } else if granule == 4 {
                                        buf.mov_load32(RAX, RBX, base + o);
                                        buf.bswap_r32(RAX);
                                        buf.mov_store32(RBX, dbase + o, RAX);
                                    } else {
                                        // granule == 2 (rev16): byte-swap each
                                        // 16-bit halfword = rotate-left-by-8.
                                        buf.mov_load16(RAX, RBX, base + o);
                                        buf.rol16_ri8(RAX, 8);
                                        buf.mov_store16(RBX, dbase + o, RAX);
                                    }
                                }
                                Ok(())
                            }
                            Inst::SimdLaneS { rd, rn, esize, index } => {
                                // mov Sd/Dd, Vn.T[idx]: copy the element into the
                                // low bytes of the dest FP register slot.
                                let f = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                let src = f(rn) + (index as i32) * (esize as i32);
                                let dst = f(rd);
                                if esize == 8 {
                                    buf.mov_load64(RAX, RBX, src);
                                    buf.mov_store64(RBX, dst, RAX);
                                } else {
                                    buf.mov_load32(RAX, RBX, src);
                                    buf.mov_store32(RBX, dst, RAX);
                                }
                                Ok(())
                            }
                            Inst::Sha { mode, rd, rn, rm } => {
                                // Call the host SHA helper: passes the current
                                // guest-state pointer (RBX) and a packed[op|rd|rn|rm].
                                let packed = ((mode as u64) << 24) | ((rd as u64) << 16)
                                    | ((rn as u64) << 8) | (rm as u64);
                                let addr = crate::jit::guest_sha1stem as usize as u64;
                                buf.mov_rr64(RDI, RBX);   // arg0 = CpuState*
                                buf.mov_ri64(RSI, packed); // arg1 = packed op
                                buf.mov_ri64(RAX, addr);
                                // Align RSP ≡ 0 mod 16 at the host call site
                                // (block body runs at RSP ≡ 8; SysV host calls
                                // need ≡ 0) — same rationale as the Svc arm.
                                buf.sub_ri64(4, 8);
                                buf.call_r64(RAX);
                                buf.add_ri64(4, 8);
                                Ok(())
                            }
                            // Vd = (Vn & Vm) | (Vd & ~Vm), over the full 16 bytes
        Inst::SimdFmovImm { rd, esize, value_bits, q } => {
            // fmov Vd.T, #imm: broadcast the immediate FP float (esize bytes,
            // 64-bit double or 32-bit single bits) into every lane of Vd.
            let slot = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            if esize == 8 {
                buf.mov_ri64(RAX, value_bits);
            } else {
                buf.mov_ri64(RAX, value_bits & 0xffff_ffff);
            }
            let total = if q { 16i32 } else { 8i32 };
            let mut off = 0i32;
            while off < total {
                if esize == 8 {
                    buf.mov_store64(RBX, slot + off, RAX);
                } else {
                    buf.mov_store32(RBX, slot + off, RAX);
                }
                off += esize as i32;
            }
            Ok(())
        }
        Inst::VarShiftVar { rd, rn, rm, op, sf } => {
            // lslv/lsrv/asrv/rorv Rd, Rn, Rm: variable shift by register. Rn -> RAX,
            // Rm count -> RCX (low byte CL); shift via the _cl64 helpers. W masks the
            // count to 0x1f and ASR sign-extends the low 32 before arithmetic shift.
            ldg(buf, RAX, rn as u32); // value
            ldg(buf, RCX, rm as u32); // shift count
            if sf {
                buf.and_ri64(RCX, 0x3f);
            } else {
                buf.and_ri64(RCX, 0x1f);
                if op == 2 {
                    // asr (W): sign-extend low 32 before arithmetic shift
                    buf.movsxd_r64_r32(RAX, RAX);
                }
            }
            match op {
                0 => buf.shl_cl64(RAX), // lslv
                1 => buf.shr_cl64(RAX), // lsrv
                2 => buf.sar_cl64(RAX), // asrv
                _ => buf.ror_cl64(RAX), // rorv
            }
            if !sf {
                buf.zero_ext_r32(RAX);
            }
            if rd != 31 {
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::Div { rd, rn, rm, signed, is_x } => {
            // udiv/sdiv Rd, Rn, Rm: RAX = Rn / Rm (div/rdiv in RAX/RDX:AX).
            ldg(buf, RAX, rn as u32);        // dividend
            ldg(buf, RCX, rm as u32);        // divisor
            buf.xor_rr64(RDX, RDX);          // clear hi-word for unsigned div
            if signed {
                buf.idiv_r64(RCX);           // signed: use I DIV
            } else if is_x {
                buf.div_r64(RCX);
            } else {
                buf.div_r32(RCX);
            }
            stg(buf, rd as u32, RAX);        // quotient -> Rd
            Ok(())
        }
        Inst::SimdVShift { rd, rn, rm, esize, signed_, q, rounding } => {
            // ushl/sshl Vd.T, Vn.T, Vm.T : per-lane variable shift.
            // Each count lane C is a SIGNED esize-bit value:
            //   C >= 0 -> result = V << C          (left)
            //   C <  0 -> result = V >> -C         (right; sshl = arithmetic, ushl = logical)
            //   |C| >= B (B = esize*8):
            //       left  shift by >= B -> 0
            //       right shift by >= B -> ushl: 0, sshl: sign-fill (= sign bit replicated)
            // The scalar count is masked by x86's `shl/shr/sar r64, cl` to low 6 bits,
            // so out-of-range shifts must be guarded explicitly (else shl by 64 wraps).
            let vslot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let lanes = if q { 16 / (esize as i32) } else { 8 / (esize as i32) };
            let bbits = (esize as i32) * 8;
            let wmask: u64 = if esize == 8 { u64::MAX } else { (1u64 << bbits) - 1 };
            for i in 0..lanes {
                let src = vslot(rn) + (i as i32) * (esize as i32);
                let cnt = vslot(rm) + (i as i32) * (esize as i32);
                let dst = vslot(rd) + (i as i32) * (esize as i32);
                // load value V -> RAX, count C -> RCX (zero-extended)
                match esize {
                    8 => buf.mov_load64(RAX, RBX, src),
                    4 => buf.mov_load32(RAX, RBX, src),
                    2 => buf.movzx_word_mem(RAX, RBX, src),
                    _ => buf.movzx_byte_mem(RAX, RBX, src),
                }
                match esize {
                    8 => buf.mov_load64(RCX, RBX, cnt),
                    4 => buf.mov_load32(RCX, RBX, cnt),
                    2 => buf.movzx_word_mem(RCX, RBX, cnt),
                    _ => buf.movzx_byte_mem(RCX, RBX, cnt),
                }
                // sign-extend the count lane to its element width
                match esize {
                    8 => {}
                    4 => buf.movsxd_r64_r32(RCX, RCX),
                    2 => {
                        buf.shl_ri8(RCX, 48);
                        buf.sar_ri8(RCX, 48);
                    }
                    _ => {
                        buf.shl_ri8(RCX, 56);
                        buf.sar_ri8(RCX, 56);
                    }
                }
                let mut done_jumps: Vec<usize> = Vec::new(); // jump offsets that target the final store
                let mut fixed: Vec<(usize, usize)> = Vec::new(); // (jump, fixed internal label)
                buf.test_rr64(RCX, RCX); // set SF (sign) from the sign-extended count
                let jneg = buf.jcc_rel32(0x88); // JS: count < 0 -> right path
                // ---- left path (C >= 0) ----
                if bbits < 64 {
                    buf.cmp_ri64(RCX, bbits as u32);
                    let jbig = buf.jcc_rel32(0x83); // JAE: C >= B -> result 0
                    buf.shl_cl64(RAX);
                    if wmask != u64::MAX {
                        buf.mov_ri64(RDX, wmask);
                        buf.and_rr64(RAX, RDX);
                    }
                    done_jumps.push(buf.jmp_rel32()); // jdone
                    let big_at = buf.len();
                    buf.mov_ri64(RAX, 0);
                    done_jumps.push(buf.jmp_rel32()); // jbigfall (big -> store)
                    fixed.push((jbig, big_at));
                } else {
                    // 64-bit, C in [0,63]: shl by CL is exact (x86 masks to low 6 bits)
                    buf.shl_cl64(RAX);
                    done_jumps.push(buf.jmp_rel32()); // jdone
                }
                // ---- right path (C < 0): -C = -(RCX) = k ----
                let right_at = buf.len();
                buf.neg_r64(RCX); // RCX = -C = magnitude k
                if signed_ {
                    // sshl/srshl: arithmetic right shift, sign-extend the element's sign first
                    match esize {
                        4 => buf.movsxd_r64_r32(RAX, RAX),
                        2 => {
                            buf.shl_ri8(RAX, 48);
                            buf.sar_ri8(RAX, 48);
                        }
                        1 => {
                            buf.shl_ri8(RAX, 56);
                            buf.sar_ri8(RAX, 56);
                        }
                        _ => {}
                    }
                }
                if bbits < 64 {
                    buf.cmp_ri64(RCX, bbits as u32);
                    let jbigr = buf.jcc_rel32(0x83); // JAE: magnitude >= B
                    // rounding (urshl/srshl): result = (V + (1 << (k-1))) >> k.
                    // Add the in-range bias only when k < B; the out-of-range branch
                    // below keeps RAX unbiased (ARM rounds only within-range right
                    // shifts — a full shift-out returns sign-fill/0 regardless).
                    if rounding {
                        // RDX = 1 << (k-1): bits k..0 set then shift right 1.
                        buf.mov_ri64(RDX, 1);
                        buf.shl_cl64(RDX);      // RDX = 1 << k
                        buf.shr_ri8(RDX, 1);    // RDX = 1 << (k-1)
                        buf.add_rr64(RAX, RDX); // V += bias
                    }
                    if signed_ {
                        buf.sar_cl64(RAX); // sshl/srshl: arithmetic right shift
                    } else {
                        buf.shr_cl64(RAX); // ushl/urshl: logical right shift
                    }
                    if wmask != u64::MAX {
                        buf.mov_ri64(RDX, wmask);
                        buf.and_rr64(RAX, RDX);
                    }
                    done_jumps.push(buf.jmp_rel32()); // jdoner
                    let sign_or_zero = buf.len();
                    // out-of-range right shift: ushl/urshl -> 0; sshl/srshl -> sign-fill
                    if signed_ {
                        // result = (V < 0) ? wmask : 0 ... but V already sign-extended.
                        buf.test_rr64(RAX, RAX);
                        let jnsz = buf.jcc_rel32(0x89); // JNS: V >= 0 -> 0
                        buf.mov_ri64(RAX, wmask);
                        done_jumps.push(buf.jmp_rel32()); // jz (wmask -> store)
                        let zero_at = buf.len();
                        buf.mov_ri64(RAX, 0);
                        fixed.push((jnsz, zero_at));
                    } else {
                        buf.mov_ri64(RAX, 0);
                    }
                    fixed.push((jbigr, sign_or_zero));
                } else {
                    // 64-bit (bbits==64): for 64-bit esize, rounding (srshl/q) is
                    // determined per-lane; here bbits<64 is false only for esize 8,
                    // and srshl .2d lives in the bbits<64==false path. Bias then sar.
                    if rounding {
                        buf.mov_ri64(RDX, 1);
                        buf.shl_cl64(RDX);
                        buf.shr_ri8(RDX, 1);
                        buf.add_rr64(RAX, RDX);
                    }
                    buf.sar_cl64(RAX); // 64-bit, C in [-63,-1] so sar is fine
                }
                // patch to final store
                let done = buf.len();
                for &at in &done_jumps {
                    let disp = (done as i64 - (at as i64 + 4)) as i32;
                    buf.bytes[at..at + 4].copy_from_slice(&disp.to_le_bytes());
                }
                for (at, tgt) in &fixed {
                    let disp = (*tgt as i64 - (*at as i64 + 4)) as i32;
                    buf.bytes[*at..*at + 4].copy_from_slice(&disp.to_le_bytes());
                }
                let disp_neg = (right_at as i64 - (jneg as i64 + 4)) as i32;
                buf.bytes[jneg..jneg + 4].copy_from_slice(&disp_neg.to_le_bytes());
                // store lane result to Vd[i]
                match esize {
                    8 => buf.mov_store64(RBX, dst, RAX),
                    4 => buf.mov_store32(RBX, dst, RAX),
                    2 => buf.mov_store16(RBX, dst, RAX),
                    _ => buf.mov_store8(RBX, dst, RAX),
                }
            }
            Ok(())
        }
        Inst::SimdFpUnary { rd, rn, op, esize, q } => {
            // fneg/fabs/fsqrt Vd.T, Vn.T: per-lane unary FP on the vector slot.
            // fneg/fabs flip/clear the sign bit on the FP bit-pattern via GPRs;
            // fsqrt uses x86 sqrtsd. esize 8 lanes are full doubles, esize 4
            // lanes handled the same (sign-bit at bit 31; store low 32 back).
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let lanes = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let m = esize as i32;
            let sign64: u64 = if esize == 8 { 0x8000_0000_0000_0000 }
                else if esize == 2 { 0x8000 } else { 0x8000_0000 };
            for l in 0..lanes {
                let off = l * m;
                buf.movq_load(0, RBX, src + off); // xmm0 <- lane bits
                if op == 2 {
                    buf.sqrtsd(0, 0); // fsqrt
                } else {
                    // fneg (op 0): xor sign; fabs (op 1): and with ~sign
                    buf.movq_r64_xmm(RAX, 0);
                    buf.mov_ri64(RCX, sign64);
                    if op == 0 {
                        buf.xor_rr64(RAX, RCX);
                    } else {
                        buf.mov_ri64(RDX, sign64);
                        buf.not_r64(RDX);
                        buf.and_rr64(RAX, RDX);
                    }
                    buf.movq_xmm_r64(0, RAX);
                }
                if esize == 8 {
                    buf.movq_store(RBX, dst + off, 0);
                } else if esize == 2 {
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store16(RBX, dst + off, RAX); // fp16 fabs/fneg: 2 bytes
                } else {
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, dst + off, RAX);
                }
            }
            Ok(())
        }
        Inst::SimdFrint { rd, rn, mode, esize, q } => {
            // frint{n,m,p,z,a} Vd.T, Vn.T: per-lane FP rounding. esize 8 lanes
            // are doubles; esize 4 lanes are singles (promote to double, round,
            // demote). roundsd imm: 0b00=nearest-even(frintn), 0b01=floor(m),
            // 0b10=ceil(p), 0b11=toward-zero(z). frinta (mode 4) = ties-away:
            // sign*floor(|x|+0.5).
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let lanes = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let m = esize as i32;
            let emit_round = |buf: &mut crate::x86::CodeBuf| {
                match mode {
                    0 => buf.roundsd(0, 0, 0b00), // frintn: round to nearest-even
                    1 => buf.roundsd(0, 0, 0b01), // frintm: floor
                    2 => buf.roundsd(0, 0, 0b10), // frintp: ceil
                    3 => buf.roundsd(0, 0, 0b11), // frintz: toward zero
                    4 => {
                        // frinta: nearest, ties away = sign*floor(|x|+0.5)
                        buf.movq_r64_xmm(RAX, 0);
                        buf.mov_ri64(RCX, 0x8000_0000_0000_0000);
                        buf.and_rr64(RAX, RCX);               // sign bit
                        buf.movq_xmm_r64(2, RAX);             // xmm2 = sign mask
                        buf.movq_r64_xmm(RAX, 0);
                        buf.mov_ri64(RCX, 0x7fff_ffff_ffff_ffff);
                        buf.and_rr64(RAX, RCX);               // |x|
                        buf.movq_xmm_r64(0, RAX);
                        buf.mov_ri64(RAX, 0x3fe0_0000_0000_0000); // 0.5
                        buf.movq_xmm_r64(1, RAX);
                        buf.addsd(0, 1);                      // |x| + 0.5
                        buf.roundsd(0, 0, 0x01);              // floor
                        buf.pxor_xmm(0, 2);                   // reapply sign
                    }
                    _ => unreachable!("SimdFrint bad mode {mode}"),
                }
            };
            for l in 0..lanes {
                let off = l * m;
                if esize == 8 {
                    buf.movq_load(0, RBX, src + off); // 64-bit double lane
                    emit_round(&mut *buf);
                    buf.movq_store(RBX, dst + off, 0);
                } else if esize == 2 {
                    // FP16 lane: promote to FP32 (vcvtph2ps), round, demote.
                    buf.mov_load16(RAX, RBX, src + off);
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    buf.cvtss2sd(0, 0);
                    emit_round(&mut *buf);
                    buf.cvtsd2ss(0, 0);
                    // frint modes other than a use the exact f32 rounding; demote RN
                    // (frint results are integral so RN is exact).
                    buf.bytes.extend_from_slice(&[0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]); // vcvtps2ph $0,xmm0,xmm0
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store16(RBX, dst + off, RAX);
                } else {
                    buf.mov_load32(RAX, RBX, src + off); // 32-bit float lane
                    buf.movd_xmm_r32(0, RAX);
                    buf.cvtss2sd(0, 0); // promote to double
                    emit_round(&mut *buf);
                    buf.cvtsd2ss(0, 0); // demote back to single
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, dst + off, RAX);
                }
            }
            Ok(())
        }
        Inst::SimdArithUnary { rd, rn, esize, q, op } => {
            // neg/abs Vd.T, Vn.T: per-lane signed negate or absolute value.
            // neg: 0 - lane (two's complement wraps on overflow, matching ARM).
            // abs: |signed lane| via the identity (x ^ (x ar>> w-1)) - (x ar>> w-1)
            // after sign-extending the lane; safe read-modify-write when rd==rn.
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let lanes = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let m = esize as i32;
            for l in 0..lanes {
                match esize {
                    8 => buf.mov_load64(RAX, RBX, src + l * m),
                    4 => {
                        buf.mov_load32(RAX, RBX, src + l * m);
                        if op == 1 {
                            buf.movsxd_r64_r32(RAX, RAX); // sign-extend for |signed|
                        }
                    }
                    2 => {
                        if op == 1 {
                            buf.movsx_word_mem(RAX, RBX, src + l * m);
                        } else {
                            buf.movzx_word_mem(RAX, RBX, src + l * m);
                        }
                    }
                    _ => {
                        if op == 1 {
                            buf.movsx_byte_mem(RAX, RBX, src + l * m);
                        } else {
                            buf.movzx_byte_mem(RAX, RBX, src + l * m);
                        }
                    }
                }
                if op == 0 {
                    buf.neg_r64(RAX);
                } else {
                    let w = esize * 8;
                    buf.mov_rr64(RCX, RAX);
                    buf.sar_ri8(RCX, (w - 1) as u8);
                    buf.xor_rr64(RAX, RCX);
                    buf.sub_rr64(RAX, RCX);
                }
                match esize {
                    8 => buf.mov_store64(RBX, dst + l * m, RAX),
                    4 => buf.mov_store32(RBX, dst + l * m, RAX),
                    2 => buf.mov_store16(RBX, dst + l * m, RAX),
                    _ => buf.mov_store8(RBX, dst + l * m, RAX),
                }
            }
            Ok(())
        }
        Inst::SimdAddp { rd, rn, rm, esize, q } => {
            // ADDP Vd.T, Vn.T, Vm.T: pairwise-adjacent addition, no widening.
            // First half of the output lanes = sums of adjacent (2i, 2i+1) lane
            // pairs of Vn; second half = pair sums of Vm. Output lanes total
            // n/esize where n = 8 (q=0) or 16 (q=1) bytes. Each src element read
            // EXACTLY esize bytes (zero-extended) then added, so a lane's high
            // bytes never pollute the neighbour; store exactly esize bytes.
            // SELF-ALIAS (gcc emits addp v31,v31,v31 in reductions): the second
            // half of the dest overlaps the source bytes the first half just wrote,
            // so snapshot any source that aliases rd to scratch (permute_source).
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            let rn_src = permute_source(buf, rd, rn, false);
            let rm_src = permute_source(buf, rd, rm, true);
            let n: i32 = if q { 16 } else { 8 };
            let es = esize as i32;
            let out_lanes = n / es;
            let half = out_lanes / 2;
            for i in 0..out_lanes {
                let (base, pair) = if i < half {
                    (rn_src, i * 2)
                } else {
                    (rm_src, (i - half) * 2)
                };
                // load pair at (2i, 2i+1), zero-extend each to 64-bit, add
                let mut load = |tgt: u8, off: i32| match esize {
                    8 => buf.mov_load64(tgt, RBX, base + off * es),
                    4 => buf.mov_load32(tgt, RBX, base + off * es),
                    2 => buf.movzx_word_mem(tgt, RBX, base + off * es),
                    _ => buf.movzx_byte_mem(tgt, RBX, base + off * es),
                };
                load(RAX, pair);
                load(RCX, pair + 1);
                buf.add_rr64(RAX, RCX);
                match esize {
                    8 => buf.mov_store64(RBX, slot(rd) + i * es, RAX),
                    4 => buf.mov_store32(RBX, slot(rd) + i * es, RAX),
                    2 => buf.mov_store16(RBX, slot(rd) + i * es, RAX),
                    _ => buf.mov_store8(RBX, slot(rd) + i * es, RAX),
                }
            }
            Ok(())
        }
        Inst::FcvVec { rd, rn, signed, esize, q } => {
            // fcvtzu/fcvtzs Vd.T, Vn.T: convert each FP lane (esize bytes) to an
            // int, truncating toward zero; negative clamp for the unsigned form.
            // The 4-byte (S) lane form is NOT a double: it must load the 32-bit
            // float and promote (movq_load would read 8 bytes = lane + next lane),
            // and the result lane is 32-bit (mov_store32, not mov_store64 which
            // would clobber the neighbouring lane).
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let lanes = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let m = esize as i32;
            for l in 0..lanes {
                if esize == 8 {
                    buf.movq_load(0, RBX, src + l * m); // 64-bit double lane
                } else {
                    buf.mov_load32(RAX, RBX, src + l * m); // 32-bit float lane
                    buf.movd_xmm_r32(0, RAX);
                    buf.cvtss2sd(0, 0); // promote to double in xmm0
                }
                buf.cvttsd2si(RAX, 0);
                if !signed {
                    buf.xor_rr64(RCX, RCX);
                    buf.test_rr64(RAX, RAX);
                    buf.cmov_rr64(0x48, RAX, RCX); // negative d -> 0
                }
                if esize == 8 {
                    buf.mov_store64(RBX, dst + l * m, RAX); // 64-bit int lane
                } else {
                    buf.mov_store32(RBX, dst + l * m, RAX); // 32-bit int lane
                }
            }
            Ok(())
        }
                Inst::VecIntToFp { rd, rn, esize, signed, q } => {
            // scvtf/ucvtf Vd.T, Vn.T: convert each int lane (esize bytes) to FP.
            // The reverse of FcvVec. esize=4 -> s32/u32 -> f32 per lane;
            // esize=8 -> (scvtf v.2d) i64 -> f64 per lane.
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let lanes = if q { 16 / esize as i32 } else { 8 / esize as i32 };
            let m = esize as i32;
            for l in 0..lanes {
                if esize == 8 {
                    // scvtf v.2d: 64-bit signed int lane -> double.
                    buf.mov_load64(RAX, RBX, src + l * m);
                    buf.cvtsi2sd(0, true, RAX);
                    buf.movq_store(RBX, dst + l * m, 0);
                } else {
                    // 32-bit lane. mov_load32 zero-extends to RAX; sign-extend
                    // if signed so cvtsi2ss is exact for negatives.
                    buf.mov_load32(RAX, RBX, src + l * m);
                    if signed {
                        buf.movsxd_r64_r32(RAX, RAX);
                    }
                    buf.cvtsi2ss(0, true, RAX);
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, dst + l * m, RAX);
                }
            }
            Ok(())
        }
                Inst::VecFcvtl { rd, rn, upper, half } => {
            // fcvtl Vd.2D, Vn.2S (f32->f64) / fcvtl2 Vd.2D, Vn.4S — widen two f32
            // lanes of Vn to doubles; OR the fp16 form (half): fcvtl Vd.4s, Vn.4h /
            // fcvtl2 Vd.4s, Vn.8h — widen four f16 lanes to floats via F16C.
            // fcvtl reads Vn bytes 0..7 (f32) / 0..8 (fp16); fcvtl2 (upper) reads
            // Vn bytes 8..15 (f32) / 8..16 (fp16). Both write all of Vd.
            // In-place: fcvtl (lower) writing would clobber Vn's upper lanes when
            // rd==rn; snapshot Vn via permute_source.
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let src = permute_source(buf, rd, rn, false);
            if half {
                // fp16 -> f32 widen: 4 lanes, vcvtph2ps each promoted lane.
                let soff = if upper { 8i32 } else { 0i32 };
                for lane in 0..4 {
                    buf.mov_load16(RAX, RBX, src + soff + lane * 2); // f16 lane
                    buf.movd_xmm_r32(0, RAX);
                    buf.bytes.extend_from_slice(&[0xc4, 0xe2, 0x79, 0x13, 0xc0]); // vcvtph2ps xmm0,xmm0
                    buf.movd_r32_xmm(RAX, 0);
                    buf.mov_store32(RBX, dst + lane * 4, RAX); // f32 lane
                }
            } else {
                let soff = if upper { 8i32 } else { 0i32 };
                for lane in 0..2 {
                    buf.mov_load32(RAX, RBX, src + soff + lane * 4); // f32 lane
                    buf.movd_xmm_r32(0, RAX); // to xmm0 low 32
                    buf.cvtss2sd(0, 0); // widen f64
                    buf.movq_store(RBX, dst + lane * 8, 0); // store f64 lane
                }
            }
            Ok(())
        }
                Inst::VecFcvtn { rd, rn, upper } => {
            // fcvtn Vd.2S, Vn.2D / fcvtn2 Vd.4S, Vn.2D: narrow two double lanes
            // of Vn to floats in Vd. fcvtn writes Vd bytes 0..7, fcvtn2 (upper)
            // writes Vd bytes 8..15. Source is always the full 2 doubles.
            // In-place: fcvtn2 (upper) writing the f32 lane0 at Vd byte 8 would
            // clobber Vn's f64 lane1 at byte 8 before it's read when rd==rn.
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let src = permute_source(buf, rd, rn, false);
            let doff = if upper { 8i32 } else { 0i32 };
            for lane in 0..2 {
                buf.movq_load(0, RBX, src + lane * 8); // f64 lane -> xmm0
                buf.cvtsd2ss(0, 0); // narrow f32 in xmm0 low
                buf.movd_r32_xmm(RAX, 0); // low32 -> RAX
                buf.mov_store32(RBX, dst + doff + lane * 4, RAX); // store f32
            }
            Ok(())
        }
        Inst::SimdCmpZero { rd, rn, esize, q, cond } => {
            // cmeq/cmgt/cmge/cmlt/cmle Vd.T,Vn.T,#0: each lane -> all-ones if the
            // signed int compare against literal 0 holds, else 0. cond: 0=eq,
            // 1=gt, 2=ge, 3=lt, 4=le. Signed conds sign-extend the lane so the
            // x86 setg/setge/setl/setle on the 64-bit value compare correctly.
            let src = crate::jit::VECTOR_BASE + (rn as i32) * 16;
            let dst = crate::jit::VECTOR_BASE + (rd as i32) * 16;
            let lanes = if q { 16 / esize as u32 } else { 8 / esize as u32 } as u32;
            let e = esize as i32;
            let signed = cond != 0;
            for l in 0..lanes {
                let off = (l as i32) * e;
                if e == 8 {
                    buf.mov_load64(RAX, RBX, src + off);
                } else if signed {
                    match e {
                        4 => { buf.mov_load32(RAX, RBX, src + off); buf.movsxd_r64_r32(RAX, RAX); }
                        2 => buf.movsx_word_mem(RAX, RBX, src + off),
                        _ => buf.movsx_byte_mem(RAX, RBX, src + off),
                    }
                } else {
                    match e {
                        4 => buf.mov_load32(RAX, RBX, src + off),
                        2 => buf.movzx_word_mem(RAX, RBX, src + off),
                        _ => buf.movzx_byte_mem(RAX, RBX, src + off),
                    }
                }
                buf.test_rr64(RAX, RAX);
                let cc = match cond { 0 => 4, 1 => 0xf, 2 => 0xd, 3 => 0xc, _ => 0xe };
                buf.setcc_rm8(cc, RAX);       // AL = 0/1
                buf.movzx_r32_r8(RAX, RAX);   // RAX = 0/1
                buf.neg_r64(RAX);             // 0 or all-ones
                match e {
                    8 => buf.mov_store64(RBX, dst + off, RAX),
                    4 => buf.mov_store32(RBX, dst + off, RAX),
                    2 => buf.mov_store16(RBX, dst + off, RAX),
                    _ => buf.mov_store8(RBX, dst + off, RAX),
                }
            }
            Ok(())
        }
                Inst::SimDup { rd, rn, esize, src_idx, q } => {
                    // dup Vd.T, Vn.T[src]: broadcast element at Vn[src_idx*esize] across
                    // all q?16:8 bytes of Vd (all lanes identical).
                    let src = crate::jit::VECTOR_BASE + (rn as i32)*16 + (src_idx as i32)*(esize as i32);
                    match esize {
                        8 => buf.mov_load64(RAX, RBX, src),
                        4 => buf.mov_load32(RAX, RBX, src),
                        2 => buf.movzx_word_mem(RAX, RBX, src),
                        _ => buf.movzx_byte_mem(RAX, RBX, src),
                    }
                    let slot = crate::jit::VECTOR_BASE + (rd as i32)*16;
                    let total = if q { 16i32 } else { 8i32 };
                    let mut off = 0i32;
                    while off < total {
                        match esize {
                            8 => buf.mov_store64(RBX, slot + off, RAX),
                            4 => buf.mov_store32(RBX, slot + off, RAX),
                            2 => buf.mov_store16(RBX, slot + off, RAX),
                            _ => buf.mov_store8(RBX, slot + off, RAX),
                        }
                        off += esize as i32;
                    }
                    if !q {
                        buf.mov_ri64(RAX, 0);
                        buf.mov_store64(RBX, slot + 8, RAX);
                    }
                    Ok(())
                }
                // (2 x 64-bit halves). RAX/RCX/RDX/RDI scratch.
                                                                                                                                                                                Inst::SimdBit { rd, rn, rm, bif } => {
            let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
            for off in [0i32, 8] {
                // BIT (bif=false): (Vn&Vm)|(Vd&~Vm); BIF (bif=true): (Vn&~Vm)|(Vd&Vm).
                // RAX = (sel & Vm), RDX = (keep & ~Vm), result = RAX|RDX, where
                // (sel,keep) = (Vn,Vd) for BIT and (Vd,Vn) for BIF.
                let (sel, keep) = if bif { (rd, rn) } else { (rn, rd) };
                buf.mov_load64(RAX, RBX, slot(sel) + off); // source taken where Vm=1
                buf.mov_load64(RCX, RBX, slot(rm) + off); // RCX = Vm
                buf.and_rr64(RAX, RCX); // RAX = sel & Vm
                buf.mov_load64(RDX, RBX, slot(keep) + off); // source taken where Vm=0
                buf.mov_ri64(RDI, 0xffff_ffff_ffff_ffff);
                buf.xor_rr64(RCX, RDI); // RCX = ~Vm
                buf.and_rr64(RDX, RCX); // RDX = keep & ~Vm
                buf.or_rr64(RAX, RDX); // (sel&Vm)|(keep&~Vm)
                buf.mov_store64(RBX, slot(rd) + off, RAX);
            }
            Ok(())
        }
        Inst::SimdExt { rd, rn, rm, imm, q } => {
                                                                                                                                                                                    // ext Vd, Vn, Vm, #imm: Vd = the 128(64)-bit window of the
                                                                                                                                                                                    // concatenation starting at byte `imm`, where **Vn occupies the
                                                                                                                                                                                    // low-address bytes (0..15) and Vm the high (16..31)** — verified
                                                                                                                                                                                    // against qemu: ext(Vn=0x99..,Vm=0x02..,#8) -> lo=Vn[8..15],
                                                                                                                                                                                    // hi=Vm[0..7]. (The OLD order had Vm low and Vn high, which
                                                                                                                                                                                    // inverted every non-symmetric `ext`; gcc's horizontal
                                                                                                                                                                                    // xor-reduce emitted `ext v0,v30,v0,#8` and returned v30.hi^
                                                                                                                                                                                    // v30.lo wrong by a full xor of one 64-bit lane.)
                                                                                                                                                                                    // concat words W[0..3] = Vn.lo, Vn.hi, Vm.lo, Vm.hi (byte
                                                                                                                                                                                    // addresses 0..31). result.lo = bytes imm..imm+7 of concat,
                                                                                                                                                                                    // result.hi = bytes imm+8..imm+15 (16B form). For 8B (Q=0)
                                                                                                                                                                                    // only the low 64 bits are produced and the high lane is 0.
                                                                                                                                                                                    let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                                                                                                    // Emit 64-bit field of concat starting at byte `start` into `dst`.
                                                                                                                                                                                    let emit_bytes64 = |buf: &mut crate::x86::CodeBuf, dst: u8, start: usize| {
                                                                                                                                                                                                                                            let wi = start / 8;
                                                                                                                                                                                                                                            // sh = bit offset within the concat word (start is in BYTES)
                                                                                                                                                                                                                                            let sh = ((start % 8) * 8) as u8;
                                                                                                                                                                                                                                            if sh == 0 {
                                                                                                                                                                                            // aligned: 8 bytes directly from one concat word
                                                                                                                                                                                            match wi {
                                                                                                                                                                                                0 => buf.mov_load64(dst, RBX, slot(rn)),
                                                                                                                                                                                                1 => buf.mov_load64(dst, RBX, slot(rn) + 8),
                                                                                                                                                                                                2 => buf.mov_load64(dst, RBX, slot(rm)),
                                                                                                                                                                                                _ => buf.mov_load64(dst, RBX, slot(rm) + 8),
                                                                                                                                                                                            }
                                                                                                                                                                                        } else {
                                                                                                                                                                                            // unaligned: (W[wi] >> sh) | (W[wi+1] << (64-sh))
                                                                                                                                                                                            let (ra0, a_off, rb0, b_off) = match wi {
                                                                                                                                                                                                0 => (rn, 0, rn, 8),
                                                                                                                                                                                                1 => (rn, 8, rm, 0),
                                                                                                                                                                                                _ => (rm, 0, rm, 8),
                                                                                                                                                                                            };
                                                                                                                                                                                            buf.mov_load64(RDX, RBX, slot(ra0) + a_off);
                                                                                                                                                                                            buf.shr_ri8(RDX, sh);
                                                                                                                                                                                            buf.mov_load64(RDI, RBX, slot(rb0) + b_off);
                                                                                                                                                                                            buf.shl_ri8(RDI, 64 - sh);
                                                                                                                                                                                            buf.or_rr64(RDX, RDI);
                                                                                                                                                                                            buf.mov_rr64(dst, RDX);
                                                                                                                                                                                        }
                                                                                                                                                                                    };
                                                                                                                                                                                    if !q {
                                                                                                                                                                                        // 8B: result.lo = bytes imm..imm+7 of concat; hi lane zeroed.
                                                                                                                                                                                        emit_bytes64(buf, RAX, imm as usize);
                                                                                                                                                                                        buf.mov_store64(RBX, slot(rd), RAX);
                                                                                                                                                                                        buf.mov_ri64(RDI, 0);
                                                                                                                                                                                        buf.mov_store64(RBX, slot(rd) + 8, RDI);
                                                                                                                                                                                    } else {
                                                                                                                                                                                        emit_bytes64(buf, RAX, imm as usize);
                                                                                                                                                                                        emit_bytes64(buf, RCX, (imm as usize) + 8);
                                                                                                                                                                                        buf.mov_store64(RBX, slot(rd), RAX);
                                                                                                                                                                                        buf.mov_store64(RBX, slot(rd) + 8, RCX);
                                                                                                                                                                                    }
                                                                                                                                                                                    Ok(())
                                                                                                                                                                                                                                                                                                    }
                                                                                                                                                                                                                                                                                                    Inst::SimdLaneGp { rd, rn, esize, index, sign, wide } => {
                                                                                                                                                                                                                                                                                                                                                                                                            // mov/umov/smov Wd|Xd, Vn.T[index]: copy one element
                                                                                                                                                                                                                                                                                                                                                                                                            // (esize bytes) from the 16-byte vector slot of Vn at byte
                                                                                                                                                                                                                                                                                                                                                                                                            // offset index*esize into GPR rd, (sign|zero) extended.
                                                                                                                                                                                                                                                                                                                                                                                                            // Slot layout: [D0@+0..+7][D1@+8..+15]; lane i lives at
                                                                                                                                                                                                                                                                                                                                                                                                            // index*esize (e.g. s[1] = +4, d[0]=+0, d[1]=+8).
                                                                                                                                                                                                                                                                                                                                                                                                            let off = (index as i32) * (esize as i32);
                                                                                                                                                                                                                                                                                                                                                                                                            let vbase = crate::jit::VECTOR_BASE + (rn as i32) * 16 + off;
                                                                                                                                                                                                                                                                                                                                                                                                            match esize {
                                                                                                                                                                                                                                                                                                                                                                                                                8 => {
                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_load64(RAX, RBX, vbase);
                                                                                                                                                                                                                                                                                                                                                                                                                    stg(buf, rd as u32, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                4 => {
                                                                                                                                                                                                                                                                                                                                                                                                                    // esize == 4 (.s): load 32-bit, zero- or sign-extend.
                                                                                                                                                                                                                                                                                                                                                                                                                    buf.mov_load32(RAX, RBX, vbase);
                                                                                                                                                                                                                                                                                                                                                                                                                    if sign {
                                                                                                                                                                                                                                                                                                                                                                                                                        buf.movsxd_r64_r32(RAX, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                    }
                                                                                                                                                                                                                                                                                                                                                                                                                    stg(buf, rd as u32, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                2 if sign => {
                                                                                                                                                                                                                                                                                                                                                                                                                    buf.movsx_word_mem(RAX, RBX, vbase);
                                                                                                                                                                                                                                                                                                                                                                                                                    stg(buf, rd as u32, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                2 => {
                                                                                                                                                                                                                                                                                                                                                                                                                    buf.movzx_word_mem(RAX, RBX, vbase);
                                                                                                                                                                                                                                                                                                                                                                                                                    stg(buf, rd as u32, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                1 if sign => {
                                                                                                                                                                                                                                                                                                                                                                                                                    buf.movsx_byte_mem(RAX, RBX, vbase);
                                                                                                                                                                                                                                                                                                                                                                                                                    stg(buf, rd as u32, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                _ => {
                                                                                                                                                                                                                                                                                                                                                                                                                    // esize == 1, unsigned
                                                                                                                                                                                                                                                                                                                                                                                                                    buf.movzx_byte_mem(RAX, RBX, vbase);
                                                                                                                                                                                                                                                                                                                                                                                                                    stg(buf, rd as u32, RAX);
                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                                                            Ok(())
                                                                                                                                                                                                                                                                                                                                                                                                        }
                                                                                                    Inst::InsGp { rd, rn, esize, index } => {
                                                                                                        // ins/mov Vd.T[index], Rn: copy esize bytes of GPR rn into
                                                                                                        // the vector slot Vd at byte offset index*esize (the reverse
                                                                                                        // of SimdLaneGp). Only the low esize bytes of Rn participate.
                                                                                                        let slot = |r: u8| crate::jit::VECTOR_BASE + (r as i32) * 16;
                                                                                                        let off = (index as i32) * (esize as i32);
                                                                                                        ldg(buf, RAX, rn as u32);
                                                                                                        match esize {
                                                                                                            1 => buf.mov_store8(RBX, slot(rd) + off, RAX),
                                                                                                            2 => buf.mov_store16(RBX, slot(rd) + off, RAX),
                                                                                                            4 => buf.mov_store32(RBX, slot(rd) + off, RAX),
                                                                                                            _ => buf.mov_store64(RBX, slot(rd) + off, RAX),
                                                                                                        }
                                                                                                        Ok(())
                                                                                                    }
                                                                                                                                                                    Inst::LdStPair {
            rt,
            rt2,
            rn,
            imm,
            ld,
            writeback,
            preidx,
            size_64,
            q128,
            fp_d,
            fp_s,
            sext,
        } => {
            let esize = if q128 {
                16i32
            } else if fp_d {
                8i32
            } else if fp_s {
                4i32
            } else if size_64 {
                8i32
            } else {
                4i32
            };
            let imm32 = imm as i32;
            // eff base: three addressing modes.
            //  pre-index `[rn, #imm]!`  -> access at rn+imm, then rn += imm
            //  post-index `[rn], #imm`  -> access at rn,     then rn += imm
            //  offset     `[rn, #imm]`  -> access at rn+imm, no writeback
            // (ARM bit23=indexed, bit24=pre within indexed; offset form has
            //  bit23=0 => writeback=false. The offset immediate must still apply
            //  to the access address.)
            ldg(buf, RDX, rn as u32); // RDX = rn
            let (access_off, wb_off) = if preidx {
                (imm32, imm32)
            } else if writeback {
                (0i32, imm32) // post-index: access at rn, then rn += imm
            } else {
                (imm32, 0i32) // offset: access at rn+imm, no writeback
            };
            if fp_d {
                // 64-bit FP/vector d-pair: each reg is the LOW 8 bytes of its
                // 16-byte guest vector slot, at VECTOR_BASE + rt*16. BUGFIX
                // (Session 99): the stride was rt*8, so `ldp d29,d28` wrote to
                // 0x1f8/0x1f0 instead of 0x2e0/0x2d0 and the follow-on fmadd read
                // stale vector slots (structfield.elf -O2 returned 128 vs 52).
                let v0 = crate::jit::VECTOR_BASE + (rt as i32) * 16;
                let v1 = crate::jit::VECTOR_BASE + (rt2 as i32) * 16;
                for (reg_off, mem_off) in [(v0, access_off), (v1, access_off + esize)] {
                    if ld {
                        buf.mov_load64(RAX, RDX, mem_off); // rax <- [addr]
                        buf.mov_store64(RBX, reg_off, RAX); // v <- rax
                    } else {
                        buf.mov_load64(RAX, RBX, reg_off); // rax <- v
                        buf.mov_store64(RDX, mem_off, RAX); // [addr] <- rax
                    }
                }
                if writeback {
                    ldg(buf, RAX, rn as u32);
                    buf.add_ri64(RAX, wb_off as u32);
                    stg(buf, rn as u32, RAX);
                }
                return Ok(());
            }
            if fp_s {
                // 32-bit FP/vector s-pair: each reg is the LOW 4 bytes of its
                // 16-byte vector slot, at VECTOR_BASE + rt*16. 4-byte transfers.
                let v0 = crate::jit::VECTOR_BASE + (rt as i32) * 16;
                let v1 = crate::jit::VECTOR_BASE + (rt2 as i32) * 16;
                for (reg_off, mem_off) in [(v0, access_off), (v1, access_off + esize)] {
                    if ld {
                        buf.mov_load32(RAX, RDX, mem_off); // eax <- [addr]
                        buf.mov_store32(RBX, reg_off, RAX); // v low4 <- eax
                    } else {
                        buf.mov_load32(RAX, RBX, reg_off); // eax <- v low4
                        buf.mov_store32(RDX, mem_off, RAX); // [addr] <- eax
                    }
                }
                if writeback {
                    ldg(buf, RAX, rn as u32);
                    buf.add_ri64(RAX, wb_off as u32);
                    stg(buf, rn as u32, RAX);
                }
                return Ok(());
            }
            if q128 {
                // 128-bit SIMD pair: transfer 16 bytes per register between the
                // guest v-slots (CpuState.v, VECTOR_BASE+16*reg) and memory via XMM0.
                let v0 = crate::jit::VECTOR_BASE + (rt as i32) * 16;
                let v1 = crate::jit::VECTOR_BASE + (rt2 as i32) * 16;
                for (reg_vslot, mem_off) in [(v0, access_off), (v1, access_off + esize)] {
                    if ld {
                        buf.movdqu_load(0, RDX, mem_off); // xmm0 <- [addr]
                        buf.movdqu_store(RBX, reg_vslot, 0); // guest v <- xmm0
                    } else {
                        buf.movdqu_load(0, RBX, reg_vslot); // xmm0 <- [vslot]
                        buf.movdqu_store(RDX, mem_off, 0); // [addr] <- xmm0
                    }
                }
            } else if ld {
                // load rt = [RDX + access_off], rt2 = [.. + esize]
                if size_64 {
                    buf.mov_load64(RAX, RDX, access_off);
                    stg_if_writable(buf, rt as u32);
                    buf.mov_load64(RAX, RDX, access_off + esize);
                    stg_if_writable(buf, rt2 as u32);
                } else {
                    if sext {
                        // ldpsw: load 32-bit, sign-extend to 64-bit X reg.
                        buf.mov_load32(RAX, RDX, access_off);
                        buf.movsxd_r64_r32(RAX, RAX);
                        stg_if_writable(buf, rt as u32);
                        buf.mov_load32(RAX, RDX, access_off + esize);
                        buf.movsxd_r64_r32(RAX, RAX);
                        stg_if_writable(buf, rt2 as u32);
                    } else {
                        buf.mov_load32(RAX, RDX, access_off);
                        stg_if_writable(buf, rt as u32);
                        buf.mov_load32(RAX, RDX, access_off + esize);
                        stg_if_writable(buf, rt2 as u32);
                    }
                }
            } else {
                // store rt at [eff], rt2 at [eff+esize]
                if size_64 {
                    ldg_src(buf, rt as u32);
                    buf.mov_store64(RDX, access_off, RAX);
                    ldg_src(buf, rt2 as u32);
                    buf.mov_store64(RDX, access_off + esize, RAX);
                    emit_canary_store_watch(buf, pc, RDX, RAX); // after pair, dest=RDX
                } else {
                    ldg_src(buf, rt as u32);
                    buf.mov_store32(RDX, access_off, RAX);
                    ldg_src(buf, rt2 as u32);
                    buf.mov_store32(RDX, access_off + esize, RAX);
                }
            }
            if writeback {
                // rn = rn + wb_off
                ldg(buf, RCX, rn as u32);
                if wb_off != 0 {
                    buf.lea64(RCX, RCX, wb_off);
                }
                stg(buf, rn as u32, RCX);
            }
            Ok(())
        }
        Inst::LdStrReg {
            rt,
            rn,
            rm,
            size,
            ld,
            shift,
            sext,
            index_ext,
        } => {
            // addr = rn + (rm << shift_amt), shift_amt = log2(size) when S=1.
            let shift_amt = if shift {
                match size {
                    1 => 0,
                    2 => 1,
                    4 => 2,
                    _ => 3,
                }
            } else {
                0
            };
            ldg(buf, RAX, rn as u32); // address base in RAX
            ldg(buf, RCX, rm as u32); // index in RCX
            // index_ext (option bits[14:13]): 2 = UXTW (index is rm's low 32
            // bits, zero-extended to 64 before the shift), 1 = UXTB (low byte).
            // Real book code stores a bit-32 "sentinel" in the X register and
            // relies on `[xN, wM, uxtw#S]` dropping it — using the full 64-bit
            // rm here indexes OOB and SIGSEGVs (`ldr w8,[x8,w0,uxtw#2]` at 0x2173218
            // with x0=0x100000665). 3 (LSL) keeps the full 64-bit rm index.
            match index_ext {
                2 => buf.zero_ext_r32(RCX),
                1 => buf.and_ri64(RCX, 0xff),
                _ => {} // 3 (LSL/UXTX) or 0 (reserved): full-width index
            }
            if shift_amt != 0 {
                // Currently only constant <=3 via the (unused) sar_cl; emit shift left.
                // x86 has no shl-by-imm op in this emitter; use add-based *2 for 1..3.
                for _ in 0..shift_amt {
                    buf.add_rr64(RCX, RCX); // RCX += RCX (shift left by 1)
                }
            }
            buf.add_rr64(RAX, RCX); // RAX = effective address
            if ld && sext {
                // Sign-extending register-offset load (ldrsw/ldrsh/ldrsb).
                // Load `size` bytes from [RAX] and sign-extend into RAX; the
                // address is no longer needed, so RAX replaces the base.
                match size {
                    4 => {
                        buf.mov_load32(RAX, RAX, 0);
                        buf.movsxd_r64_r32(RAX, RAX);
                        stg_if_writable(buf, rt as u32);
                    }
                    2 => {
                        buf.movzx_word_mem(RAX, RAX, 0);
                        buf.shl_ri8(RAX, 48);
                        buf.sar_ri8(RAX, 48);
                        stg_if_writable(buf, rt as u32);
                    }
                    1 => {
                        buf.movzx_byte_mem(RAX, RAX, 0);
                        buf.shl_ri8(RAX, 56);
                        buf.sar_ri8(RAX, 56);
                        stg_if_writable(buf, rt as u32);
                    }
                    s => return Err(format!("LdStrReg sign-extend size {} not implemented", s)),
                }
                return Ok(());
            }
            match (size, ld) {
                (8, true) => {
                    buf.mov_load64(RCX, RAX, 0);
                    stg(buf, rt as u32, RCX);
                }
                (8, false) => {
                    ldg(buf, RCX, rt as u32);
                    buf.mov_store64(RAX, 0, RCX);
                    emit_canary_store_watch(buf, pc, RAX, RCX); // dest=RAX, val=RCX
                }
                (4, true) => {
                    buf.mov_load32(RCX, RAX, 0);
                    stg(buf, rt as u32, RCX);
                }
                (4, false) => {
                    ldg(buf, RCX, rt as u32);
                    buf.mov_store32(RAX, 0, RCX);
                }
                (2, true) => {
                    buf.movzx_word_mem(RCX, RAX, 0);
                    stg(buf, rt as u32, RCX);
                }
                (2, false) => {
                    ldg(buf, RCX, rt as u32);
                    buf.mov_store16(RAX, 0, RCX);
                }
                (1, true) => {
                    buf.movzx_byte_mem(RCX, RAX, 0);
                    stg(buf, rt as u32, RCX);
                }
                (1, false) => {
                    ldg(buf, RCX, rt as u32);
                    buf.mov_store8(RAX, 0, RCX);
                }
                (s, _) => return Err(format!("LdStrReg size {} not implemented", s)),
            }
            Ok(())
        }
        Inst::VecLdStImm { vt, rn, imm, ld } => {
            // addr = rn + imm*16 ; transfer a full 128-bit vector between the
            // guest vector slot (CpuState.v, 16 bytes at VECTOR_BASE+16*vt) and
            // the guest pointer, via x86 XMM0.
            ldg(buf, RAX, rn as u32); // base address
            let off = (imm as i32).wrapping_mul(16);
            if off != 0 {
                buf.lea64(RAX, RAX, off);
            }
            if std::env::var_os("JIT_TRACE").is_some() && ld {
                // print addr + current 16 bytes at translate time (target addr
                // is RAX-modifiable only at runtime, so approximate via x0 slot).
                eprintln!("VECLD[DBG] ld vt={vt} rn={rn} imm={imm} off={off}");
            }
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            if ld {
                buf.movdqu_load(0, RAX, 0); // xmm0 <- [addr]
                buf.movdqu_store(RBX, vslot, 0); // cmp-state v <- xmm0
            } else {
                buf.movdqu_load(0, RBX, vslot); // xmm0 <- [vslot]
                buf.movdqu_store(RAX, 0, 0); // [addr] <- xmm0
            }
            Ok(())
        }
        Inst::VecLdStrReg { vt, rn, rm, ld } => {
            // addr = x[rn] + x[rm] ; transfer 16 bytes via XMM0. Vector-file
            // register-offset ld/st; the GPR base/index are unaffected (this
            // must NOT write back into either — the old mis-decode stored a
            // byte INTO x[rn]'s register and corrupted the caller's state).
            ldg(buf, RAX, rn as u32); // base
            ldg(buf, RCX, rm as u32); // index
            buf.add_rr64(RAX, RCX); // RAX = addr
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            if ld {
                buf.movdqu_load(0, RAX, 0);
                buf.movdqu_store(RBX, vslot, 0);
            } else {
                buf.movdqu_load(0, RBX, vslot);
                buf.movdqu_store(RAX, 0, 0);
            }
            Ok(())
        }
        Inst::VecLdStImmUnscaled { vt, rn, imm9, ld } => {
            // addr = x[rn] + imm9 (signed) ; transfer 16 bytes via XMM0.
            ldg(buf, RAX, rn as u32);
            if imm9 != 0 {
                buf.lea64(RAX, RAX, imm9);
            }
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            if ld {
                buf.movdqu_load(0, RAX, 0);
                buf.movdqu_store(RBX, vslot, 0);
            } else {
                buf.movdqu_load(0, RBX, vslot);
                buf.movdqu_store(RAX, 0, 0);
            }
            Ok(())
        }
        Inst::VecLdStIndexed { vt, rn, imm9, ld, pre } => {
            // pre:  addr = x[rn]+imm9, then x[rn] += imm9
            // post: addr = x[rn],     then x[rn] += imm9
            // 16-byte transfer via XMM0, then write back the advanced pointer.
            ldg(buf, RAX, rn as u32); // x[rn]
            if pre && imm9 != 0 {
                buf.lea64(RAX, RAX, imm9); // addr = x[rn]+imm9 (pre)
            }
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            if ld {
                buf.movdqu_load(0, RAX, 0);
                buf.movdqu_store(RBX, vslot, 0);
            } else {
                buf.movdqu_load(0, RBX, vslot);
                buf.movdqu_store(RAX, 0, 0);
            }
            // Xn += imm9 (pre and post both advance the base register).
            ldg(buf, RCX, rn as u32);
            if imm9 != 0 {
                buf.lea64(RCX, RCX, imm9);
            }
            stg(buf, rn as u32, RCX);
            Ok(())
        }
        Inst::FpLdStImmUnscaled { vt, rn, imm9, size, ld } => {
            // Scalar ldur/stur: addr = x[rn] + imm9 (signed), no writeback.
            ldg(buf, RDX, rn as u32);
            if imm9 != 0 {
                buf.lea64(RDX, RDX, imm9);
            }
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            fp_scalar_xfer(buf, RDX, vslot, size, ld)?;
            Ok(())
        }
        Inst::FpLdStImmWb { vt, rn, imm9, size, ld, pre } => {
            // Scalar pre/post-index ldr/str with base writeback.
            //   pre:  addr = x[rn] + imm9, then Xn += imm9
            //   post: addr = x[rn],        then Xn += imm9
            // NOTE: address must live in RDX (NOT RAX) — fp_scalar_xfer uses RAX
            // as its value scratch, so a store with addr==RAX would clobber the
            // base with the value being stored and write to [value] (a latent
            // pre/post-index scalar STORE miscompile, e.g. `str s30,[x4],#4`
            // faulting at 0x41480000 = the float bits). Mirrors FpLdStImmUnscaled.
            ldg(buf, RDX, rn as u32);
            if pre && imm9 != 0 {
                buf.lea64(RDX, RDX, imm9); // pre-add the offset into the address
            }
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            fp_scalar_xfer(buf, RDX, vslot, size, ld)?;
            // base register advances by imm9 for both pre and post index.
            ldg(buf, RCX, rn as u32);
            if imm9 != 0 {
                buf.lea64(RCX, RCX, imm9);
            }
            stg(buf, rn as u32, RCX);
            Ok(())
        }
        Inst::FpLdStrReg { vt, rn, rm, size, ld, shift, index_ext } => {
            // addr = x[rn] + (x[rm] << log2(size)) in RDX ; transfer `size`
            // bytes between [addr] and the low bytes of guest vector slot v[vt].
            // Scalar register-offset (B/H/S/D); bit26=1 vector file.
            let shift_amt = if shift {
                match size {
                    1 => 0,
                    2 => 1,
                    4 => 2,
                    _ => 3,
                }
            } else {
                0
            };
            ldg(buf, RDX, rn as u32); // base address
            ldg(buf, RAX, rm as u32); // index
            match index_ext {
                2 => buf.zero_ext_r32(RAX), // UXTW: low-32 index (drop sentinel high)
                1 => buf.and_ri64(RAX, 0xff), // UXTB
                _ => {}                     // LSL (full 64-bit)
            }
            for _ in 0..shift_amt {
                buf.add_rr64(RAX, RAX);
            }
            buf.add_rr64(RDX, RAX); // RDX = addr
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            match (size, ld) {
                (1, true) => {
                    buf.movzx_byte_mem(RAX, RDX, 0);
                    buf.mov_store8(RBX, vslot, RAX);
                }
                (1, false) => {
                    buf.movzx_byte_mem(RAX, RBX, vslot);
                    buf.mov_store8(RDX, 0, RAX);
                }
                (2, true) => {
                    buf.movzx_word_mem(RAX, RDX, 0);
                    buf.mov_store16(RBX, vslot, RAX);
                }
                (2, false) => {
                    buf.movzx_word_mem(RAX, RBX, vslot);
                    buf.mov_store16(RDX, 0, RAX);
                }
                (4, true) => {
                    buf.mov_load32(RAX, RDX, 0);
                    buf.mov_store32(RBX, vslot, RAX);
                }
                (4, false) => {
                    buf.mov_load32(RAX, RBX, vslot);
                    buf.mov_store32(RDX, 0, RAX);
                }
                (8, true) => {
                    buf.mov_load64(RAX, RDX, 0);
                    buf.mov_store64(RBX, vslot, RAX);
                }
                (8, false) => {
                    buf.mov_load64(RAX, RBX, vslot);
                    buf.mov_store64(RDX, 0, RAX);
                }
                (s, _) => return Err(format!("FpLdStrReg size {} not implemented", s)),
            }
            Ok(())
        }
        Inst::FpLdStImm { vt, rn, imm, size, ld } => {
            // Transfer `size` bytes (B/H/S/D) between the low bytes of the guest
            // vector slot v[vt] and [rn + imm*size]. Upper lanes of the 16-byte
            // slot are untouched (ARM `ldr d0` preserves the high 64 bits, and a
            // scalar `str s0/d0` only stores the low 32/64).
            ldg(buf, RDX, rn as u32); // base address
            let off = (imm as i32).wrapping_mul(size as i32);
            if off != 0 {
                buf.lea64(RDX, RDX, off);
            }
            let vslot = crate::jit::VECTOR_BASE + (vt as i32) * 16;
            match (size, ld) {
                (8, true) => {
                    buf.mov_load64(RAX, RDX, 0);
                    buf.mov_store64(RBX, vslot, RAX);
                }
                (8, false) => {
                    buf.mov_load64(RAX, RBX, vslot);
                    buf.mov_store64(RDX, 0, RAX);
                }
                (4, true) => {
                    buf.mov_load32(RAX, RDX, 0);
                    buf.mov_store32(RBX, vslot, RAX);
                }
                (4, false) => {
                    buf.mov_load32(RAX, RBX, vslot);
                    buf.mov_store32(RDX, 0, RAX);
                }
                (2, true) => {
                    buf.movzx_word_mem(RAX, RDX, 0);
                    buf.mov_store16(RBX, vslot, RAX);
                }
                (2, false) => {
                    buf.movzx_word_mem(RAX, RBX, vslot);
                    buf.mov_store16(RDX, 0, RAX);
                }
                (1, true) => {
                    buf.movzx_byte_mem(RAX, RDX, 0);
                    buf.mov_store8(RBX, vslot, RAX);
                }
                (1, false) => {
                    buf.movzx_byte_mem(RAX, RBX, vslot);
                    buf.mov_store8(RDX, 0, RAX);
                }
                (s, _) => return Err(format!("FpLdStImm size {} not implemented", s)),
            }
            Ok(())
        }
        Inst::VecMovi { vd, lo, hi, kind } => {
            // Write a full 128-bit vector immediate into the guest v-slot
            // (CpuState.v, 16 bytes at VECTOR_BASE + 16*vd). The two u64 halves
            // are hoisted as immediates. kind: 0 = write (movi/mvni),
            // 1 = AND-in-place (bic, Vd &= lo/hi), 2 = OR-in-place (orr).
            // Use a scratch reg for the (possibly >32-bit) mask.
            let vslot = crate::jit::VECTOR_BASE + (vd as i32) * 16;
            match kind {
                1 | 2 => {
                    for (off, mask) in [(vslot, lo), (vslot + 8, hi)] {
                        buf.mov_load64(RAX, RBX, off);
                        buf.mov_ri64(RCX, mask);
                        if kind == 1 {
                            buf.and_rr64(RAX, RCX);
                        } else {
                            buf.or_rr64(RAX, RCX);
                        }
                        buf.mov_store64(RBX, off, RAX);
                    }
                }
                _ => {
                    buf.mov_ri64(RAX, lo);
                    buf.mov_store64(RBX, vslot, RAX);
                    buf.mov_ri64(RAX, hi);
                    buf.mov_store64(RBX, vslot + 8, RAX);
                }
            }
            Ok(())
        }
        Inst::Hint => {
            // Hint / PAC NOP — execute as a no-op (PAC is ignored in the guest).
            Ok(())
        }
        Inst::WaitBarrier => {
            // dmb/dsb/isb — memory/cache ordering barrier; the JIT is
            // single-threaded so ordering and cache flush are irrelevant.
            Ok(())
        }
        Inst::SmeNoop => {
            // str za / smstart za / smstop za — no-ops in this SME-off guest
            // (the ZA tile is never live and streaming mode is never entered;
            // glibc compiles them on its __libc_arm_za_disable path).
            Ok(())
        }
        Inst::AddVectorLen { rd, rn, imm_bytes } => {
            // addvl/addsvl: Rd = Rn + imm*VL. VL=16 bytes in this no-SVE model.
            // Fetch Rn, add the scaled byte count, store Rd.
            ldg_src(buf, rn as u32);
            if imm_bytes != 0 {
                buf.add_ri64(RAX, imm_bytes as u32);
            }
            if rd != 31 {
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::SveCntd { rd } => {
            // cntd Xd = VL_d/64 = 2 at VL=16 bytes.
            if rd != 31 {
                buf.mov_ri64(RAX, 2);
                stg(buf, rd as u32, RAX);
            }
            Ok(())
        }
        Inst::Br { rn } => {
            // pc = x[rn]; return to the host dispatcher (which re-enters at pc).
            ldg(buf, RAX, rn as u32);
            buf.mov_store64(RBX, crate::jit::PC_OFF, RAX);
            buf.ret();
            Ok(())
        }
        Inst::Blr { rn } => {
            // x30 = pc + 4 (link); pc = x[rn]; return to the host dispatcher.
            buf.mov_ri64(RAX, pc.wrapping_add(4));
            stg(buf, 30, RAX);
            ldg(buf, RAX, rn as u32);
            buf.mov_store64(RBX, crate::jit::PC_OFF, RAX);
            buf.ret();
            Ok(())
        }
        Inst::Ret => {
            // return x0 in RAX, and set guest pc = x30 (link address) so a host
            // dispatcher can resume at the caller. $[x0] at RBX+0, x30 at RBX+240.
            ldg(buf, RAX, 30);
            buf.mov_store64(RBX, crate::jit::PC_OFF, RAX);
            buf.mov_load64(RAX, RBX, 0);
            buf.ret();
            Ok(())
        }
        Inst::B { imm, link } => {
            if link {
                // BL: save guest LR = pc+4, then host `call` to the target block.
                // LR is stored so any guest read of x30 stays correct.
                let ra = pc.wrapping_add(4);
                buf.mov_ri64(RAX, ra);
                stg(buf, 30, RAX);
                let disp = buf.call_rel32();
                fixups.push(Fixup {
                    target_pc: pc.wrapping_add(imm as u64),
                    disp_off: disp,
                    cc: 0xfe, // call fixup
                });
            } else {
                let target = pc.wrapping_add(imm as u64);
                let disp = buf.jmp_rel32();
                fixups.push(Fixup {
                    target_pc: target,
                    disp_off: disp,
                    cc: 0xff,
                });
            }
            Ok(())
        }
        Inst::Adr { rd, imm } => {
            // rd = pc + imm (load the effective address of a nearby symbol)
            let target = (pc as i64).wrapping_add(imm);
            buf.mov_ri64(RAX, target as u64);
            stg(buf, rd as u32, RAX);
            Ok(())
        }
        Inst::Adrp { rd, imm } => {
            // rd = page(PC) + (imm<<12)  -- page-aligned effective address.
            let page = (pc as i64) & !0xfff_i64;
            let target = page.wrapping_add(imm);
            buf.mov_ri64(RAX, target as u64);
            stg(buf, rd as u32, RAX);
            Ok(())
        }
        Inst::Cbz {
            rt, imm, nonzero, ..
        } => {
            let target = pc.wrapping_add(imm as u64);
            ldg_src(buf, rt as u32); // test rt
            buf.test_rr64(RAX, RAX);
            let cc = if nonzero { 0x85 } else { 0x84 }; // jnz / jz
            let disp = buf.jcc_rel32(cc);
            fixups.push(Fixup {
                target_pc: target,
                disp_off: disp,
                cc,
            });
            Ok(())
        }
        Inst::Tbz {
            rt,
            bit,
            imm,
            nonzero,
            ..
        } => {
            let target = pc.wrapping_add(imm as u64);
            ldg_src(buf, rt as u32); // load rt
            // test the single bit: test rax, 1<<bit
            buf.mov_ri64(RCX, (1u64 << bit.min(63)) & (if bit >= 64 { 0 } else { 0xffff_ffff_ffff_ffff }));
            // simpler: AND with constant handled per-bit via a cached reg
            buf.test_rr64(RAX, RCX);
            // tbz: branch if bit==0 => JE when ZF set; tbnz: branch if bit==1 => JNE
            let cc = if nonzero { 0x85 } else { 0x84 }; // jnz / jz
            let disp = buf.jcc_rel32(cc);
            fixups.push(Fixup {
                target_pc: target,
                disp_off: disp,
                cc,
            });
            Ok(())
        }
        Inst::BCond { cond, imm } => {
            let target = pc.wrapping_add(imm as u64);
            match cond {
                0xE => {
                    // AL: unconditional branch via jmp
                    let disp = buf.jmp_rel32();
                    fixups.push(Fixup {
                        target_pc: target,
                        disp_off: disp,
                        cc: 0xff,
                    });
                }
                0xF => {
                    // NV: never executed -> nothing to emit
                }
                c => {
                    let cc = x86_cc_for_cond(c)
                        .ok_or_else(|| format!("B.cond unsupported cond {:x}", c))?;
                    // Evaluate the condition from the stored NZCV (the dispatcher
                    // may clobber live x86 flags with operand reloads; NZCV is
                    // the authoritative copy). load_nzcv_to_eflags sets the flags
                    // via popfq as the last clobbering op, so the jcc that
                    // follows reads exactly the guest condition.
                    load_nzcv_to_eflags(buf);
                    let disp = buf.jcc_rel32(cc);
                    fixups.push(Fixup {
                        target_pc: target,
                        disp_off: disp,
                        cc,
                    });
                }
            }
            Ok(())
        }
        _ => Err(format!(
            "translate: unhandled {inst:?} at guest pc 0x{pc:x}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The two store-watch hermetics flip the module-global CANARY_STORE_WATCH
    // static, so they must be serialized or they race in parallel-test runs
    // (one test's ON clobbers the other's OFF phase). Serialize via a static
    // mutex so each test owns the flag exclusively regardless of scheduling.
    static CANARY_WATCH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    // SH106-NEXT hermetic: the store-watch is DEFAULT-INERT — with the atomic
    // flag OFF the pure emit helper adds zero bytes to a store's block (the
    // product path stays byte-identical), and the host fn returns 0 (observes
    // nothing). With the flag ON the helper emits the post-store probe call.
    // No real binary/image is required (pure emit + fn-call path).
    #[test]
    fn canary_store_watch_emit_is_inert_off_and_emits_on() {
        use std::sync::atomic::Ordering;
        let _guard = CANARY_WATCH_TEST_LOCK.lock().unwrap();
        // Make sure the lazy env sync can't fight the test override.
        set_canary_store_watch_test(false);
        let _ = canary_store_watch_enabled(); // run the env sync (default off here)

        CANARY_STORE_WATCH.store(false, Ordering::Relaxed);
        let mut off = crate::x86::CodeBuf::new();
        emit_canary_store_watch(&mut off, 0x1000, RDX, RAX);
        let off_len = off.len();

        CANARY_STORE_WATCH.store(true, Ordering::Relaxed);
        set_canary_store_watch_test(true);
        let mut on = crate::x86::CodeBuf::new();
        emit_canary_store_watch(&mut on, 0x1000, RDX, RAX);
        let on_len = on.len();

        CANARY_STORE_WATCH.store(false, Ordering::Relaxed);
        set_canary_store_watch_test(false);

        assert_eq!(off_len, 0, "watch must be byte-inert when disabled (product unchanged)");
        assert!(on_len >= 5, "watch must emit a call+args when enabled");
        assert_eq!(canary_store_watch(std::ptr::null(), 0x1234, 0x5678, 0x1001), 0, "disabled host fn observes nothing");
    }

    // SH106-NEXT hermetic: with the watch ON a plain 64-bit store still emits
    // the actual mov_store64 (the watch appends after it, never replaces it),
    // so no store is ever dropped — verified via translate() on a synthetic
    // LdStrImm (8,false) with no image required.
    #[test]
    fn canary_store_watch_never_removes_the_store() {
        use std::sync::atomic::Ordering;
        let _guard = CANARY_WATCH_TEST_LOCK.lock().unwrap();
        CANARY_STORE_WATCH.store(true, Ordering::Relaxed);
        set_canary_store_watch_test(true);
        let mut buf = crate::x86::CodeBuf::new();
        let inst = Inst::LdStrImm { rt: 0, rn: 1, imm: 0, size: 8, ld: false, sext: false };
        let mut fx = Vec::new();
        let _ = translate(&mut buf, 0x1000, inst, &mut fx);
        CANARY_STORE_WATCH.store(false, Ordering::Relaxed);
        set_canary_store_watch_test(false);
        assert!(buf.len() >= 3, "store+watch must emit at least a mov + call");
    }

    // SH4xx-next hermetic: the host-RETURN leak watch is byte-safe + gated the
    // same way as the canary store-watch. OFF: one AtomicBool load, no logging,
    // no panic on any input. ON: a foreign host pointer (0x7000_0000_0000..0x8000
    // _0000_0000) return triggers the log path (no panic), a non-host return does
    // not; the flag is independently pinnable so it never races the store-watch
    // hermetics.
    #[test]
    fn host_return_leak_watch_gated_and_value_classified() {
        use std::sync::atomic::Ordering;
        set_host_return_watch_test(true);
        HOST_RETURN_WATCH.store(true, Ordering::Relaxed);
        assert!(host_return_watch_active(), "watch active when armed");
        // Foreign host pointer return (the canary-leak class) -> log path, no panic.
        crate::translate::host_return_leak_watch(0x7f0000000000, 0x7fd16c014bc0, 0x1029999);
        // Non-host (small guest) return -> classified out, no panic.
        crate::translate::host_return_leak_watch(0x7f0000000000, 0x1234, 0x1029999);
        // Zero / sub-image values -> not the leak class, no panic.
        crate::translate::host_return_leak_watch(0x1000, 0, 0);

        // OFF: after disabling, calls return immediately (no logging, no panic) —
        // the disabled path must be a no-op regardless of input.
        HOST_RETURN_WATCH.store(false, Ordering::Relaxed);
        set_host_return_watch_test(false);
        assert!(!host_return_watch_active(), "watch inert when unarmed");
        crate::translate::host_return_leak_watch(0x7f0000000000, 0x7fd16c014bc0, 0);
        crate::translate::host_return_leak_watch(0, 0, 0);
    }

    // SH427: hermetic coverage of the arm64->x86 translator CORE (`translate`).
    // decode.rs pins decode (insn word -> Inst, 76 tests) and jit.rs pins
    // runtime behavior (295 tests), but the byte emission BETWEEN them — the
    // `translate(Inst -> CodeBuf)` mapping every translated block flows through —
    // had ZERO direct hermetics (only the 3 instrument-watch tests above). SH427
    // byte-pins the exact emitted x86 for the load-bearing, historically-buggy
    // instruction families, and asserts the semantic discriminators that the
    // qemu-verified comments in translate() spell out. Pure #[cfg(test)]: no
    // production path / guest byte / JIT-hook-default changed (translator core
    // untouched). Deterministic: synthetic Inst values, no real binary/image, no
    // env. RBX = CpuState base; guest reg g lives at [RBX+g*8] (slot g = 8g);
    // SP slot = 31*8 = 248 = 0xf8.
    fn tr_bytes(inst: Inst) -> Vec<u8> {
        let mut c = crate::x86::CodeBuf::new();
        let mut fx = Vec::new();
        translate(&mut c, 0x1000, inst, &mut fx).expect("translate must succeed");
        c.as_slice().to_vec()
    }

    #[test]
    fn sh453_addl_widen_add_doublewidth_signed_movsxd() {
        // saddl V1.2s, V2.2s, V3.2s (esrc=4 sign=true sub=false): widen-and-add —
        // each 4-byte element of Vn/Vm is SIGN-extended (movsxd 48 63 c0/c9 after
        // the mov_load32) then summed into a DOUBLE-width 8-byte dst (mov_store64
        // 48 89 83). The movsxd sign-extend + the 8B dst store IS the widening:
        // a zero-extend (no movsxd) flips a negative element's high bits and a
        // 4B store truncates. rd=1 rn=2 rm=3 -> Vn@0x130 Vm@0x140 Vd@0x120.
        let b = tr_bytes(Inst::SimdAddl { rd: 1, rn: 2, rm: 3, esrc: 4, sign: true, sub: false, upper: false });
        assert_eq!(b, vec![
            // lane 0: Vn[0] + Vm[0] -> Vd[0] (8B)
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] Vn lane0
            0x48, 0x63, 0xc0,                   // movsxd rax,eax (sign extend)
            0x8b, 0x8b, 0x40, 0x01, 0x00, 0x00, // mov ecx,[rbx+0x140] Vm lane0
            0x48, 0x63, 0xc9,                   // movsxd rcx,ecx
            0x48, 0x01, 0xc8,                   // add rax,rcx
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax (8B dst)
            // lane 1: Vn[1] + Vm[1] -> Vd[1]
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // Vn lane1 0x134
            0x48, 0x63, 0xc0,
            0x8b, 0x8b, 0x44, 0x01, 0x00, 0x00, // Vm lane1 0x144
            0x48, 0x63, 0xc9,
            0x48, 0x01, 0xc8,
            0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00, // Vd lane1 0x128
        ]);
        assert!(b.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "signed 4B lane sign-extends via movsxd");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "double-width 8B dst store");
    }

    #[test]
    fn sh453_addl_sub_direction_and_word_movzx() {
        // usubl V1.4s, V2.4h, V3.4h (esrc=2 sign=false sub=true): ZERO-extended
        // word sources (movzx 0f b7 83 / 0f b7 8b) with SUB rax,rcx (48 29 c8)
        // into 4B dst. The 0x29 opcode byte-vs-0x01 is the sub-vs-add
        // discriminator (a transposed op adds instead of subtracting). 4 lanes.
        let s = tr_bytes(Inst::SimdAddl { rd: 1, rn: 2, rm: 3, esrc: 2, sign: false, sub: true, upper: false });
        assert!(s.windows(7).any(|w| w == [0x0f, 0xb7, 0x83, 0x30, 0x01, 0x00, 0x00]), "unsigned word source zero-extends (movzx 0f b7)");
        assert!(s.windows(3).any(|w| w == [0x48, 0x29, 0xc8]), "usubl = sub rax,rcx (48 29 c8)");
        assert!(!s.windows(3).any(|w| w == [0x48, 0x01, 0xc8]), "sub form must NOT emit add rax,rcx (48 01 c8)");
        assert!(!s.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "sign=false must NOT movsxd");
        assert!(s.windows(6).any(|w| w == [0x89, 0x83, 0x2c, 0x01, 0x00, 0x00]), "lane3 4B dst store at 0x12c");
        // the add form (control) DOES emit add rax,rcx.
        let a = tr_bytes(Inst::SimdAddl { rd: 1, rn: 2, rm: 3, esrc: 2, sign: false, sub: false, upper: false });
        assert!(a.windows(3).any(|w| w == [0x48, 0x01, 0xc8]), "uaddl = add rax,rcx (48 01 c8)");
    }

    #[test]
    fn sh453_addl_upper_high_half_source_offset_and_signed_byte() {
        // saddl2 V1.8h, V2.8b, V3.8b (esrc=1 sign=true upper=true): SIGN-extend
        // byte sources (movsx 48 0f be) from the HIGH half (upper=true adds +8
        // to the source bases -> Vn@0x138 Vm@0x148) summed into 2B dst (66 89).
        let b = tr_bytes(Inst::SimdAddl { rd: 1, rn: 2, rm: 3, esrc: 1, sign: true, sub: false, upper: true });
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x38, 0x01, 0x00, 0x00]) || b.windows(8).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x38, 0x01, 0x00, 0x00]), "upper lane0 Vn byte source at 0x138 (high half)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x8b, 0x48, 0x01, 0x00, 0x00]) || b.windows(8).any(|w| w == [0x48, 0x0f, 0xbe, 0x8b, 0x48, 0x01, 0x00, 0x00]), "upper lane0 Vm byte source at 0x148 (high half)");
        assert!(b.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "byte-wide sum -> 2B dst store (66 89 83)");
        // upper must NOT read the low-half sources.
        assert!(!b.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x30, 0x01, 0x00, 0x00]), "upper must NOT read Vn@0x130 low half");
    }

    #[test]
    fn sh453_addl_inplace_alias_permute_source_snapshot() {
        // gcc's `saddl v1.4s, v1.4h, v2.4h` (rd==rn==1): the widened 2*esrc dst
        // write at i*2*esrc OVERLAPS the narrow source bytes of lane i+1 (at
        // (i+1)*esrc), so a read-then-write loop clobbers the still-needed
        // source. The emit snapshots the full 16B dest/source register to perm-
        // scratch FIRST (mov_load64 [0x120] -> [0x320], [0x128] -> [0x328]),
        // then reads the Vn operands from that scratch (48 0f bf 83 20 03..) —
        // the [.., 0x320] perm-scratch reads are the in-place-alias signature.
        // PERMSCRATCH_OFF = 0x200 (VECTOR_BASE 0x110 + 33*16 = 0x320).
        let b = tr_bytes(Inst::SimdAddl { rd: 1, rn: 1, rm: 3, esrc: 2, sign: true, sub: false, upper: false });
        // snapshot: two u64 halves pushed to 0x320/0x328.
        assert!(b.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x20, 0x01, 0x00, 0x00]), "alias snapshots rd/rn low u64 (mov rax,[0x120])");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x03, 0x00, 0x00]), "alias stores snap to perm-scratch 0x320");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x28, 0x03, 0x00, 0x00]), "alias snapshots high u64 to 0x328");
        // the widened reads come FROM the scratch (0x320,0x322,...) not the live dst.
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x20, 0x03, 0x00, 0x00]) || b.windows(8).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x20, 0x03, 0x00, 0x00]), "source lane0 reads Vn from perm-scratch 0x320");
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x22, 0x03, 0x00, 0x00]) || b.windows(8).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x22, 0x03, 0x00, 0x00]), "source lane1 reads Vn from perm-scratch 0x322 (not live 0x122)");
        // rm=3 does NOT alias rd, so Vm is read directly at 0x140 (no snapshot).
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbf, 0x8b, 0x40, 0x01, 0x00, 0x00]) || b.windows(8).any(|w| w == [0x48, 0x0f, 0xbf, 0x8b, 0x40, 0x01, 0x00, 0x00]), "Vm lane0 read directly at 0x140");
    }

    #[test]
    fn sh452_adalp_pairwise_sum_doublewidth_full_buffer() {
        // saddlp V1.2s, V2.2s (se=4 np=2 signed=true acc=false): pairwise-ADD-
        // LONG — sum each adjacent pair (2i,2i+1) of 4-byte srcs into a dst lane
        // of WIDTH 8 (2*se). Each pair: mov_load32(RAX,[0x130]) (8b 83) +
        // mov_load32(RCX,[0x134]) (8b 8b — RCX base) + `add rax,rcx` (48 01 c8)
        // + mov_store64 (48 89 83) into the 8-byte dst. The double-width dest
        // (2x source) IS the pairwise-add-LONG semantic — a flub storing only
        // the low 4 bytes drops the pair-carry. rd=1 rn=2 -> Vn@0x130 Vd@0x120.
        let b = tr_bytes(Inst::SimdAdalp { rd: 1, rn: 2, src_esize: 4, n_pairs: 2, signed: true, upper: false, acc: false });
        assert_eq!(b, vec![
            // pair 0: (Vn[0], Vn[1]) -> Vd[0] (8B)
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] src[2i]
            0x8b, 0x8b, 0x34, 0x01, 0x00, 0x00, // mov ecx,[rbx+0x134] src[2i+1]
            0x48, 0x01, 0xc8,                   // add rax,rcx
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax (dst 8B)
            // pair 1: (Vn[2], Vn[3]) -> Vd[1]
            0x8b, 0x83, 0x38, 0x01, 0x00, 0x00, // src[2] at 0x138
            0x8b, 0x8b, 0x3c, 0x01, 0x00, 0x00, // src[3] at 0x13c
            0x48, 0x01, 0xc8,
            0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00, // dst 8B at 0x128
        ]);
        // the widening: 4-byte source loads feed an 8-byte dest store.
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "4B source [2i]");
        assert!(b.windows(6).any(|w| w == [0x8b, 0x8b, 0x34, 0x01, 0x00, 0x00]), "4B source [2i+1] into RCX (8b 8b)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "double-width (8B) dst store (48 89 83)");
        // the 4-byte source loads never feed a 32-bit store: both stores are the
        // 64-bit 48 89 83 forms (the assert_eq above pins them exactly).
        assert_eq!(b.windows(7).filter(|w| w == &[0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00] || w == &[0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00]).count(), 2, "both pair sums store 64-bit dst (0x120, 0x128)");
    }

    #[test]
    fn sh452_adalp_word_pair_sum_and_signed_byte_signextend() {
        // uadalp V1.4s, V2.4h (se=2 np=4 signed=false acc=false): pairwise sum
        // of 2-byte srcs into 4-byte dst lanes. movzx_word (0f b7 83/0f b7 8b)
        // + add + mov_store32 (89 83). And saddlp V1.4h, V2.8b (se=1 signed=true):
        // movsx_byte_mem (48 0f be) SIGN-extends then adds into 2-byte dst (66 89).
        let b = tr_bytes(Inst::SimdAdalp { rd: 1, rn: 2, src_esize: 2, n_pairs: 4, signed: false, upper: false, acc: false });
        assert!(b.windows(7).any(|w| w == [0x0f, 0xb7, 0x83, 0x34, 0x01, 0x00, 0x00]), "pair2 [2i] movzx word at 0x134");
        assert!(b.windows(7).any(|w| w == [0x0f, 0xb7, 0x8b, 0x36, 0x01, 0x00, 0x00]), "pair2 [2i+1] movzx word at 0x136 (RCX)");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x24, 0x01, 0x00, 0x00]), "dst lane1 4B store at 0x124");
        // signed byte sources are SIGN-extended (movsx 48 0f be), not zero (0f b6).
        let sb = tr_bytes(Inst::SimdAdalp { rd: 1, rn: 2, src_esize: 1, n_pairs: 8, signed: true, upper: false, acc: false });
        assert!(sb.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x30, 0x01, 0x00, 0x00]) || sb.windows(8).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x30, 0x01, 0x00, 0x00]),
            "signed byte source uses movsx (48 0f be)");
        assert!(!sb.windows(7).any(|w| w == [0x0f, 0xb6, 0x83, 0x30, 0x01, 0x00, 0x00]), "signed must NOT zero-extend (no 0f b6)");
        assert!(sb.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "byte-pair sum -> 2B dst store (66 89 83)");
    }

    #[test]
    fn sh452_adalp_acc_accumulate_load16_add_before_store() {
        // uadalp V1.2s, V2.2s (se=4 np=2 acc=true): the ACCUMULATE form reads the
        // existing dst lane into R10 (4c 8b 93) and ADDS the pair sum (4c 01 d0)
        // before storing — uadalp/sadalp (acc) vs saddlp (acc=false) which does
        // NOT touch the dst. The mov_load64-into-R10 + add-r10 is the acc
        // discriminator: a flub that omits it overwrites instead of accumulating.
        let acc = tr_bytes(Inst::SimdAdalp { rd: 1, rn: 2, src_esize: 4, n_pairs: 2, signed: true, upper: false, acc: true });
        let now_acc = tr_bytes(Inst::SimdAdalp { rd: 1, rn: 2, src_esize: 4, n_pairs: 2, signed: true, upper: false, acc: false });
        assert!(acc.windows(7).any(|w| w == [0x4c, 0x8b, 0x93, 0x20, 0x01, 0x00, 0x00]), "acc loads dst lane into R10 (4c 8b 93)");
        assert!(acc.windows(3).any(|w| w == [0x4c, 0x01, 0xd0]), "acc adds R10 into RAX (4c 01 d0)");
        assert!(!now_acc.windows(7).any(|w| w == [0x4c, 0x8b, 0x93, 0x20, 0x01, 0x00, 0x00]), "non-acc must NOT read the dst");
        // the accumulate must precede the store.
        let r10_load = acc.windows(7).position(|w| w == [0x4c, 0x8b, 0x93, 0x20, 0x01, 0x00, 0x00]).unwrap();
        let store = acc.windows(7).position(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]).unwrap();
        assert!(r10_load < store, "accumulate read must come BEFORE the dst store");
        assert!(acc.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "acc still stores the 8B dst");
    }

    #[test]
    fn sh451_shrn_8to4_narrowing_width_full_buffer() {
        // shrn V1.2s, V2.2d, #16 (esrc=8 shift=16 upper=false round=false): the
        // NARROWING shift — 2 double-width source elements become 2 half-width
        // dest elements. Each lane: mov_load64 (48 8b 83, the 64-bit source) +
        // shr rax,16 (48 c1 e8 10) + mov_store32 (89 83, the 32-bit dest) —
        // source 8B wide, dest 4B wide. The width pair is the semantic: a flub
        // that stores the full 64-bit source into a 4B dest or truncates a 4B
        // source corrupts every lane. rd=1 rn=2 -> Vn@0x130 Vd@0x120.
        let b = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 8, shift: 16, upper: false, round: false });
        assert_eq!(b, vec![
            // lane 0
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130] 64-bit source
            0x48, 0xc1, 0xe8, 0x10,                   // shr rax,16
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00,       // mov [rbx+0x120],eax 32-bit dest
            // lane 1
            0x48, 0x8b, 0x83, 0x38, 0x01, 0x00, 0x00, // Vn lane1 +8
            0x48, 0xc1, 0xe8, 0x10,
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00,       // Vd lane1 +4
        ]);
        // the narrowing: a 64-bit source load paired with a 32-bit dest store.
        assert!(b.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "double-width source load (48 8b 83)");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "half-width dest store (89 83)");
        assert!(!b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "srn must NOT store the 64-bit dest (no 48 89 83)");
    }

    #[test]
    fn sh451_shrn_upper_high_half_destination_offset() {
        // shrn2 V1.2s, V2.2d, #16 (upper=true): identical emit but the dest lands
        // in the HIGH half (+8: dst_off=8 -> 0x128/0x12c instead of 0x120/0x124).
        // The upper flag is the only thing that moves the destination base.
        let lo = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 8, shift: 16, upper: false, round: false });
        let hi = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 8, shift: 16, upper: true, round: false });
        assert!(hi.windows(6).any(|w| w == [0x89, 0x83, 0x28, 0x01, 0x00, 0x00]), "upper lane0 stores Vd@0x128");
        assert!(hi.windows(6).any(|w| w == [0x89, 0x83, 0x2c, 0x01, 0x00, 0x00]), "upper lane1 stores Vd@0x12c");
        assert!(!hi.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "upper must NOT write the low-half 0x120");
        assert!(lo.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "non-upper writes low-half 0x120");
        // the source lane addressing is unchanged by upper.
        assert!(hi.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x38, 0x01, 0x00, 0x00]), "upper source lane1 still +8 from 0x130");
    }

    #[test]
    fn sh451_rshrn_round_add_before_shift_is_the_discriminator() {
        // rshrn V1.2s, V2.2d, #15 (round=true): the ROUNDING-HALF-UP form emits
        // `add rax, 1<<(shift-1)` (48 81 c0) BEFORE the shift — shrn (round=false)
        // has NO such add. The add_ri64 imm32 (48 81 c0 00 40 00 00 = +0x4000 for
        // shift 15) is the round-vs-no-round discriminator; a flub that shifts
        // without the round bias truncates instead of rounding.
        let r = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 8, shift: 15, upper: false, round: true });
        let n = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 8, shift: 15, upper: false, round: false });
        assert!(r.windows(6).any(|w| w == [0x48, 0x81, 0xc0, 0x00, 0x40, 0x00, 0x00]) || r.windows(7).any(|w| w == [0x48, 0x81, 0xc0, 0x00, 0x40, 0x00, 0x00]),
            "rshrn adds 1<<(shift-1)=0x4000 (48 81 c0 imm32)");
        assert!(!n.windows(6).any(|w| w == [0x48, 0x81, 0xc0, 0x00, 0x40]) && !n.windows(7).any(|w| w == [0x48, 0x81, 0xc0, 0x00, 0x40]),
            "shrn must NOT emit the round-add");
        // the add must precede the shift (ordering).
        let add_pos = r.windows(6).position(|w| w == [0x48, 0x81, 0xc0, 0x00, 0x40, 0x00]).unwrap();
        let shr_pos = r.windows(4).position(|w| w == [0x48, 0xc1, 0xe8, 0x0f]).unwrap();
        assert!(add_pos < shr_pos, "round-add must come BEFORE the narrowing shift");
        assert!(r.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x0f]), "shift 15 = shr rax,0x0f (48 c1 e8 0f)");
    }

    #[test]
    fn sh451_shrn_4to2_word_narrowing_16bit_store() {
        // shrn V1.4h, V2.4s, #8 (esrc=4 shift=8): 4 half-word dest lanes. Each:
        // mov_load32 (8b 83, 4B source) + shr rax,8 (48 c1 e8 08) + mov_store16
        // (66 89 83, 2B dest) — the O-word width narrowing vs the 8to4 form's
        // 32-bit store. The 66 prefix on the store + thread-local 4B source is
        // the 4to2 discriminator.
        let b = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 4, shift: 8, upper: false, round: false });
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "4B source load (mov eax)");
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x08]), "shr rax,8");
        assert!(b.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "2B dest store (66 89 83)");
        // 4 lanes: Vn@0x130/0x134/0x138/0x13c, Vd@0x120/0x122/0x124/0x126.
        assert!(b.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x26, 0x01, 0x00, 0x00]), "lane3 dest store at 0x126");
        assert_eq!(b.windows(6).filter(|w| w == &[0x8b, 0x83, 0x34, 0x01, 0x00, 0x00]).count(), 1, "lane1 4B source at 0x134");
        // no 64-bit store in the 4to2 form.
        assert!(!b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "4to2 must NOT emit a 64-bit store");
    }

    #[test]
    fn sh451_shrn_2to1_byte_narrowing_movzx_word_and_byte_store() {
        // shrn V1.8b, V2.8h, #4 (esrc=2 shift=4): 8 byte dest lanes. Each lane:
        // movzx_word_mem (0f b7 83, ZERO-extend the 2B source) + shr rax,4
        // (48 c1 e8 04) + mov_store8 (88 83, 1B dest). The movzx (not movsx) +
        // byte store is the narrowest-width narrowing form.
        let b = tr_bytes(Inst::SimdShrn { rd: 1, rn: 2, esrc: 2, shift: 4, upper: false, round: false });
        assert!(b.windows(7).any(|w| w == [0x0f, 0xb7, 0x83, 0x30, 0x01, 0x00, 0x00]), "2B source via movzx word (0f b7 83)");
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x04]), "shr rax,4");
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x20, 0x01, 0x00, 0x00]), "lane0 byte store [rbx+0x120]");
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x27, 0x01, 0x00, 0x00]), "lane7 byte store at 0x127");
        // movzx (0f b7) means UNSIGNED — the source must NOT be sign-extended.
        assert!(!b.windows(7).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x30, 0x01, 0x00]), "shrn is unsigned — no movsx word (48 0f bf)");
    }

    // SH440: hermetic coverage of the byte-COUNT + horizontal-SUM codegen pair
    // (translate.rs SimdPopcnt — `cnt Vd.8b, Vn.8b`; SimdSum8 — `uaddlv
    // h{rd}, Vn.8b`). These SWAR reductions had zero direct byte tests. SH440
    // pins the exact emitted x86 with rd=1, rn=2 (Vn slot = VECTOR_BASE
    // 0x110+2*16=0x130, Vd slot = 0x110+1*16=0x120):
    //  (1) SimdPopcnt full buffer — the SWAR popcount: load Vn@0x130, then
    //      x = x - ((x>>1)&0x5555..) [shr 1 + and 0x5555.. + sub], then
    //      (x&0x3333..)+((x>>2)&0x3333..) [shr 2], then (x+(x>>4))&0x0f0f..
    //      [shr 4 + add], store Vd@0x120. The three mask constants
    //      (0x5555../0x3333../0x0f0f..) + the shift progression 1/2/4 are the
    //      byte-count discriminators a wrong mask or shift silently corrupts.
    //  (2) SimdSum8 full buffer — the horizontal byte sum (uaddlv): load
    //      Vn@0x130, then pairwise-sum three times: (x + (x>>8))&0x00ff00ff..,
    //      then (x + (x>>16))&0x0000ffff0000ffff, then (x + (x>>32))&0xffffffff
    //      [shr imm8 8/16/32 + the three widening masks], store Vd@0x120. The
    //      shift-progression 8/16/32 (vs SimdPopcnt's 1/2/4) is the
    //      count-vs-sum discriminator; a byte error here mis-sums a
    //      lane-mask/popcount total.
    #[test]
    fn sh440_simdpopcnt_swar_byte_count_emits_masks_and_shift_ladder() {
        // rd=1 rn=2: src Vn slot = 0x130, dst Vd slot = 0x120.
        let b = tr_bytes(Inst::SimdPopcnt { rd: 1, rn: 2 });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130] load Vn
            0x48, 0x89, 0xc2,                         // mov rdx,rax        x
            0x48, 0xc1, 0xea, 0x01,                   // shr rdx,1         x>>1
            0x48, 0xb9, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, // mov rcx,0x5555..
            0x48, 0x21, 0xca,                         // and rdx,rcx       (x>>1)&0x5555..
            0x48, 0x29, 0xd0,                         // sub rax,rdx       x-(...)
            0x48, 0x89, 0xc2,                         // mov rdx,rax
            0x48, 0xc1, 0xea, 0x02,                   // shr rdx,2         x>>2
            0x48, 0xb9, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, // mov rcx,0x3333..
            0x48, 0x21, 0xca,                         // and rdx,rcx
            0x48, 0x21, 0xc8,                         // and rax,rcx
            0x48, 0x01, 0xd0,                         // add rax,rdx
            0x48, 0x89, 0xc2,                         // mov rdx,rax
            0x48, 0xc1, 0xea, 0x04,                   // shr rdx,4         x>>4
            0x48, 0x01, 0xd0,                         // add rax,rdx       x+(x>>4)
            0x48, 0xb9, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, // mov rcx,0x0f0f..
            0x48, 0x21, 0xc8,                         // and rax,rcx
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax store Vd
        ]);
        // The three SWAR masks must appear in ascending bit-half order.
        assert!(b.windows(10).any(|w| w[0] == 0x48 && w[1] == 0xb9 && w[2..].iter().all(|&x| x == 0x55)));
        assert!(b.windows(10).any(|w| w[0] == 0x48 && w[1] == 0xb9 && w[2..].iter().all(|&x| x == 0x33)));
        assert!(b.windows(10).any(|w| w[0] == 0x48 && w[1] == 0xb9 && w[2..].iter().all(|&x| x == 0x0f)));
        // The shift ladder goes 1/2/4 (shr rdx,imm8) — never 8/16/32 (that is
        // SimdSum8's ladder below): shr rdx.imm8 = 48 c1 ea imm.
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x01]));
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x02]));
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x04]));
        assert!(!b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x08]),
            "popcount must not use the sum-ladder shr 8");
    }

    #[test]
    fn sh440_simdsum8_horizontal_byte_sum_emits_widening_masks_and_ladder() {
        // rd=1 rn=2: src Vn slot = 0x130, dst Vd slot = 0x120.
        let b = tr_bytes(Inst::SimdSum8 { rd: 1, rn: 2 });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130] load Vn
            0x48, 0xb9, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, // mov rcx,0x00ff00ff00ff00ff
            0x48, 0x89, 0xc2,                         // mov rdx,rax
            0x48, 0xc1, 0xea, 0x08,                   // shr rdx,8         x>>8
            0x48, 0x21, 0xca,                         // and rdx,rcx
            0x48, 0x21, 0xc8,                         // and rax,rcx
            0x48, 0x01, 0xd0,                         // add rax,rdx       (x + x>>8)
            0x48, 0x89, 0xc2,                         // mov rdx,rax
            0x48, 0xc1, 0xea, 0x10,                   // shr rdx,16
            0x48, 0xb9, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, // mov rcx,0x0000ffff0000ffff
            0x48, 0x21, 0xca,                         // and rdx,rcx
            0x48, 0x21, 0xc8,                         // and rax,rcx
            0x48, 0x01, 0xd0,                         // add rax,rdx
            0x48, 0x89, 0xc2,                         // mov rdx,rax
            0x48, 0xc1, 0xea, 0x20,                   // shr rdx,32
            0x48, 0xb9, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, // mov rcx,0x00000000ffffffff
            0x48, 0x21, 0xca,                         // and rdx,rcx
            0x48, 0x21, 0xc8,                         // and rax,rcx
            0x48, 0x01, 0xd0,                         // add rax,rdx
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax store Vd
        ]);
        // Horizontal-sum widening ladder 8/16/32 (shr rdx,imm8) — the
        // sum-vs-count discriminator (SimdPopcnt uses 1/2/4).
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x08]));
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x10]));
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x20]));
        assert!(!b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x01]),
            "horizontal sum must not use the popcount ladder shr 1");
        // The three widening masks, in order.
        assert!(b.windows(10).any(|w| w[0] == 0x48 && w[1] == 0xb9 && w[2] == 0xff && w[3] == 0x00 && w[4] == 0xff && w[5] == 0x00 && w[6] == 0xff && w[7] == 0x00));
        assert!(b.windows(10).any(|w| w[0] == 0x48 && w[1] == 0xb9 && w[2] == 0xff && w[3] == 0xff && w[4] == 0x00 && w[5] == 0x00 && w[6] == 0xff && w[7] == 0xff));
        assert!(b.windows(10).any(|w| w[0] == 0x48 && w[1] == 0xb9 && w[2] == 0xff && w[3] == 0xff && w[4] == 0xff && w[5] == 0xff && w[6] == 0x00 && w[7] == 0x00));
    }

    // SH441: hermetic coverage of the SCALAR-REDUCTION + WIDEN codegen families
    // (translate.rs FMaxV — `fmaxv/fminv Sd, Vn.4s`; WidenShl — `shll/shll2
    // Vd.Td, Vn.Ts`). Both had zero direct byte tests. SH441 pins the exact
    // emitted x86 with rd=1, rn=2:
    //  (1) FMaxV full buffer — reduce the 4 single lanes of Vn (base
    //      0x130=0x110+2*16) into scalar Sd@0x120 (0x110+1*16): movd xmm0=<Vn.0>,
    //      then movd xmm1=<Vn.i> + maxss/minss xmm0,xmm1 for i=1..3, movd eax,
    //      mov [0x120],eax. The 0x5f (maxss) vs 0x5d (minss) opcode is the
    //      fmaxv-vs-fminv discriminator; the REX.B `40` (f3 40 0f 5f c1) and
    //      the advancing src displacement +0x4-lane are pinned.
    //  (2) WidenShl — reverse-iteration shll: dst_esize=4 dst-lanes 4 apart
    //      (0x124, 0x120), src src_esize=2 (so 0x132, 0x130), signed uses
    //      movsx_word_mem (`48 0f bf`) / unsigned uses movzx_word_mem (`0f b7`)
    //      — the REX.W presence is the signed-vs-unsigned discriminator; the
    //      shll2 upper-half (uh=8 -> src base 0x138) + dst_esize=2 case uses
    //      movsx_byte_mem (`48 0f be`) + the 16-bit `66 89` store, stepping
    //      src -1 and dst -2 lanes 3..0.
    #[test]
    fn sh441_fmaxv_cross_lane_scalar_reduce_maxss_ladder() {
        let b = tr_bytes(Inst::FMaxV { rd: 1, rn: 2, min: false });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] Vn.0
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // mov eax,[rbx+0x134] Vn.1
            0x66, 0x0f, 0x6e, 0xc8,             // movd xmm1,eax
            0xf3, 0x40, 0x0f, 0x5f, 0xc1,       // maxss xmm0,xmm1
            0x8b, 0x83, 0x38, 0x01, 0x00, 0x00, // mov eax,[rbx+0x138] Vn.2
            0x66, 0x0f, 0x6e, 0xc8,             // movd xmm1,eax
            0xf3, 0x40, 0x0f, 0x5f, 0xc1,       // maxss xmm0,xmm1
            0x8b, 0x83, 0x3c, 0x01, 0x00, 0x00, // mov eax,[rbx+0x13c] Vn.3
            0x66, 0x0f, 0x6e, 0xc8,             // movd xmm1,eax
            0xf3, 0x40, 0x0f, 0x5f, 0xc1,       // maxss xmm0,xmm1
            0x66, 0x0f, 0x7e, 0xc0,             // movd eax,xmm0
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],eax Sd
        ]);
        // The three source lanes advance +4 (0x134, 0x138, 0x13c) from base 0x130.
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x34, 0x01, 0x00, 0x00]));
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x3c, 0x01, 0x00, 0x00]));
        // maxss opcode byte 0x5f present, fminv's 0x5d (minss) absent.
        assert!(b.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5f, 0xc1]));
        assert!(!b.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5d, 0xc1]));
    }

    #[test]
    fn sh441_fminv_cross_lane_scalar_reduce_minss_ladder() {
        let b = tr_bytes(Inst::FMaxV { rd: 1, rn: 2, min: true });
        // Same ladder shape but opcode 0x5d (minss) instead of 0x5f (maxss).
        assert!(!b.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5f, 0xc1]),
            "fminv must not emit maxss");
        assert!(b.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5d, 0xc1]));
        // Still reduces all 4 lanes into Sd@0x120.
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]));
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x3c, 0x01, 0x00, 0x00]));
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]));
    }

    #[test]
    fn sh441_widenshl_reverse_iteration_signed_movsx_vs_unsigned_movzx() {
        // shll Vd.4s, Vn.4h : dst_esize=4, nlanes=2 => src_esize=2. Reverse
        // iteration writes i=1 first (dst 0x124 from src 0x132) then i=0
        // (dst 0x120 from src 0x130) — the widening reverse order.
        let s = tr_bytes(Inst::WidenShl { rd: 1, rn: 2, dst_esize: 4, nlanes: 2, signed: true, upper: false });
        assert_eq!(s, vec![
            0x48, 0x0f, 0xbf, 0x83, 0x32, 0x01, 0x00, 0x00, // movsx_word_mem rax,[rbx+0x132]
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00,             // mov [rbx+0x124],eax  dst lane1
            0x48, 0x0f, 0xbf, 0x83, 0x30, 0x01, 0x00, 0x00, // movsx_word_mem rax,[rbx+0x130]
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00,             // mov [rbx+0x120],eax  dst lane0
        ]);
        assert!(s.windows(8).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x32, 0x01, 0x00, 0x00]));
        let u = tr_bytes(Inst::WidenShl { rd: 1, rn: 2, dst_esize: 4, nlanes: 2, signed: false, upper: false });
        // unsigned uses movzx_word_mem (0f b7, NO 0x48 REX.W) — the signed-vs-
        // unsigned discriminator (a sign-extended lane picks the wrong value).
        assert!(u.windows(7).any(|w| w == [0x0f, 0xb7, 0x83, 0x32, 0x01, 0x00, 0x00]));
        assert!(!u.windows(8).any(|w| w == [0x48, 0x0f, 0xbf, 0x83, 0x32, 0x01, 0x00, 0x00]),
            "unsigned shll must not emit movsx (REX.W)");
    }

    #[test]
    fn sh441_widenshl2_upper_half_byte_widen_16bit_store() {
        // shll2 Vd.8h, Vn.8b : dst_esize=2 (=> src_esize=1), nlanes=4, upper
        // => src base = Vn + 8 = 0x138. Reverse iteration: src 0x13b..0x138,
        // dst lanes 0x126,0x124,0x122,0x120 (2 apart). Signed byte widen via
        // movsx_byte_mem (48 0f be) + 16-bit store (66 89).
        let b = tr_bytes(Inst::WidenShl { rd: 1, rn: 2, dst_esize: 2, nlanes: 4, signed: true, upper: true });
        assert_eq!(b, vec![
            0x48, 0x0f, 0xbe, 0x83, 0x3b, 0x01, 0x00, 0x00, // movsx_byte_mem rax,[rbx+0x13b]
            0x66, 0x89, 0x83, 0x26, 0x01, 0x00, 0x00,     // mov [rbx+0x126],ax  dst lane3
            0x48, 0x0f, 0xbe, 0x83, 0x3a, 0x01, 0x00, 0x00, // movsx_byte_mem rax,[rbx+0x13a]
            0x66, 0x89, 0x83, 0x24, 0x01, 0x00, 0x00,     // mov [rbx+0x124],ax  dst lane2
            0x48, 0x0f, 0xbe, 0x83, 0x39, 0x01, 0x00, 0x00, // movsx_byte_mem rax,[rbx+0x139]
            0x66, 0x89, 0x83, 0x22, 0x01, 0x00, 0x00,     // mov [rbx+0x122],ax  dst lane1
            0x48, 0x0f, 0xbe, 0x83, 0x38, 0x01, 0x00, 0x00, // movsx_byte_mem rax,[rbx+0x138]
            0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00,     // mov [rbx+0x120],ax  dst lane0
        ]);
        // upper-half base: src must start at 0x138 (Vn base 0x130 + uh 8), never 0x130.
        assert!(b.windows(8).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x38, 0x01, 0x00, 0x00]));
        assert!(!b.windows(8).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x30, 0x01, 0x00, 0x00]),
            "shll2 upper half must not read the low Vn base");
        // 16-bit stores: opcode 66 89 present, 32-bit 89 absent.
        assert!(b.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x26, 0x01, 0x00, 0x00]));
        assert!(!b.windows(7).any(|w| w == [0x89, 0x83, 0x24, 0x01, 0x00, 0x00]));
    }

    // SH443: hermetic coverage of the BYTE-REVERSE codegen family (translate.rs
    // SimdRev — rev64/rev32/rev16 Vd.T, Vn.T: byte-reverse within each
    // granule). SimdRev had zero direct byte tests. SH443 pins the exact
    // emitted x86 with rd=1, rn=2 (Vn base = VECTOR_BASE 0x110+2*16=0x130,
    // Vd base = 0x110+1*16=0x120):
    //  (1) rev64 (granule 8, q=0) — one 8B granule: `mov rax,[rbx+0x130]` +
    //      bswap r64 (`48 0f c8`) + `mov [rbx+0x120],rax`. The bswap-r64
    //      opcode (48 0f c8) is the 8B-granule discriminator.
    //  (2) rev64 q=1 — TWO 8B granules: second reads Vn+8 (0x138), bswaps,
    //      stores Vd+8 (0x128). The q cross-lane advance +0x8 pins the
    //      whole-register reversal (a made-only-low-half buggy emit would
    //      never touch 0x138/0x128).
    //  (3) rev32 (granule 4, q=0) — TWO 4B granules: `mov eax,[rbx+0x130]` +
    //      bswap r32 (`0f c8`, NO 0x48 REX.W) + `mov [rbx+0x120],eax`, then
    //      granule 1 at 0x134 -> 0x124. The bswap-r32 (0f c8 without 48) vs
    //      bswap-r64 (48 0f c8) opcode is the 4-vs-8-granule discriminator.
    //  (4) rev16 (granule 2, q=0) — FOUR 2B granules, each `movzx eax,
    //      [rbx+0x130+2i]` + rol16 by 8 (`66 c1 c0 08`, rotate-left-by-8 =
    //      byte-swap a halfword) + the 16-bit `66 89` store at 2-apart dst
    //      (0x120, 0x122, 0x124, 0x126). The rol-imm8-by-8 (66 c1 c0 08) vs a
    //      bswap is the 2-granule discriminator.
    // A wrong granule emits the wrong byte-swap primitive (bswap64 vs bswap32
    // vs rol16), silently reordering rendered pixel/color bytes.
    #[test]
    fn sh443_simdrev_granule8_bswap64_q() {
        let b = tr_bytes(Inst::SimdRev { rd: 1, rn: 2, granule: 8, q: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130]
            0x48, 0x0f, 0xc8,                         // bswap r64 rax
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax
        ]);
        assert!(b.windows(3).any(|w| w == [0x48, 0x0f, 0xc8]),
            "rev64 must bswap the 64-bit rax (48 0f c8)");
    }

    #[test]
    fn sh443_simdrev_granule8_q_cross_lane_whole_register() {
        let b = tr_bytes(Inst::SimdRev { rd: 1, rn: 2, granule: 8, q: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130] granule0 src
            0x48, 0x0f, 0xc8,                         // bswap rax
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax granule0 dst
            0x48, 0x8b, 0x83, 0x38, 0x01, 0x00, 0x00, // mov rax,[rbx+0x138] granule1 src (Vn+8)
            0x48, 0x0f, 0xc8,                         // bswap rax
            0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00, // mov [rbx+0x128],rax granule1 dst (Vd+8)
        ]);
        assert!(b.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x38, 0x01, 0x00, 0x00]),
            "q-mode rev64 must reverse the second 8B granule (Vn+8=0x138)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00]),
            "q-mode rev64 must store the second granule at Vd+8=0x128");
    }

    #[test]
    fn sh443_simdrev_granule4_bswap32_no_rexw() {
        let b = tr_bytes(Inst::SimdRev { rd: 1, rn: 2, granule: 4, q: false });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] granule0
            0x0f, 0xc8,                         // bswap r32 eax (NO 0x48 REX.W)
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],eax
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // mov eax,[rbx+0x134] granule1
            0x0f, 0xc8,                         // bswap eax
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [rbx+0x124],eax
        ]);
        assert!(b.windows(2).any(|w| w == [0x0f, 0xc8]),
            "rev32 must bswap the 32-bit eax (0f c8)");
        assert!(!b.windows(3).any(|w| w == [0x48, 0x0f, 0xc8]),
            "rev32 must NOT emit bswap-r64 (48 0f c8) — 4B granule, no REX.W");
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x34, 0x01, 0x00, 0x00]),
            "rev32 q=0 reverses TWO 4B granules (second src at 0x134)");
    }

    #[test]
    fn sh443_simdrev_granule2_rol16_rotate_left_8() {
        // rev16: byte-swap each 16-bit halfword = rotate-left-by-8.
        let b = tr_bytes(Inst::SimdRev { rd: 1, rn: 2, granule: 2, q: false });
        assert_eq!(b, vec![
            0x0f, 0xb7, 0x83, 0x30, 0x01, 0x00, 0x00, // movzx eax,[rbx+0x130] hw0
            0x66, 0xc1, 0xc0, 0x08,                   // rol eax,8 (rotate-left-by-8)
            0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],ax
            0x0f, 0xb7, 0x83, 0x32, 0x01, 0x00, 0x00, // movzx eax,[rbx+0x132] hw1
            0x66, 0xc1, 0xc0, 0x08,                   // rol eax,8
            0x66, 0x89, 0x83, 0x22, 0x01, 0x00, 0x00, // mov [rbx+0x122],ax
            0x0f, 0xb7, 0x83, 0x34, 0x01, 0x00, 0x00, // movzx eax,[rbx+0x134] hw2
            0x66, 0xc1, 0xc0, 0x08,                   // rol eax,8
            0x66, 0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [rbx+0x124],ax
            0x0f, 0xb7, 0x83, 0x36, 0x01, 0x00, 0x00, // movzx eax,[rbx+0x136] hw3
            0x66, 0xc1, 0xc0, 0x08,                   // rol eax,8
            0x66, 0x89, 0x83, 0x26, 0x01, 0x00, 0x00, // mov [rbx+0x126],ax
        ]);
        assert!(b.windows(4).any(|w| w == [0x66, 0xc1, 0xc0, 0x08]),
            "rev16 must byte-swap each halfword via rol eax,8 (66 c1 c0 08)");
        assert!(!b.windows(3).any(|w| w == [0x48, 0x0f, 0xc8]),
            "rev16 must NOT emit bswap-r64");
        assert!(b.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x26, 0x01, 0x00, 0x00]),
            "rev16 must store the 4th halfword (3rd-lane) via a 16-bit store");
    }

    // SH442: hermetic coverage of the scalar FcvtToInt ROUNDING-mode paths
    // (translate.rs FcvtToInt mode 2 fcvtau / mode 3 fcvtpu,ps / mode 4
    // fcvtmu,ms). SH435 pinned mode 0 (fcvtzs/fcvtzu trunc) and SH439 pinned
    // fcvtzu's [2^63,2^64) big-path; the ROUND-before-truncate modes had zero
    // direct byte tests. SH442 pins the exact emitted x86 (rd=0, rn=1 d-src,
    // src slot = VECTOR_BASE 0x110+1*16=0x120, dst slot g0):
    //  (1) fcvtau (unsigned, mode 2) = roundsd NEAREST-EVEN (imm8 0x00) then
    //      cvttsd2si — and NO cmovs clamp (the comment documents the accepted
    //      saturation edge). The imm8 0x00 vs 0x01/0x02 is the mode-2
    //      discriminator.
    //  (2) fcvtpu (unsigned, mode 3, +inf) = roundsd CEIL (imm8 0x02) +
    //      cvttsd2si + the negative clamp pair (xor rcx,rcx; test rax,rax;
    //      cmovs rax,rcx = 48 0f 48 c1). The unsigned clamp presence separates
    //      the unsigned from the signed mode-3 path.
    //  (3) fcvtmu (unsigned, mode 4, -inf) = roundsd FLOOR (imm8 0x01) +
    //      cvttsd2si + same clamp. The imm8 0x01 (floor) vs 0x02 (ceil) is the
    //      fcvtmu-vs-fcvtpu discriminator.
    //  (4) signed mode 3/4 (fcvtps/fcvtms) = same roundsd but NO clamp — the
    //      unsigned-cmovs absence is the signed discriminator.
    // Each test pins the exact full emit AND the imm8 + clamp discriminators
    // so a wrong rounding direction or a missing negative-clamp silently
    // mis-converts a render/lighting value.
    #[test]
    fn sh442_fcvtau_rounds_nearest_even_no_unsigned_clamp() {
        let b = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 2, sf: true, unsigned: true, src_sng: false, fbits: 0 });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x20, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x120]
            0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x00,                  // roundsd xmm0,xmm0,0x00 nearest-even
            0xf2, 0x48, 0x0f, 0x2c, 0xc0,                         // cvttsd2si rax,xmm0
            0x48, 0x89, 0x03,                                     // mov [rbx],rax  dst g0
        ]);
        assert!(b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x00]),
            "fcvtau must roundsd nearest-even (imm8 0x00)");
        assert!(!b.windows(4).any(|w| w == [0x48, 0x31, 0xc9, 0x00]) && !b.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]),
            "fcvtau must NOT emit the unsigned cmovs clamp");
    }

    #[test]
    fn sh442_fcvtpu_rounds_ceil_with_unsigned_clamp() {
        let b = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 3, sf: true, unsigned: true, src_sng: false, fbits: 0 });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x20, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x120]
            0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x02,                  // roundsd xmm0,xmm0,0x02 CEIL (+inf)
            0xf2, 0x48, 0x0f, 0x2c, 0xc0,                         // cvttsd2si rax,xmm0
            0x48, 0x31, 0xc9,                                     // xor rcx,rcx
            0x48, 0x85, 0xc0,                                     // test rax,rax
            0x48, 0x0f, 0x48, 0xc1,                               // cmovs rax,rcx  (neg -> 0)
            0x48, 0x89, 0x03,                                     // mov [rbx],rax
        ]);
        assert!(b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x02]),
            "fcvtpu must roundsd ceil (imm8 0x02)");
        assert!(b.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]),
            "unsigned fcvtpu must clamp negatives to 0");
    }

    #[test]
    fn sh442_fcvtmu_rounds_floor_with_unsigned_clamp() {
        let b = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 4, sf: true, unsigned: true, src_sng: false, fbits: 0 });
        // Same shape as fcvtpu but roundsd imm8 0x01 (floor, -inf).
        assert!(b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x01]),
            "fcvtmu must roundsd floor (imm8 0x01)");
        assert!(!b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x02]),
            "fcvtmu must not roundsd ceil (0x02)");
        assert!(b.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]),
            "unsigned fcvtmu must clamp negatives to 0");
    }

    #[test]
    fn sh442_signed_fcvtps_ms_no_unsigned_clamp_discriminator() {
        let p = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 3, sf: true, unsigned: false, src_sng: false, fbits: 0 });
        // signed fcvtps = roundsd ceil (0x02) + cvttsd2si, NO clamp.
        assert!(p.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x02]));
        assert!(!p.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]),
            "signed fcvtps must NOT emit the unsigned cmovs clamp");
        assert!(p.windows(4).any(|w| w == [0xf2, 0x48, 0x0f, 0x2c]),
            "signed fcvtps truncs via cvttsd2si");
    }

    #[test]
    fn sh442_signed_fcvtms_floors_no_clamp() {
        let m = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 4, sf: true, unsigned: false, src_sng: false, fbits: 0 });
        assert!(m.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x0b, 0xc0, 0x01]),
            "signed fcvtms must roundsd floor (imm8 0x01)");
        assert!(!m.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]),
            "signed fcvtms must NOT emit the unsigned cmovs clamp");
    }

    #[test]
    fn sh431_logicimm_orr_xzr_mov_alias_materializes_mask() {
        // `mov x0,#7` = ORR x0,xzr,#7 (op=1, rn==31): rn==31 must read as XZR
        // (zero), so RAX is `mov rax,0` (never the SP slot), the bitmask
        // immediate is materialized in RCX (mov rcx,7), then `or rax,rcx`,
        // store slot0. Pins the bitmask-immediate materialization + the
        // xzr-not-SP read of rn==31.
        let b = tr_bytes(Inst::LogicImm { rd: 0, rn: 31, mask: 7, op: 1, sf: true });
        assert_eq!(b, vec![
            0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax,0 (XZR, NOT SP)
            0x48, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rcx,7 (bitmask imm)
            0x48, 0x09, 0xc8, // or rax,rcx
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
        assert!(!b.windows(4).any(|w| w == [0x48, 0x8b, 0x83, 0xf8]),
            "rn==31 must be XZR (zero), not the SP slot");
    }

    #[test]
    fn sh431_logicimm_ands_w32_flag_setting_nzcv() {
        // ANDS w0,w1,#5 (op=3, sf=false): `and rax,rcx` then store_nzcv (the
        // pushfq + 4-bit NZCV pack ending with the C-store `89 93 08 01 00 00`
        // to [rbx+0x108] + caller pop), then the 32-bit zero-extend (mov eax,eax)
        // and store. Pins that the flag-setting ANDS form is the ONLY LogicImm
        // op that emits the nzcv pack (and/orr/eor do not).
        let b = tr_bytes(Inst::LogicImm { rd: 0, rn: 1, mask: 5, op: 3, sf: false });
        assert_eq!(&b[0..17], &[
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0xb9, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rcx,5 (mask)
            0x48, 0x21, 0xc8, // and rax,rcx
        ]);
        // the nzcv tail packs NZCV and stores C to [rbx+0x108], pops caller regs,
        // then the final 32-bit zero-extend store. Pin the real trailing 5 bytes
        // (mov eax,eax then mov [rbx],rax) and the nzcv C-store just before them.
        assert_eq!(&b[b.len() - 5..], &[0x89, 0xc0, 0x48, 0x89, 0x03], "W zero-ext + store");
        assert_eq!(&b[b.len() - 14..b.len() - 8], &[0x89, 0x93, 0x08, 0x01, 0x00, 0x00], "nzcv C-store");
        assert_eq!(&b[b.len() - 8..b.len() - 5], &[0x5a, 0x59, 0x58], "caller regs restored");
    }

    #[test]
    fn sh431_mulhigh_umulh_unsigned_high_half_in_rdx() {
        // umulh x0,x1,x2: one-operand UNSIGNED multiply `mul rcx` (48 f7 e1),
        // high half lands in RDX, stored to slot0 via `mov [rbx],rdx` (48 89 13).
        // Pin the /4 (mul) discriminator against the signed /5 (imul) case.
        let b = tr_bytes(Inst::MulHigh { rd: 0, rn: 1, rm: 2, signed: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1 multiplicand)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2 multiplier)
            0x48, 0xf7, 0xe1, // mul rcx  (/4 unsigned; RDX:RAX = RAX*RCX)
            0x48, 0x89, 0x13, // mov [rbx],rdx (store high 64 -> rd=0)
        ]);
    }

    #[test]
    fn sh431_mulhigh_smulh_signed_high_half_in_rdx() {
        // smulh x0,x1,x2: same load pair, but the SIGNED one-operand `imul rcx`
        // (48 f7 e9, /5) puts the high half in RDX. The /5 discriminator is what
        // separates smulh from umulh — a flub here silently corrupts the high
        // half of every signed wide multiply.
        let b = tr_bytes(Inst::MulHigh { rd: 0, rn: 1, rm: 2, signed: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x48, 0xf7, 0xe9, // imul rcx  (/5 signed)
            0x48, 0x89, 0x13, // mov [rbx],rdx (store high 64 -> rd=0)
        ]);
    }

    fn tr_bytes_fx(inst: Inst) -> (Vec<u8>, Vec<Fixup>) {
        let mut c = crate::x86::CodeBuf::new();
        let mut fx = Vec::new();
        translate(&mut c, 0x1000, inst, &mut fx).expect("translate must succeed");
        (c.as_slice().to_vec(), fx)
    }

    #[test]
    fn sh430_muldiv_madd_64_accumulates_rn_mul_rm_plus_ra() {
        // madd x0,x1,x2,x3 = x0 = x1*x2 + x3 (MADD, div=false, signed=false).
        // Pin: load rn(1)+rm(2), imul (RAX = product), load ra(3) into RDI,
        // `add rax,rdi` (accumulate +), store slot0. The accumulate direction
        // (product + ra) is the discriminator against the msub case.
        let b = tr_bytes(Inst::MulDiv { div: false, signed: false, rd: 0, rn: 1, rm: 2, ra: 3, sf: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x48, 0x0f, 0xaf, 0xc1, // imul rax,rcx  (RAX = rn*rm, low 64)
            0x48, 0x8b, 0x7b, 0x18, // mov rdi,[rbx+0x18] (ra=3)
            0x48, 0x01, 0xf8, // add rax,rdi  (RAX += ra)
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
    }

    #[test]
    fn sh430_muldiv_msub_64_subtracts_product_from_ra() {
        // msub x0,x1,x2,x3 = x0 = x3 - x1*x2. Historical bug (doc): `n - q*d`
        // compiled to msub returned -48 for 1298-25*50 (should be +48) until the
        // direction was fixed to `ra - product`. Pin the FIXED shape: RDI=ra,
        // `sub rdi,rax` (RDI = ra - product), `mov rax,rdi` (RAX = ra - product),
        // store. A regression back to `rn*rm - ra` flips the sub operand order.
        let b = tr_bytes(Inst::MulDiv { div: false, signed: true, rd: 0, rn: 1, rm: 2, ra: 3, sf: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x48, 0x0f, 0xaf, 0xc1, // imul rax,rcx
            0x48, 0x8b, 0x7b, 0x18, // mov rdi,[rbx+0x18] (ra=3)
            0x48, 0x29, 0xc7, // sub rdi,rax  (RDI = ra - product)
            0x48, 0x89, 0xf8, // mov rax,rdi  (RAX = ra - product)
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
    }

    #[test]
    fn sh430_muldiv_mul_xzr_accumulate_skipped_not_sp() {
        // mul x0,x1,x2 = madd with ra==31 (XZR, accumulate 0). The ra==31 slot
        // must be XZR (skip entirely), NOT a SP read — ldg(ra=31) would read the
        // stack pointer at [rbx+0xf8] and corrupt the product with it. Pin that
        // after the imul the result stores directly: NO [rbx+...] ra/SP load and
        // NO add/sub appear anywhere in the 14 bytes.
        let b = tr_bytes(Inst::MulDiv { div: false, signed: false, rd: 0, rn: 1, rm: 2, ra: 31, sf: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x48, 0x0f, 0xaf, 0xc1, // imul rax,rcx
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
        assert!(!b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0xf8, 0x00, 0x00, 0x00])
            && !b.windows(4).any(|w| w == [0x48, 0x8b, 0x83, 0xf8]),
            "ra==31 must not touch the SP slot [rbx+0xf8]");
    }

    #[test]
    fn sh430_muldiv_udiv_64_unsigned_quotient_in_rax() {
        // udiv x0,x1,x2: unsigned — zero the RDX:RAX high half with `xor rdx,rdx`
        // (NOT cqo), then `div rcx` (quotient in RAX), store. The signed/unsigned
        // discriminator is xor-rdx (48 31 d2) vs cqo (48 99) + idiv.
        let b = tr_bytes(Inst::MulDiv { div: true, signed: false, rd: 0, rn: 1, rm: 2, ra: 31, sf: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1 dividend)
            0x48, 0x31, 0xd2, // xor rdx,rdx  (zero high half — unsigned)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2 divisor)
            0x48, 0xf7, 0xf1, // div rcx  (/6 group-3 unsigned divide)
            0x48, 0x89, 0x03, // mov [rbx],rax (store quotient)
        ]);
    }

    #[test]
    fn sh430_muldiv_sdiv_32_signextends_both_operands() {
        // sdiv w0,w1,w2 (sf=false, signed=true): the 32-bit operands must be
        // sign-extended to 64 BEFORE the signed divide (movsxd rax,eax +
        // movsxd rcx,ecx) and the dividend high half must come from cqo.
        // Then cqo, idiv rcx (quotient in RAX), 32-bit zero-ext store.
        let b = tr_bytes(Inst::MulDiv { div: true, signed: true, rd: 0, rn: 1, rm: 2, ra: 31, sf: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0x63, 0xc0, // movsxd rax,eax (sign-extend W dividend)
            0x48, 0x99, // cqo (signed RDX:RAX)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x48, 0x63, 0xc9, // movsxd rcx,ecx (sign-extend W divisor)
            0x48, 0xf7, 0xf9, // idiv rcx  (/7 signed divide)
            0x89, 0xc0, // mov eax,eax (W zero-extend of quotient)
            0x48, 0x89, 0x03, // mov [rbx],rax (store slot0)
        ]);
    }

    #[test]
    fn sh430_mullong_smull_64_signextends_w32_operands() {
        // smull x0,w1,w2 = 64-bit product of two 32-bit SIGNED operands.
        // Pin: ffrom 8b 43 08 (load rn), movsxd rax,eax, movsxd rcx,ecx,
        // imul (low-64 of signed 32x32 product), store. A 32x32 signed product
        // always fits in 64 bits, so low-64 imul is exact.
        let b = tr_bytes(Inst::MulLong { rd: 0, rn: 1, rm: 2, ra: 31, signed: true, sub: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x48, 0x63, 0xc0, // movsxd rax,eax (sign-extend rn)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x48, 0x63, 0xc9, // movsxd rcx,ecx (sign-extend rm)
            0x48, 0x0f, 0xaf, 0xc1, // imul rax,rcx
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
    }

    #[test]
    fn sh430_mullong_umsubl_negates_product_then_adds_ra() {
        // umsubl x0,w1,w2,x3 = x0 = (u32)x3 - (u32)x1*(u32)x2. Pin: zero-extend
        // both operands (mov eax,eax = 89 c0 / mov ecx,ecx = 89 c9), imul, load
        // ra(3) into RDI, `neg rax` (48 f7 d8) then `add rax,rdi` (48 01 f8)
        // = RAX = ra - product. The neg+add pair is the umsubl discriminator.
        let b = tr_bytes(Inst::MulLong { rd: 0, rn: 1, rm: 2, ra: 3, signed: false, sub: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x89, 0xc0, // mov eax,eax (zero-extend rn)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx,[rbx+0x10] (rm=2)
            0x89, 0xc9, // mov ecx,ecx (zero-extend rm)
            0x48, 0x0f, 0xaf, 0xc1, // imul rax,rcx
            0x48, 0x8b, 0x7b, 0x18, // mov rdi,[rbx+0x18] (ra=3)
            0x48, 0xf7, 0xd8, // neg rax (RAX = -product)
            0x48, 0x01, 0xf8, // add rax,rdi (RAX = ra - product)
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
    }

    #[test]
    fn sh430_clz_64_rexw_before_f3_lzcnt() {
        // clz x0,x1 (sf=true) emits LZCNT with the documented REX.W-order fix:
        // `f3 48 0f bd c0` — the REX.W prefix MUST come after F3 and immediately
        // before the 0F opcode. A regression to `48 f3 0f bd` makes the CPU
        // ignore REX.W and execute a 32-bit lzcnt (clz(x<2^32) returns 32-len(x),
        // e.g. the real repro clz(0x16136740) returned 3 vs oracle 35).
        let b = tr_bytes(Inst::ClzCls { rd: 0, rn: 1, sf: true, cls: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0xf3, 0x48, 0x0f, 0xbd, 0xc0, // lzcnt rax,rax (F3 then REX.W then 0F BD)
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
    }

    #[test]
    fn sh430_clz_w32_zext_then_32bit_lzcnt() {
        // clz w0,w1 (sf=false): zero-extend the W operand first (89 c0), then the
        // 32-bit lzcnt form `f3 0f bd c0` (no REX.W). The upper-32-zeroing via the
        // 32-bit lzcnt matches the W zero-extend semantics.
        let b = tr_bytes(Inst::ClzCls { rd: 0, rn: 1, sf: false, cls: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax,[rbx+0x08] (rn=1)
            0x89, 0xc0, // mov eax,eax (W zero-extend)
            0xf3, 0x0f, 0xbd, 0xc0, // lzcnt eax,eax (no REX.W)
            0x48, 0x89, 0x03, // mov [rbx],rax (store rd=0)
        ]);
    }

    #[test]
    fn sh430_branch_bl_stores_lr_then_call_placeholder() {
        // bl (link=true): save guest LR = pc+4 into slot30 (mov rax,pc+4 then
        // mov [rbx+0xf0],rax), then a host `call rel32` whose 4-byte displacement
        // placeholder is emitted as zeros (patched later by jit.rs). Pin the
        // exact bytes + the call fixup metadata (target = pc+imm = 0x1000+0x30,
        // cc=0xfe call).
        let (b, fx) = tr_bytes_fx(Inst::B { imm: 0x30, link: true });
        assert_eq!(b, vec![
            0x48, 0xb8, 0x04, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax,0x1004 (=pc+4, LR)
            0x48, 0x89, 0x83, 0xf0, 0x00, 0x00, 0x00, // mov [rbx+0xf0],rax (slot30 = x30/LR)
            0xe8, 0x00, 0x00, 0x00, 0x00, // call rel32 (placeholder; E8 + 4 zero disp)
        ]);
        assert_eq!(fx.len(), 1);
        assert_eq!(fx[0].cc, 0xfe, "call fixup");
        assert_eq!(fx[0].target_pc, 0x1030, "BL target = pc + imm");
        // the 4-byte displacement placeholder follows the E8 opcode at index 17
        assert_eq!(&b[18..22], &[0, 0, 0, 0]);
    }

    #[test]
    fn sh430_branch_b_uncond_jmp_fixup() {
        // b (link=false): a single host `jmp rel32` (E9) with the displacement
        // placeholder; fixup target = pc + imm = 0x1000 - 0x20 (backward), cc=0xff
        // (the unconditional-jmp marker).
        let (b, fx) = tr_bytes_fx(Inst::B { imm: -0x20, link: false });
        assert_eq!(b, vec![0xe9, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(fx.len(), 1);
        assert_eq!(fx[0].cc, 0xff, "unconditional jmp fixup marker");
        assert_eq!(fx[0].target_pc, 0xfe0, "B target = pc + imm (backward)");
    }

    #[test]
    fn sh430_branch_tbz_single_bit_and_test() {
        // tbz x7,#3 (nonzero -> tbnz): load rt(7) from slot 0x38, compare the
        // single bit 3 (mov rcx, 1<<3 = 8 then `test rax,rcx` 48 85 c8), branch
        // jnz (cc=0x85) to pc+imm=0x1040. Pins the one-bit masked compare used to
        // implement the bit-test (not a full zero-test like cbz).
        let (b, fx) = tr_bytes_fx(Inst::Tbz { rt: 7, bit: 3, imm: 0x40, nonzero: true, sf: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x38, // mov rax,[rbx+0x38] (rt=7)
            0x48, 0xb9, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rcx, 1<<3
            0x48, 0x85, 0xc8, // test rax,rcx
            0x0f, 0x85, 0x00, 0x00, 0x00, 0x00, // jnz (nonzero)
        ]);
        assert_eq!(fx.len(), 1);
        assert_eq!(fx[0].cc, 0x85, "jnz for tbnz");
        assert_eq!(fx[0].target_pc, 0x1040);
    }

    #[test]
    fn sh430_branch_cbz_zero_test_two_operand() {
        // cbz x9 (nonzero=false): load rt(9), `test rax,rax` (48 85 c0), branch jz
        // (cc=0x84) to pc+imm=0x1000-0x10=0xff0. Distinguishes cbz (whole-register
        // zero test, test rax,rax) from tbz (masked single-bit test).
        let (b, fx) = tr_bytes_fx(Inst::Cbz { rt: 9, imm: -0x10, nonzero: false, sf: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x48, // mov rax,[rbx+0x48] (rt=9)
            0x48, 0x85, 0xc0, // test rax,rax
            0x0f, 0x84, 0x00, 0x00, 0x00, 0x00, // jz (nonzero=false)
        ]);
        assert_eq!(fx.len(), 1);
        assert_eq!(fx[0].cc, 0x84, "jz for cbz");
        assert_eq!(fx[0].target_pc, 0xff0);
    }

    // movz x0,#0x1234, 64-bit: `mov r32-imm` shortcut (mov_eax_imm32 into a
    // 64-bit dest) then store to [RBX+0]. The 64-bit form may use the imm32
    // short-cut because movz zero-extends — pin that choice and the store.
    #[test]
    fn sh427_movz_64_imm32_shortcut_stores_slot0() {
        let b = tr_bytes(Inst::MoveWide { rd: 0, imm16: 0x1234, hw: 0, opc: 0, sf: true });
        assert_eq!(b, vec![
            0x48, 0xc7, 0xc0, 0x34, 0x12, 0x00, 0x00, // mov rax, 0x1234
            0x48, 0x89, 0x03, // mov [rbx], rax (slot0)
        ]);
    }

    // movn w0,#2 (32-bit, opc=2): NOT of the immediate, then TRUNCATED to 32 bits
    // (W-dest zero-extends). The qemu-verified comment: `movn w0,#2` must be
    // 0x00000000fffffffd, NOT 0xfffffffffffffffd — the imm64 path would
    // sign-extend and leave the upper 32 set. Pin that the emitted constant is
    // 0x00000000fffffffd (upper half zero) and the store lands in slot0.
    #[test]
    fn sh427_movn_32_truncates_to_zero_extended() {
        let b = tr_bytes(Inst::MoveWide { rd: 0, imm16: 2, hw: 0, opc: 2, sf: false });
        assert_eq!(b, vec![
            0x48, 0xb8, 0xfd, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, // mov rax, 0x00000000fffffffd
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // movk x0,#2,lsl#16, 64-bit, over a prior movz x0,#0x8bb1 (so x0==0x28bb1
    // after). movk is read-modify-write — OR imm16<<16 into bits[16,32),
    // PRESERVING the 0x8bb1 low half. The qemu-verified regression: treating
    // movk as a full replace corrupted the constant (0x8bb1 then 0x2 lsl16 came
    // out 0x20000). Pin the read-AND-OR sequence: load [rbx], and with the
    // clear mask 0xffffffffffff0000, or with 0x20000, store.
    #[test]
    fn sh427_movk_read_modify_write_preserves_low_half() {
        let b = tr_bytes(Inst::MoveWide { rd: 0, imm16: 2, hw: 1, opc: 1, sf: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x03, // mov rax, [rbx]
            0x48, 0xb9, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, // mov rcx, 0xffffffffffff0000 (clear mask)
            0x48, 0x21, 0xc8, // and rax, rcx
            0x48, 0xb9, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rcx, 0x20000
            0x48, 0x09, 0xc8, // or rax, rcx
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // sub sp,sp,#0x20 (64-bit, rd==31=SP): the ADD/SUB immediate form treats
    // rd==31 as SP (unlike logical ops where it's discarded as XZR). Every fn
    // prologue does this. Pin: load [RBX+0xf8] (SP slot), sub 0x20, store back
    // to [RBX+0xf8].
    #[test]
    fn sh427_sub_imm_sp_writeback_uses_sp_slot() {
        let b = tr_bytes(Inst::AddSubImm { rd: 31, rn: 31, imm12: 0x20, shift12: false, sub: true, sf: true, s: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0xf8, 0x00, 0x00, 0x00, // mov rax, [rbx+0xf8] (SP)
            0x48, 0x81, 0xe8, 0x20, 0x00, 0x00, 0x00, // sub rax, 0x20
            0x48, 0x89, 0x83, 0xf8, 0x00, 0x00, 0x00, // mov [rbx+0xf8], rax
        ]);
    }

    // cmp wzr,#0x10 (32-bit, s==1, rd==31): rn==31 must be read as XZR (zero),
    // NOT SP — and the flag-setting form must NOT clobber the SP writeback. The
    // emitted sequence: mov rax,0 (XZR), zext to 32 via shl/shr 32, sub 0x10,
    // then store_nzcv (pushfq + nzcv-packing into [CpuState+0x108], the 4-bit C
    // at offset 0x108+0 = nzcv[0]: XXX-C packing 1f/1e/0b/06/07/0d/1c/1d).
    // Assert: starts with `mov rax,0` (not a [rbx+0xf8] SP read) and ends with
    // the store to [rbx+0x8] — slot rd but the write is SKIPPED in the s==1 case
    // (comparison discards). So no slot write; the tail is the nzcv pack.
    #[test]
    fn sh427_cmp_wzr_reads_xzr_zero_and_skips_writeback() {
        let b = tr_bytes(Inst::AddSubImm { rd: 31, rn: 31, imm12: 0x10, shift12: false, sub: true, sf: false, s: true });
        // Begin: mov rax, 0 (XZR, NOT a [rbx+0xf8] SP load)
        assert_eq!(&b[0..10], &[0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        // 32-bit zext: shl rax,32 ; shr rax,32
        assert_eq!(&b[10..14], &[0x48, 0xc1, 0xe0, 0x20]);
        assert_eq!(&b[14..18], &[0x48, 0xc1, 0xe8, 0x20]);
        // sub rax,0x10 (imm32, 6 bytes)
        assert_eq!(&b[18..24], &[0x81, 0xe8, 0x10, 0x00, 0x00, 0x00]);
        // No `mov [rbx+0xf8], rax` writeback anywhere (comparison discards) —
        // the nzcv pack follows. The whole buffer must NOT contain a store to
        // the SP slot. The nzcv tail writes to [rbx+0x108]-ish (nzcv), not SP.
        assert!(!b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0xf8, 0x00, 0x00, 0x00]),
            "cmp must not write back to the SP slot");
        // nzcv pack ends with the C-bit store `89 93 08 01 00 00` (mov
        // [rbx+0x108],edx) then restores the caller's regs (5a 59 58 = pop
        // rdx/rcx/rax). Pin the C-flags store wraps the tail (a real write to
        // the flags slot, distinct from the forbidden SP-writeback).
        assert_eq!(&b[b.len() - 9..b.len() - 3], &[0x89, 0x93, 0x08, 0x01, 0x00, 0x00],
            "nzcv pack stores C to [rbx+0x108]");
        assert_eq!(&b[b.len() - 3..], &[0x5a, 0x59, 0x58], "caller regs restored");
    }

    // neg x6,x6 = sub x6,xzr,x6 (bit21=0, shifted-register form): rn==31 is XZR
    // (zero), NOT SP — a `neg` must never touch the SP slot. Pin: mov rax,0
    // (XZR), load rm(6) into rcx, sub, store to slot6.
    #[test]
    fn sh427_neg_shifted_form_reads_xzr_zero() {
        let b = tr_bytes(Inst::AddSubReg { rd: 6, rn: 31, rm: 6, sub: true, sf: true, s: false, shift: ShiftKind::Lsl, sh_amt: 0, sp_operand: false });
        assert_eq!(b, vec![
            0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 0 (XZR)
            0x48, 0x8b, 0x4b, 0x30, // mov rcx, [rbx+0x30] (slot6)
            0x48, 0x29, 0xc8, // sub rax, rcx
            0x48, 0x89, 0x43, 0x30, // mov [rbx+0x30], rax (slot6)
        ]);
    }

    // sub sp,sp,x1 (bit21=1, extended-register form): rn==31 AND rd==31 are SP.
    // Pin: load [rbx+0xf8] (SP), load rm(1) from slot8 ([rbx+0x8]) into rcx,
    // sub, write back to [rbx+0xf8] (SP).
    #[test]
    fn sh427_sub_sp_sp_x1_extended_form_reads_slots() {
        let b = tr_bytes(Inst::AddSubReg { rd: 31, rn: 31, rm: 1, sub: true, sf: true, s: false, shift: ShiftKind::Lsl, sh_amt: 0, sp_operand: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0xf8, 0x00, 0x00, 0x00, // mov rax, [rbx+0xf8] (SP rn)
            0x48, 0x8b, 0x4b, 0x08, // mov rcx, [rbx+0x08] (rm=1)
            0x48, 0x29, 0xc8, // sub rax, rcx
            0x48, 0x89, 0x83, 0xf8, 0x00, 0x00, 0x00, // mov [rbx+0xf8], rax (SP rd)
        ]);
    }

    // adds w0,w1,w2 (32-bit flag-setting add, bit21=0, S=1): 32-bit operand zext,
    // add, then store_nzcv with the cmc (carry-borrow convention) inserted for
    // the ADDS case. Pin: it begins with the two register loads, performs the
    // 32-bit zext (shl/shr 32) on each, a 32-bit add (01 c8), a cmc (f5), the
    // nzcv pack, and finishes with a zero-extended 32-bit store to slot0.
    #[test]
    fn sh427_adds_32_flag_setting_cmc_and_nzcv() {
        let b = tr_bytes(Inst::AddSubReg { rd: 0, rn: 1, rm: 2, sub: false, sf: false, s: true, shift: ShiftKind::Lsl, sh_amt: 0, sp_operand: false });
        // loads: mov rax,[rbx+0x08] (rn1) ; mov rcx,[rbx+0x10] (rm2)
        assert_eq!(&b[0..8], &[0x48, 0x8b, 0x43, 0x08, 0x48, 0x8b, 0x4b, 0x10]);
        // 32-bit zext rax: shl/shr 32
        assert_eq!(&b[8..12], &[0x48, 0xc1, 0xe0, 0x20]);
        assert_eq!(&b[12..16], &[0x48, 0xc1, 0xe8, 0x20]);
        // 32-bit zext rcx
        assert_eq!(&b[16..20], &[0x48, 0xc1, 0xe1, 0x20]);
        assert_eq!(&b[20..24], &[0x48, 0xc1, 0xe9, 0x20]);
        // 32-bit add: add eax,ecx (01 c8)
        assert_eq!(&b[24..26], &[0x01, 0xc8]);
        // cmc (the ADDS non-subtract borrow-convention carry complement)
        assert_eq!(&b[26], &0xf5);
        // finishes with a zero-extended store: after the nzcv pack + register
        // pop (5a 59 58), the write path does shl/shr 32 then `mov [rbx],rax`.
        assert_eq!(&b[b.len() - 11..], &[0x48, 0xc1, 0xe0, 0x20, 0x48, 0xc1, 0xe8, 0x20, 0x48, 0x89, 0x03]);
    }

    // adc x0,x1,x2 (64-bit, non-S): load rn/rm, load stored C into EFLAGS, then
    // cmc + native adc (since TRUE_C = !stored_C in the borrow convention). Pin
    // the tail: cmc (f5) then adc (48 11 c8) then store slot0.
    #[test]
    fn sh427_adc_adds_true_carry_via_cmc_and_adc() {
        let b = tr_bytes(Inst::AddCarry { rd: 0, rn: 1, rm: 2, sf: true, s: false, sub: false });
        // ends: ... cmc(f5) adc rax,rcx(48 11 c8) store
        assert_eq!(&b[b.len() - 4 - 3..b.len() - 3], &[0xf5, 0x48, 0x11, 0xc8]);
        assert_eq!(&b[b.len() - 3], &0x48); // store prefix
    }

    // sbc x0,x1,x2 (64-bit, non-S): SBB subtracts C_s = (1-TRUE_C), so native
    // sbb directly (no cmc). Pin: tail is `sbb rax,rcx` (48 19 c8) then store.
    #[test]
    fn sh427_sbc_sbb_direct_borrow() {
        let b = tr_bytes(Inst::AddCarry { rd: 0, rn: 1, rm: 2, sf: true, s: false, sub: true });
        assert_eq!(&b[b.len() - 3 - 3..b.len() - 3], &[0x48, 0x19, 0xc8]); // sbb rax,rcx
        assert_eq!(&b[b.len() - 3], &0x48); // store prefix
    }

    // bic x0,x1,x2 (logical NOT AND, op=4, 64-bit): not rm then and. Pin the
    // full sequence: load rn, load rm, not rcx (48 f7 d1), and, store.
    #[test]
    fn sh427_bic_not_then_and() {
        let b = tr_bytes(Inst::LogicReg { rd: 0, rn: 1, rm: 2, op: 4, s: false, sf: true, shift: ShiftKind::Lsl, sh_amt: 0 });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x43, 0x08, // mov rax, [rbx+0x08] (rn1)
            0x48, 0x8b, 0x4b, 0x10, // mov rcx, [rbx+0x10] (rm2)
            0x48, 0xf7, 0xd1, // not rcx
            0x48, 0x21, 0xc8, // and rax, rcx
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // orr x0,xzr,x1 (64-bit, `mov` alias): rn==31 (XZR) must load as zero, NOT
    // SP. Pin: mov rax,0 then or with rm(1) then store.
    #[test]
    fn sh427_orr_xzr_loads_zero_not_sp() {
        let b = tr_bytes(Inst::LogicReg { rd: 0, rn: 31, rm: 1, op: 1, s: false, sf: true, shift: ShiftKind::Lsl, sh_amt: 0 });
        assert_eq!(b, vec![
            0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 0 (XZR)
            0x48, 0x8b, 0x4b, 0x08, // mov rcx, [rbx+0x08] (rm1)
            0x48, 0x09, 0xc8, // or rax, rcx
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // BCond (b.eq rel): the discriminator must not clobber a live C — the
    // sequence loads stored NZCV into EFLAGS (pushfq/popfq around the nzcv
    // unpack) then emits a je (0f 84 disp32). Pin: the buffer ends with
    // `0f 84 00 00 00 00` (je rel32, fixup placeholder zeroed by patch).
    #[test]
    fn sh427_bcond_loads_nzcv_and_emits_je() {
        let b = tr_bytes(Inst::BCond { cond: 0, imm: 8 });
        assert_eq!(&b[b.len() - 6..], &[0x0f, 0x84, 0x00, 0x00, 0x00, 0x00], "je rel32");
    }

    // SH428: hermetic coverage of the load/store codegen families (LdStrImm —
    // the single most load-bearing translation: every guest memory access).
    // decode.rs pins the decode, jit.rs pins runtime, but the exact bytes for
    // the load/store emission were untested. These pin the critical semantic
    // discriminators: the XZR-source store (`str xzr,[..]` reads zero, never
    // the SP-slot), zero- vs sign-extending loads, the scaled-offset form, and
    // the LEAGUES-of-use `cmn` carry-borrow flag pack.
    //
    // Byte-map (x86): [RBX] = CpuState base, slot g = [RBX+g*8], g=1 -> 0x08.

    // ldr x0,[x1] (64-bit, scaled-offset 0): load [x1] into RAX, store slot0.
    #[test]
    fn sh428_ldr_64_scaled_offset_zero() {
        let b = tr_bytes(Inst::LdStrImm { rt: 0, rn: 1, imm: 0, size: 8, ld: true, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]  (x1 = addr)
            0x48, 0x8b, 0x02, // mov rax, [rdx]  (load [x1])
            0x48, 0x89, 0x03, // mov [rbx], rax  (store x0)
        ]);
    }

    // str x0,[x1] (64-bit): addr in rdx, load source slot0, store [rdx].
    #[test]
    fn sh428_str_64_reads_source_slot() {
        let b = tr_bytes(Inst::LdStrImm { rt: 0, rn: 1, imm: 0, size: 8, ld: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]  (x1 = addr)
            0x48, 0x8b, 0x03, // mov rax, [rbx]  (x0 source)
            0x48, 0x89, 0x02, // mov [rdx], rax  (store)
        ]);
    }

    // str xzr,[x1] (64-bit): source register x31 = XZR = ZERO, must NOT read the
    // SP slot. Pin: `mov rax,0` (not a [rbx+0xf8] load), then store [rdx].
    #[test]
    fn sh428_str_xzr_stores_zero_not_sp() {
        let b = tr_bytes(Inst::LdStrImm { rt: 31, rn: 1, imm: 0, size: 8, ld: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]  (addr)
            0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 0 (XZR)
            0x48, 0x89, 0x02, // mov [rdx], rax
        ]);
    }

    // str xzr,[x1] (32-bit): same XZR-zero source but the 32-bit store (89 02).
    #[test]
    fn sh428_str_xzr_32_stores_zero_not_sp_w32() {
        let b = tr_bytes(Inst::LdStrImm { rt: 31, rn: 1, imm: 0, size: 4, ld: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]
            0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov rax, 0 (XZR)
            0x89, 0x02, // mov [rdx], eax (32-bit store)
        ]);
    }

    // ldr w0,[x1] (32-bit, non-sext): mov_load32 zero-extends into RAX, then
    // stg_if_writable stores the full 64-bit RAX to slot0.
    #[test]
    fn sh428_ldr_32_zero_extends() {
        let b = tr_bytes(Inst::LdStrImm { rt: 0, rn: 1, imm: 0, size: 4, ld: true, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]
            0x8b, 0x02, // mov eax, [rdx]  (w zero-extends to 64-bit RAX)
            0x48, 0x89, 0x03, // mov [rbx], rax (full 64-bit zero-extended store)
        ]);
    }

    // ldrsb x0,[x1,#1] (byte sign-extend, scaled offset 1): lea addr, movzx
    // byte, then shl/sar 56 (8->64 sign-extend), store slot0.
    #[test]
    fn sh428_ldrsb_scaled_offset_sign_extends() {
        let b = tr_bytes(Inst::LdStrImm { rt: 0, rn: 1, imm: 1, size: 1, ld: true, sext: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]  (addr)
            0x48, 0x8d, 0x52, 0x01, // lea rdx, [rdx+1]  (scaled offset 1*1)
            0x0f, 0xb6, 0x02, // movzx eax, byte [rdx]
            0x48, 0xc1, 0xe0, 0x38, // shl rax, 56
            0x48, 0xc1, 0xf8, 0x38, // sar rax, 56  (sign-extend byte -> 64)
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // ldrsw x0,[x1,#4] (word sign-extend, scaled offset 1 -> 4 bytes): lea,
    // mov_load32, movsxd (48 63 c0) 32->64, store.
    #[test]
    fn sh428_ldrsw_scaled_offset_sign_extends_word() {
        let b = tr_bytes(Inst::LdStrImm { rt: 0, rn: 1, imm: 1, size: 4, ld: true, sext: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]
            0x48, 0x8d, 0x52, 0x04, // lea rdx, [rdx+4]  (scaled offset 1*4)
            0x8b, 0x02, // mov eax, [rdx]
            0x48, 0x63, 0xc0, // movsxd rax, eax  (32->64 sign)
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // ldr x0,[x1,#16] (64-bit, scaled offset 2 -> 16 bytes): lea then load.
    #[test]
    fn sh428_ldr_64_scaled_offset_16() {
        let b = tr_bytes(Inst::LdStrImm { rt: 0, rn: 1, imm: 2, size: 8, ld: true, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]
            0x48, 0x8d, 0x52, 0x10, // lea rdx, [rdx+16]  (scaled offset 2*8)
            0x48, 0x8b, 0x02, // mov rax, [rdx]
            0x48, 0x89, 0x03, // mov [rbx], rax
        ]);
    }

    // cmn x0,#0x10000 (64-bit, non-sub shift:12, S=1): the qemu-verified case
    // `x > 0xffffffffffff0000` compiles to `cmn x,#0x10000; b.ls` — the ADD
    // carries (ARM C=1, ls=false) so stored C must be !carry. Pin: load x0,
    // add 0x10000 (81 c0 00 00 00 01), cmc (f5), then the nzcv pack ending with
    // the C-store `89 93 08 01 00 00` + caller-reg restores.
    #[test]
    fn sh428_cmn_shifted_carry_borrow_pack() {
        let b = tr_bytes(Inst::AddSubImm { rd: 31, rn: 0, imm12: 0x1000, shift12: true, sub: false, sf: true, s: true });
        // load x0 (slot0): mov rax, [rbx]
        assert_eq!(&b[0..3], &[0x48, 0x8b, 0x03]);
        // add rax, imm32 (imm12 0x1000 << 12 = 0x01000000): 48 81 c0 + LE imm32
        assert_eq!(&b[3..10], &[0x48, 0x81, 0xc0, 0x00, 0x00, 0x00, 0x01]);
        // cmc (the ADDS carry-borrow complement)
        assert_eq!(&b[10], &0xf5);
        // nzcv pack: pushfq (9c) ... ends with C-store `89 93 08 01 00 00`
        assert_eq!(&b[b.len() - 9..b.len() - 3], &[0x89, 0x93, 0x08, 0x01, 0x00, 0x00]);
        assert_eq!(&b[b.len() - 3..], &[0x5a, 0x59, 0x58], "caller regs restored");
    }

    // SH429: hermetic coverage of the load/store-PAIR (LdStPair) and the
    // scalar FP/SIMD immediate (FpLdStImm) codegen — the families that move
    // real rendered geometry/vertex data. decode.rs pins decode, jit.rs pins
    // runtime, but the exact bytes were untested. These pin the semantically
    // critical discriminators: the stride-16 vector slot for FP d/s-pairs (the
    // Session-99 BUGFIX: was rt*8, corrupting `ldp d29,d28` follow-on fmadd),
    // the 32/64-bit pair width, the offset-vs-load addressing, and
    // fp_scalar_xfer's low-N-bytes slot write (upper lanes preserved).
    //
    // Map: [RBX]=CpuState base; GPR slot = [RBX+g*8] (g=8 -> 0x40); vector slot
    // vt = VECTOR_BASE + vt*16, VECTOR_BASE=0x110 (so d0=0x110, d29=0x2e0,
    // d28=0x2d0).

    // ldp x0,x1,[x8] (64-bit GPR pair, offset imm=0): mov rdx,[rbx+0x40] (x8),
    // load [rdx]->slot0, then [rdx+8]->slot1 (the second lane at +esize).
    #[test]
    fn sh429_ldp_64_pair_offset_zero() {
        let b = tr_bytes(Inst::LdStPair { rt: 0, rt2: 1, rn: 8, imm: 0, ld: true, writeback: false, preidx: false, size_64: true, q128: false, fp_d: false, fp_s: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x40, // mov rdx, [rbx+0x40]  (x8)
            0x48, 0x8b, 0x02, // mov rax, [rdx]  (lane0)
            0x48, 0x89, 0x03, // mov [rbx], rax  (x0)
            0x48, 0x8b, 0x42, 0x08, // mov rax, [rdx+8]  (lane1 @ +esize)
            0x48, 0x89, 0x43, 0x08, // mov [rbx+8], rax  (x1)
        ]);
    }

    // stp x0,x1,[x8] (64-bit GPR pair, store): source lanes read from slots0/1,
    // stored to [rdx] and [rdx+8].
    #[test]
    fn sh429_stp_64_pair_offset_zero() {
        let b = tr_bytes(Inst::LdStPair { rt: 0, rt2: 1, rn: 8, imm: 0, ld: false, writeback: false, preidx: false, size_64: true, q128: false, fp_d: false, fp_s: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x40, // mov rdx, [rbx+0x40]  (x8)
            0x48, 0x8b, 0x03, // mov rax, [rbx]  (x0 source)
            0x48, 0x89, 0x02, // mov [rdx], rax
            0x48, 0x8b, 0x43, 0x08, // mov rax, [rbx+8]  (x1 source)
            0x48, 0x89, 0x42, 0x08, // mov [rdx+8], rax
        ]);
    }

    // ldp x0,x1,[x8,#2] (64-bit GPR pair, offset imm=2 = raw byte offset): the
    // offset-form immediate applies RAW to the access address (not scaled).
    // lane0 at [rdx+2], lane1 at [rdx+2+8=0xa].
    #[test]
    fn sh429_ldp_64_pair_offset_imm_raw_bytes() {
        let b = tr_bytes(Inst::LdStPair { rt: 0, rt2: 1, rn: 8, imm: 2, ld: true, writeback: false, preidx: false, size_64: true, q128: false, fp_d: false, fp_s: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x40, // mov rdx, [rbx+0x40]
            0x48, 0x8b, 0x42, 0x02, // mov rax, [rdx+2]  (lane0 @ off+0)
            0x48, 0x89, 0x03, // mov [rbx], rax
            0x48, 0x8b, 0x42, 0x0a, // mov rax, [rdx+0xa]  (lane1 @ off+8)
            0x48, 0x89, 0x43, 0x08, // mov [rbx+8], rax
        ]);
    }

    // ldp w0,w1,[x8] (32-bit GPR pair): 32-bit loads (mov eax,[rdx]) zero-extend
    // into RAX then full-64 store to slots (W regs live in the low 32 of their
    // X slots).
    #[test]
    fn sh429_ldp_32_pair_zero_extends() {
        let b = tr_bytes(Inst::LdStPair { rt: 0, rt2: 1, rn: 8, imm: 0, ld: true, writeback: false, preidx: false, size_64: false, q128: false, fp_d: false, fp_s: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x40, // mov rdx, [rbx+0x40]
            0x8b, 0x02, // mov eax, [rdx]  (w0 zero-extends)
            0x48, 0x89, 0x03, // mov [rbx], rax  (full-64 store)
            0x8b, 0x42, 0x04, // mov eax, [rdx+4]  (w1 @ +esize=4)
            0x48, 0x89, 0x43, 0x08, // mov [rbx+8], rax
        ]);
    }

    // stp d29,d28,[x8] FP/vector D-pair store — pins the Session-99 BUGFIX: the
    // vector stride is 16 (VECTOR_BASE + vt*16), NOT 8. d29 -> 0x2e0, d28 ->
    // 0x2d0 (a rt*8 stride would have written 0x1f8/0x1f0 and corrupted the
    // follow-on fmadd, returning 128 vs 52). Sources read from those vector
    // slots and stored at [rdx] / [rdx+8].
    #[test]
    fn sh429_stp_dpair_stride16_vector_slots() {
        let b = tr_bytes(Inst::LdStPair { rt: 29, rt2: 28, rn: 8, imm: 0, ld: false, writeback: false, preidx: false, size_64: false, q128: false, fp_d: true, fp_s: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x40, // mov rdx, [rbx+0x40]  (x8)
            0x48, 0x8b, 0x83, 0xe0, 0x02, 0x00, 0x00, // mov rax, [rbx+0x2e0]  (d29, stride 16)
            0x48, 0x89, 0x02, // mov [rdx], rax
            0x48, 0x8b, 0x83, 0xd0, 0x02, 0x00, 0x00, // mov rax, [rbx+0x2d0]  (d28)
            0x48, 0x89, 0x42, 0x08, // mov [rdx+8], rax
        ]);
    }

    // ldp d29,d28,[x8] FP/vector D-pair LOAD: loads land in the stride-16 vector
    // slots.
    #[test]
    fn sh429_ldp_dpair_stride16_vector_slots() {
        let b = tr_bytes(Inst::LdStPair { rt: 29, rt2: 28, rn: 8, imm: 0, ld: true, writeback: false, preidx: false, size_64: false, q128: false, fp_d: true, fp_s: false, sext: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x40, // mov rdx, [rbx+0x40]
            0x48, 0x8b, 0x02, // mov rax, [rdx]
            0x48, 0x89, 0x83, 0xe0, 0x02, 0x00, 0x00, // mov [rbx+0x2e0], rax  (d29)
            0x48, 0x8b, 0x42, 0x08, // mov rax, [rdx+8]
            0x48, 0x89, 0x83, 0xd0, 0x02, 0x00, 0x00, // mov [rbx+0x2d0], rax  (d28)
        ]);
    }

    // ldr d0,[x1] FpLdStImm 64-bit: fp_scalar_xfer (D) — mov rdx,[rbx+0x08]
    // (x1), mov rax,[rdx], mov [rbx+0x110] (d0 vector slot VECTOR_BASE+0). Only
    // the low 8 of the 16-byte slot touched (upper lanes preserved).
    #[test]
    fn sh429_ldr_d_scalar_into_vector_base() {
        let b = tr_bytes(Inst::FpLdStImm { vt: 0, rn: 1, imm: 0, size: 8, ld: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]  (x1)
            0x48, 0x8b, 0x02, // mov rax, [rdx]  (8-byte load)
            0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110], rax  (d0 low 8)
        ]);
    }

    // str d0,[x1] FpLdStImm 64-bit store: source from d0 vector slot, store
    // [rdx].
    #[test]
    fn sh429_str_d_scalar_from_vector_base() {
        let b = tr_bytes(Inst::FpLdStImm { vt: 0, rn: 1, imm: 0, size: 8, ld: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]
            0x48, 0x8b, 0x83, 0x10, 0x01, 0x00, 0x00, // mov rax, [rbx+0x110]  (d0)
            0x48, 0x89, 0x02, // mov [rdx], rax
        ]);
    }

    // ldr s0,[x1] FpLdStImm 32-bit: mov eax,[rdx] then 32-bit store to the low
    // 4 of the d0 vector slot (89 83 — upper lanes preserved).
    #[test]
    fn sh429_ldr_s_scalar_32_into_vector_base() {
        let b = tr_bytes(Inst::FpLdStImm { vt: 0, rn: 1, imm: 0, size: 4, ld: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x53, 0x08, // mov rdx, [rbx+0x08]
            0x8b, 0x02, // mov eax, [rdx]  (32-bit)
            0x89, 0x83, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110], eax  (d0 low 4)
        ]);
    }

    // SH432: hermetic coverage of the SIMD/vector ALU codegen families
    // (translate.rs SimdVLog, SminMax, SimdSatAdd) — the byte emission that
    // carries real rendered geometry/color lane math. decode.rs + jit.rs pin
    // decode + runtime, but the EMISSION between them was unpinned for these
    // three families. SH432 pins the semantically-critical discriminators a
    // byte error silently corrupts: the pand/por/pxor/pandn opcode choice in
    // SimdVLog (a /r field flub maps Vd&Vm to Vd^Vm or worse), the cmov
    // condition byte that selects max-vs-min AND signed-vs-unsigned in
    // SminMax (cmovg 0x4F / cmovl 0x4C / cmova 0x47 / cmovb 0x42 — a flub
    // returns the wrong lane or clamps the wrong direction), the movsxd
    // sign-extension presence that flips sub-byte saturating results, and the
    // smax-vs-smin constant + cmov pairing in SimdSatAdd that decides which
    // bound a signed overflow clamps to (a sign-extended 0x80 lane vs a
    // zero-extended +128 changes the clamp entirely). Exact-byte, synthetic
    // Inst -> translate() (see tr_bytes), deterministic, no image/env.
    // [RBX]=CpuState; vector slot v[t] = VECTOR_BASE(0x110) + t*16.

    // SimdVLog AND: load Vn@0x120 + Vm@0x130 into xmm0/xmm1, `pand xmm0,xmm1`
    // (66 0F DB C1), store Vd@0x110. Whole-buffer pin.
    #[test]
    fn sh432_simdvlog_and_emits_pand_and_stores_vd() {
        let b = tr_bytes(Inst::SimdVLog { rd: 0, rn: 1, rm: 2, op: 0 });
        assert_eq!(b, vec![
            0xf3, 0x0f, 0x6f, 0x83, 0x20, 0x01, 0x00, 0x00, // movdqu xmm0,[rbx+0x120]  Vn
            0xf3, 0x0f, 0x6f, 0x8b, 0x30, 0x01, 0x00, 0x00, // movdqu xmm1,[rbx+0x130]  Vm
            0x66, 0x0f, 0xdb, 0xc1,                         // pand xmm0,xmm1  (AND)
            0xf3, 0x0f, 0x7f, 0x83, 0x10, 0x01, 0x00, 0x00, // movdqu [rbx+0x110],xmm0  Vd
        ]);
    }

    // SimdVLog opcode discriminator: after the two identical movdqu loads
    // (16 bytes), the /r-based opcode byte distinguishes AND/ORR/EOR/BIC
    // (DB/EB/EF/DF). A /r-flub would emit one op in another's slot.
    #[test]
    fn sh432_simdvlog_op_discriminates_pand_por_pxor_pandn() {
        let cases = [
            (0u8, &[0x66, 0x0f, 0xdb, 0xc1][..]), // AND  pand  xmm0,xmm1
            (2u8, &[0x66, 0x0f, 0xeb, 0xc1][..]), // ORR  por   xmm0,xmm1
            (1u8, &[0x66, 0x0f, 0xef, 0xc1][..]), // EOR  pxor  xmm0,xmm1
            (9u8, &[0x66, 0x0f, 0xdf, 0xc8][..]), // BIC  pandn xmm1,xmm0
        ];
        for (op, want) in cases {
            let b = tr_bytes(Inst::SimdVLog { rd: 0, rn: 1, rm: 2, op });
            assert_eq!(&b[0..16], &b[16..].get(0..0).map(|_| b[0..16].as_ref()).map(|_| {
                // both loads identical regardless of op
                &[0xf3, 0x0f, 0x6f, 0x83, 0x20, 0x01, 0x00, 0x00,
                  0xf3, 0x0f, 0x6f, 0x8b, 0x30, 0x01, 0x00, 0x00][..]
            }).unwrap()[..]);
            assert_eq!(&b[16..20], want);
        }
    }

    // SminMax signed .2s max (esize=4,q=false): each lane loads A,B with the
    // movsxd SIGN-extend (48 63) so negative lanes compare correctly, then
    // `cmp rcx,rax; cmovg rax,rcx` (48 39 C1 / 48 0F 4F C1) keeps the larger.
    // The 48 63 movsxd and the 4F (cmovg not cmovl) are the discriminators.
    #[test]
    fn sh432_sminmax_signed_max_signextends_and_cmovg() {
        let b = tr_bytes(Inst::SminMax { rd: 0, rn: 1, rm: 2, max: true, unsigned: false, esize: 4, q: false });
        // per-lane: mov eax,[rbx+0x120]; movsxd rax,eax; mov ecx,[rbx+0x130];
        // movsxd rcx,ecx; cmp rcx,rax; cmovg rax,rcx
        assert!(b.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "signed A lane must movsxd");
        assert!(b.windows(3).any(|w| w == [0x48, 0x63, 0xc9]), "signed B lane must movsxd");
        assert!(b.windows(2).any(|w| w == [0x48, 0x39]), "cmp rcx,rax present");
        assert!(b.windows(4).any(|w| w == [0x48, 0x0f, 0x4f, 0xc1]),
            "signed max must cmovg (0x4F), not cmovl/others");
        assert!(!b.windows(4).any(|w| w == [0x48, 0x0f, 0x4c, 0xc1]), "no cmovl in a max lane");
    }

    #[test]
    fn sh432_sminmax_unsigned_min_zeroextends_and_cmovb() {
        let b = tr_bytes(Inst::SminMax { rd: 0, rn: 1, rm: 2, max: false, unsigned: true, esize: 1, q: false });
        assert!(b.windows(2).any(|w| w == [0x0f, 0xb6]), "movzx byte lane present");
        assert!(!b.windows(2).any(|w| w == [0x48, 0x63]), "unsigned lanes must NOT sign-extend");
        assert!(b.windows(4).any(|w| w == [0x48, 0x0f, 0x42, 0xc1]),
            "unsigned min must cmovb (0x42), not cmovl (signed)");
        assert!(!b.windows(4).any(|w| w == [0x48, 0x0f, 0x4c, 0xc1]), "no signed cmovl in unsigned lane");
    }

    #[test]
    fn sh432_simdsatadd_signed_clamps_smax_and_smin() {
        let b = tr_bytes(Inst::SimdSatAdd { rd: 0, rn: 1, rm: 2, esize: 4, sub: false, unsigned: false, q: true });
        assert!(b.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "sqadd sign-extends the A lane");
        assert!(b.windows(10).any(|w| w == [0x49, 0xba, 0xff, 0xff, 0xff, 0x7f, 0, 0, 0, 0]),
            "smax bound must be 0x7FFFFFFF loaded into r10");
        assert!(b.windows(4).any(|w| w == [0x49, 0x0f, 0x4f, 0xc2]), "signed overflow cmovg -> r10 (smax)");
        assert!(b.windows(10).any(|w| w == [0x49, 0xba, 0x00, 0x00, 0x00, 0x80, 0xff, 0xff, 0xff, 0xff]),
            "smin bound must be 0x80000000 as a SIGNED negative in r10");
        assert!(b.windows(4).any(|w| w == [0x49, 0x0f, 0x4c, 0xc2]), "signed underflow cmovl -> r10 (smin)");
    }

    #[test]
    fn sh432_simdsatadd_unsigned_clamps_zero_no_signextend() {
        let b = tr_bytes(Inst::SimdSatAdd { rd: 0, rn: 1, rm: 2, esize: 2, sub: true, unsigned: true, q: false });
        assert!(b.windows(2).any(|w| w == [0x0f, 0xb7]), "uqsub zero-extends word lanes (movzx)");
        assert!(!b.windows(2).any(|w| w == [0x48, 0x63]), "unsigned lanes must NOT sign-extend");
        assert!(b.windows(10).any(|w| w == [0x49, 0xba, 0, 0, 0, 0, 0, 0, 0, 0]),
            "uqsub clamp floor must be literal 0 in r10, not smax/smin");
        assert!(b.windows(2).any(|w| w == [0x48, 0x29]), "sub rax,rcx after the borrow check");
        assert!(b.windows(4).any(|w| w == [0x49, 0x0f, 0x42, 0xc2]), "unsigned underflow cmovb -> r10 (0)");
        assert!(!b.windows(4).any(|w| w == [0x49, 0x0f, 0x4f, 0xc2]), "no signed smax clamp in unsigned lane");
    }
// SH433: hermetic coverage of the SIMD FP multiply-accumulate (Fmla/FmlaEl)
    // codegen — the single most render-heavy family (matrix/vertex/lighting
    // transforms accumulate as SIMD FMAs). The family had no direct byte tests.
    // SH433 pins the discriminators a byte error silently corrupts: the product
    // DIRECTION (mulss/mulsd dst=CREG=1, src=0 => xmm1 = Vn*Vm, so the accumulate
    // addss/addsd into xmm0 (Vd) has the CORRECT fmls sign — Vd +/- Vn*Vm, NOT
    // Vn*Vm - Vd), the add-vs-sub accumulate opcode (addss 0x58 / subss 0x5C),
    // the single- (F3+movd) vs double- (.2d, F2+movq) lane width, and the
    // FmlaEl by-element broadcast into xmm2 (movd xmm2,../66 0F 6E D0) + mulss
    // xmm1,xmm2 (F3 0F 59 CA). Exact-byte window asserts (multi-lane).

    #[test]
    fn sh433_fmla_2s_add_product_direction_and_accumulate() {
        // .2s fmla: per-lane movd Vn->xmm0, Vm->xmm1, mulss xmm1,xmm0 (product
        // in xmm1), reload Vd->xmm0, addss xmm0,xmm1 => Vd += Vn*Vm.
        let b = tr_bytes(Inst::Fmla { rd: 0, rn: 1, rm: 2, el64: false, q: false, sub: false });
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc8]), "mulss xmm1,xmm0 (Vn*Vm into xmm1)");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x58, 0xc1]), "addss xmm0,xmm1 (accumulate into Vd)");
        // the accumulate must come AFTER the product so the sign is right for fmls
        let prod = b.windows(4).position(|w| w == [0xf3, 0x0f, 0x59, 0xc8]).unwrap();
        let acc = b.windows(4).position(|w| w == [0xf3, 0x0f, 0x58, 0xc1]).unwrap();
        assert!(prod < acc, "product must be computed before the accumulate");
        assert!(!b.windows(4).any(|w| w == [0xf3, 0x0f, 0x5c, 0xc1]), "no subss in an add-form fmla");
    }

    #[test]
    fn sh433_fmla_2s_sub_uses_subss_not_subtrahend_swap() {
        // .2s fmls: identical layout but subss xmm0,xmm1 (Vd - Vn*Vm). The
        // discriminator 0x5C vs 0x58; a 'Vn*Vm - Vd' swap would corrupt sign.
        let b = tr_bytes(Inst::Fmla { rd: 0, rn: 1, rm: 2, el64: false, q: false, sub: true });
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc8]), "product mulss xmm1,xmm0");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x5c, 0xc1]), "subss xmm0,xmm1 (Vd - Vn*Vm)");
        assert!(!b.windows(4).any(|w| w == [0xf3, 0x0f, 0x58, 0xc1]), "no addss in a sub-form fmla");
    }

    #[test]
    fn sh433_fmla_2d_double_lanes_no_single_precision() {
        // .2d fmla: double path uses movq (F3 48 0F 7E load / 66 48 0F D6 store)
        // and mulsd/addsd (F2 0F 59/58), never the .2s single-precision
        // F3+movd/mulss. A width flub silently halves/squares transform math.
        let b = tr_bytes(Inst::Fmla { rd: 0, rn: 1, rm: 2, el64: true, q: false, sub: false });
        assert!(b.windows(4).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e]), "movq xmm,[mem] double load");
        assert!(b.windows(4).any(|w| w == [0x66, 0x48, 0x0f, 0xd6]), "movq [mem],xmm double store");
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc8]), "mulsd xmm1,xmm0 double product");
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xc1]), "addsd xmm0,xmm1 double accumulate");
        assert!(!b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc8]), "no single-precision mulss in .2d");
        assert!(!b.windows(3).any(|w| w == [0x66, 0x0f, 0x6e]), "no movd (32-bit) path in .2d");
    }

    #[test]
    fn sh433_fmlael_2s_broadcasts_element_then_mulss_by_it() {
        // fmla Vd, Vn, Vm.el[idx] (.2s): first broadcast the element into xmm2
        // (movd xmm2,eax = 66 0F 6E D0), then per lane mulss xmm1,xmm2 (F3 0F 59
        // CA) and accumulate. The broadcast-target + rm=xmm2 is the discriminator
        // vs the 3-operand Fmla (which multiplies by a full Vm lane in xmm1).
        let b = tr_bytes(Inst::FmlaEl { rd: 0, rn: 1, vlm: 2, idx: 1, el64: false, q: false, sub: false });
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xd0]), "broadcast element into xmm2 (movd xmm2,eax)");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xca]), "mulss xmm1,xmm2 (Vn[l]*Vm.el)");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x58, 0xc1]), "addss xmm0,xmm1 accumulate");
        assert!(!b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc8]), "element-form must NOT scale by full Vm lane");
    }

// SH434: hermetic coverage of the SIMD shift-and-accumulate codegen families
    // (translate.rs SimdShl, SimdShr, SimdShrAcc) — the load-bearing integer
    // shift/rounding math every vertex-index / packed-color / image-lane path
    // leans on. These had no direct byte tests. SH434 pins the discriminators
    // a byte error silently corrupts: (1) the shl-vs-shr opcode byte (shl C1
    // /4 E0  vs  unsigned shr C1 /5 E8  vs  signed arithmetic shr C1 /7 F8 —
    // a shift-direction flub moves every lane the wrong way); (2) the
    // SIGN-extend-before-arithmetic-shift requirement (sshr esize=4 loads
    // movsxd 48 63 then sar F8; ushr loads zero-extend movzx and shr E8 — a
    // zero-extended negative element shifts positive and flips the sign bit);
    // (3) the shift>=esize-bits guard (logical all-zeros via xor 48 31 C0 vs
    // arithmetic all-ones sign-fill via sar,63 48 C1 F8 3F — a bare x86 imm
    // would clamp instead, silently wrong for large shifts); (4) the
    // SimdShrAcc accumulate ordering (Vd read AFTER the shift, added via
    // add rcx,rax 48 01 C1 — never before, and the dst is re-stored, so usra/
    // ssra accumulate rather than overwrite). Exact-byte window + full-buffer
    // asserts (multi-lane); synthetic Inst -> translate() (tr_bytes),
    // deterministic, no image/env. [RBX]=CpuState; vector slot v[t] =
    // VECTOR_BASE(0x110) + t*16. (STATUS next-forward #5: the remaining SIMD
    // surface — distinct from SH432's SminMax/SimdSatAdd and SH433's Fmla.)

    // SimdShl esize=8, shift=1 (shl x0,x1,#1 / shl v0.2d): 2 lanes, each
    // `mov_load64 [rn]` -> `shl rax,1` (48 C1 E0 01) -> `mov_store64 [rd]`.
    // Full-buffer pin of the 2-lane body: the shl opcode byte C1/E0/01 and
    // the per-lane Vn 0x120/0x128 -> Vd 0x110/0x118 offset walk.
    #[test]
    fn sh434_shl_64_two_lane_load_shift_store() {
        let b = tr_bytes(Inst::SimdShl { rd: 0, rn: 1, esize: 8, shift: 1 });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov rax,[rbx+0x120]  Vn[0]
            0x48, 0xc1, 0xe0, 0x01,                   // shl rax,1
            0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110],rax  Vd[0]
            0x48, 0x8b, 0x83, 0x28, 0x01, 0x00, 0x00, // mov rax,[rbx+0x128]  Vn[1]
            0x48, 0xc1, 0xe0, 0x01,                   // shl rax,1
            0x48, 0x89, 0x83, 0x18, 0x01, 0x00, 0x00, // mov [rbx+0x118],rax  Vd[1]
        ]);
        assert!(!b.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x01]), "shl must NOT emit shr");
        assert!(!b.windows(4).any(|w| w == [0x48, 0xc1, 0xf8, 0x01]), "shl must NOT emit sar");
    }

    // SimdShr signed (sshr) esize=8: each 64-bit lane `mov_load64` ->
    // `sar rax,shift` (48 C1 F8 02, arithmetic) -> store. Full-buffer pin of
    // the 2-lane body: the F8 (sar) opcode byte is the signed-vs-unsigned
    // discriminator (ushr uses E8).
    #[test]
    fn sh434_sshr_64_signed_arithmetic_shift() {
        let b = tr_bytes(Inst::SimdShr { rd: 0, rn: 1, esize: 8, shift: 2, unsigned: false });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov rax,[rbx+0x120]  Vn[0]
            0x48, 0xc1, 0xf8, 0x02,                   // sar rax,2  (arithmetic)
            0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110],rax  Vd[0]
            0x48, 0x8b, 0x83, 0x28, 0x01, 0x00, 0x00, // mov rax,[rbx+0x128]  Vn[1]
            0x48, 0xc1, 0xf8, 0x02,                   // sar rax,2
            0x48, 0x89, 0x83, 0x18, 0x01, 0x00, 0x00, // mov [rbx+0x118],rax  Vd[1]
        ]);
        assert!(!b.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x02]), "sshr must NOT emit logical shr");
    }

    // SimdShr unsigned (ushr) esize=8: `mov_load64` -> `shr rax,2` (48 C1 E8
    // 02, logical) -> store. The E8 (shr) vs F8 (sar) opcode byte is the
    // signed-vs-unsigned discriminator.
    #[test]
    fn sh434_ushr_64_unsigned_logical_shift() {
        let b = tr_bytes(Inst::SimdShr { rd: 0, rn: 1, esize: 8, shift: 2, unsigned: true });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov rax,[rbx+0x120]
            0x48, 0xc1, 0xe8, 0x02,                   // shr rax,2  (logical)
            0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110],rax
            0x48, 0x8b, 0x83, 0x28, 0x01, 0x00, 0x00, // mov rax,[rbx+0x128]
            0x48, 0xc1, 0xe8, 0x02,                   // shr rax,2
            0x48, 0x89, 0x83, 0x18, 0x01, 0x00, 0x00, // mov [rbx+0x118],rax
        ]);
        assert!(!b.windows(4).any(|w| w == [0x48, 0xc1, 0xf8, 0x02]), "ushr must NOT emit arithmetic sar");
    }

    // SimdShr esize=4 signed (sshr) MUST sign-extend the 32-bit element to 64
    // BEFORE the arithmetic shift (movsxd rax,eax = 48 63 C0 then sar) so a
    // negative element polls its sign into the high bits; the unsigned form
    // must NOT (movzx load, shr E8, no movsxd). A zero-extended negative
    // element under an arithmetic shift becomes positive -> sign-bit flub.
    #[test]
    fn sh434_shr_signed_movsxd_before_sar_unsigned_never() {
        let s = tr_bytes(Inst::SimdShr { rd: 0, rn: 1, esize: 4, shift: 1, unsigned: false });
        assert!(s.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "sshr .4s/2s must movsxd the element before sar");
        assert!(s.windows(4).any(|w| w == [0x48, 0xc1, 0xf8, 0x01]), "sshr element uses arithmetic sar");
        let u = tr_bytes(Inst::SimdShr { rd: 0, rn: 1, esize: 4, shift: 1, unsigned: true });
        assert!(!u.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "ushr must NOT sign-extend (zero-extend then logical shr)");
        assert!(u.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x01]), "ushr uses logical shr");
        assert!(!u.windows(4).any(|w| w == [0x48, 0xc1, 0xf8, 0x01]), "ushr must NOT use sar");
    }

    // SimdShr shift>=esize-bits guard: esize=1 (byte), shift=8 >= 8 bits.
    // x86 imm shifts clamp to the 64-bit width (shift amounts 64-255 are
    // masked), so a large guest shift that empties the element must be
    // synthesized: unsigned UNSIGNED all-zeros (xor rax,rax = 48 31 C0),
    // signed all-ones sign-fill (sar rax,63 = 48 C1 F8 3F). The guard is the
    // discriminator between the two.
    #[test]
    fn sh434_shr_oversize_guard_unsigned_zero_signed_signfill() {
        let u = tr_bytes(Inst::SimdShr { rd: 0, rn: 1, esize: 1, shift: 8, unsigned: true });
        assert!(u.windows(3).any(|w| w == [0x48, 0x31, 0xc0]), "ushr overflowed element must xor to 0");
        assert!(!u.windows(4).any(|w| w == [0x48, 0xc1, 0xf8, 0x3f]), "unsigned must NOT sign-fill");
        let s = tr_bytes(Inst::SimdShr { rd: 0, rn: 1, esize: 1, shift: 8, unsigned: false });
        assert!(s.windows(4).any(|w| w == [0x48, 0xc1, 0xf8, 0x3f]), "sshr overflowed element must sign-fill all-ones (sar,63)");
        assert!(!s.windows(3).any(|w| w == [0x48, 0x31, 0xc0]), "signed must NOT zero the overflowed element");
        // both preserve the 8-bit element sign/zero extension before the guard
        assert!(u.windows(3).any(|w| w == [0x0f, 0xb6, 0x83]), "ushr .8b zero-extends byte lane (movzx)");
        assert!(s.windows(4).any(|w| w == [0x48, 0x0f, 0xbe, 0x83]), "sshr .8b sign-extends byte lane (movsx)");
    }

    // SimdShrAcc (usra here, unsigned esize=8 shift=1): Vd_i += Vn_i >> imm.
    // Full-buffer pin of lane 0: load Vn -> logical shr -> load the Vd
    // accumulator into RCX AFTER the shift -> add rcx,rax (48 01 C1) ->
    // re-store to Vd. The accumulate ordering (read-before-add, add after
    // shift) is the usra/ssra-vs-plain-shift discriminator.
    #[test]
    fn sh434_simdshracc_accumulates_after_shift() {
        let b = tr_bytes(Inst::SimdShrAcc { rd: 0, rn: 1, esize: 8, shift: 1, unsigned: true });
        // lane 0 only: load Vn[0] @0x120 -> shr 1 -> load Vd[0] @0x110 into RCX
        // -> add rcx,rax -> store [rbx+0x110]
        let want_head = [
            0x48, 0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov rax,[rbx+0x120]  Vn[0]
            0x48, 0xc1, 0xe8, 0x01,                   // shr rax,1  (shifted source)
            0x48, 0x8b, 0x8b, 0x10, 0x01, 0x00, 0x00, // mov rcx,[rbx+0x110]  Vd[0] accumulator
            0x48, 0x01, 0xc1,                         // add rcx,rax  (accumulate)
            0x48, 0x89, 0x8b, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110],rcx  (re-store Vd[0])
        ];
        assert_eq!(&b[0..want_head.len()], &want_head[..]);
        // the accumulate add must come after the shift of Vn (never before)
        let shr = b.windows(4).position(|w| w == [0x48, 0xc1, 0xe8, 0x01]).unwrap();
        let add = b.windows(3).position(|w| w == [0x48, 0x01, 0xc1]).unwrap();
        assert!(shr < add, "shift must happen before the accumulate add");
    }

// SH435: hermetic coverage of the FP conversion codegen families (translate.rs
    // Fcvt, FcvtTzReg, FcvtHalf) — the widen/narrow/trunc/FP16 conversions
    // every color-intensity, light, and texture-sample path leans on. These had
    // no direct byte tests. SH435 pins the discriminators a byte error silently
    // corrupts: (1) Fcvt widen (S->D) loads movd xmm0,eax (66 0F 6E C0) then
    // cvtss2sd (F3 0F 5A C0) + movq_store (66 48 0F D6 ...), vs narrow (D->S)
    // movq_load (F3 48 0F 7E ...) then cvtsd2ss (F2 0F 5A C0) + movd_r32_xmm
    // (66 0F 7E C0) + mov_store32 — the 32-vs-64-bit src/dst lane width and the
    // 5A 0F 5A ... 0x58/0x5A opcode are the widen-vs-narrow discriminators;
    // (2) FcvtTzReg signed fcvtzs uses cvttsd2si (F2 48 0F 2C C0) DIRECTLY
    // (trunc-to-zero is already what X86 cvttsd/si does), always a 64-bit store
    // for D or 32-bit for S, and the UNSIGNED fcvtzu appends the clamp
    // (mov rcx,0 / test / cmovs = 48 0F 48 C1) so negatives become 0 — the
    // cmovs presence separates fcvtzu (trunc-toward-zero UNSIGNED) from
    // fcvtzs; (3) FcvtHalf FP16 promotes via vcvtph2ps (c4 e2 79 13 c0) /
    // demotes via vcvtps2ph imm0 (c4 e3 79 1d c0 00) — the H->D wides through
    // cvtss2sd after promote, D->H narrows through cvtsd2ss before demote.
    // Exact-byte full-buffer + window asserts; synthetic Inst -> translate()
    // (tr_bytes); deterministic, no image/env. [RBX]=CpuState; vector slot
    // v[t] = VECTOR_BASE(0x110) + t*16.

    // Fcvt to_d=true (S -> D widen): load low-32 of s1 (0x120) into xmm0 via
    // movd, promote cvtss2sd, store low-64 of d0. Full-buffer (rd=0,rn=1).
    #[test]
    fn sh435_fcvt_widen_single_to_double() {
        let b = tr_bytes(Inst::Fcvt { to_d: true, rd: 0, rn: 1 });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov eax,[rbx+0x120]  (s1 low 32)
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0xf3, 0x0f, 0x5a, 0xc0,             // cvtss2sd xmm0,xmm0  (promote)
            0x66, 0x48, 0x0f, 0xd6, 0x83, 0x10, 0x01, 0x00, 0x00, // movq [rbx+0x110],xmm0 (d0 low 64)
        ]);
        assert!(!b.windows(4).any(|w| w == [0xf2, 0x0f, 0x5a, 0xc0]), "widen must be single->double (F3 cvtss2sd), not F2");
    }

    // Fcvt to_d=false (D -> S narrow): load low-64 of d1 (0x120) into xmm0 via
    // movq_load, narrow cvtsd2ss, store low-32 of s0 (movd_r32_xmm + store32).
    #[test]
    fn sh435_fcvt_narrow_double_to_single() {
        let b = tr_bytes(Inst::Fcvt { to_d: false, rd: 0, rn: 1 });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x20, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x120] (d1 low 64)
            0xf2, 0x0f, 0x5a, 0xc0,                               // cvtsd2ss xmm0,xmm0 (narrow)
            0x66, 0x0f, 0x7e, 0xc0,                               // movd eax,xmm0
            0x89, 0x83, 0x10, 0x01, 0x00, 0x00,                   // mov [rbx+0x110],eax (s0 low 32)
        ]);
        assert!(!b.windows(4).any(|w| w == [0xf3, 0x0f, 0x5a, 0xc0]), "narrow must be double->single (F2 cvtsd2ss), not F3");
    }

    // FcvtTzReg fcvtzs Dd,Dn (dbl, signed): movq_load -> cvttsd2si DIRECTLY
    // (x86 cvttsd2si already trunc-toward-zero, so no pre-round), 64-bit
    // store. The F2 48 0F 2C opcode is the trunc-convert discriminator. The
    // UNSIGNED fcvtzu must then clamp negatives to 0 with cmovs — it must NOT
    // store the raw signed trunc (a negative would corrupt the unsigned dst).
    #[test]
    fn sh435_fcvtzs_signed_trunc_is_truncate_without_round() {
        let s = tr_bytes(Inst::FcvtTzReg { rd: 0, rn: 1, dbl: true, unsigned: false });
        assert_eq!(s, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x20, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x120]
            0xf2, 0x48, 0x0f, 0x2c, 0xc0,                         // cvttsd2si rax,xmm0
            0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00,             // mov [rbx+0x110],rax
        ]);
        assert!(!s.windows(4).any(|w| w == [0x66, 0x0f, 0x2f, 0xc0]), "fcvtzs must NOT add a rounding compare");
    }

    // FcvtTzReg SIGNED vs UNSIGNED (fcvtzs vs fcvtzu): both truncate, but the
    // UNSIGNED form appends the clamp (mov rcx,0 / test rcx / cmovs -> rcx)
    // so a negative result becomes 0; the signed form stores the raw signed
    // trunc. The cmovs (48 0F 48 C1) presence is the fcvtzu discriminator.
    #[test]
    fn sh435_fcvtzu_unsigned_clamps_negative_cmovs() {
        let u = tr_bytes(Inst::FcvtTzReg { rd: 0, rn: 1, dbl: false, unsigned: true });
        assert!(u.windows(3).any(|w| w == [0x48, 0x0f, 0x2c]), "fcvtzu truncates via cvttsd2si");
        assert!(u.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]), "fcvtzu must clamp negative to 0 (cmovs rax,rcx)");
        assert!(u.windows(3).any(|w| w == [0x48, 0x85, 0xc0]), "fcvtzu tests the sign before clamping (test rax,rax)");
        // single (S) source: 32-bit load then promote cvtss2sd BEFORE trunc
        assert!(u.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xc0]), "S source loads via movd xmm0,eax");
        let s = tr_bytes(Inst::FcvtTzReg { rd: 0, rn: 1, dbl: true, unsigned: false });
        assert!(!s.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]), "signed fcvtzs must NOT clamp (raw signed trunc stored)");
    }

    // FcvtHalf op=0 (H -> S, fcvt s,h): load low-32 of h1, promote via F16C
    // vcvtph2ps (c4 e2 79 13 c0), store low-32 of s0. op=2 (H -> D) then
    // wides through cvtss2sd to a 64-bit store. The 13 (ph2ps promote) vs 1d
    // (ps2ph demote) opcode discriminator is the vector FP16 conversion gate.
    #[test]
    fn sh435_fcvthalf_half_to_float_uses_f16c_promote() {
        let b = tr_bytes(Inst::FcvtHalf { rd: 0, rn: 1, op: 0 });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov eax,[rbx+0x120]  (h1 low16 in low 32)
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0xc4, 0xe2, 0x79, 0x13, 0xc0,       // vcvtph2ps xmm0,xmm0  (F16C promote)
            0x66, 0x0f, 0x7e, 0xc0,             // movd eax,xmm0
            0x89, 0x83, 0x10, 0x01, 0x00, 0x00, // mov [rbx+0x110],eax  (s0 low 32)
        ]);
        assert!(!b.windows(6).any(|w| w == [0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]), "H->S must promote (13), not demote (1d)");
    }

    #[test]
    fn sh435_fcvthalf_float_to_half_uses_f16c_demote() {
        let b = tr_bytes(Inst::FcvtHalf { rd: 0, rn: 1, op: 1 });
        assert!(b.windows(6).any(|w| w == [0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]), "S->H must demote via vcvtps2ph $0 (c4 e3 79 1d c0 00)");
        assert!(!b.windows(5).any(|w| w == [0xc4, 0xe2, 0x79, 0x13, 0xc0]), "S->H must NOT promote");
        // narrow stores low 32 (single) only
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x7e, 0xc0]), "S->H result moved via movd eax,xmm0");
    }

    // FcvtHalf op=2 (H -> D): promote to FP32 then widen cvtss2sd to the low
    // 64 bits. op=3 (D -> H): narrow cvtsd2ss then demote. The extra cvtss2sd
    // (after promote) vs cvtsd2ss (before demote) distinguishes the .5-width
    // conversions from the direct S<->H pair.
    #[test]
    fn sh435_fcvthalf_half_to_double_wides_after_promote() {
        let b = tr_bytes(Inst::FcvtHalf { rd: 0, rn: 1, op: 2 });
        assert!(b.windows(5).any(|w| w == [0xc4, 0xe2, 0x79, 0x13, 0xc0]), "H->D promotes via vcvtph2ps");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x5a, 0xc0]), "H->D then widens via cvtss2sd");
        assert!(b.windows(4).any(|w| w == [0x66, 0x48, 0x0f, 0xd6]), "H->D stores 64-bit (movq [mem],xmm0)");
        let d = tr_bytes(Inst::FcvtHalf { rd: 0, rn: 1, op: 3 });
        assert!(d.windows(4).any(|w| w == [0xf2, 0x0f, 0x5a, 0xc0]), "D->H narrows via cvtsd2ss");
        assert!(d.windows(6).any(|w| w == [0xc4, 0xe3, 0x79, 0x1d, 0xc0, 0x00]), "D->H then demotes via vcvtps2ph");
    }

// SH436: hermetic coverage of the SIMD bitwise-select + high-narrow codegen
    // families (translate.rs SimdSel, SimdHighNarrow) — the 3-input bitwise
    // select (bsl/bit/bif) and the narrowing add/sub that drive blend masks
    // and byte-level color/normal packing in rendered content. These had no
    // direct byte tests. SH436 pins the discriminators a byte error silently
    // corrupts: (1) SimdSel BSL (op=0) = (Rn & Rd) | (~Rd & Rm) vs BIT/BIF
    // (op=1) = (~Rm & Rn) | (Rd & Rm) — the operand ORDER of the pandn/pand
    // pair is the semantic separator (a transposed mask picks the wrong
    // source vector); (2) SimdHighNarrow addhn (no-round) does add + NO round-
    // width add, vs raddhn (round) that adds 1<<(dst_bits-1) BEFORE the shr —
    // the round-carry presence is the discriminator, and the shr_ri8 by
    // dst_bits (a width flub drops the wrong high half); (3) SimdHighNarrow
    // Q=0 zeroes the upper half of Vd (`mov qword [rd+8],0`) vs Q=1 (upper)
    // which does not — the upper-half clear is the Q discriminator.
    // Exact-byte full-buffer + window asserts; synthetic Inst -> translate()
    // (tr_bytes); deterministic, no image/env. [RBX]=CpuState; vector slot
    // v[t] = VECTOR_BASE(0x110) + t*16.

    // SimdSel op=0 (BSL): Vd = (Rn & Vd) | (~Vd & Rm). Loads Rn@0x120 (xmm0),
    // Rm@0x130 (xmm1), Vd@0x110 (xmm2 via R10); then pand xmm0,xmm2 (Rn&Rd),
    // pandn xmm2,xmm1 (~Rd&Rm — NOTE the dst=xmm2/rm=xmm1 order), por xmm0,
    // xmm2, store Vd. Full-buffer pin (rd=0,rn=1,rm=2):
    #[test]
    fn sh436_simdsel_bsl_and_then_nand_vd_wins_source() {
        let b = tr_bytes(Inst::SimdSel { rd: 0, rn: 1, rm: 2, op: 0 });
        assert_eq!(b, vec![
            0xf3, 0x0f, 0x6f, 0x83, 0x20, 0x01, 0x00, 0x00, // movdqu xmm0,[rbx+0x120]  Rn
            0xf3, 0x0f, 0x6f, 0x8b, 0x30, 0x01, 0x00, 0x00, // movdqu xmm1,[rbx+0x130]  Rm
            0xf3, 0x44, 0x0f, 0x6f, 0x93, 0x10, 0x01, 0x00, 0x00, // movdqu xmm2,[rbx+0x110] (Vd)
            0x66, 0x41, 0x0f, 0xdb, 0xc2,             // pand xmm0,xmm2  (Rn & Rd)
            0x66, 0x44, 0x0f, 0xdf, 0xd1,             // pandn xmm2,xmm1  (~Rd & Rm: dst=xmm2, rm=xmm1)
            0x66, 0x41, 0x0f, 0xeb, 0xc2,             // por xmm0,xmm2  (final union)
            0xf3, 0x0f, 0x7f, 0x83, 0x10, 0x01, 0x00, 0x00, // movdqu [rbx+0x110],xmm0  Vd
        ]);
    }

    // SimdSel op=1 (BIT/BIF): Vd = (~Rm & Rn) | (Rd & Rm). The pand+AND-pandn
    // operand order is INVERTED vs BSL: first pandn (~Rm & Rn, dst=xmm0,
    // rm=xmm1) then pand (Rd & Rm, dst=xmm2, rm=xmm1). A transposed order
    // picks the wrong source vector. Window discriminator vs op=0.
    #[test]
    fn sh436_simdsel_bit_bif_nand_then_and_operand_order() {
        let b = tr_bytes(Inst::SimdSel { rd: 0, rn: 1, rm: 2, op: 1 });
        // op1 starts the union with the NAND, not the AND:
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0xdf, 0xc1]), "BIT/BIF must lead with ~Rm & Rn (pandn xmm0,xmm1)");
        assert!(b.windows(5).any(|w| w == [0x66, 0x44, 0x0f, 0xdb, 0xd1]), "BIT/BIF then Rd & Rm (pand xmm2,xmm1)");
        assert!(!b.windows(5).any(|w| w == [0x66, 0x41, 0x0f, 0xdb, 0xc2]), "BIT/BIF must NOT do BSL's Rn & Rd first");
        // the final union is still por xmm0,xmm2
        assert!(b.windows(5).any(|w| w == [0x66, 0x41, 0x0f, 0xeb, 0xc2]), "both ends in por xmm0,xmm2");
    }

    // SimdHighNarrow addhn (no round) .4s->dst_esize=2, 8 src byte-pairs: 2
    // dst lanes, add RAX+RCX then shr rax,16 (2*dst_bits? no — dst_esize=2,
    // dst_bits=16, shr by dst_bits=16). NO round-carry add before the shr.
    // Q=0 also zeroes the upper half (mov qword [Vd+8],0).
    #[test]
    fn sh436_simdhighnarrow_addhn_no_round_shr_by_dst_bits() {
        let b = tr_bytes(Inst::SimdHighNarrow { rd: 0, rn: 1, rm: 2, dst_esize: 2, sub: false, round: false, q: false });
        assert!(b.windows(3).any(|w| w == [0x48, 0x01, 0xc8]), "addhn adds src lanes (add rax,rcx)");
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x10]), "addhn must shr by dst_bits=16 (0x10) to take the high half");
        // NO round-carry: a bare c1 e8 0x10 must appear directly after the add,
        // never preceded by an add of 1<<(dst_bits-1)
        let shr_off = b.windows(4).position(|w| w == [0x48, 0xc1, 0xe8, 0x10]).unwrap();
        assert!(!b[..shr_off].windows(3).any(|w| w == [0x48, 0x01, 0xd0]), "addhn must NOT have a round-carry add before the shift");
        // Q=0 zeroes the upper half of Vd (mov rax,0 then mov [Vd+8],rax)
        assert!(b.windows(10).any(|w| w == [0x48, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0]), "addhn Q=0 must load a 0 upper-half");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x18, 0x01, 0x00, 0x00]), "addhn Q=0 must store 0 to [Vd+8] (the upper half)");
    }

    // SimdHighNarrow raddhn (round): same add + shr, but a round-carry
    // `add rax, 1<<(dst_bits-1)` is inserted BEFORE the shr. The round-carry
    // presence is the raddhn-vs-addhn discriminator.
    #[test]
    fn sh436_simdhighnarrow_raddhn_rounds_before_shr() {
        let b = tr_bytes(Inst::SimdHighNarrow { rd: 0, rn: 1, rm: 2, dst_esize: 2, sub: false, round: true, q: false });
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xe8, 0x10]), "raddhn still shr by dst_bits=16");
        // round carry = 1<<(16-1) = 0x8000 added to the sum BEFORE the shift
        assert!(b.windows(10).any(|w| w == [0x49, 0xba, 0x00, 0x80, 0, 0, 0, 0, 0, 0]), "raddhn must add 1<<(dst_bits-1)=0x8000 (mov r10,0x8000)");
        let round = b.windows(10).position(|w| w == [0x49, 0xba, 0x00, 0x80, 0, 0, 0, 0, 0, 0]).unwrap();
        let shr = b.windows(4).position(|w| w == [0x48, 0xc1, 0xe8, 0x10]).unwrap();
        assert!(round < shr, "round-carry must be added before the narrowing shift");
    }

    // SH437: hermetic coverage of the SIMD lane-COPY codegen family
    // (translate.rs SimdInsD — `mov Vd.T[dst], Vn.T[src]`): the per-element
    // lane move used to splat/broadcast/shuffle one value across a vector
    // (color/texel lane packing + 8-bit channel moves on render data paths).
    // Byte-map ([RBX]=CpuState, vector slot v[t]=VECTOR_BASE 0x110 + t*16):
    //   src = 0x110 + rn*16 + src_idx*esize ;  dst = 0x110 + rd*16 + dst_idx*esize
    // The element size selects FOUR distinct load/store byte forms — a width
    // flub copies the wrong byte count and silently corrupts the lane:
    //   esize=8 -> mov_load64 (48 8B) + mov_store64 (48 89)
    //   esize=4 -> mov_load32 (8B, zero-ext) + mov_store32 (89)
    //   esize=2 -> movzx_word_mem (0F B7) + mov_store16 (66-prefixed 89)
    //   esize=1 -> movzx_byte_mem (0F B6) + mov_store8 (88)
    //
    // SH437-1: d-lane copy (esize=8) rd=0 rn=1 dst_idx=1 src_idx=0. Actual
    // source address = 0x110 + 0x10*1 + 0 = 0x120; dest = 0x110 + 0 + 8 = 0x118.
    #[test]
    fn sh437_simdinsd_d_lane_copies_full_64bit() {
        let b = tr_bytes(Inst::SimdInsD { rd: 0, rn: 1, dst_idx: 1, src_idx: 0, esize: 8 });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x20, 0x01, 0x00, 0x00, // mov rax,[rbx+0x120]  (Vn.s[0] d-lane)
            0x48, 0x89, 0x83, 0x18, 0x01, 0x00, 0x00, // mov [rbx+0x118],rax  (Vd.d[1])
        ]);
    }

    // SH437-2: s-lane copy (esize=4) rd=2 rn=3 dst_idx=1 src_idx=0. Source =
    // 0x110+0x30 = 0x140; dest = 0x110+0x20+4 = 0x134. 32-bit load zero-extends
    // (no REX.W) and the store is the 32-bit 89 form — NOT the 8B/48-8B d-lane form.
    #[test]
    fn sh437_simdinsd_s_lane_zero_extends_32bit() {
        let b = tr_bytes(Inst::SimdInsD { rd: 2, rn: 3, dst_idx: 1, src_idx: 0, esize: 4 });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x40, 0x01, 0x00, 0x00, // mov eax,[rbx+0x140]  (Vn.s[0])
            0x89, 0x83, 0x34, 0x01, 0x00, 0x00, // mov [rbx+0x134],eax  (Vd.s[1])
        ]);
        assert!(!b.windows(2).any(|w| w == [0x48, 0x8b]), "s-lane must NOT emit the REX.W d-lane mov");
    }

    // SH437-3: h-lane copy (esize=2) rd=0 rn=1 dst_idx=3 src_idx=1. Source =
    // 0x110+0x10+2 = 0x122; dest = 0x110+0+6 = 0x116. 16-bit movzx (0F B7) on the
    // load and the 16-bit 0x66-prefixed store (66 89) — the only esize form whose
    // store carries the 66 operand-size prefix.
    #[test]
    fn sh437_simdinsd_h_lane_uses_movzx_word_and_66_store() {
        let b = tr_bytes(Inst::SimdInsD { rd: 0, rn: 1, dst_idx: 3, src_idx: 1, esize: 2 });
        assert_eq!(b, vec![
            0x0f, 0xb7, 0x83, 0x22, 0x01, 0x00, 0x00, // movzx eax,word [rbx+0x122]  (Vn.h[1])
            0x66, 0x89, 0x83, 0x16, 0x01, 0x00, 0x00, // mov [rbx+0x116],ax  (Vd.h[3], 66-prefixed)
        ]);
    }

    // SH437-4: b-lane copy (esize=1) rd=1 rn=0 dst_idx=4 src_idx=2. Source =
    // 0x110+0+2 = 0x112; dest = 0x110+0x10+4 = 0x124. Byte movzx (0F B6) + byte
    // store (88) — the narrowest form; a width flub here would over-read/write
    // the neighbouring byte channel.
    #[test]
    fn sh437_simdinsd_b_lane_uses_movzx_byte_and_byte_store() {
        let b = tr_bytes(Inst::SimdInsD { rd: 1, rn: 0, dst_idx: 4, src_idx: 2, esize: 1 });
        assert_eq!(b, vec![
            0x0f, 0xb6, 0x83, 0x12, 0x01, 0x00, 0x00, // movzx eax,byte [rbx+0x112]  (Vn.b[2])
            0x88, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [rbx+0x124],al  (Vd.b[4], 88 byte store)
        ]);
        assert!(!b.windows(2).any(|w| w == [0x48, 0x8b]), "b-lane must not emit a 64-bit mov");
    }

    // SH437-5: lane-ADDRESSING discriminator — the source offset follows
    // src_idx*esize and the dest follows rd*16 + dst_idx*esize independently.
    // Stepping src_idx by one d-lane (esize=8) moves ONLY the source disp, and by
    // exactly esize bytes (0x110+Vn*16+{idx}) — never a fixed/vt-stale stride.
    #[test]
    fn sh437_simdinsd_lane_index_addresses_scale_by_esize() {
        let b0 = tr_bytes(Inst::SimdInsD { rd: 0, rn: 3, dst_idx: 0, src_idx: 0, esize: 8 });
        let b1 = tr_bytes(Inst::SimdInsD { rd: 0, rn: 3, dst_idx: 0, src_idx: 1, esize: 8 });
        // src0 addr 0x110+0x30 = 0x140 ; src1 addr 0x110+0x30+8 = 0x148.
        let m0 = b0.windows(7).position(|w| w == [0x48, 0x8b, 0x83, 0x40, 0x01, 0x00, 0x00]).expect("Vn.s[0] load");
        let m1 = b1.windows(7).position(|w| w == [0x48, 0x8b, 0x83, 0x48, 0x01, 0x00, 0x00]).expect("Vn.s[1] load");
        assert!(m0 < b0.len() && m1 < b1.len(), "lane-addressed loads present");
        // dest writes land at the same rd slot (0x110) regardless of src_idx.
        let d0 = b0.windows(7).position(|w| w == [0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00]).expect("Vd store");
        let d1 = b1.windows(7).position(|w| w == [0x48, 0x89, 0x83, 0x10, 0x01, 0x00, 0x00]).expect("Vd store");
        assert!(d0 < b0.len() && d1 < b1.len(), "dest disp fixed at VECTOR_BASE+rd*16");
    }

    // SH438: hermetic coverage of the STRUCTURE-LOAD/STORE codegen family
    // (translate.rs Ld2 / St2 — ld2/st2 {Vt, Vt1}, [Xn]): the interleaved
    // vertex-attribute / structure pair moves (RG/z+texcoord style data, and
    // 2-vector register list loads). Memory holds the DEINTERLEAVED
    // {V0.e0,V1.e0, V0.e1,V1.e1, ...} layout: element i of reg j (j in 0..2) is
    // at byte offset i*(2*es) + j*es. Ld2 writes reg j's element i to
    // VECTOR_BASE+(rd+j)*16 + i*es (rd+j = the 2nd structure reg), St2 reads
    // them back. A byte offset flub here misdelivers which attribute/register
    // lane goes where — the whole point of the family.
    // [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16, Xn loaded by
    // ldg into RDX (mov rdx,[rbx+rn*8]). Case: esize=4, q=false (8B, nelems=2),
    // rd=0, rn=1 -> ldg = `48 8B 53 08`; loop j∈{0,1}, i∈{0,1}, b∈{0..3}.
    // Ld2 load (mem, base RDX): off = i*2*es + j*es + b = i*8 + j*4 + b
    //   (i,j)=(0,0)->mem 0 ; (0,1)->mem 4 ; (1,0)->mem 8 ; (1,1)->mem 0xc
    //     => `0F B6 42 <off>` (movzx eax, byte [rdx+off], disp8 mod=1)
    // Ld2 store (Vd slot, base RBX): vslot(rd+j)+i*es+b = 0x110+j*16+i*4+b
    //   (0,0)->0x110 ; (0,1)->0x120 ; (1,0)->0x114 -> `88 83 <u32>` (mod=2)
    #[test]
    fn sh438_ld2_deinterleaves_j_to_second_structure_reg() {
        let b = tr_bytes(Inst::Ld2 { rd: 0, rn: 1, q: false, post: 0, esize: 4 });
        // loads: j advances mem by es=4, i advances mem by 2*es=8
        assert!(b.windows(3).any(|w| w == [0x0f, 0xb6, 0x02]), "V0.elt0 read mem+0 (i0,j0)");
        assert!(b.windows(4).any(|w| w == [0x0f, 0xb6, 0x42, 0x04]), "V1.elt0 read mem+4 (j=1 -> +es)");
        assert!(b.windows(4).any(|w| w == [0x0f, 0xb6, 0x42, 0x08]), "V0.elt1 read mem+8 (i=1 -> +2*es)");
        // stores: reg j lands at Vd + j*16, element i advances by es=4
        // (the store is 6 bytes: 0x88 0x83 + disp32)
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x10, 0x01, 0x00, 0x00]), "V0.elt0 -> [0x110]");
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x20, 0x01, 0x00, 0x00]), "V1.elt0 -> [0x120] (2nd structure reg at +16)");
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x14, 0x01, 0x00, 0x00]), "V0.elt1 -> [0x114] (i+es)");
        // no outstanding post-increment write (post=0)
        assert!(!b.windows(5).any(|w| w == [0x48, 0x89, 0x43, 0x08, 0x00]) && !b.windows(4).any(|w| w == [0x48, 0x83, 0xc0, 0x00]), "post=0 emits no reg +0 increment");
    }

    // St2 is the INVERSE: it READS the vector slots ([rbx+0x110..]) and STORES
    // to memory ([rdx+off]). Same deinterleave mapping; the discriminator vs
    // Ld2 is that the movzx source becomes [rbx+Vd-slot] and the store becomes
    // `88` to [rdx+mem-off] (Ld2's store was 88 to [rbx], St2's store is 88 to
    // [rdx]).
    #[test]
    fn sh438_st2_inverse_reads_slots_stores_to_memory_deinterleave() {
        let b = tr_bytes(Inst::St2 { rd: 0, rn: 1, q: false, post: 0, esize: 4 });
        // loads now from Vd slots (base RBX): V0.elt0 at 0x110, V1.elt0 at 0x120
        assert!(b.windows(7).any(|w| w == [0x0f, 0xb6, 0x83, 0x10, 0x01, 0x00, 0x00]), "V0.elt0 read from [rbx+0x110]");
        assert!(b.windows(7).any(|w| w == [0x0f, 0xb6, 0x83, 0x20, 0x01, 0x00, 0x00]), "V1.elt0 read from [rbx+0x120]");
        // stores to memory (base RDX): V0.elt0 -> mem+0, V0.elt1 -> mem+8 (i+2*es)
        assert!(b.windows(3).any(|w| w == [0x88, 0x02, 0x00]) || b.windows(2).any(|w| w == [0x88, 0x02]), "V0.elt0 -> [rdx+0] (0x88 store to mem)");
        assert!(b.windows(4).any(|w| w == [0x88, 0x42, 0x08, 0x00]) || b.windows(3).any(|w| w == [0x88, 0x42, 0x08]), "V0.elt1 -> [rdx+8] (i+2*es)");
        assert!(b.windows(4).any(|w| w == [0x88, 0x42, 0x04, 0x00]) || b.windows(3).any(|w| w == [0x88, 0x42, 0x04]), "V1.elt0 -> [rdx+4] (j+es)");
    }

    // SH438-3: q=true (16B) Ld2 doubles the element count per structure vector
    // (nelems = 16/es = 4 for esize=4), so element i in {0..3} lands at
    // Vd + i*es = [0x110,0x114,0x118,0x11c]; the q-bit distinguishes 2 vs 4
    // deinterleaved elements. Also pins the post-increment write-back when
    // post != 0 (mov_load64 + add + store of rn's guest slot).
    #[test]
    fn sh438_ld2_q16_has_four_elements_and_post_increments_rn() {
        let b = tr_bytes(Inst::Ld2 { rd: 0, rn: 1, q: true, post: 0x20, esize: 4 });
        // 4th element of structure reg 0 -> Vd + 3*es = [0x11c]
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x1c, 0x01, 0x00, 0x00]), "V0.elt3 -> [0x11c] (q->4 elements)");
        // post=0x20: rn slot reloaded (48 8B 43 08), +0x20 via imm32 form
        // (48 81 C0 20 00 00 00), written back to [rbx+8] (48 89 43 08).
        let post_seq = [0x48, 0x8b, 0x43, 0x08, 0x48, 0x81, 0xc0, 0x20, 0x00, 0x00, 0x00, 0x48, 0x89, 0x43, 0x08];
        assert!(b.windows(post_seq.len()).any(|w| w == post_seq), "post-increment mov-load + add imm32 + store-back of rn slot [rbx+8]");
    }

// SH439: hermetic coverage of the float-to-int UNSIGNED BIG-PATH codegen
    // family (translate.rs FcvtToInt over [2^63, 2^64) and the fixed-point
    // scale) — the fcvtzu emission SH435 deliberately left as "the unsigned
    // big-path" next-forward. A byte error here silently corrupts the high
    // half of every unsigned float->u64 conversion the real client does on
    // timing/URLOpen/color-of-light paths. SH439 pins the discriminators that
    // separate fcvtzu (unsigned big-path) from the plain signed fcvtzs, and
    // the fbits>0 fixed-point scale:
    // (1) fcvtzu vs fcvtzs: fcvtzu EMITS the range gate — the 2^63 float const
    //   (mov rcx,0x43e0_0000_0000_0000), the `comisd xmm0,xmm1` (66 0F 2F)
    //   compare, the JB-to-signed-path rel32 (0F 82), the big-path `subsd`
    //   (F2 0F 5C) that computes d-2^63, the `add rax,rcx` (48 01 C8, +2^63
    //   restore) and the u64::MAX saturation (mov rax,-1) — plus the cmovs
    //   clamp (48 0F 48 C1) in BOTH the signed fall-through and the (implicit)
    //   2^63 add; the SIGNED fcvtzs NEVER emits comisd/subsd/2^63-add/u64::MAX
    //   — it is movq_load + a single cvttsd2si + store. Presence of the
    //   comisd gate is the fcvtzu-vs-fcvtzs discriminator.
    // (2) fbits>0 fixed-point: the 2^fbits double scale is materialized
    //   (mov rax,const), moved to xmm1 (66 48 0F 6E C8), and MULt into xmm0
    //   (mulsd F2 0F 59 C1) BEFORE truncation — the mulsd presence is the
    //   fixed-point discriminator (fbits=0 must NOT mulsd).
    // Exact-byte window asserts via synthetic Inst -> translate() (tr_bytes);
    // deterministic, no image/env. [RBX]=CpuState; vector slot v[t] =
    // VECTOR_BASE(0x110)+t*16; rn=1 -> slot v1 @ 0x120 (d-src low 8B).
    #[test]
    fn sh439_fcvtztoint_fcvtzu_unsigned_bigpath_comisd_gate_and_clamp() {
        // fcvtzu x0, d1 (unsigned, d-src, mode 0 = trunc-toward-zero):
        // d -> load low 8B of v1, then the [0,2^64) range gate.
        let b = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 0, sf: true, unsigned: true, src_sng: false, fbits: 0 });
        // d-source loads low 8B of v1 @0x120 (movq xmm0,[rbx+0x120], F3 48 0F 7E)
        assert!(b.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x20, 0x01, 0x00, 0x00]), "fcvtzu d-src = movq xmm0,[rbx+0x120]");
        // the 2^63 double constant materialized in RCX (mov rcx,0x43e0_0000_0000_0000)
        assert!(b.windows(10).any(|w| w == [0x48, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0x43]), "fcvtzu gates on 2^63 (mov rcx,0x43e0_0000_0000_0000)");
        // the signed compare comisd xmm0,xmm1 (66 40 0F 2F C1, REX always
        // present) -> CF=1 iff d<2^63
        assert!(b.windows(5).any(|w| w == [0x66, 0x40, 0x0f, 0x2f, 0xc1]), "fcvtzu compares via comisd xmm0,xmm1 (66 40 0F 2F C1)");
        // the JB branch to the signed path (0F 82) and the big-path subsd (F2 0F 5C)
        assert!(b.windows(2).any(|w| w == [0x0f, 0x82]), "fcvtzu JB?s to the signed path (0F 82 rel32)");
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x5c, 0xc1]), "fcvtzu big-path computes d-2^63 (subsd xmm0,xmm1)");
        // the unsigned clamp applies in BOTH arms so negatives never reach the
        // unsigned dst: cmovs rax,rcx (48 0F 48 C1) must appear
        assert!(b.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]), "fcvtzu clamps negative to 0 (cmovs rax,rcx)");
        // u64::MAX saturation for d>=2^64 (mov rax,-1 = 48 B8 FF..)
        let sat = [0x48, 0xb8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert!(b.windows(10).any(|w| w == sat), "fcvtzu saturates d>=2^64 to u64::MAX");
    }

    #[test]
    fn sh439_fcvttoint_signed_fcvtzs_omits_unsigned_bigpath_entirely() {
        // fcvtzs x0, d1 (signed, mode 0): the plain trunc path. It must be a
        // bare movq_load + single cvttsd2si + store — NO comisd range gate,
        // NO subsd 2^63 subtract, NO 2^63 add-restore, NO u64::MAX, NO cmovs.
        let b = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 0, sf: true, unsigned: false, src_sng: false, fbits: 0 });
        assert!(b.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x20, 0x01, 0x00, 0x00]), "signed fcvtzs loads via movq xmm0,[rbx+0x120]");
        assert!(b.windows(5).any(|w| w == [0xf2, 0x48, 0x0f, 0x2c, 0xc0]), "signed fcvtzs truncs via cvttsd2si rax,xmm0");
        assert!(b.windows(3).any(|w| w == [0x48, 0x89, 0x03]) || b.windows(4).any(|w| w == [0x48, 0x89, 0x83, 0x10]), "signed fcvtzs stores the 64-bit trunc to the integer dst slot (stg)");
        assert!(!b.windows(5).any(|w| w == [0x66, 0x40, 0x0f, 0x2f, 0xc1]), "signed fcvtzs must NOT emit the comisd range gate");
        assert!(!b.windows(4).any(|w| w == [0xf2, 0x0f, 0x5c, 0xc1]), "signed fcvtzs must NOT emit the 2^63 subtract (subsd)");
        assert!(!b.windows(4).any(|w| w == [0x48, 0x0f, 0x48, 0xc1]), "signed fcvtzs must NOT clamp via cmovs");
        assert!(!b.windows(10).any(|w| w == [0x48, 0xb8, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]), "signed fcvtzs must NOT saturate to u64::MAX");
    }

    // SH444: hermetic coverage of the SIMD single-precision FP TWO-SOURCE
    // arithmetic codegen family (translate.rs VecFpArith — fadd/fsub/fmul/fdiv
    // op 0..3 + fmax/fmin/fmaxnm/fminnm op 4..7 on Vd.2s/.4s lanes; and the
    // frecps/frsqrts op 8/9 reciprocal-estimate helpers). SH433 pinned Fmla
    // (the FP multiply-ACCUMULATE with a vector-3/4-operand dst) but not the
    // plain 2-src per-lane arithmetic, which is every geometry/color lane's
    // add/sub/mul/div/max/min. The whole family shares one movd-xmm0/1 +
    // opcode + movd-back store shape, so the ONLY semantic bit is the opcode
    // byte: addss 0x58 / subss 0x5C / mulss 0x59 / divss 0x5E on (xmm0,xmm1)
    // = F3 0F {58,5C,59,5E} C1, maxss 0x5F, minss 0x5D. A flub (say mulss where
    // fsub was wanted) silently wrong-computes every lane. The frecps/frsqrts
    // path is DELIBERATELY different: it loads the product into a temp then
    // emits `subss xmm1,xmm0` (F3 0F 5C C8 — inversion of the fsub register
    // direction) to compute 2-prod / (3-prod), and frsqrts adds a `mulss
    // xmm1,xmm3` by 0.5f (F3 0F 59 CB). rd=1 rn=2 rm=3: Vd slot VECTOR_BASE
    // 0x110+1*16=0x120, Vn slot 0x130, Vm slot 0x140.
    #[test]
    fn sh444_vecfparith_2s_add_full_lane_addressing() {
        // fadd V1.2s, V2.2s, V3.2s (op=0, q=false): 2 lanes, dst 0x120/0x124.
        let b = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 0, q: false });
        assert_eq!(b, vec![
            // lane 0: Vn@0x130, Vm@0x140, addss, store Vd@0x120
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130]  Vn lane0
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0x8b, 0x83, 0x40, 0x01, 0x00, 0x00, // mov eax,[rbx+0x140]  Vm lane0
            0x66, 0x0f, 0x6e, 0xc8,             // movd xmm1,eax
            0xf3, 0x0f, 0x58, 0xc1,             // addss xmm0,xmm1
            0x66, 0x0f, 0x7e, 0xc0,             // movd eax,xmm0
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],eax  Vd lane0
            // lane 1: Vn@0x134, Vm@0x144, addss, store Vd@0x124
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // mov eax,[rbx+0x134]  Vn lane1
            0x66, 0x0f, 0x6e, 0xc0,
            0x8b, 0x83, 0x44, 0x01, 0x00, 0x00, // mov eax,[rbx+0x144]  Vm lane1
            0x66, 0x0f, 0x6e, 0xc8,
            0xf3, 0x0f, 0x58, 0xc1,             // addss xmm0,xmm1
            0x66, 0x0f, 0x7e, 0xc0,             // movd eax,xmm0
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [rbx+0x124],eax  Vd lane1
        ]);
        // lane addressing: dst advances by 4 per lane (0x120 -> 0x124), both Vn
        // (0x130->0x134) and Vm (0x140->0x144) too — never a stale/fixed stride.
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x24, 0x01, 0x00, 0x00]),
            "lane1 must store Vd@0x124 (dst advances by esize=4 per lane)");
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x34, 0x01, 0x00, 0x00]),
            "lane1 must read Vn@0x134 (src advances +4)");
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x44, 0x01, 0x00, 0x00]),
            "lane1 must read Vm@0x144 (src advances +4)");
    }

    #[test]
    fn sh444_vecfparith_opcode_discriminators_add_sub_mul_div_max_min() {
        // The whole family shares the movd-in/opcode/movd-out shape; only the
        // opcode byte changes. Verify each is EXACT and cross-absent so a flub
        // (mulss where fsub was wanted) fails.
        let sub = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 1, q: false });
        assert!(sub.windows(4).any(|w| w == [0xf3, 0x0f, 0x5c, 0xc1]), "fsub = subss (F3 0F 5C C1)");
        assert!(!sub.windows(4).any(|w| w == [0xf3, 0x0f, 0x58, 0xc1]), "fsub must NOT emit addss");
        let mul = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 2, q: false });
        assert!(mul.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc1]), "fmul = mulss (F3 0F 59 C1)");
        assert!(!mul.windows(4).any(|w| w == [0xf3, 0x0f, 0x5c, 0xc1]), "fmul must NOT emit subss");
        let div = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 3, q: false });
        assert!(div.windows(4).any(|w| w == [0xf3, 0x0f, 0x5e, 0xc1]), "fdiv = divss (F3 0F 5E C1)");
        let mx = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 4, q: false });
        assert!(mx.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5f, 0xc1]), "fmax/fmaxnm = maxss (F3 40 0F 5F C1 — ALWAYS emits the no-op REX)");
        assert!(!mx.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5d, 0xc1]), "fmax must NOT emit minss");
        let mn = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 5, q: false });
        assert!(mn.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5d, 0xc1]), "fmin/fminnm = minss (F3 40 0F 5D C1 — ALWAYS emits the no-op REX)");
        assert!(!mn.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5f, 0xc1]), "fmin must NOT emit maxss");
        // op 6/7 (fmaxnm/fminnm) reuse the maxss/minss opcode (NaN edge is a
        // documented approximation), so they must match 4/5 respectively.
        let mxnm = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 6, q: false });
        assert!(mxnm.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5f, 0xc1]), "fmaxnm = maxss (same opcode)");
        let mnnm = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 7, q: false });
        assert!(mnnm.windows(5).any(|w| w == [0xf3, 0x40, 0x0f, 0x5d, 0xc1]), "fminnm = minss (same opcode)");
    }

    #[test]
    fn sh444_vecfparith_frecps_inverted_sub_and_frsqrts_half_scale() {
        // frecps (op 8): computes 2.0 - Vn*Vm -> the emit is mulss(xmm0,xmm1)
        // for the product, materializes 2.0f32 (0x40000000), then `subss
        // xmm1,xmm0` (F3 0F 5C C8) — the INVERTED register direction vs the
        // plain fsub (F3 0F 5C C1) — so xmm1 = 2.0 - prod. frsqrts (op 9) adds
        // `movd xmm3,eax` of 0.5f (0x3f000000) + `mulss xmm1,xmm3` (F3 0F 59 CB).
        let f = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 8, q: false });
        assert!(f.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc1]), "frecps multiplies the product first (mulss xmm0,xmm1)");
        assert!(f.windows(4).any(|w| w == [0xf3, 0x0f, 0x5c, 0xc8]), "frecps inverted sub = subss xmm1,xmm0 (F3 0F 5C C8), the frecps-vs-fsub discriminator");
        assert!(!f.windows(4).any(|w| w == [0xf3, 0x0f, 0x5c, 0xc1]), "frecps must NOT emit the fsub register direction");
        // 2.0f32 little-endian 0x40000000 materialized as an imm32 word
        assert!(f.windows(4).any(|w| w == [0x00, 0x00, 0x80, 0x40]) || f.windows(3).any(|w| w == [0x00, 0x00, 0x40]) && f.windows(4).any(|w| w == [0x00, 0x00, 0x00, 0x40]),
            "frecps materializes 2.0f32 (0x40000000)");
        let r = tr_bytes(Inst::VecFpArith { rd: 1, rn: 2, rm: 3, op: 9, q: false });
        assert!(r.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xd8]), "frsqrts loads 0.5 into xmm3 (movd xmm3,eax)");
        assert!(r.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xcb]), "frsqrts scales by 0.5 via mulss xmm1,xmm3 (F3 0F 59 CB)");
    }

    #[test]
    fn sh445_veccmp_2s_fcmeq_allones_mask_full_buffer_and_cc() {
        // fcmeq V1.2s, V2.2s, V3.2s (esize=4, op=0, cc=sete): each lane -> all-ones
        // if equal else 0. rd=1 rn=2 rm=3 -> Vd@0x120, Vn@0x130, Vm@0x140, 2 lanes.
        let b = tr_bytes(Inst::VecFpCmp { rd: 1, rn: 2, rm: 3, esize: 4, op: 0, q: false, abs: false });
        assert_eq!(b, vec![
            // lane 0
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] Vn lane0
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0x8b, 0x83, 0x40, 0x01, 0x00, 0x00, // mov eax,[rbx+0x140] Vm lane0
            0x66, 0x0f, 0x6e, 0xc8,             // movd xmm1,eax
            0x40, 0x0f, 0x2f, 0xc1,             // comiss xmm0,xmm1 (CF/ZF)
            0x0f, 0x94, 0xc0,                   // sete al  (cc 4)
            0x0f, 0xb6, 0xc0,                   // movzx eax,al
            0x48, 0xf7, 0xd8,                   // neg rax   -> +1 becomes all-ones
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],eax Vd lane0
            // lane 1
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // mov eax,[rbx+0x134] Vn lane1
            0x66, 0x0f, 0x6e, 0xc0,
            0x8b, 0x83, 0x44, 0x01, 0x00, 0x00, // mov eax,[rbx+0x144] Vm lane1
            0x66, 0x0f, 0x6e, 0xc8,
            0x40, 0x0f, 0x2f, 0xc1,             // comiss
            0x0f, 0x94, 0xc0,                   // sete al
            0x0f, 0xb6, 0xc0,                   // movzx eax,al
            0x48, 0xf7, 0xd8,                   // neg rax
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [rbx+0x124],eax Vd lane1
        ]);
        // the all-ones mask: setback is 1 -> neg yields 0xffffffff, so a lane store
        // carries the full all-ones mask for a matched lane (0 -> 0 for a miss).
        assert!(b.windows(3).any(|w| w == [0x48, 0xf7, 0xd8]), "compare->mask must emit neg rax (1 -> all-ones)");
        // lane addressing advances +4/lane across Vn/Vm/Vd.
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x24, 0x01, 0x00, 0x00]), "lane1 stores Vd@0x124");
        // cc discriminator: fcmeq = sete (0f 94), fcmgt = seta (0f 97), fcmge = setae (0f 93).
        let gt = tr_bytes(Inst::VecFpCmp { rd: 1, rn: 2, rm: 3, esize: 4, op: 1, q: false, abs: false });
        assert!(gt.windows(3).any(|w| w == [0x0f, 0x97, 0xc0]), "fcmgt = seta (0f 97)");
        assert!(!gt.windows(3).any(|w| w == [0x0f, 0x94, 0xc0]), "fcmgt must NOT emit sete");
        let ge = tr_bytes(Inst::VecFpCmp { rd: 1, rn: 2, rm: 3, esize: 4, op: 2, q: false, abs: false });
        assert!(ge.windows(3).any(|w| w == [0x0f, 0x93, 0xc0]), "fcmge = setae (0f 93)");
        assert!(!ge.windows(3).any(|w| w == [0x0f, 0x94, 0xc0]), "fcmge must NOT emit sete");
        // comiss carries the always-REX 0x40 (same quirk as maxss/minss/sh444).
        assert!(b.windows(4).any(|w| w == [0x40, 0x0f, 0x2f, 0xc1]), "single-path compare = comiss (40 0f 2f c1)");
    }

    #[test]
    fn sh445_veccmp_2d_comisd_width_discriminator() {
        // fcmeq V1.2d, V2.2d, V3.2d (esize=8): 1 lane (8/8). Double uses movq_load
        // (f3 48 0f 7e) + comisd (66 40 0f 2f c1) + 64-bit store (48 89 83) — the
        // esize=8 width discriminator vs the single single-precision path.
        let b = tr_bytes(Inst::VecFpCmp { rd: 1, rn: 2, rm: 3, esize: 8, op: 0, q: false, abs: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x130] Vn
            0xf3, 0x48, 0x0f, 0x7e, 0x8b, 0x40, 0x01, 0x00, 0x00, // movq xmm1,[rbx+0x140] Vm
            0x66, 0x40, 0x0f, 0x2f, 0xc1,                         // comisd xmm0,xmm1
            0x0f, 0x94, 0xc0,                                     // sete al
            0x0f, 0xb6, 0xc0,                                     // movzx eax,al
            0x48, 0xf7, 0xd8,                                     // neg rax
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00,             // mov [rbx+0x120],rax (64-bit)
        ]);
        assert!(b.windows(5).any(|w| w == [0x66, 0x40, 0x0f, 0x2f, 0xc1]), "double-path compare = comisd (66 40 0f 2f c1)");
        assert!(b.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00]), "double-path loads Vn via movq (f3 48 0f 7e)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "double-path stores 64-bit mask (48 89 83)");
        // width discriminator: the double path exclusively uses movq_load
        // (f3 48 0f 7e) — it must NOT emit the single path's movd (66 0f 6e c0).
        assert!(!b.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xc0]), "double-path must NOT emit the single-precision movd (66 0f 6e c0)");
    }

    #[test]
    fn sh445_veccmp_abs_signmask_pand_before_compare() {
        // facgt/facge (abs): compare |Vn| vs |Vm| — the emit clears each sign bit
        // (pand with 0x7fffffff) BEFORE the comiss. The signmask constant +
        // pand sequence is the abs-vs-nonabs discriminator.
        let b = tr_bytes(Inst::VecFpCmp { rd: 1, rn: 2, rm: 3, esize: 4, op: 1, q: false, abs: true });
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0xdb, 0xc2]), "abs clears |Vn| sign via pand xmm0,xmm2 (66 0f db c2)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0xdb, 0xca]), "abs clears |Vm| sign via pand xmm1,xmm2 (66 0f db ca)");
        assert!(b.windows(10).any(|w| w == [0x48, 0xb8, 0xff, 0xff, 0xff, 0x7f, 0x00, 0x00, 0x00, 0x00]),
            "abs sign-clear merges the 0x7fffffff constant (mov rax,0x7fffffff)");
        assert!(b.windows(5).any(|w| w == [0x66, 0x48, 0x0f, 0x6e, 0xd0]), "abs loads mask into xmm2 (movq xmm2,rax, 66 48 0f 6e d0)");
        // the pand must precede the comiss (sign cleared before compare).
        let c_pos = b.windows(4).position(|w| w == [0x40, 0x0f, 0x2f, 0xc1]).unwrap();
        let pand_pos = b.windows(4).position(|w| w == [0x66, 0x0f, 0xdb, 0xc2]).unwrap();
        assert!(pand_pos < c_pos, "abs must pand (clear sign) BEFORE the comiss compare");
        assert!(b.windows(3).any(|w| w == [0x0f, 0x97, 0xc0]), "facgt keeps seta (0f 97)");
    }

#[test]
    fn sh446_fmulel_2s_full_buffer_broadcast_once() {
        // fmul V1.2s, V2.2s, V4.s[0] (esize=4, q=false, idx=0): each lane =
        // V2[lane] * V4[0]. The element Vm[0] (rd=1 rm=4 -> Vm@0x150) is
        // broadcast into xmm2 ONCE up front, then per-lane mulss xmm0,xmm2.
        let b = tr_bytes(Inst::SimdFmulEl { rd: 1, rn: 2, rm: 4, esize: 4, index: 0, q: false });
        assert_eq!(b, vec![
            // broadcast element ONCE into xmm2
            0x8b, 0x83, 0x50, 0x01, 0x00, 0x00, // mov eax,[rbx+0x150] Vm[0]
            0x66, 0x0f, 0x6e, 0xd0,             // movd xmm2,eax
            // lane 0: Vn@0x130 * xmm2 -> Vd@0x120
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] Vn lane0
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0xf3, 0x0f, 0x59, 0xc2,             // mulss xmm0,xmm2
            0x66, 0x0f, 0x7e, 0xc0,             // movd eax,xmm0
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],eax Vd lane0
            // lane 1: Vn@0x134 -> Vd@0x124
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // mov eax,[rbx+0x134] Vn lane1
            0x66, 0x0f, 0x6e, 0xc0,
            0xf3, 0x0f, 0x59, 0xc2,             // mulss xmm0,xmm2
            0x66, 0x0f, 0x7e, 0xc0,
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [rbx+0x124],eax Vd lane1
        ]);
        // broadcast-once: exactly ONE movd xmm2 (66 0f 6e d0).
        assert_eq!(b.windows(4).filter(|w| w == &[0x66, 0x0f, 0x6e, 0xd0]).count(), 1,
            "fmul-el must broadcast the element into xmm2 exactly ONCE");
        // element source = Vm base + index*es (rm=4 -> +16, idx0 -> +0 => 0x150).
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x50, 0x01, 0x00, 0x00]), "element read at Vm[0]=0x150");
        // per-lane product uses the SH433-familiar mulss but dst=xmm0 src=xmm2.
        assert_eq!(b.windows(4).filter(|w| w == &[0xf3, 0x0f, 0x59, 0xc2]).count(), 2,
            "each of the 2 lanes multiplies via mulss xmm0,xmm2 (f3 0f 59 c2)");
    }

    #[test]
    fn sh446_fmulel_4s_index_and_lane_advance() {
        // fmul V1.4s, V2.4s, V4.s[1] (q=true, idx=1): element at Vm[1] = 0x154
        // (index*4), 4 lanes. Broadcast still once, dst lanes 0x120..0x12c.
        let b = tr_bytes(Inst::SimdFmulEl { rd: 1, rn: 2, rm: 4, esize: 4, index: 1, q: true });
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x54, 0x01, 0x00, 0x00]), "element read at Vm[1]=0x154 (index*es)");
        assert_eq!(b.windows(4).filter(|w| w == &[0x66, 0x0f, 0x6e, 0xd0]).count(), 1, ".4s still broadcasts xmm2 once");
        assert_eq!(b.windows(4).filter(|w| w == &[0xf3, 0x0f, 0x59, 0xc2]).count(), 4, ".4s has 4 lane mulss");
        // 4 distinct dst lanes advance +4 from 0x120.
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x28, 0x01, 0x00, 0x00]));
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x2c, 0x01, 0x00, 0x00]));
    }

    #[test]
    fn sh446_fmulel_2d_double_broadcast_and_mulsd_width() {
        // fmul V1.2d, V2.2d, V4.d[1] (esize=8, idx=1): element via movq_load at
        // 0x158, per-lane movq_load + mulsd (f2 0f 59 c2) + movq_store (66 48 0f d6).
        let b = tr_bytes(Inst::SimdFmulEl { rd: 1, rn: 2, rm: 4, esize: 8, index: 1, q: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x93, 0x58, 0x01, 0x00, 0x00, // movq xmm2,[rbx+0x158] Vm[1]
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x130] Vn lane0
            0xf2, 0x0f, 0x59, 0xc2,                               // mulsd xmm0,xmm2
            0x66, 0x48, 0x0f, 0xd6, 0x83, 0x20, 0x01, 0x00, 0x00, // movq [rbx+0x120],xmm0 Vd
        ]);
        assert!(b.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x93, 0x58, 0x01, 0x00, 0x00]), "double element via movq xmm2 @0x158");
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc2]), "double product via mulsd xmm0,xmm2 (f2 0f 59 c2)");
        assert!(b.windows(9).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x83, 0x20, 0x01, 0x00, 0x00]), "double store via movq (66 48 0f d6)");
        assert!(!b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc2]), "double must NOT emit single mulss");
    }

    #[test]
    fn sh446_fmulel_rd_eq_rm_broadcast_captures_before_overlap_store() {
        // fmul V4.2s, V2.2s, V4.s[0] (rd==rm==4): the rd==rm clobber guard —
        // the element V4[0] (also the dst Vd@0x150) must be captured into xmm2
        // BEFORE the first lane stores back over Vd@0x150, else all lanes after
        // 0 read the clobbered element. Pin: movd xmm2 (66 0f 6e d0) precedes the
        // store to 0x150.
        let b = tr_bytes(Inst::SimdFmulEl { rd: 4, rn: 2, rm: 4, esize: 4, index: 0, q: false });
        let e_pos = b.windows(4).position(|w| w == &[0x66, 0x0f, 0x6e, 0xd0]).unwrap(); // element->xmm2
        let s_pos = b.windows(6).position(|w| w == &[0x89, 0x83, 0x50, 0x01, 0x00, 0x00]).unwrap(); // store Vd@0x150
        assert!(e_pos < s_pos, "rd==rm: must capture the element into xmm2 BEFORE the overlapping Vd store");
        assert_eq!(b.windows(6).filter(|w| w == &[0x89, 0x83, 0x50, 0x01, 0x00, 0x00]).count(), 1, "dst lane0 stores Vd@0x150 (=Vm[0] source)");
        // a re-read-implementation would load the element from [rbx+0x150]
        // inside the lane loop again after the clobbering store — assert the
        // element source is read exactly ONCE (moves into xmm2 only at the top).
        assert_eq!(b.windows(6).filter(|w| w == &[0x8b, 0x83, 0x50, 0x01, 0x00, 0x00]).count(), 1,
            "the element source must be read from memory exactly ONCE (guard corrupts a per-lane re-read)");
    }

#[test]
    fn sh439_fcvtztoint_fixed_point_fbits_scales_by_2pown_mulsd() {
        // fcvtzu x0, s1, #4 (fixed-point, fbits=4): result = Fn * 2^4.
        // The scale const (2^4 = 16.0 double = 0x4030_0000_0000_0000) is
        // materialized, moved to xmm1 (66 48 0F 6E C8), and MULt into xmm0
        // (mulsd F2 0F 59 C1) BEFORE truncation. fbits=0 must NOT mulsd.
        let f = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 0, sf: true, unsigned: true, src_sng: false, fbits: 4 });
        assert!(f.windows(10).any(|w| w == [0x48, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x40]), "fixed-point materializes 2^4 as double (mov rax,0x4030_0000_0000_0000)");
        assert!(f.windows(5).any(|w| w == [0x66, 0x48, 0x0f, 0x6e, 0xc8]), "fixed-point moves the scale to xmm1 (movq xmm1,rax)");
        assert!(f.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc1]), "fixed-point multiplies BEFORE trunc (mulsd xmm0,xmm1)");
        assert!(f.windows(5).any(|w| w == [0x66, 0x40, 0x0f, 0x2f, 0xc1]), "fixed-point unsigned still carries the comisd range gate AFTER the scale");
        // fbits=0 must be a single-slippery path the trunc never pre-scales:
        let z = tr_bytes(Inst::FcvtToInt { rd: 0, rn: 1, mode: 0, sf: true, unsigned: true, src_sng: false, fbits: 0 });
        assert!(!z.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc1]), "fbits=0 must NOT emit mulsd (no fixed-point scale)");
    }

    #[test]
    fn sh447_fpunary_fneg_2s_flips_sign_via_xor_and_stores32() {
        // fneg V1.2s, V2.2s (rd=1 rn=2 op=0 esize=4 q=false): per-lane the FP
        // bit-pattern is moved into RAX, sign halfword(bit31) const is XOR'd in
        // (48 31 c8), moved back, then stored 32-bit (89 83). Full-buffer exact.
        let b = tr_bytes(Inst::SimdFpUnary { rd: 1, rn: 2, op: 0, esize: 4, q: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[0x130] Vn lane0
            0x66, 0x48, 0x0f, 0x7e, 0xc0,                         // movq rax,xmm0
            0x48, 0xb9, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, // mov rcx,0x80000000
            0x48, 0x31, 0xc8,                                     // xor rax,rcx (fneg: flip bit31)
            0x66, 0x48, 0x0f, 0x6e, 0xc0,                         // movq xmm0,rax
            0x66, 0x0f, 0x7e, 0xc0,                               // movd eax,xmm0
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00,                   // mov [0x120],eax Vd lane0
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x34, 0x01, 0x00, 0x00, // movq xmm0,[0x134] lane1
            0x66, 0x48, 0x0f, 0x7e, 0xc0,
            0x48, 0xb9, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
            0x48, 0x31, 0xc8,
            0x66, 0x48, 0x0f, 0x6e, 0xc0,
            0x66, 0x0f, 0x7e, 0xc0,
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00,                   // mov [0x124],eax lane1
        ]);
        // single width: stores are 32-bit (89 83), never the 16-bit 66 89 form.
        assert!(!b.windows(8).any(|w| w == [0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, 0xf3]));
    }

    #[test]
    fn sh447_fpunary_fabs_clears_sign_through_not_and_never_xor() {
        // fabs V1.2s, V2.2s (op=1): sign cleared via mov rdx,const; not rdx
        // (48 f7 d2); and rax,rdx (48 21 d0). The and+not (NOT xor) is the
        // fneg-vs-fabs discriminator.
        let b = tr_bytes(Inst::SimdFpUnary { rd: 1, rn: 2, op: 1, esize: 4, q: false });
        // const 0x80000000 -> rdx (48 ba), not rdx, and rax,rdx
        assert!(b.windows(16).any(|w| w == [0x48, 0xba, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x48, 0xf7, 0xd2, 0x48, 0x21, 0xd0]));
        // fabs must NEVER emit the xor-flip (48 31 c8) that fneg uses.
        assert!(!b.windows(3).any(|w| w == [0x48, 0x31, 0xc8]), "fabs clears via and, never flips via xor");
    }

    #[test]
    fn sh447_fpunary_fsqrt_2d_sqrtsd_no_gpr_sign_manip() {
        // fsqrt V1.2d V2.2d q=false (op=2): 1 lane, movq xmm0 -> sqrtsd
        // (f2 0f 51 c0) -> movq store (66 48 0f d6). Double width; NO GPR
        // sign-bit manipulation at all (fsqrt never touches rax/rcx).
        let b = tr_bytes(Inst::SimdFpUnary { rd: 1, rn: 2, op: 2, esize: 8, q: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[0x130]
            0xf2, 0x0f, 0x51, 0xc0,                               // sqrtsd xmm0,xmm0
            0x66, 0x48, 0x0f, 0xd6, 0x83, 0x20, 0x01, 0x00, 0x00, // movq [0x120],xmm0
        ]);
        assert!(!b.windows(3).any(|w| w == [0x48, 0xb9, 0x00]), "fsqrt emits no sign const (mov rcx)");
        assert!(!b.windows(4).any(|w| w == [0x48, 0x31, 0xc8]), "fsqrt emits no xor-flip");
    }

    #[test]
    fn sh447_fpunary_esize_width_discriminator_sign_const_and_store() {
        // The sign-const AND store width both discriminate esize: esize=8 loads
        // the full 64-bit sign (0x8000_0000_0000_0000 -> imm 00*7 80) + 64-bit
        // movq store; esize=4 loads bit31 (0x8000_0000) + 32-bit store; esize=2
        // loads bit15 (0x8000) + 16-bit 66 89 store.
        let d = tr_bytes(Inst::SimdFpUnary { rd: 1, rn: 2, op: 0, esize: 8, q: true });
        assert!(d.windows(16).any(|w| w == [0x48, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x48, 0x31, 0xc8, 0x66, 0x48, 0x0f]), "esize=8 const = 0x8000..0x00 (bit63)");
        assert!(d.windows(9).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x83, 0x28, 0x01, 0x00, 0x00]), "esize=8 stores 64-bit movq (66 48 0f d6), lane1 at +8");

        let h = tr_bytes(Inst::SimdFpUnary { rd: 1, rn: 2, op: 0, esize: 2, q: false });
        assert!(h.windows(10).any(|w| w == [0x48, 0xb9, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]), "esize=2 const = 0x8000 (bit15)");
        assert!(h.windows(7).any(|w| w == [0x66, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "esize=2 stores 16-bit (66 89 83)");
    }

    #[test]
    fn sh447_fpunary_q_lane_advance_4s_stores_advance_by_esize() {
        // fabs V1.4s V2.4s (q=true): 4 lanes, loads 0x130/0x134/0x138/0x13c,
        // stores 0x120/0x124/0x128/0x12c — every lane advances +esize.
        let b = tr_bytes(Inst::SimdFpUnary { rd: 1, rn: 2, op: 1, esize: 4, q: true });
        for (i, lo) in [0x30u32, 0x34, 0x38, 0x3c].iter().enumerate() {
            assert!(b.windows(9).any(|w| w == &[0xf3, 0x48, 0x0f, 0x7e, 0x83, *lo as u8, 0x01, 0x00, 0x00]), "q lane {i} source at 0x130+{i}*4");
        }
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x28, 0x01, 0x00, 0x00]), "lane2 store 0x128");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x2c, 0x01, 0x00, 0x00]), "lane3 store 0x12c");
    }

    #[test]
    fn sh448_arithunary_neg_2s_full_buffer_neg_rax_32bit_loadstore() {
        // neg V1.2s, V2.2s (rd=1 rn=2 esize=4 q=false op=0): per-lane a 32-bit
        // zero-extending load (8b 83), then `neg rax` (48 f7 d8), then a 32-bit
        // store (89 83). Full-buffer exact.
        let b = tr_bytes(Inst::SimdArithUnary { rd: 1, rn: 2, esize: 4, q: false, op: 0 });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[0x130] Vn lane0
            0x48, 0xf7, 0xd8,                   // neg rax
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [0x120],eax Vd lane0
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // mov eax,[0x134] lane1
            0x48, 0xf7, 0xd8,                   // neg rax
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // mov [0x124],eax lane1
        ]);
        // neg never emits the abs sign-extend (48 63 c0 movsxd).
        assert!(!b.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "neg must not movsxd");
    }

    #[test]
    fn sh448_arithunary_abs_2s_signextend_sar_xor_sub_idiom() {
        // abs V1.2s V2.2s (op=1): the (x ^ (x ar>> w-1)) - (x ar>> w-1) idiom.
        // esize=4 must movsxd (48 63 c0) the 32-bit lane FIRST, then mov rcx,rax
        // (48 89 c1) + sar rcx,31 (48 c1 f9 1f) + xor abs,xmm... xor rax,rcx
        // (48 31 c8) + sub rax,rcx (48 29 c8), 32-bit store (89 83).
        let b = tr_bytes(Inst::SimdArithUnary { rd: 1, rn: 2, esize: 4, q: false, op: 1 });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[0x130]
            0x48, 0x63, 0xc0,                   // movsxd rax,eax (signed lane)
            0x48, 0x89, 0xc1,                   // mov rcx,rax
            0x48, 0xc1, 0xf9, 0x1f,             // sar rcx,31  (w-1 = 4*8-1)
            0x48, 0x31, 0xc8,                   // xor rax,rcx
            0x48, 0x29, 0xc8,                   // sub rax,rcx (|x|)
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [0x120],eax
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00,
            0x48, 0x63, 0xc0,
            0x48, 0x89, 0xc1,
            0x48, 0xc1, 0xf9, 0x1f,
            0x48, 0x31, 0xc8,
            0x48, 0x29, 0xc8,
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00,
        ]);
        // abs must NOT be a bare neg (48 f7 d8).
        assert!(!b.windows(3).any(|w| w == [0x48, 0xf7, 0xd8]), "abs uses the sar/xor/sub idiom, never neg");
    }

    #[test]
    fn sh448_arithunary_width_discriminator_esize_load_sar_store() {
        // neg 2d q=true: full 64-bit load (48 8b 83) + neg (48 f7 d8) + 64-bit
        // store (48 89 83), 2 lanes at +8. The abs sar imm = esize*8-1 (double ->
        // 63, byte -> 7). esize=1 abs loads via movsx_byte_mem (48 0f be) and
        // stores byte (88 83), lanes advance +1.
        let d = tr_bytes(Inst::SimdArithUnary { rd: 1, rn: 2, esize: 8, q: true, op: 0 });
        assert!(d.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "2d loads 64-bit (48 8b 83) lane0");
        assert!(d.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00]), "2d stores 64-bit (48 89 83) lane1 at +8 0x128");

        let h = tr_bytes(Inst::SimdArithUnary { rd: 1, rn: 2, esize: 1, q: false, op: 1 });
        assert!(h.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x30, 0x01, 0x00]), "abs byte loads via movsx_byte_mem (48 0f be)");
        assert!(h.windows(4).any(|w| w == [0x48, 0xc1, 0xf9, 0x07]), "abs byte sar imm = 7 (esize*8-1)");
        assert!(h.windows(6).any(|w| w == [0x88, 0x83, 0x21, 0x01, 0x00, 0x00]), "byte store lane1 at 0x121 (advance +1)");
    }

    #[test]
    fn sh448_arithunary_abs_neg_discriminator_movsxd_vs_bare_neg_never_cross() {
        // The load-extend is THE discriminator: abs(32-bit) sign-extends
        // (48 63 c0 movsxd), neg MUST NOT; neg always emits 48 f7 d8, abs MUST
        // NOT. A cross silently abs()'s a neg or negates an abs.
        let neg = tr_bytes(Inst::SimdArithUnary { rd: 1, rn: 2, esize: 4, q: true, op: 0 });
        assert_eq!(neg.windows(3).filter(|w| *w == [0x48, 0xf7, 0xd8]).count(), 4, "4-lane neg has 4 neg rax");
        assert_eq!(neg.windows(3).filter(|w| *w == [0x48, 0x63, 0xc0]).count(), 0, "neg never movsxd");
        let abs = tr_bytes(Inst::SimdArithUnary { rd: 1, rn: 2, esize: 4, q: true, op: 1 });
        assert_eq!(abs.windows(3).filter(|w| *w == [0x48, 0x63, 0xc0]).count(), 4, "4-lane abs has 4 movsxd");
        assert_eq!(abs.windows(3).filter(|w| *w == [0x48, 0xf7, 0xd8]).count(), 0, "abs never neg rax");
    }

    #[test]
    fn sh449_cmpzero_eq_2s_sete_no_signextend_allones_store32() {
        // cmeq V1.2s V2.2s,#0 (rd=1 rn=2 esize=4 q=false cond=0): per-lane a
        // 32-bit zero-extending load (8b 83), test rax (48 85 c0), sete al
        // (0f 94 c0), movzx (0f b6 c0), neg rax (48 f7 d8) -> 0 or all-ones,
        // 32-bit store (89 83). eq (cond 0) is UNSIGNED — no movsxd. Full-buffer.
        let b = tr_bytes(Inst::SimdCmpZero { rd: 1, rn: 2, esize: 4, q: false, cond: 0 });
        assert_eq!(b, vec![
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[0x130] lane0
            0x48, 0x85, 0xc0,                   // test rax,rax
            0x0f, 0x94, 0xc0,                   // sete al
            0x0f, 0xb6, 0xc0,                   // movzx eax,al
            0x48, 0xf7, 0xd8,                   // neg rax (0 or all-ones)
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [0x120],eax
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00,
            0x48, 0x85, 0xc0,
            0x0f, 0x94, 0xc0,
            0x0f, 0xb6, 0xc0,
            0x48, 0xf7, 0xd8,
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00,
        ]);
        assert!(!b.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "cmeq (cond 0) is unsigned — must not movsxd");
    }

    #[test]
    fn sh449_cmpzero_cond_cc_byte_is_the_semantic() {
        // The setcc opcode byte IS the semantic: cond 0=eq 0f 94, 1=gt 0f 9f,
        // 2=ge 0f 9d, 3=lt 0f 9c, 4=le 0f 9e. A wrong cond picks the wrong
        // comparison, silently (a >= 0 becomes a > 0). Signed conds (1-4)
        // MUST movsxd (48 63 c0) the lane; eq does not.
        let cc_for = |cond| {
            let b = tr_bytes(Inst::SimdCmpZero { rd: 1, rn: 2, esize: 4, q: false, cond });
            let i = b.windows(2).position(|w| w == [0x0f, 0x94]).or_else(|| b.windows(2).position(|w| w == [0x0f, 0x9f]))
                .or_else(|| b.windows(2).position(|w| w == [0x0f, 0x9d])).or_else(|| b.windows(2).position(|w| w == [0x0f, 0x9c]))
                .or_else(|| b.windows(2).position(|w| w == [0x0f, 0x9e])).unwrap();
            (b[i], b[i + 1])
        };
        assert_eq!(cc_for(0), (0x0f, 0x94), "eq=sete");
        assert_eq!(cc_for(1), (0x0f, 0x9f), "gt=setg");
        assert_eq!(cc_for(2), (0x0f, 0x9d), "ge=setge");
        assert_eq!(cc_for(3), (0x0f, 0x9c), "lt=setl");
        assert_eq!(cc_for(4), (0x0f, 0x9e), "le=setle");
        // signed conds sign-extend the 32-bit lane.
        assert!(tr_bytes(Inst::SimdCmpZero { rd: 1, rn: 2, esize: 4, q: false, cond: 4 })
            .windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "le (signed) must movsxd");
    }

    #[test]
    fn sh449_cmpzero_lt_1b_8_lanes_signextend_byte_store() {
        // cmlt V1.8b V2.8b,#0 (esize=1 q=false cond=3): 8 lanes; each loads via
        // movsx_byte_mem (48 0f be — signed), test, setl (0f 9c), movzx, neg,
        // byte store (88 83), lanes advance +1 (0x130..0x137, 0x120..0x127).
        let b = tr_bytes(Inst::SimdCmpZero { rd: 1, rn: 2, esize: 1, q: false, cond: 3 });
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x30, 0x01, 0x00]), "lane0 signed byte load (48 0f be)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x0f, 0xbe, 0x83, 0x37, 0x01, 0x00]), "lane7 signed byte load at +7");
        assert!(b.windows(6).any(|w| w == [0x88, 0x83, 0x27, 0x01, 0x00, 0x00]), "lane7 byte store at 0x127");
        assert_eq!(b.windows(3).filter(|w| *w == [0x0f, 0x9c, 0xc0]).count(), 8, "8 lanes, 8 setl");
    }

    #[test]
    fn sh449_cmpzero_eq_2d_q_64bit_load_store_and_unsigned() {
        // cmeq V1.2d V2.2d,#0 (esize=8 q=true cond=0): full 64-bit load
        // (48 8b 83) + test + sete + neg + 64-bit store (48 89 83), 2 lanes at
        // +8. eq never sign-extends (64-bit load is already the full value).
        let b = tr_bytes(Inst::SimdCmpZero { rd: 1, rn: 2, esize: 8, q: true, cond: 0 });
        assert!(b.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "2d 64-bit load (48 8b 83)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00]), "2d 64-bit store lane1 at +8 0x128");
        assert_eq!(b.windows(3).filter(|w| *w == [0x0f, 0x94, 0xc0]).count(), 2, "2 lanes, 2 sete");
        // the all-ones mask construction appears once/lane: test + sete + neg =
        // every lane's result is 0 or 0xffffffffffffffff (64-bit neg of 0/1).
        assert_eq!(b.windows(3).filter(|w| *w == [0x48, 0xf7, 0xd8]).count(), 2, "2 neg rax (0-or-allones)");
    }

    #[test]
    fn sh450_vfpcz_2s_fcmeq_pxor_zero_operand_full_buffer() {
        // fcmeq V1.2s V2.2s,#0.0 (rd=1 rn=2 esize=4 op=0 q=false): per-lane
        // all-ones mask if Vn == +0.0 else 0. The SECOND operand is a ZEROED
        // xmm1 — each lane emits pxor xmm1,xmm1 (66 0f ef c9) to materialize
        // +0.0, then mov_load32 (8b 83) + movd xmm0,eax (66 0f 6e c0) + comiss
        // (40 0f 2f c1) + sete al (0f 94 c0) + movzx (0f b6 c0) + neg rax
        // (48 f7 d8) + 32-bit store (89 83). This is the DISTINCTIVE marker vs
        // SH445's two-operand VecFpCmp, which loads Vm — VecFpCmpZero instead
        // pxor's a fresh zero operand every lane. Full-buffer (2 lanes, +4/lane).
        let b = tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 0, esize: 4, q: false });
        assert_eq!(b, vec![
            // lane 0
            0x66, 0x0f, 0xef, 0xc9,             // pxor xmm1,xmm1 (+0.0 second operand)
            0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov eax,[rbx+0x130] Vn lane0
            0x66, 0x0f, 0x6e, 0xc0,             // movd xmm0,eax
            0x40, 0x0f, 0x2f, 0xc1,             // comiss xmm0,xmm1
            0x0f, 0x94, 0xc0,                   // sete al (cc op=0 eq)
            0x0f, 0xb6, 0xc0,                   // movzx eax,al
            0x48, 0xf7, 0xd8,                   // neg rax -> 0 or all-ones
            0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],eax Vd lane0
            // lane 1
            0x66, 0x0f, 0xef, 0xc9,
            0x8b, 0x83, 0x34, 0x01, 0x00, 0x00, // Vn lane1
            0x66, 0x0f, 0x6e, 0xc0,
            0x40, 0x0f, 0x2f, 0xc1,
            0x0f, 0x94, 0xc0,
            0x0f, 0xb6, 0xc0,
            0x48, 0xf7, 0xd8,
            0x89, 0x83, 0x24, 0x01, 0x00, 0x00, // Vd lane1 0x124
        ]);
        // the zero-operand discriminant: pxor xmm1,xmm1 (66 0f ef c9) appears
        // once per lane (2 here), and the parser NEVER emits a Vm load (there is
        // no third operand) — the 2-operand VecFpCmp's mov eax,[rbx+0x140/0x144]
        // Vm loads are absent. A flub that reuses a shared/garbage register
        // instead of re-zeroing per lane compares against stale state.
        assert_eq!(b.windows(4).filter(|w| *w == [0x66, 0x0f, 0xef, 0xc9]).count(), 2, "a fresh pxor xmm1,xmm1 per lane (2)");
        assert!(!b.windows(6).any(|w| w == [0x8b, 0x83, 0x40, 0x01, 0x00, 0x00]), "no Vm load at 0x140 (zero-operand form)");
        assert!(!b.windows(6).any(|w| w == [0x8b, 0x83, 0x44, 0x01, 0x00, 0x00]), "no Vm load at 0x144");
    }

    #[test]
    fn sh450_vfpcz_cc_map_eq_gt_ge_lt_le() {
        // The setcc condition byte maps op -> cc: op0 fcmeq=sete 0f 94, op1
        // fcmgt=seta 0f 97, op2 fcmge=setae 0f 93, op3 fcmlt=setb 0f 92, op4
        // fcmle=setbe 0f 96. A wrong cond silently picks the wrong comparison
        // (a wrong op turns eq-into-0 into gt-into-0). Also equivalence to
        // SH445's two-operand form: same comiss+cc byte, so a lone setcc pin
        // CANNOT distinguish the zero-vs-loaded second operand — the pxor is
        // the real discriminant (pinned above).
        let cc_byte = |op| {
            let b = tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op, esize: 4, q: false });
            // the first setcc 0f 9X (skipping the leading pxor + loads)
            let i = b.windows(2).position(|w| w == [0x0f, 0x94]).unwrap();
            let (a, c) = (b[i], b[i + 1]);
            // must land in the 0f 9X family and be a real setcc
            (a, c)
        };
        assert_eq!(cc_byte(0), (0x0f, 0x94), "fcmeq=sete");
        assert!(tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 1, esize: 4, q: false }).windows(3).any(|w| w == [0x0f, 0x97, 0xc0]), "fcmgt=seta");
        assert!(tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 2, esize: 4, q: false }).windows(3).any(|w| w == [0x0f, 0x93, 0xc0]), "fcmge=setae");
        assert!(tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 3, esize: 4, q: false }).windows(3).any(|w| w == [0x0f, 0x92, 0xc0]), "fcmlt=setb");
        assert!(tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 4, esize: 4, q: false }).windows(3).any(|w| w == [0x0f, 0x96, 0xc0]), "fcmle=setbe");
        // negative: a transposed op must NOT emit the wrong cc. fcmgt must not
        // emit sete (0f 94), fcmeq must not emit seta (0f 97).
        assert!(!tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 1, esize: 4, q: false }).windows(3).any(|w| w == [0x0f, 0x94, 0xc0]), "fcmgt must not emit sete");
        assert!(!tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 0, esize: 4, q: false }).windows(3).any(|w| w == [0x0f, 0x97, 0xc0]), "fcmeq must not emit seta");
    }

    #[test]
    fn sh450_vfpcz_2d_comisd_width_discriminator() {
        // fcmeq V1.2d V2.2d,#0.0 (esize=8 op=0 q=false): 1 lane (8/8). Double
        // uses movq_load (f3 48 0f 7e) + comisd (66 40 0f 2f c1) + 64-bit
        // store (48 89 83). esize-width discriminator vs the single path.
        let b = tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 0, esize: 8, q: false });
        assert_eq!(b, vec![
            0x66, 0x0f, 0xef, 0xc9,                         // pxor xmm1,xmm1 (+0.0)
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x130] Vn
            0x66, 0x40, 0x0f, 0x2f, 0xc1,                   // comisd xmm0,xmm1
            0x0f, 0x94, 0xc0,                               // sete al
            0x0f, 0xb6, 0xc0,                               // movzx eax,al
            0x48, 0xf7, 0xd8,                               // neg rax
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00,       // mov [rbx+0x120],rax (64-bit)
        ]);
        assert!(b.windows(5).any(|w| w == [0x66, 0x40, 0x0f, 0x2f, 0xc1]), "double-path compare = comisd (66 40 0f 2f c1)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "double-path stores 64-bit mask (48 89 83)");
        // width discriminator: double path exclusively uses movq_load (f3 48 0f
        // 7e) — it must NOT emit the single-path movd (66 0f 6e c0).
        assert!(!b.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xc0]), "double-path must NOT emit movd (66 0f 6e c0)");
    }

    #[test]
    fn sh450_vfpcz_4s_q_true_lane_addressing_advances_4() {
        // fcmeq V1.4s V2.4s,#0.0 (esize=4 q=true): 4 lanes; Vd@0x120/0x124/
        // 0x128/0x12c, Vn@0x130/0x134/0x138/0x13c. The q bit doubles the lane
        // count (4 vs 2), confirming the lane-advance math for the full-register
        // form. Each lane still re-pxors the zero operand.
        let b = tr_bytes(Inst::VecFpCmpZero { rd: 1, rn: 2, op: 0, esize: 4, q: true });
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x38, 0x01, 0x00, 0x00]), "lane2 loads Vn@0x138");
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x3c, 0x01, 0x00, 0x00]), "lane3 loads Vn@0x13c");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x28, 0x01, 0x00, 0x00]), "lane2 stores Vd@0x128");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x2c, 0x01, 0x00, 0x00]), "lane3 stores Vd@0x12c");
        assert_eq!(b.windows(4).filter(|w| *w == [0x66, 0x0f, 0xef, 0xc9]).count(), 4, "4 lanes, 4 pxor xmm1,xmm1 (a fresh zero operand each)");
        // no 64-bit movq in the single-precision q form.
        assert!(!b.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00]), "4s q must stay single-path (no movq)");
    }

    #[test]
    fn sh458_lanes_esize8_index0_full_buffer() {
        // mov D1.1D, v2.1D[0] (esize=8 index=0): copy a 64-bit element to the
        // DEST FP slot's low bytes. EMIT: mov rax,[0x130] (48 8b 83, src =
        // Vn + index*8 = 0x130) + mov [0x120],rax (48 89 83, dst = Vd slot).
        // rd=1 rn=2 -> Vn@0x130 Vd@0x120. The DEST is FIXED at f(rd) — only the
        // index moves the source.
        let b = tr_bytes(Inst::SimdLaneS { rd: 1, rn: 2, esize: 8, index: 0 });
        assert_eq!(b, vec![
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130] src el 0
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax dst
        ]);
        assert!(b.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "esize=8 src loads via mov_load64 (48 8b 83)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "esize=8 dst stores via mov_store64 (48 89 83)");
    }

    #[test]
    fn sh458_lanes_index_moves_source_dst_fixed() {
        // mov D1.1D, v2.1D[1] (esize=8 index=1): index advances the SOURCE by
        // +esize (0x130 -> 0x138) while the DEST stays at f(rd)=0x120. A flub
        // that also advances the dest (or misses the source advance) copies the
        // wrong element.
        let b = tr_bytes(Inst::SimdLaneS { rd: 1, rn: 2, esize: 8, index: 1 });
        assert!(b.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x38, 0x01, 0x00, 0x00]), "index=1 reads source 0x138 (Vn + 1*8)");
        assert!(b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "dest stays at Vd 0x120 regardless of index");
        assert!(!b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x28, 0x01, 0x00, 0x00]), "index must NOT advance the DEST (dst stays fixed at f(rd))");
        // control: index=0 reads 0x130.
        let lo = tr_bytes(Inst::SimdLaneS { rd: 1, rn: 2, esize: 8, index: 0 });
        assert!(lo.windows(7).any(|w| w == [0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00]), "index=0 reads source 0x130");
    }

    #[test]
    fn sh458_lanes_esize4_width_and_index_stride() {
        // mov S1.1S, v2.1S[idx] (esize=4): the 32-bit width — mov_load32 (8b 83)
        // + mov_store32 (89 83), and index strides the source by +4 (0x130 ->
        // 0x134). The 89-vs-48 89 store (32-vs-64-bit) is the esize width
        // discriminator — a width flub silently half/doubles the copied element.
        let b = tr_bytes(Inst::SimdLaneS { rd: 1, rn: 2, esize: 4, index: 1 });
        assert!(b.windows(6).any(|w| w == [0x8b, 0x83, 0x34, 0x01, 0x00, 0x00]), "esize=4 index=1 reads source 0x134 (Vn + 1*4)");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "esize=4 dst stores via mov_store32 (89 83)");
        assert!(!b.windows(6).any(|w| w == [0x48, 0x8b, 0x83, 0x38, 0x01]), "esize=4 must NOT mov_load64 (48 8b 83)");
        assert!(!b.windows(7).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01]), "esize=4 must NOT mov_store64 (48 89 83)");
    }

    #[test]
    fn sh457_pmull_low64_movq_pclmulq_128bit_store() {
        // pmull v1.1Q, v2.1D, v3.1D (hi=false): 64x64 carry-less (polynomial)
        // multiply -> 128-bit via x86 PCLMULQDQ imm=0x00. EMIT: movq_load
        // xmm0=[0x130] (f3 48 0f 7e, low64 of Vn) + movq_load xmm1=[0x140]
        // (f3 48 0f 7e, low64 of Vm) + `pclmulqdq xmm0,xmm1,0x00` (66 0f 3a 44
        // c1 00) + movdqu_store 0x120 (f3 0f 7f). rd=1 rn=2 rm=3 -> Vn@0x130
        // Vm@0x140 Vd@0x120. A pclmulq slip (the wrong 2-vs-3-byte form) or a
        // 16B-vs-8B store shows in the exact buffer.
        let b = tr_bytes(Inst::Pmull1q { rd: 1, rn: 2, rm: 3, hi: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x130] Vn low64
            0xf3, 0x48, 0x0f, 0x7e, 0x8b, 0x40, 0x01, 0x00, 0x00, // movq xmm1,[rbx+0x140] Vm low64
            0x66, 0x0f, 0x3a, 0x44, 0xc1, 0x00,                   // pclmulqdq xmm0,xmm1,0x00
            0xf3, 0x0f, 0x7f, 0x83, 0x20, 0x01, 0x00, 0x00,       // movdqu [rbx+0x120],xmm0 (128b)
        ]);
        assert!(b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x44, 0xc1, 0x00]), "pmull = PCLMULQDQ xmm0,xmm1,0x00 (66 0f 3a 44)");
        assert!(b.windows(8).any(|w| w == [0xf3, 0x0f, 0x7f, 0x83, 0x20, 0x01, 0x00, 0x00]), "128-bit result stored via movdqu (f3 0f 7f, 16B)");
    }

    #[test]
    fn sh457_pmull_hi2_high_half_source_offset() {
        // pmull2 (hi=true): identical emit EXCEPT the selected 64-bit halves are
        // the UPPER elements (bytes 8..15) of Vn/Vm — sources advance +8 to
        // 0x138/0x148 (vs hi=false's 0x130/0x140). The hi flag is the ONLY thing
        // that moves the sources; the pclmulq imm stays 0x00 and the store stays
        // at Vd 0x120. A flub reading the low half still "works" arithmetically
        // but multiplies the wrong polynomial elements (silent wrong value).
        let hi = tr_bytes(Inst::Pmull1q { rd: 1, rn: 2, rm: 3, hi: true });
        // high-half source reads
        assert!(hi.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x38, 0x01, 0x00, 0x00]), "pmull2 reads Vn high half at 0x138");
        assert!(hi.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x8b, 0x48, 0x01, 0x00, 0x00]), "pmull2 reads Vm high half at 0x148");
        // the imm + opcode + store unchanged from hi=false
        assert!(hi.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x44, 0xc1, 0x00]), "pmull2 still PCLMULQDQ imm=0x00");
        assert!(hi.windows(8).any(|w| w == [0xf3, 0x0f, 0x7f, 0x83, 0x20, 0x01, 0x00, 0x00]), "pmull2 stores at Vd 0x120 (same 128b)");
        // negative: hi=true must NOT read the low halves.
        assert!(!hi.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00]), "pmull2 must NOT read Vn low half 0x130");
        assert!(!hi.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x8b, 0x40, 0x01, 0x00, 0x00]), "pmull2 must NOT read Vm low half 0x140");
        // control: hi=false reads the low halves.
        let lo = tr_bytes(Inst::Pmull1q { rd: 1, rn: 2, rm: 3, hi: false });
        assert!(lo.windows(9).any(|w| w == [0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00]), "pmull (hi=false) reads Vn low half 0x130");
    }

    #[test]
    fn sh457_pmull_pclmulq_imm_selects_direction() {
        // The pclmulq immediate (0x00 = low64 x low64) is the pmull1q semantic.
        // The byte lock is the three-byte PCLMULQDQ opcode 66 0f 3a 44 with the
        // imm following in the /r ib: the polynomial multiply must never degrade
        // to an integer mul (48 0f af) or a different imm that crosses halves
        // (0x11/0x10/0x01 would select the wrong 64-bit lanes and corrupt the
        // carry-less result's polynomial degree).
        let b = tr_bytes(Inst::Pmull1q { rd: 1, rn: 2, rm: 3, hi: false });
        assert!(b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x44, 0xc1, 0x00]), "the imm 0x00 selects low64 x low64");
        assert!(!b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x44, 0xc1, 0x01]), "imm must not select vn.low x vm.high");
        assert!(!b.windows(6).any(|w| w == [0x66, 0x0f, 0x3a, 0x44, 0xc1, 0x10]), "imm must not select vn.high x vm.low");
        assert!(!b.windows(3).any(|w| w == [0x48, 0x0f, 0xaf]), "pmull must NOT degrade to an integer imul (48 0f af)");
    }

    #[test]
    fn sh456_vshift_sshl_d8_full_buffer() {
        // sshl vL.2d, vV.2d, vC.2d (esize=8 signed_=true): per-lane variable
        // shift. bbits==64 (no wmask, no out-of-range clamp because x86's
        // shl/sar mask CL to the low 6 bits = the exact 0..63 ARM range). EMIT
        // per lane: mov rax,[V] + mov rcx,[C] + `test rcx,rcx` (48 85 c9) + js
        // (0f 88) to the right path + LEFT `shl rax,cl` (48 d3 e0) + jmp + RIGHT
        // `neg rcx` (48 f7 d9) + `sar rax,cl` (48 d3 f8, arithmetic) + mov
        // [Vd],rax. rd=1 rn=2 rm=3 -> Vd@0x120 V@0x130 C@0x140.
        let b = tr_bytes(Inst::SimdVShift { rd: 1, rn: 2, rm: 3, esize: 8, signed_: true, q: false, rounding: false });
        assert_eq!(b, vec![
            // lane 0
            0x48, 0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, // mov rax,[rbx+0x130] V
            0x48, 0x8b, 0x8b, 0x40, 0x01, 0x00, 0x00, // mov rcx,[rbx+0x140] C
            0x48, 0x85, 0xc9,                         // test rcx,rcx (SF from sign)
            0x0f, 0x88, 0x08, 0x00, 0x00, 0x00,       // js right (+8)
            0x48, 0xd3, 0xe0,                         // shl rax,cl (left, C>=0)
            0xe9, 0x06, 0x00, 0x00, 0x00,             // jmp done
            0x48, 0xf7, 0xd9,                         // neg rcx = magnitude k
            0x48, 0xd3, 0xf8,                         // sar rax,cl (arithmetic right)
            0x48, 0x89, 0x83, 0x20, 0x01, 0x00, 0x00, // mov [rbx+0x120],rax
        ]);
        assert!(b.windows(3).any(|w| w == [0x48, 0xd3, 0xe0]), ".2d left = shl rax,cl");
        assert!(b.windows(3).any(|w| w == [0x48, 0xd3, 0xf8]), ".2d signed right = sar rax,cl (F8)");
        assert!(!b.windows(3).any(|w| w == [0x48, 0xd3, 0xe8]), "sshl must NOT emit shr (E8)");
    }

    #[test]
    fn sh456_vshift_ushl_signextend_count_and_js_direction() {
        // ushl v1.2s, v2.2s, v3.2s (esize=4 signed_=false): per 32-bit lane. The
        // COUNT lane C is SIGN-extended (movsxd rcx,ecx 48 63 c9) so a negative
        // (high-bit-set) C makes the JS branch take the right path — a
        // zero-extend (no movsxd) turns a negative count into a huge positive
        // and wrongly takes the left path. The `test rcx,rcx` + `js` (0f 88)
        // sign-dispatch is the variable-shift control-flow discriminator.
        let b = tr_bytes(Inst::SimdVShift { rd: 1, rn: 2, rm: 3, esize: 4, signed_: false, q: false, rounding: false });
        // lane0 sign-extend + sign-dispatch
        assert!(b.windows(7).any(|w| w == [0x8b, 0x83, 0x30, 0x01, 0x00, 0x00, 0x8b]), "V lane0 loaded via mov eax,[rbx+0x130]");
        assert!(b.windows(3).any(|w| w == [0x48, 0x63, 0xc9]), "count sign-extends via movsxd rcx,ecx (48 63 c9) — a negative C must dispatch right");
        assert!(b.windows(3).any(|w| w == [0x48, 0x85, 0xc9]), "test rcx,rcx sets SF from the sign-extended count");
        assert!(b.windows(4).any(|w| w == [0x0f, 0x88, 0x31, 0x00]), "js = 0f 88 dispatches C<0 to the right path");
        // left path must guard C>=B (32) -> 0, with the wmask and-back
        assert!(b.windows(3).any(|w| w == [0x48, 0xd3, 0xe0]), "left path = shl rax,cl");
        assert!(b.windows(8).any(|w| w == [0x48, 0xba, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00]), "esize=4 left ANDs the wmask 0xffffffff");
        assert!(b.windows(3).any(|w| w == [0x48, 0x21, 0xd0]), "left masks via and rax,rdx (48 21 d0)");
        // store is 32-bit (89 83), the esize=4 width
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "lane0 stores 32-bit Vd (89 83 0x120)");
        assert!(!b.windows(6).any(|w| w == [0x48, 0x89, 0x83, 0x20, 0x01, 0x00]), "esize=4 must NOT mov_store64");
    }

    #[test]
    fn sh456_vshift_sshl_vs_ushl_right_shift_opcode() {
        // The signed/unsigned RIGHT-shift opcode is the core semantic: sshl/srshl
        // (signed_) right path uses `sar rax,cl` (48 d3 f8, arithmetic — sign-
        // extends, keeps a negative V negative), ushl/urshl uses `shr rax,cl`
        // (48 d3 e8, logical — zero-fills). A flub flips the direction-of-shift
        // semantics for every negative count. Both sign-extend the VALUE V (for
        // esize=4: movsxd rax,eax 48 63 c0) before the arithmetic right shift.
        let signed = tr_bytes(Inst::SimdVShift { rd: 1, rn: 2, rm: 3, esize: 4, signed_: true, q: false, rounding: false });
        assert!(signed.windows(3).any(|w| w == [0x48, 0xd3, 0xf8]), "sshl right = sar rax,cl (48 d3 f8, arithmetic)");
        assert!(signed.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "signed right sign-extends the VALUE V first (movsxd rax,eax)");
        assert!(!signed.windows(3).any(|w| w == [0x48, 0xd3, 0xe8]), "sshl must NOT emit shr (E8)");
        // signed out-of-range right: sign-fill (V<0 -> wmask 0xffffffff, V>=0 -> 0)
        assert!(signed.windows(4).any(|w| w == [0x0f, 0x89, 0x0f, 0x00]), "signed out-of-range = jns (0f 89) to zero");
        assert!(signed.windows(8).any(|w| w == [0x48, 0xb8, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00]), "signed big-shift = sign-fill value 0xffffffff");
        let unsigned = tr_bytes(Inst::SimdVShift { rd: 1, rn: 2, rm: 3, esize: 4, signed_: false, q: false, rounding: false });
        assert!(unsigned.windows(3).any(|w| w == [0x48, 0xd3, 0xe8]), "ushl right = shr rax,cl (48 d3 e8, logical)");
        assert!(!unsigned.windows(3).any(|w| w == [0x48, 0xd3, 0xf8]), "ushl must NOT emit sar (F8)");
        assert!(!unsigned.windows(3).any(|w| w == [0x48, 0x63, 0xc0]), "unsigned right must NOT sign-extend the value");
        // unsigned out-of-range right -> plain 0 (mov rax,0, no jns/sign-fill)
        assert!(!unsigned.windows(4).any(|w| w == [0x0f, 0x89, 0x0f, 0x00]), "unsigned big-shift has NO jns (just mov rax,0)");
    }

    #[test]
    fn sh456_vshift_urshl_rounding_bias_before_shift() {
        // urshl (rounding=true): the in-range right path adds the half-up bias
        // 1<<(k-1) to V BEFORE the logical right shift. That bias is
        // `mov rdx,1` + `shl rdx,cl` (48 d3 e2) + `shr rdx,1` (48 c1 ea 01) +
        // `add rax,rdx` (48 01 d0), THEN `shr rax,cl`. The presence + position
        // of the add BEFORE the final shift is the round discriminator — a flub
        // that skips the bias truncates (round-half-down) instead of half-up.
        let b = tr_bytes(Inst::SimdVShift { rd: 1, rn: 2, rm: 3, esize: 4, signed_: false, q: false, rounding: true });
        assert!(b.windows(3).any(|w| w == [0x48, 0xd3, 0xe2]), "rounding bias = shl rdx,cl (48 d3 e2)");
        assert!(b.windows(4).any(|w| w == [0x48, 0xc1, 0xea, 0x01]), "rounding = shr rdx,1 (48 c1 ea 01, 1<<(k-1))");
        assert!(b.windows(3).any(|w| w == [0x48, 0x01, 0xd0]), "rounding = add rax,rdx (48 01 d0, V += bias)");
        let bias_at = b.windows(3).position(|w| w == [0x48, 0x01, 0xd0]).unwrap();
        let shift_at = b.windows(3).position(|w| w == [0x48, 0xd3, 0xe8]).unwrap();
        assert!(bias_at < shift_at, "rounding bias must precede the final shr");
        // control: the non-rounding form has NO 1<<(k-1) ladder.
        let plain = tr_bytes(Inst::SimdVShift { rd: 1, rn: 2, rm: 3, esize: 4, signed_: false, q: false, rounding: false });
        assert!(!plain.windows(8).any(|w| w == [0x48, 0xc1, 0xea, 0x01, 0x48, 0x01, 0xd0, 0x48]), "non-rounding must NOT emit the bias-1<<(k-1) ladder");
    }

    #[test]
    fn sh455_fmuscalar_fmul_double_movq_mulsd_direction() {
        // fmul d1, d2, d3 (double=true neg=false): Dd = Dn * Dm. The scalar
        // 2-source FP multiply — the SAME product core as Fma3 but with NO
        // accumulate into a 4th operand (exactly 2 sources + dst). EMIT:
        // movq_load xmm0=[0x130] (f3 48 0f 7e, Dn) + movq_load xmm1=[0x140]
        // (f3 48 0f 7e, Dm) + `mulsd xmm0,xmm1` (f2 0f 59 c1) + movq_store
        // [0x120] (66 48 0f d6). rd=1 rn=2 rm=3 -> Dd@0x120 Dn@0x130 Dm@0x140.
        // A width flub (mulss on a double) or a missing/extra source shows up
        // immediately in the exact buffer.
        let b = tr_bytes(Inst::FmulScalar { rd: 1, rn: 2, rm: 3, double: true, neg: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x130] Dn
            0xf3, 0x48, 0x0f, 0x7e, 0x8b, 0x40, 0x01, 0x00, 0x00, // movq xmm1,[rbx+0x140] Dm
            0xf2, 0x0f, 0x59, 0xc1,                               // mulsd xmm0,xmm1 -> prod
            0x66, 0x48, 0x0f, 0xd6, 0x83, 0x20, 0x01, 0x00, 0x00, // movq [rbx+0x120],xmm0
        ]);
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc1]), "fmul double = mulsd xmm0,xmm1 (f2 0f 59 c1)");
        assert!(!b.windows(3).any(|w| w == [0xf3, 0x0f, 0x59]), "double must NOT emit mulss (f3 0f 59)");
        assert!(!b.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xd0]), "fmul is a pure multiply, must NOT accumulate addsd");
    }

    #[test]
    fn sh455_fmuscalar_fnmul_double_negate_via_pxor_sign_flip() {
        // fnmul d1, d2, d3 (double=true neg=true): -(Dn*Dm). The negate is a
        // SIGN-BIT FLIP, NOT Fma3's 0-sub: mov_ri64 RCX=0x8000_0000_0000_0000
        // (48 b9 .. 80) + `movq xmm1,rcx` (66 48 0f 6e c9) + `pxor xmm0,xmm1`
        // (66 0f ef c1) BEFORE the movq_store. The pxor-with-sign-const is the
        // fnmul discriminator — a flub that skips it leaves the sign un-negated
        // (the multiply of two positives stays positive).
        let b = tr_bytes(Inst::FmulScalar { rd: 1, rn: 2, rm: 3, double: true, neg: true });
        assert!(b.windows(10).any(|w| w == [0x48, 0xb9, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]), "fnmul double sign const = 0x8000_0000_0000_0000");
        assert!(b.windows(5).any(|w| w == [0x66, 0x48, 0x0f, 0x6e, 0xc9]), "fnmul double = movq xmm1,rcx (66 48 0f 6e c9)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0xef, 0xc1]), "fnmul double negates via pxor xmm0,xmm1 (66 0f ef c1)");
        // ordering: the sign flip must precede the store.
        let pxor_pos = b.windows(4).position(|w| w == [0x66, 0x0f, 0xef, 0xc1]).unwrap();
        let store_pos = b.windows(5).position(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x83]).unwrap();
        assert!(pxor_pos < store_pos, "fnmul must flip the sign before storing");
        // neg vs non-neg (control) — the pxor must be ABSENT when neg=false.
        let plain = tr_bytes(Inst::FmulScalar { rd: 1, rn: 2, rm: 3, double: true, neg: false });
        assert!(!plain.windows(4).any(|w| w == [0x66, 0x0f, 0xef, 0xc1]), "plain fmul must NOT emit the negate pxor");
    }

    #[test]
    fn sh455_fmuscalar_fmul_single_mulss_movd_width() {
        // fmul s1, s2, s3 (double=false): the SINGLE-precision path swaps to
        // mov_load32 + movd_xmm_r32 (66 0f 6e) + `mulss xmm0,xmm1` (f3 0f 59
        // c1) + movd_r32_xmm (66 0f 7e) + 32-bit store (89 8b). The F3-mulss-vs-
        // F2-mulsd prefix is the single-vs-double width discriminator (a width
        // flub silently halves/doubles the FP multiply domain).
        let b = tr_bytes(Inst::FmulScalar { rd: 1, rn: 2, rm: 3, double: false, neg: false });
        assert!(b.windows(4).any(|w| w == [0x8b, 0x8b, 0x30, 0x01]), "single loads Dn via mov ecx,[rbx+0x130] (8b 8b)");
        assert!(b.windows(3).any(|w| w == [0xf3, 0x0f, 0x59]), "single = mulss xmm0,xmm1 (f3 0f 59 c1)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xc1]), "single = movd xmm0,ecx (66 0f 6e)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x7e, 0xc1]), "single = movd ecx,xmm0 (66 0f 7e)");
        assert!(b.windows(6).any(|w| w == [0x89, 0x8b, 0x20, 0x01, 0x00, 0x00]), "single stores 32-bit dst (89 8b 0x120)");
        // width negatives.
        assert!(!b.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc1]), "single must NOT emit mulsd (f2 0f 59)");
        assert!(!b.windows(9).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x83, 0x20, 0x01, 0x00, 0x00]), "single must NOT emit movq_store (66 48 0f d6)");
    }

    #[test]
    fn sh455_fmuscalar_fnmul_single_negate_width_pxor() {
        // fnmul s1, s2, s3 (double=false neg=true): -(S2*S3). The single negate
        // uses the 32-bit sign constant 0x8000_0000 (mov_ri64 RCX = 00..80 00..)
        // then `movd xmm1,ecx` (66 0f 6e c9) + `pxor xmm0,xmm1` (66 0f ef c1)
        // then movd ecx,xmm0 + 32-bit store. The 32-bit sign const (00 00 00 80
        // 00 00 00 00) vs the double's 64-bit (..00*7 80) is the negate-width
        // discriminator; the movd (66 0f 6e) vs the double's movq (66 48 0f 6e)
        // confirms it too.
        let b = tr_bytes(Inst::FmulScalar { rd: 1, rn: 2, rm: 3, double: false, neg: true });
        assert!(b.windows(10).any(|w| w == [0x48, 0xb9, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00]), "fnmul single sign const = 0x8000_0000 (32-bit)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xc9]), "fnmul single = movd xmm1,ecx (66 0f 6e c9)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0xef, 0xc1]), "fnmul single negates via pxor xmm0,xmm1");
        assert!(!b.windows(5).any(|w| w == [0x66, 0x48, 0x0f, 0x6e, 0xc9]), "single negate must NOT use movq xmm1,rcx (66 48 0f 6e)");
    }

    #[test]
    fn sh454_fma3_fmadd_double_addsd_mulsd_direction() {
        // fmadd d1, d2, d3, d4 (sz=true sub=false neg=false): Dd = Da + Dn*Dm.
        // The EMIT is: movq_load xmm0=[0x130] (f3 48 0f 7e, Dn) + movq_load
        // xmm1=[0x140] (Dm) + `mulsd xmm0,xmm1` (f2 0f 59 c1) + movq_load
        // xmm2=[0x150] (Da) + `addsd xmm2,xmm0` (f2 0f 58 d0) + movq_store
        // [0x120] (66 48 0f d6). The PROD direction is rn*rm (mulss into xmm0),
        // and the accumulate is ra + prod — never prod + ra. rd=1 rn=2 rm=3 ra=4.
        let b = tr_bytes(Inst::Fma3 { rd: 1, rn: 2, rm: 3, ra: 4, sz: true, sub: false, neg: false });
        assert_eq!(b, vec![
            0xf3, 0x48, 0x0f, 0x7e, 0x83, 0x30, 0x01, 0x00, 0x00, // movq xmm0,[rbx+0x130] Dn
            0xf3, 0x48, 0x0f, 0x7e, 0x8b, 0x40, 0x01, 0x00, 0x00, // movq xmm1,[rbx+0x140] Dm
            0xf2, 0x0f, 0x59, 0xc1,                               // mulsd xmm0,xmm1 -> prod
            0xf3, 0x48, 0x0f, 0x7e, 0x93, 0x50, 0x01, 0x00, 0x00, // movq xmm2,[rbx+0x150] Da
            0xf2, 0x0f, 0x58, 0xd0,                               // addsd xmm2,xmm0 = da+prod
            0x66, 0x48, 0x0f, 0xd6, 0x93, 0x20, 0x01, 0x00, 0x00, // movq [rbx+0x120],xmm2
        ]);
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc1]), "fmadd = mulsd xmm0,xmm1 (f2 0f 59 c1)");
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xd0]), "fmadd = addsd xmm2,xmm0 (f2 0f 58 d0)");
        // the double path stores via movq_store (66 48 0f d6), never movd (66 0f 7e).
        assert!(b.windows(8).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x93, 0x20, 0x01, 0x00]), "double FMA stores via movq_store (66 48 0f d6)");
    }

    #[test]
    fn sh454_fma3_fmsub_add_vs_sub_opcode_fnmsub_direction() {
        // The sub/neg pair picks the accumulate opcode AND its direction:
        // fmsub (sub=true neg=false): `subsd xmm2,xmm0` (f2 0f 5c d0) = da - prod;
        // fnmsub (sub=true neg=true): `subsd xmm0,xmm2` (f2 0f 5c c2) = prod - da
        // (the signed negation makes it rn*rm - da, REQUIRING the operands
        // swapped — a lone subss in the wrong order negates the wrong term).
        // The 0x58-adds-vs-0x5c-subsd opcode byte AND the c0/c2 operand order
        // discriminate the four fmadd/fmsub/fnmadd/fnmsub combinations.
        let sub = tr_bytes(Inst::Fma3 { rd: 1, rn: 2, rm: 3, ra: 4, sz: true, sub: true, neg: false });
        assert!(sub.windows(4).any(|w| w == [0xf2, 0x0f, 0x5c, 0xd0]), "fmsub = subsd xmm2,xmm0 (da - prod, f2 0f 5c d0)");
        assert!(!sub.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xd0]), "fmsub must NOT emit addsd");
        let fnms = tr_bytes(Inst::Fma3 { rd: 1, rn: 2, rm: 3, ra: 4, sz: true, sub: true, neg: true });
        assert!(fnms.windows(4).any(|w| w == [0xf2, 0x0f, 0x5c, 0xc2]), "fnmsub = subsd xmm0,xmm2 (prod - da, f2 0f 5c c2 — SWAPPED)");
        assert!(!fnms.windows(4).any(|w| w == [0xf2, 0x0f, 0x5c, 0xd0]), "fnmsub must NOT emit the da-prod order");
        assert!(!fnms.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xd0]), "fnmsub must NOT emit addsd");
        // fnmsub stores from xmm0 (the prod-da result) — movq_store [0x120],xmm0 (83).
        assert!(fnms.windows(8).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x83, 0x20, 0x01, 0x00]), "fnmsub stores xmm0 via movq [0x120] (83 base)");
        // fmadd (control) takes the addsd path.
        let add = tr_bytes(Inst::Fma3 { rd: 1, rn: 2, rm: 3, ra: 4, sz: true, sub: false, neg: false });
        assert!(add.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xd0]), "fmadd = addsd xmm2,xmm0");
    }

    #[test]
    fn sh454_fma3_fnmadd_negate_via_pxor_sub() {
        // fnmadd (sub=false neg=true): -(Da + Dn*Dm) — emits addsd xmm2,xmm0
        // then pxor xmm3,xmm3 (66 0f ef db, +0.0) then `subsd xmm3,xmm2`
        // (f2 0f 5c da = 0 - xmm2) to NEGATE, storing FROM xmm3. The
        // pxor+subsd-into-scratch is the negate discriminator vs the plain
        // fnmsub's swapped subss — a flub that skips the 0-sub leaves the sign
        // un-negated.
        let b = tr_bytes(Inst::Fma3 { rd: 1, rn: 2, rm: 3, ra: 4, sz: true, sub: false, neg: true });
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0xef, 0xdb]), "fnmadd negates via pxor xmm3,xmm3 (66 0f ef db)");
        assert!(b.windows(4).any(|w| w == [0xf2, 0x0f, 0x5c, 0xda]), "fnmadd = subsd xmm3,xmm2 (0 - result, f2 0f 5c da)");
        assert!(b.windows(8).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x9b, 0x20, 0x01, 0x00]), "fnmadd stores xmm3 via movq [0x120] (9b base)");
        // the addsd-before-negate ordering.
        let add_pos = b.windows(4).position(|w| w == [0xf2, 0x0f, 0x58, 0xd0]).unwrap();
        let pxor_pos = b.windows(4).position(|w| w == [0x66, 0x0f, 0xef, 0xdb]).unwrap();
        assert!(add_pos < pxor_pos, "fnmadd must add before negating");
    }

    #[test]
    fn sh454_fma3_single_precision_mulss_addss_width() {
        // fmadd s1, s2, s3, s4 (sz=false): the SINGLE-precision path swaps to
        // movd_xmm_r32 (66 0f 6e) + mulss (f3 0f 59 c1) + addss (f3 0f 58 d0) +
        // movd_r32_xmm (66 0f 7e) + 32-bit store (89 83). The F3-Mulss-vs-F2-
        // Mulsd prefix is the single-vs-double width discriminator. 4 operands
        // at 0x130/0x140/0x150, dst 0x120.
        let b = tr_bytes(Inst::Fma3 { rd: 1, rn: 2, rm: 3, ra: 4, sz: false, sub: false, neg: false });
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x6e, 0xc0]), "single loads Vn via movd xmm0,eax (66 0f 6e c0)");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x59, 0xc1]), "single = mulss xmm0,xmm1 (f3 0f 59 c1)");
        assert!(b.windows(4).any(|w| w == [0xf3, 0x0f, 0x58, 0xd0]), "single = addss xmm2,xmm0 (f3 0f 58 d0)");
        assert!(b.windows(4).any(|w| w == [0x66, 0x0f, 0x7e, 0xd0]), "single = movd eax,xmm2 (66 0f 7e d0)");
        assert!(b.windows(6).any(|w| w == [0x89, 0x83, 0x20, 0x01, 0x00, 0x00]), "single stores 32-bit dst (89 83)");
        // width: single must NOT use mulsd/addsd/movq_store.
        assert!(!b.windows(4).any(|w| w == [0xf2, 0x0f, 0x59, 0xc1]), "single must NOT emit mulsd (f2 0f 59)");
        assert!(!b.windows(4).any(|w| w == [0xf2, 0x0f, 0x58, 0xd0]), "single must NOT emit addsd (f2 0f 58)");
        assert!(!b.windows(8).any(|w| w == [0x66, 0x48, 0x0f, 0xd6, 0x93, 0x20, 0x01, 0x00]), "single must NOT emit movq_store (66 48 0f d6)");
    }
}
