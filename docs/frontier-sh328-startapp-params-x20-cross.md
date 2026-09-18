# Frontier SH328 — supply fn 0x25f52b4's x20 (params obj); the 0x25f5328 ldr wall CROSSED, gate advances to a real-AppStarted live-member wall

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh328` (new, real-image guard) beside
`sh327`/`sh325`/`sh324`. elfjit examples 152/0 (was 151); arm64jit lib 408/0; workspace green.
Route-B live-DM gate UNCHANGED at the top level (no make_shared<DataModel>), but the app-start
fencepost ADVANCED one more full step past SH327's endpoint. This cycle implemented the SH327-file
declared NEXT forward ("supply the params-obj +0x140 member") and it worked: the `ldr w3,[x20,#320]`
@0x25f5328 wall SH327 left (fault=0x140) is now CROSSED, fn 0x25f52b4 completes, and the fault
moved to 0x25f5050 on a real-AppStarted live member. Deterministic 2/2.

## 1. The gate SH328 clears (SH327's declared NEXT)

SH327 pinned: after `[appstart+24]` (AppStarted factory construction) is forced deterministic, the
app-start continuation at fn **0x25f52b4** (nativeAppBridgeV2StartAppWithParams body) immediately
faulted at `ldr w3,[x20,#320]` @0x25f5328 (fault=0x140) because x20=x0 (the per-caller V2StartApp
params object, entered via the 0x258b144 vtable-slot-248 virtual dispatch) is **0** headlessly.

Disassembly of fn 0x25f52b4 (real libroblox.so, robbox):
- entry `mov x21,x1; mov x20,x0` (x20=params obj, x21=[x1]);
- `bl 0x233bf20` nativeAppBridgeAppStart -> `ldr x19,[sp,#328]` (**AppStarted out-param, REAL heap obj**, verified 0x5588b4471ae0);
- version gate `ldr x0,[0x6a701c8]; cmp #6...` -> appendix `ldr w3,[x20,#320]` @0x25f5328 (FAULT) / `ldr w8,[x20,#320]` @0x25f533c;
- params-copy `add x1,x20,#0x148` @0x25f5350, then `mov x0,x20; bl 0x25f55b0` (reads [x20]..#0x110 window), then `[x19]vt+16` dispatch @0x25f5460.

`x20` is **only read** (never written by guest) across a ~0x300-byte window (offsets #320,#328,
#0x18..#0x110, #0x148) — it does not need a constructed object, only a READABLE base that yields
benign zeros.

## 2. The fix (`routeb_patch_startapp_params_x20`)

Patch `mov x20,x0` @0x25f52d8 (0xaa0003f4) -> `adrp x20,0x107334000` (0xf00269f4), a **single
instruction** of identical footprint. 0x107334000 is the JIT's reserved *guarded RW tail*
(402MB @ 0x107334000, already mapped by the boot `[tail] reserved ... RW tail @0x107334000`; the
same region backs the SH156-seeded thread-mutex/session objects). Page-base offset 0, so adrp alone
yields the exact base. All `[x20,#imm]` reads (<= +0x140) land inside the mapped zeroed tail ->
benign empty-string/zero reads. Idempotent (before==want short-circuit), page RW->write->RX,
`block_cache_drop_region(0x1025f52b4,0x1025f5550)`. Gated on JIT_ROUTEB_DM_SEED/DMFORCE (inert on
plain runs).

## 3. Measured (real libroblox.so, robbox, 2/2 deterministic)

- Patch fires every run (`[elfjit:routeB] SH328 params-x20 @0x1025f52d8 ...`).
- `JIT_DUMP_PC=0x1025f5460` fires (2/2): fn 0x25f52b4 now **COMPLETES** and reaches the
  `[x19]vt+16` AppStarted dispatch @0x25f5460 — the wall SH327 left is crossed.
- New fault deterministic `guestpc=0x1025f501c` fault=0x0 (was 0x1025f5300 fault=0x140):
  the merged-fn continuation `ldr x8,[x0]` @0x25f5050 where x0=`[x19,#1032]` (or the
  `bl 2ea3a84` result) is **0**; x19 = **REAL AppStarted heap obj** (0x5588b4471ae0). The gate
  is now a genuine AppStarted **live member** (+0x408 / +0x418 / +0x420, devpath `cbz w8,25f504c`
  on byte [0x6a67488]).

## 4. Verify

- `cargo test -p arm64jit --example elfjit -- sh328` = 1 passed (5 real-image pins: mov x20,x0
  0x25f52d8, ldr w3 0x25f5328, ldr w8 0x25f533c, [x19]vt+16 ldr 0x25f5460, fn 0x25f55b0 stp).
- `cargo test --workspace` EXIT 0; elfjit examples 152/0; arm64jit lib 408/0.
- Probe: <full seed set in /home/hermes-worker/runs/STATUS.md> `... --startapp 0x258b144 --v2boot
  --v2boot-session --v2boot-skip-appstart --v2boot-session-bus` -> first_sigsegv 0x1025f501c;
  0x1025f5460 dump hit; repro /tmp/sh328-probe.txt.
- elfjit.rs held under the 1MB pre-commit hook.

## 5. Honest

No make_shared<DataModel> (DM-root 0, MH_* false); Route-B live-DM structural gate UNCHANGED.
The declared SH327 next-forward (supply the params-obj +0x140 member) is DONE and MEASURED: the
`x20`-params-object wall is fully crossed (fn 0x25f52b4 completes), and the app-start fencepost
advanced to 0x25f5050 — a **real-AppStarted live member** (+0x408/+0x418) demand. That is the
SESSION-CTOR wall class; a fabricated-object seed there is SH251's *proven dead-end* (feeding fake
objects upstream does not change the map construction). Per the SESSION-CTOR PRIMARY LEVER, the
forward is the REAL Activity/AppBridge session drive so the upstream session ctor RUNS and builds
the +0x408 member — NOT another static seed. SH328 is inert by default and did not change that
honest state, only advanced the app-start fencepost one more deterministic step.