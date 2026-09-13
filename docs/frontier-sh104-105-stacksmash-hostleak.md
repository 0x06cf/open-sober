# SH104–SH105 — stack-smash gate analysis (host-pointer leak into guest stack canary)

## Where we are
SH100/101/102/103 cleared the globalinit walker + URL-log + keyed-registry gates. The
ladder then advances into the scheduler/cond path and hits a NEW stack-smash:
`*** stack smashing detected ***: terminated` (glibc `__stack_chk_fail`, guestpc=0x0
on a worker thread).

## SH104 negative result (cond_broadcast ruled out)
An initial recon (deleg_73834a52) blamed `pthread_cond_broadcast` running as raw glibc
over a bionic cond at 0x10683a168, writing a 48-byte gadget near/over the guest
`__stack_chk_guard` .bss. We added bionic-safe no-op shims for cond_broadcast/signal/
destroy + mutex_destroy and instrumented (SH104_DEBUG): **the no-op ran 9x and the
stack-smash persisted at the SAME site.** cond_broadcast was NOT the writer. The change
was reverted (no fix, only cruft).

## SH105 probe — the real mechanism
Instrumented the failing frame's canary compare (fn epilogue restoring the canary):
```
[SH105] pc=0x102dae364 x22=0x559095543960 [x22]=guard=0x559095543960
        x29=0x559097645e20 [x29-16]=stored=0x559097645e50 match=false
```
i.e. the frame's **stored canary slot [x29-16] (guest stack) was overwritten with a
HOST pointer** (0x559097645e50 is a host-ASLR VA, and it sits just above this frame's
own x29). The guard slot itself ([x22]) is intact. So a HOST pointer is being written
into the GUEST STACK CANARY SLOT of a scheduler-frame by some host-call excursion —
the same host-pointer-into-guest-state leak class as SH103's x20 (`ldr x24,[x20]` where
x20 held 0x564900000001), now manifesting higher in the stack where it clobbers the
saved canary.

This is a genuine JIT **hosting** bug: a host call (or host-call BL crossing) leaks a
host-register pointer into a guest callee-saved/stack slot. It is NOT a specific
unshimmed struct call (SH98 class), so per-call struct shims do not fix it; forcing the
canary check would be UNSAFE (it's active memory corruption, not a false positive).

## Recon refinement (deleg_5e7fec4d)
Closer analysis corrects the value reading: `x29=0x559097645e20`, stored
`[x29-16]=0x559097645e50 = x29+0x30` — BOTH in the process 0x55 region, because the
guest STACK is a host-malloc'd buffer. So the canary slot holds a guest-owned
self-stack pointer, not a foreign host-execution pointer (a true foreign pointer would
be 0x7f.../0x7fff...). Verified in code: the host-call bridge (jit.rs ~2227-2237) writes
only s.x[0] (return) and s.px (pc); guest SP (x31), x1..x18 are untouched by the bridge
(no live-range syncing from host); memmove/memcmp bind RAW glibc guest-to-guest (no
foreign bytes); the routeb dispatcher hooks (jit.rs ~2450+) overwrite guest x0/x19 at
block boundaries. Net: the writer is guest code storing a self-stack pointer into its
canary slot, downstream of a host-call RETURN value leaking into guest x0 (the SH103
class — e.g. memmove returns dst; a lengthy out-pointer or a shim returning its guest-buf
address that guest code then stores onto its frame). The exact guest STR instruction is
NOT yet named — the JIT_TRACE tail was deleted before the recon ran. Fix direction per
recon: (a) a bridge sanitize for the string-helper-return class (if a non-pointer
libc helper's return >= 0x100000000, force 0 — the SH97 safe_cstr_len guard generalized
to the bridge), OR (b) trace the exact hostcall whose return lands at the canary slot
first. Blanket-zeroing is WRONG (guest stack pointers are also 0x55).

## Standing blocker
The stack-smash persists because the underlying host-pointer-into-guest-state leak
surfaced at the scheduler/cond frame's canary. Root fix is bridge-level: the JIT
host-call dispatch (jit.rs host_call_at + the BL/BRGOT/PLT crossing) must never leave a
host address in a guest register/stack slot. Tracked; next recon focuses on pinning the
exact writer (which host call / bridge path writes 0x5590... into [x29-16]).