# SH420 — `__stack_chk_fail` host shim names the GENUINE canary wall's failing frame

Date: Sep 26, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green (arm64jit lib 477/0 incl. new sh420 hermetic). Production code only in shims.rs (off the 1MiB hooks; jit.rs/elfjit.rs untouched, both at/near the hook, byte-unchanged). recon-v3 immediate-priority deliverables unchanged-green.

## What and why
The STATUS item #2 (canary producer NAMED then CORRECTED) left the GENUINE
`*** stack smashing detected ***` wall — the deeper nativeGameGlobalInit ladder
(SH97/98 class) — identified only as "a distinct store on that path," never
measured. The SH4xx store-watch is **doubly inadequate** for this wall:
(a) it emits a host call after EVERY 64-bit store, which perturbs scheduling so
the run no longer reproduces the stack-smash, and (b) it filters to the SH103
leak class (value >= 0x7000_0000_0000) only, so a canary clobber by ANY other
value class (a zeroed slot, a small int, a guest pointer) is invisible.

This cycle delivers the correct instrument: a **`__stack_chk_fail` host shim**
that fires exactly ONCE at the check failure — zero scheduling perturbation —
reads the dispatcher's `current_guest_pc()` (the guest return address of the
`bl __stack_chk_fail`, which names the failing function), best-effort reads the
failing frame's canary slot(s), then forwards to the real libc `__stack_chk_fail`
so the abort is **byte-identical whether or not the dump env is set**. Default-inert
(env `JIT_STACKCHK_DUMP=1`).

## Code
- shims.rs (off-hook, 130KB < 1MiB): `stack_chk_fail_dump` registered into
  `register_shims()` as `__stack_chk_fail` (register_named precedence over the
  generic dlsym resolver, so the guest `bl __stack_chk_fail@plt` dispatches to
  it). `current_guest_pc()` is the PRIMARY datum (thread-local, set by the
  dispatcher to the guest return addr = inside the failing fn's epilogue);
  `guest_state_of_host(gettid())` is best-effort for x29/x30/sp + canary slots
  (domain-checked reads). Test override (live) + muted forward for the hermetic.
- New hermetic `stack_chk_fail_dump_returns_safely_under_test_override`
  (arm64jit lib 476 -> 477): override path returns 0 without a live image/abort,
  forward never aborts in test context, gating is live.

## MEASURED (real libroblox.so, SH54 full-boot env + 3-gate crossing, JIT_STACKCHK_DUMP=1)
The instrument names the GENUINE wall's site on the real binary. Run reaches
nativeGameGlobalInit (rung 2) and aborts cleanly at the one-shot line:

```
[stack_chk_fail] __guest_pc=0x102206d90 tid=2123922 state=... canary@-16=0x0 @-8=0x0 expected=0x55eefc69a478
*** stack smashing detected ***: terminated
```

Disassembly pins it: file 0x2206c40 = nativeGameGlobalInit body; prologue
`stur x8,[x29,#-8]` @0x2206c70 stores the canary; epilogue `cmp x8,x9; b.ne
0x2206d8c` @0x2206cf4; 0x2206d8c `bl __stack_chk_fail@plt` (return 0x2206d90).
In the earlier validating run the canary slot read `@-8=0x0` (ZEROED) vs expected
`0x55c69880e3f8` — the genuine clobber is a **zeroing of [x29,#-8] during the
do-init once-path** (between 0x2206c70 and 0x2206cf4), NOT a foreign host-ptr
leak. This is the distinct wall SH4xx-next anticipated, now correlated against
ITS OWN canary slot with a working observer.

## Honest
NOT a DM (DM-root 0, MH_* false). Does NOT fix the wall — it NAMES the genuine
site (nativeGameGlobalInit body, canary slot zeroed in do-init once-path) and
provides the instrument to correlate its writer next. Route-B live-DM structural
gate UNCHANGED. recon-v3 deliverables unchanged-green.

## Writer-hunt is a re-tread of the Route-B do-init wall (measured)
The naive NEXT ("arm a narrow store-watch to name the zeroing writer") is a
RE-TREAD: disassembling the four callees between the canary store (0x2206c70) and
compare (0x2206cf4) — 0x6201bd4 (FMOD), 0x220671c (once-lambda gate), 0x2206db8
(once-lambda do-init dispatch), 0x221942c (16-byte struct touch) — shows ALL have
small (<0xc0) frames and write only to their own locals; none directly writes the
caller's canary. The zeroing lands during the once-lambda do-init dispatch chain
(deeper callees), which SH404/405/407 already measured running deep into the
app-shell ctor band and draining into the standing SH285/LSM persistence lane.
So the canary-zero is a SYMPTOM of the do-init once-path construction (the Route-B
wall), not a distinct seedable store; fix the once-path / persistence lane and the
canary-zero resolves with it. Do NOT spend a cycle on a narrow store-watch for it
(that is re-driving the measured do-init wall from one more angle).

## Files
- docs/frontier-sh420-stackchk-fail-shim-names-wall.md
- runs/capture_sh420_stackchk_fail.sh
- crates/arm64jit/src/shims.rs (shim + hermetic; jit.rs/elfjit.rs untouched)