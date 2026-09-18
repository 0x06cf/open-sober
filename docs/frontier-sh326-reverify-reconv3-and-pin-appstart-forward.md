# SH326 — re-verify recon-v3 deliverables + pin the exact SEP-17 forward contract; observe
# SetInitParams deep type-4 drain (nondeterministic, run-variable — NOT a stable gate)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh325` extended (2 new anchors, 1 passed)
beside `sh324` (unchanged). Workspace green (tracked below). Route-B live-DM gate UNCHANGED
(DM-root [0x106a68818]=0, MH_* false). This cycle: VERIFIED the two recon-v3 deliverables are
already landed + green at HEAD, and sharpened the standing 0x1025f5300 gate's forward contract.

## 1. Recon-v3 deliverables — already implemented, re-verified green at HEAD

The IMMEDIATE-PRIORITY items in docs/recon-selfdrive-seed-jsonfix.md are BOTH present in HEAD and
green (no new code needed this cycle):

- (1) type4_frame_thunk self-driven frames: `--taskv4-seed frame` installs the registered
  non-recursive host thunk at [0x106829ea8]; the engine's real drain dispatches type-4 nodes to it.
  Re-verified: **24 task-driven frames; present #19..#23 swap Ok(0x1)** with distinct colors;
  197 node pops (real drain activity); EXIT 124 (stable idle). Markers: `[elfjit:taskv4] type4_frame_thunk
  registered`, `TASK-DRIVEN FRAME PRESENTED`, `present #N swap Ok(0x1)`.
- (2) json-zero-fix: JIT_JSON_ZERO_FIX clamps the leaked std::string length at append-check
  0x102355d40 (`len>cap` -> len=0 SSO empty). Re-verified: **0 json abort** on the same run.

## 2. Standing gate 0x1025f5300 — deterministic 3/3, forward contract now pinned

The terminal is stable: first SIGSEGV guestpc=0x1025f5300 (fault=0x140), 3/3 clean runs
(probe_sh324_terminal_regs.sh). This is V2StartAppWithParams' field-copy gate: after `bl 0x25f54e8`
(a field-copy helper), the caller faults reading its source object. The source is `ldr x0,[x19,#24]`
(x0 = [appstart_obj+24]) where x19 = [sp+328] = the **genuine AppStarted object** built by
nativeAppBridgeAppStart (bl @0x25f52ec). [appstart_obj+24]==0 headlessly -> fault=0x140.

The SEP-17 forward contract is now pinned in the sh325 hermetic guard (2 new anchors + the existing
field-copy source pin):
- 0x1025f52ec = 0x97f51b0d  `bl nativeAppBridgeAppStart 0x233bf20` (the AppStarted source fn)
- 0x1025f52f0 = 0xf940a7f3  `ldr x19,[sp,#328]` (AppStarted out-param -> field-copy source)
- 0x1025f52f8 = 0xf9400e60  `ldr x0,[x19,#24]`  (the field-copy source object)

**The cross is `[appstart_obj+24] != NULL`** — a genuine session-constructed AppStarted field, the
SEP-17 real-session construct. A host seed there is the SH324-PROVEN dead-end class (fn consumes the
x8 out-param a host leaf cannot write); only the real session drive builds it. This sharpens, not
relocates, the standing SEP-17 lever.

## 2b. The cross IS constructible (recon): [appstart+24] = [0x106a6f480] written by a REAL ctor

Full forwarding of the gate's object chain reveals the cross is a DRIVABLE ctor output, not a
fabricate-only cell:
- nativeAppBridgeAppStart (0x233bf20, bl @0x25f52ec) is once-guarded on [0x106a6f490]. When
  [0x106a6f490].bit0==1 it calls `x0=0x106a6f468; bl 2e890c4` (the AppStarted factory; x0 = the AppStarted
  object) and stores the result to the out-param `[sp+328]` = x19. When bit0==0 it takes the call_once
  path (bl 284ce54) first.
- 0x2e890c4's construction branch (reached after its counter/flag checks) computes
  `str x0,[x19,#24]` @0x2e89150 — i.e. **writes [AppStarted+24] = the exact cross cell**. x0 there is the
  return of `bl 35d0608` (a build helper) after `bl 21daef8` (a co-initializer on obj+0x8/+0x30). The
  branch is gated: `ldr x8,[x19,#24]; cbnz x8, skip` @0x2e89130 (already-set short-circuit).
## 2c. MEASURED: the construction write FIRES headlessly — premise overturned

JIT_DUMP_PC probe on the standing run (--startapp 0x258b144, --v2boot --v2boot-session, full seed
set) shows 0x2e89150 (`str x0,[x19,#24]`, the AppStarted+24 constructor write) DOES execute headlessly:
`DUMPPC pc=0x102e89150 x0=0x5588d0f085e0 x19=0x106a6f468` — so `[0x106a6f468+24]=[0x106a6f480]` is
populated with a REAL heap object. The prior "cross is unconstructible headlessly / only a real Java
session builds it" premise (SH324-325) is **OVERTURNED**: the AppStarted ctor runs and writes the
cross cell. The gate STILL faults (x0=[AppStarted+24]=0 at the field-copy 0x1025f5300) — leading
hypothesis (NEXT to test): a synchronization/determinism gap, the construction write does not land
before the main thread's field-copy reads the cell (once-guard bl 233bf20 -> bl 2e890c4 may run
asynchronously/post-ladder or on a second AppStarted copy), so the read sees 0. SEP-17 forward is
therefore to test determinism, not construction: drive 0x2e890c4 synchronously on the SAME thread and
re-read [0x106a6f480] after bl 233bf20 returns (or seed once-guard [0x106a6f490].bit0=1 + verify);
whether the write raced a clear or a different object is the open question. No code changed this
cycle; repro steps in this doc + JIT_DUMP_PC=0x102e89150.

## 3. New observation: SetInitParams drives the real type-4 TaskScheduler drain (NONDETERMINISTIC)

The sh325 ladder probe (runs/probe_sh325_ladder_session.sh, --v2boot-session-set + full seed set)
runs SetInitParams (0x102bcc814) deep enough that the engine's REAL TaskScheduler wakes and drains
type-4 task nodes (the recon-v3 producer region). In ~2/3 runs it faults on a **leaked host pointer**
in a task node's processor-vtable slot ([node+112] = a host box address) -> SIGSEGV in the node-proc
drain dispatch, then SIGABRT. In ~1/3 runs SetInitParams soft-returns benignly and InitClientSettings
returns Ok(0x1).

Measured (5 runs, identical cmd): RUN1 SIGSEGV guestpc=0x102855fd0; RUN2 benign (InitClientSettings
Ok(0x1)); RUN4 guestpc=0x10624e6c0; RUN5 guestpc=0x1021df3cc; RUN6 benign (run_loop pc 0x30 outside
image). **Different pcs each crash** => run-variable host-pointer leak (the known ASLR-flaky base,
SH320-class), NOT a stable deterministic gate => not a build target this cycle. It does confirm the
SEP-17 client-settings feed now reaches real engine task dispatch — a signal, not a milestone.

## 4. Verify

- `cargo test -p arm64jit --example elfjit -- sh325` = 1 passed (8 real-image pins: 2 SH160-NOP
  sites; new-terminal adrp/ldr; fn 0x25f54e8; ldr [x19,#24]; +2 NEW bl-nativeAppBridgeAppStart 0x25f52ec
  + ldr x19,[sp,#328] 0x25f52f0 — the SEP-17 AppStarted source chain).
- `cargo test --workspace` EXIT 0; elfjit examples ~150/0; arm64jit lib 408/0.
- Recon-v3 re-verified: 24 task-driven frames present #19..#23 swap Ok(0x1), 197 node pops, 0 json abort,
  EXIT 124.
- Standing gate deterministic 3/3 at 0x1025f5300.
- elfjit.rs ~1023.9 KB, under the 1MB pre-commit hook.

## 5. Honest

No DM (DM-root 0, MH_* false). Route-B live-DM structural gate UNCHANGED. The recon-v3 deliverables
were already green at HEAD (this cycle verified + documented them); the code change this cycle is the
extension of the sh325 hermetic guard with the SEP-17 forward-contract anchors. Single-agent,
default-inert, no production path edited.