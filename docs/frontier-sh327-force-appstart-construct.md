# Frontier SH327 — force AppStarted factory construction DETERMINISTIC; gate advances one fencepost

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh327` (new, real-image guard) beside
`sh325`/`sh324` (unchanged). elfjit examples 151/0 (was 150); arm64jit lib 408/0; workspace green.
elffit.rs held under the 1MB pre-commit hook (condensed SH-prose to fit). Route-B live-DM gate
UNCHANGED at the top level (no DM), but the APP-START fencepost `0x1025f5300` was OVERTURNED and
ADVANCED one step deeper. This cycle: implemented the SEP-17 "force the construction branch
deterministically" forward (SH326's measured premise reversal -> a real code change).

## 1. The problem SH327 fixes (SH326c premise -> a concrete leak)

SH326c measured that the AppStarted construction write (`str x0,[x19,#24]` @0x2e89150, target
[0x106a6f480] = [appstart+24]) DOES fire headlessly, overturning the SH324/325 "unconstructible
blindly" premise. But the standing `0x1025f5300` field-copy gate STILL faulted reading
[appstart+24]==0 across runs — a NONDETERMINISTIC construction.

Root cause (from the 0x2e890c4 disassembly, real libroblox.so): after the producer-counter
`bl 2baaac0`, the factory does `lsr x8,x0,#62; cmp x8,#0x2; b.ne 0x2e89118`. When the counter
returns tag==2, the branch FALLS THROUGH to `mov x0,x19; ret` — returning the appstart
**unconstructed** ([appstart+24] stays 0), and the field-copy gate faults. So construction is
run-variable (the tag).

## 2. The fix (`routeb_patch_appstart_construct_force`)

Patch `0x2e890f4` from `b.ne 0x2e89118` (0x54000121) to unconditional `b 0x2e89118` (0x14000009),
removing the sole non-construction exit. Now every invocation of the factory runs its construction
loop. Idempotent: the loop short-circuits at `0x2e89130` (`ldr x8,[x19,#24]; cbnz x8, 0x2e89160`)
when [appstart+24] is already built. Gated on the same JIT_ROUTEB_DM_SEED/DMFORCE flags as the
SH325 init3 hoist (inert on plain runs). Drops the factory's cached blocks so it recompiles from the
patched bytes.

## 3. Measured (real libroblox.so, 3/3 + dual-PC dump)

- Patch fires every run (`[elfjit:routeB] SH327 force AppStarted construct @0x102e890f4 ...`).
- JIT_DUMP_PC=0x102e89150,0x1025f52f0: the construction write FIRES and lands —
  `[appstart0x106a6f480]=0x55cb343ab000` (real heap obj) is read at the gate's `ldr x19,[sp,#328]`.
  Construction is now DETERMINISTIC (was run-variable).
- The gate ADVANCED one fencepost: the field-copy helper 0x25f54e8 now COMPLETES and returns
  (lr back at 0x25f5300); the fault moved to `ldr w3,[x20,#320]` @0x25f5328 (fault=0x140) where
  x20 = V2StartAppWithParams' params object is 0. This is a NEW, deeper gate —
  `<V2StartAppWithParams params obj>+0x140` — one stop past the appstart construction.

## 4. Verify

- `cargo test -p arm64jit --example elfjit -- sh327` = 1 passed (9 real-image pins: factory bl
  2baaac0, the force-target b.ne 0x2e890f4, unconstructed-return ret 0x2e89104, construction body
  ldr/str 0x2e89130/0x2e89150, plus counter/cmp words).
- `cargo test --workspace` EXIT 0; elfjit examples 151/0; arm64jit lib 408/0.
- Probe: `timeout 200 env <full seed set> ./target/debug/examples/elfjit $SO 0x2173ff4 --jni
  --startapp 0x258b144 --v2boot --v2boot-session --v2boot-skip-appstart --v2boot-session-bus`
  (SH327 patch marker + first_sigsegv advanced; repro in /tmp/sh327_probe.sh).
- elffit.rs held under the 1MB pre-commit hook.

## 5. Honest

No make_shared<DataModel> (DM-root 0, MH_* false); Route-B live-DM structural gate UNCHANGED.
What is new + measured: the AppStarted factory construction is now DETERMINISTIC (the SEP-17
forward), and the app-start fencepost moved from [appstart+24] to the V2StartAppWithParams params
object +0x140 (x20, param0, currently 0). The NEXT forward is that +0x140 member of the params
object (what x0/x20 of 0x25f5270 must hold for the field-copy consumer to pass). Default-inert.