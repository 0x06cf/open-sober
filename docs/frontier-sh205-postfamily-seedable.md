# Frontier SH205 — the "post-family" fault is a seedable SH116-class singleton, not the migration gate

**Verdict:** SH203's dismissal of the post-family fault (SIGSEGV host-call slot 0x22b0
pthread_mutex_lock, x0=0x28, lr 0x102b53a78) as "the same live-world-build migration gate, 0 direct
callers" was **wrong on both counts**. A new env-gated guest-stack dump proves the caller is
`nativeInitializeNativeFlags` (return-address 0x102320a98) locking `[*(0x10672739b0) + 0x28]` where the
flag-manager global reads 0 — a **fixed `.bss` pointer on the flags page** (the SH116 seedable class),
not a live-DM world-build. The fix patches that read site; the flag-manager lock crash is eliminated
(zero faulting runs show its caller). The ladder overall is NOT deterministic-clean — the residual
SH55/64 run-variable faults are separate, pre-existing sites (see Measurement).

## Discovery: a guest-stack dump (GSDSP) that identifies a crash's *caller*

The existing crash report dumps the guest register file, but for a crash **inside a cross-called host
thunk** (a guest `bl` to a host bridge like pthread_mutex_lock), CpuState only shows `guestpc` = the
host-call slot and `x30` = the *leaf's* own return target — **not** the guest caller's return address.
The caller sits on the guest stack: a leaf prologue `stp x29,x30,[sp,#-16]!` places caller-ra at
`[sp + 8]`.

Added `JIT_GUEST_STACK_DUMP=1` (default-inert, elfjit.rs crash handler): when a fault's REX-matched
CpuState is real, dump the guest words at `[sp..sp+56]`, tagging words that fall in the in-image guest
text range `[0x100000000, 0x120000000)` as `GUEST(..)`. This turns "crashes at out-of-image slot X"
into "the caller bl-site is guest 0x102320a98".

## The measured caller

```
[SIGSEGV] guestpc=0x7f00000022b0   (host-call slot 1110 = pthread_mutex_lock bridge)
  x0=0x28 x29=sp lr(x30)=0x102b53a78   (leaf ret target)
  GSDSP[+8=GUEST(0x102320a98) ...]     (the CALLER's return address)
```

`bl 0x2b53a68` at file 0x2320a94, i.e. the shared lock helper is called from **file 0x2320a94**
(guest 0x102320a94):

```asm
; nativeInitializeNativeFlags, file 0x2320a24..0x2320a94
2320a30: ldr x8,[x8,#2480]     ; x8 = *(0x10672739b0)  <- flag-manager global, = 0 headlessly
...
2320a6c: add x0, x0, #0x28      ; lock target = &obj+0x28
2320a94: bl 2b53a68             ; pthread_mutex_lock(x0) -> x0=0x28 faults
```

SH116 had already patched the *sibling* read site (file 0x2320710, which reads the same global) because
that path's `.bss` page is unmapped at harness seed time (mprotect RW → ENOMEM). But this run advanced
**one site deeper** into the SAME function to a second read site (0x2320a24) that SH116 did not cover —
so SH203 counted the shared *helper's* logic and concluded "0 direct callers / live-world-build gate".
In fact the helper has **500+ direct `bl` callers**; only this one was reached with a NULL object.

## Fix (SH116b)

Mirror SH116's proven **code-patch** mechanism (a store to the `.bss` global at seed time is impossible —
page unmapped until the engine boots). Patch file 0x2320a24's load slot (adrp+ldr → movz/movk x8) to
materialize a **low fixed zeroed page** (mmap 0x60000000, 2 slots = fits the low-32-bit address range)
whose `+0x28` is all-zero = a valid `PTHREAD_MUTEX_INITIALIZER`. The lock then targets a real mapped
zeroed page instead of `&0+0x28`.

- `routeb_patch_nativeinit_flagmanager()` — idempotent, shift-guarded (refuses if slot0 != `adrp x8,7273000`), non-vtable-widening, gated under `JIT_SH115_SINGLETON_PATCH` (same chain as SH116/117/119/200).
- `sh116b_flagmanager_words(obj)` — pure movz/movk x8 hw0/hw1 encoding, hermetic-tested.
- `routeb_flagmanager_obj()` — leaks the low mapped zeroed page (MAP_FIXED_NOREPLACE, falls back to MAP_FIXED).

## Measurement (HONEST — corrected after further runs)

- **Flag-manager lock site: FIXED.** Before: that exact site (caller ra 0x102320a98, `[*(0x10672739b0)+0x28]`) faulted run-variably. After: **zero** faulting runs show GUEST 0x102320a98 in the GSDSP; the flag-lock crash is gone.
- **Ladder overall: NOT deterministic-clean.** A corrected 10-run characterization with the slot fix = **8 clean / 2 fault**. The 2 residual faults are **different, pre-existing sites** in the SH55/64 run-variable flake family — `[0x104c393f0]` (system-dialog handler region) and `[0x10284ce54]` (the do-init `__call_once` `blr x2`, SH196/SH55 territory) — NOT the flag-manager site this patch targets. The ladder's post-SH116b clean-through is *higher* than before but still run-variable; earlier "10/10 clean" was a lucky streak, retracted.
- **Default path** (env off, no scale): clean, SH116b not fired — unregressed.
- Workspace green (564/0); arm64jit example 65/0 (incl. `sh116b_flagmanager_words_roundtrip_and_real_site_guard`).

## Why it matters / next

- Corrects the record: the post-family fault is **not** the concluded Route-B migration gate. It is a
  per-boot `.bss` flag-manager pointer that a code-patch materializes — a real fix for one specific
  crash the earlier SH116 sibling did not cover. Do NOT overstate it to "ladder deterministic"; the
  remaining SH55/64 run-variable faults (0x104c393f0 / 0x10284ce54) are separate, pre-existing and
  outside this patch's scope.

## Repro

```
env JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
    JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_SETWORLDBUILD=1 \
    JIT_ROUTEB_V2_ONDEMAND=1 \
    ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
    --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent
expect: SH116b patched ... ; SendAppEventOnAppReady returned Ok ; EXIT 124 (no SIGSEGV)
```