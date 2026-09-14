# SH159 — Route-B: the AppBridgeV2 governor 0x102e9fa84 now EXECUTES (first time); union-init guard-globals zeroed

## Outcome
The do-init body's dispatch into the AppBridgeV2 governor (0x102e9fa84) now
actually FUNCTIONS: region-watch confirms the governor is entered (0x102e9fa84,
0x102e9faf0, 0x102e9fb10) and runs into its MODERN appendix (nativeAppBridge
V2InitWithParams param sub-reader chain). This is the deepest any run has driven
the engine — the SH156-identified frontier marker is now reachable. Opt-in
(JIT_ROUTEB_DM_SEED=1); default path unregressed (EXIT 0, 24 real task frames,
0 SIGSEGV/SIGABRT).

## What was wrong (empirically corrected two recon premises)
1. **NOT "zero data relocations"** — the .so's .rela.dyn is ~2MB of ANDROID_RELA
   PACKED relocations (readelf -r cannot decode that format → an earlier subagent
   falsely concluded vtables are statically all-zero). Our loader DOES apply them:
   runtime probe `[0x1063a3428]=0x102e9fa84` (vt[+0x18]) and `[0x106a705e8]=
   0x1063a3410` (singleton vtable) — correctly relocated, so NO hand-seeding of the
   governor vtable slot is needed (and doing so would clobber the correct value).
2. **The governor was never reached because the body SOFT-DIVERGED in its union-init
   (0x2366694) sub-constructor** — not because the dispatch was wrong. The union
   init's first sub-constructor 0x2366848 guards on globals G=[0x6a63da0] and
   W=[0x6a63d70]. At runtime both hold non-NULL HOST-heap pointers (0x55d8...,
   i.e. not live guest objects), so the guard is taken and the sub-constructor
   treats that pointer as `this`, garbage virtual-dispatches, and never returns to
   the body's continuation (0x23effac) → the governor dispatch blr never ran.
   (Recon deleg_0eff24ca predicted the guard held rodata feature-flag name strings;
   probed values were host pointers, but either way non-NULL = dead-end.)

## The fix
Zero both guard globals (guest rw- .bss, in the mapped region, direct host write):
- `[0x106a63da0] = 0`  (G)
- `[0x106a63d70] = 0`  (W)
With both 0, all 11 union-init sub-constructors collapse at their `cbz` guard and
0x2366694 returns at 0x2366810 → the body's `blr x8` at 0x23effbc reaches the
governor 0x102e9fa84.

Plus an (insurance-only) coherent impl seed: wrapper[+0x20] = [0x106a70608] = a
live guest impl buffer; impl[+0x408] = a DISPATCH object whose all-benign-leaf vt
makes the MODERN sub-dispatch (gov 0x2e9fb54 blr) resolvable. Runtime shows this
did not change the observed fault (identical with/without), so it is inert for this
gate; kept as harmless opt-in insurance.

## NEXT GATE (honest)
With the governor entered, it runs into its MODERN appendix and faults at
**guest 0x102e9fb10** (`ldrb w8,[sp,#408]`) right after `bl 23c1504` returns —
SIGSEGV fault=0x0, x0=0x0. 0x23c1504 is `nativeAppBridgeV2InitWithParams`'s
param sub-reader (reads x5 param union). The appendix at 0x2e9faf8 passes x5=
sp+0x198 (the stack union). Either the union is uninitialized (values feed a NULL
read) or 23c1504 expects a real param-union content the harness doesn't populate.
Next work: give the appendix a coherent param union (or seed the specific field
23c1504 reads), so the governor proceeds to `bl 0x258c6e4 nativeAppBridgeV2StartAppWithParams`.

## Hermetic tests (elfjit.rs, +0 this commit — probes only; no behavior change to code paths gated by the fix)
The SH159 regression surface is the runtime governor-reach region-watch, not a
unit test; the two guard-global writes are plain opt-in data seeds. Elfjit example
61/0 still green.

## Repro
```
timeout 100 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_DM_SEED=1 JIT_REGION_WATCH=0x102e9fa00-0x102e9fd00 ./target/debug/examples/elfjit ... --v2boot ... 
```
(runs/sh159-seed.txt: EXIT 134, but region-watch shows the governor entered;
runs/sh159-default.txt: EXIT 0 / 24 frames / 0 faults, default unregressed.)