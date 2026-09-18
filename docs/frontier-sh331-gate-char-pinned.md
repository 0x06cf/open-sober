# SH331 — app-start +0x408 gate characteristics pinned: guard 100% deterministic,
# downstream run-variable confirmed (8-run), SH330b x21-reseed tested-and-reverted (measured-negative)

Status: `dev`, single-agent (cone suppressed). Pure characterization cycle — NO production code
change beyond a doc note on `routeb_appstart_408_guard` (SH330). Workspace green (arm64jit lib
408/0; elfjit examples 154/0; all crates 0 fail). Recon-v3 re-verified green at this HEAD.

## Why

STATUS listed candidate #2 for the SH330 forward: "continue clearing the run-variable host-ptr
leaks (SH320-class) / FMOD AAudio (SH212) on the app-start path ... to find the next STABLE gate."
This cycle (a) re-measures whether the app-start +0x408 gate's crossed-downstream is really
run-variable or whether a defensive epilogue x21 reseed can make it deterministic, and (b) pins
whether the leaked host PCs on the session rungs (SetInitParams/V2Init) are one shared slot.

## Measured (real libroblox.so, full SH330 recipe + JIT_ROUTEB_APPSART_408SEED, 8 runs)

`runs/probe_sh330_repro.sh` (added this cycle; canonical SH330 repro, ~200s each):

| metric | result |
|--------|--------|
| guard fires ([x19+0x408] seeded) | **8/8 (100%)** at pc=0x1025f501c |
| gate 0x25f5050 crossed (DUMPPC 0x1025f5060 reach) | **8/8 (100%)** |
| downstream EXIT | **7/8 EXIT 0 (clean)**, 1/8 EXIT 134 (guest abort) |

So the gate-cross itself is fully deterministic, but the DOWNSTREAM is genuinely run-variable even
on the full recipe — exactly matching SH330's original doc (the binary does not crash at
0x25f5050 anymore, but the continuation variably aborts ~1/8, SH320/SH212-class host-pc/AAudio,
non-seedable, deliver NOT a stable gate). No new stable gate found on the app-start path.

## SH330b: defensive epilogue x21 reseed = measured no-op, reverted

Hypothesis: the epilogue `ldr x8,[x21]` @0x25f5060 re-reads the stack CH canary through x21, and if
the host-leaf dispatch clobbered x21 the canary check fails -> stack_chk_fail / run-variable arm.
Tested a guard that reseeds `x21 = [0x1067d16f0]` (guardGOT slot) at epilogue block entry.

MEASURED: 0 firings (x21 already === guardGOT in every run — the host leaf preserves CpuState
callee-saved regs), and NO distribution change (7/8 clean → still 7/8 clean WITH the reseed). So
the epilogue is NOT the leak source; the run-variable arm is a real downstream host-pc/AAudio site
(SH320/SH212-class). Reverted the code (SH184/SH186 no-cruft: don't ship an unfired guard against
an empty endpoint); kept only a terse measured-negative NOTE on the SH330 fn.

## SetInitParams / V2Init session-rung leak = shared leaked-host-pc slot (already-known class)

Driving the SEP-17 lifecycle rungs (--v2boot-session), SetInitParams (0x102bcc814) and
V2InitWithParams both soft-return `run_loop: pc <host-or-subimage> outside image` — the leaked pc
is identical WITHIN one process run (e.g. 0xe9c148c18948ca09 both times) but different ACROSS runs
(ASLR) and sometimes a tiny sub-image value (0xfd3/0x159/0x38). The JNIEnv fn-table slot 31
(GET_OBJECT_CLASS) is correctly populated (jni_get_object_class, jni.rs:1069) and resolves; the
leak is downstream of it, in the real settings/telemetry marshalling (21fcda0/21fcc84/2335e34 +
bl 626b6d0 FMOD AAudio) — the same SH320-class host-pc family already documented
(frontier-sh320/sh212), NOT a seedable gate. Do not re-derive.

## Honest

No DM (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM gate UNCHANGED. This cycle
characterizes/pins (a) the app-start +0x408 gate is 100/100 deterministic at the fork and
(7/8-clean) run-variable downstream, and (b) the SH330b x21 hypothesis is a measured negative.
No production code path altered (doc-note only). The next forward remains the SESSION-CTOR real
Activity/AppBridge session drive (SEP-17 PRIMARY LEVER) so the upstream ctor builds a REAL
+0x408 object (real vt[+136] work) instead of the fabricated leaf — the standing migration-gate
direction; single-agent cone remains suppressed.

## Verify

- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (arm64jit lib 408/0; elfjit
  examples 154/0 incl sh330/sh329 still passing).
- recon-v3 re-verified: 24 task-driven frames, present #21..#23 swap Ok(0x1), 0 json abort,
  EXIT 124.
- Repro: `bash runs/probe_sh330_repro.sh` (guard=2, crossed=2, EXIT distribution as above).

Single-agent, default-inert. Doc-note only; elfjit.rs size unchanged.