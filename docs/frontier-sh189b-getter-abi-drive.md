# SH189b — ScreenGui + StarterGui class-register once-GETTERS (find + ABI + drive recipe)

Task: find the once-wrapped CALLER getters for ScreenGui (body 0x10201f4f0) and StarterGui
(body 0x10202014c), verify ABI, explain the null-deref, deliver drive recipe.
Binary: libroblox.so (ARM64), guest = file vaddr + 0x100000000. .text filev 0x1d95980, len 0x4540104.
Read-only; no harness run (loop owns it).

================================================================
1. THE GETTERS (concrete, file + guest)
================================================================
PlayerGui reference (verified working headless): getter 0x201fce0 / 0x10201fce0
  - cached [0x6c97f28], latch [0x6c97f30]=0x106c97f30, source-builder 0x201e95c (nested latch 0x6c883a0),
    register body 0x201fda4, desc object 0x106c980b8.

SCREENGUI  -> CALLER GETTER = 0x201f42c / 0x10201f42c
  - prolog at 0x201f42c (sub sp,#0x40; stp x29/x30; .. ; stack canary x19=0x67d1000[+1776])
  - cached-desc slot check: adrp 6c98000 + ldr x0,[x8,#0xa20] @0x201f450 (guest 0x106c980a20)
  - once-latch bucket: adrp 6c98000 + add x8,#0xa28 @0x201f45c/0x201f460 ldarb (guest 0x106c980a28)
  - source-descriptor builder: bl 0x1e639ac @0x201f4ac (guest 0x101e639ac); its own nested
    latch cached [0x6c96860] / guard 0x6c96868 (guest 0x106c96868), checked inside 0x1e639ac
  - register BODY: bl 0x201f4f0 @0x201f4c4 (guest 0x10201f4f0) — the once-BODY, NOT self-guarding
  - classid 0x1b87 (mov w3), name 0x54807d 'ScreenGui'
  - desc OBJECT: 0x6c98000+0xa40 = 0x6c980a40 (guest 0x106c980a40) — matches recon
  - single bl-caller of body 0x201f4f0: the getter @0x201f4c4. (scan confirmed).
  - ABI: PARAMETERLESS ✓ (never reads incoming x0/x1; x1=sp set locally @0x201f4c0;
    x0 for the body comes from the return of source-builder 0x1e639ac). Callable with [0;8].

STARTERGUI -> CALLER GETTER = 0x2020120 / 0x102020120
  - prolog @0x2020120 (sub sp,#0x80; stp x29/x30; stp x20,x19). THE recon "body 0x202014c"
    is mid-function (adrp name 0x5168ef @0x202014c), i.e. it is NOT a self-guarding body —
    0x2020120 is the once-wrapped getter, classid 0x892 set inline @0x2020158.
  - no upfront cached-desc check (unlike PlayerGui); instead it ALWAYS calls the register
    helper first, then a once-guard protects the descriptor/type wiring:
    - bl 0x5fb225c (register helper) @0x2020180  — unconditional
    - ldarb w8,[x19] @0x2020184 where x19 = 0x6c9a000+0x710 = LATCH 0x6c9a710 (guest 0x106c9a710)
    - tbz -> 0x202019c once-path: bl 0x284ce54 (lock), write vtable [0x6c9a708]=0x664db10
      (guest [0x106c9a708]=0x10664db10), bl 0x1dcf7dc (desc build at 0x6c9b060), unlock via
      0x284cf5c (tail-branch @0x20201d8).
  - desc OBJECT: 0x6c9a710+0x950 = 0x6c9b060 (guest 0x106c9b060); vt-family slot
    [0x106c9a708] = 0x10664db10
  - classid 0x892 (adrp name StarterGui 0x5168ef + w3=0x892 @0x2020158)
  - ABI: NOT parameterless-in-the-PlayerGui/mirror sense. It FORWARDS caller x0->regstack x1
    and caller x1->x7 into the register helper 0x5fb225c (mov x7,x1 @0x202013c; mov x1,x0
    @0x2020140) BEFORE the desc-computation. So an all-zero call feeds x1=source=0 into a
    helper that copies source descriptor members (add x3,x21,#0x70 @0x5fb22d8 etc.) -> repeat
    null-deref class. Verdict (2): ScreenGui getter is parameterless-safe; StarterGui getter
    is NOT safely all-zero — it wants a REAL source descriptor in x0 (the object that the
    register-helper copies FROM).

================================================================
2. ABI SUMMARY
================================================================
- ScreenGui getter 0x10201f42c: parameterless. Internally builds its own source via
  0x101e639ac. SAFE to run_guest_callback([0;8]). Mirrors PlayerGui exactly.
- StarterGui getter 0x102020120: expects x0 = valid source class-descriptor (its own
  source-builder is NOT called inside — the getter forwards x0/x1 straight to 0x105fb225c).
  All-zero x0 likely reproduces the same helper null-deref. To drive it headlessly you must
  first materialize a valid source descriptor (see caveat in §4).

================================================================
3. THE NULL-DEREF (ScreenGui body driven directly)
================================================================
The observed SIGSEGV at guestpc 0x101db7e38 = file 0x1db7e38 is `str x0,[x22,#8]` inside
0x1db7e04 (a "build one class-descriptor member" helper, callers incl. 0x5fb22b0 from the
register helper 0x5fb225c and 0x1dc44cc/0x1dc4d8c from the deeper source-copy path).
x22 = x0 of 0x1db7e04 = a descriptor sub-object pointer. When the register BODY 0x201f4f0 is
run directly with all-zero args, the source/sub-descriptor pointer chain it hands downward is
NULL/empty (the body reads x0=0 for source and forwards it), so the helper walks a NULL member
and the store-through at 1db7e38 faults.
The GETTER avoids this because its once-setup FIRST calls the class's source-descriptor builder
(0x1e639ac for ScreenGui) to materialize a real source descriptor (nested latch 0x106c96868),
THEN passes that valid source + desc object into the body. I.e. yes: the body alone derefs a
member that the getter's source-builder initializes first. Driving the getter supplies it.

================================================================
4. CORRECTED DRIVE RECIPE (for the loop)
================================================================
SCREENGUI — safe, mirrors PlayerGui:
1. Clear latches (both chained, mirroring PlayerGui's 0x106c97f30+0x106c883a0):
     *(0x106c980a28) = 0   (getter once-guard)
     *(0x106c96868) = 0   (nested source-builder 0x101e639ac guard)
   optionally zero cached (0x106c980a20)=0 and source-cached (0x106c96860)=0 so the register
   path re-runs from scratch.
2. run_guest_callback(0x10201f42c, [0;8], sp)   // parameterless, expect ret x0 = 0x106c980a40
3. Success probes:
     [0x106c980a20] == 0x106c980a40          (cached desc)
     [0x106c980a40]      == 0x1067a6230      (desc vtable, written by 0x1db7e30: x8=0x67a6230)
     [0x106c980a40+0x230]== 0x106649c98      (ScreenGui vt-family slot — recon's 0x106649c98 ✓;
                                             body writes 0x6649c98 @0x201f55c str [x19,#560])
     class-desc counter [0x106dca0e28] note: stayed 0 for PlayerGui too (known recon caveat —
     weaker probe, use vt-family/vtable as the strong proof).

STARTERGUI — CAUTION (ABI not all-zero-safe):
- Getter 0x102020120, latch to clear 0x106c9a710, desc object 0x106c9b060,
  vt-family slot [0x106c9a708] = 0x10664db10.
- Because the getter forwards x0 (source) to the register helper, do NOT drive it with all-zero
  x0. The loop needs to first build a valid source descriptor for it. Recon's "body" 0x10202014c
  is mid-getter and is NOT a callable body (0 bl-callers). If a parameterless path is required,
  the cleaner analogue is to drive a source-builder that materializes the StarterGui source
  (the PlayerGui/ScreenGui analogue)
  — exact StarterGui source-builder not isolated in this pass (0x102020120 builds desc via
  0x101dcf7dc after the once-guard, but its SOURCE must be supplied in x0). Flag for loop:
  verify with a real source pointer, or extend the recon to isolate StarterGui's source-builder
  before driving, else expect the helper null-deref again.

================================================================
Notes / evidence
================================================================
- ScreenGui body 0x201f4f0 single caller = getter @0x201f4c4 (bl). StarterGui "body" 0x202014c:
  0 bl-callers (it is mid-function of 0x2020120).
- register helper 0x5fb225c (793 bl-callers) is the shared engine: takes x0=dest desc,
  x1=source, classid/typeid + stack args; calls 0x1db7e04 (source 0x5fb22b0) and 0x1dc4744
  value-copies FROM source (x21) — hence the null-deref when source is empty.
- Guard functions: 0x284ce54=acquire/lock, 0x284cf5c=release, 0x284cfbc=retry — same
  once-idiom as SH156 (recognized by recon).
- Session was read-only: no git/cargo/test writes, no harness runs.