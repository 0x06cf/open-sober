# SH332 — combined MAIN-path reach measured: do-init done-path MAIN binder-dispatch +
# full SH320/322/323 stack + SH330 +0x408 cross on ONE run, terminating at the FMOD/AAudio
# live-object wall (0x106240cb8) — no new stable gate, but a new combined reach (SESSION-CTOR)

Status: `dev`, single-agent (cone suppressed). No production code changed this cycle.
Workspace green (arm64jit lib 408/0; elfjit examples 154/0; all crates 0 fail). Recon-v3
re-verified green at HEAD (24 task-driven frames, present #19..#23 swap Ok(0x1), 192-197 node
pops, 0 json abort, EXIT 124). New repro probe runs/probe_sh332c_mainpath_regok.sh. elfjit.rs
byte-identical (1048570 B, unchanged).

## Why this cycle

STATUS candidate #1 (SESSION-CTOR, the SEP-17 primary lever) has been worked many cycles. The
SH320/322/323 guards (do-init done-path MAIN dispatch + lifecycle-wall early-return + settings-SSO
seed) are the documented line that carries the do-init MAIN binder-dispatch through the SH273 wall
into engine-settings-init. SH330 independently crossed the app-start +0x408 fork on the harness
`--startapp` path. What was **never measured together**: the full MAIN-path dispatch stack AND the
+0x408 app-start cross on ONE jit_run, with the service-registration walk / name->service lookup
under region-watch, to answer the operator's "does the crossed app-start drive the 'App' service
registration" question (SH313/315/316/317/318).

## Measured (real libroblox.so, runs/probe_sh332c_mainpath_regok.sh, single ladder run)

Full recipe, no `--v2boot-skip-appstart` (so V2StartAppWithParams reaches the fork):

- **do-init done-path MAIN binder-dispatch executes**: `[routeb-sh320] seeded do-init done-path
  main-id [0x106863a68]=... -> b.eq NOT taken -> MAIN binder-dispatch 0x206df4 (DM-ctor entry)`.
- **SH322/SH323 stack runs live**: `lifecycle_seed=22` (early-return guard fired across BOTH
  lifecycle-notifier copies 0x1021f3748 + 0x1021f4538), `sso_seed=2` (both settings-SSO std::string
  cells [0x106ed7a18]+[0x106ed7a28] populated when NULL). Reach windows all confirmed.
- **The app-start +0x408 gate CROSSES on this MAIN path**: `guard408=2`, `fork_501c=1`, `fork_5060=1`.
  DUMPPC at 0x1025f5060 shows x8=0x7f00000001d8 (the benign host leaf) — the same cross SH330
  established on the harness --startapp path, now reached from the do-init done-path MAIN dispatch.
- **The name->service lookup AND the service-registration walk both execute on this path**:
  `lookup_hits=4` (region 0x102168798-0x102168840), `regwalk_hits=2` (region 0x1021e2a90-0x1021e2b40)
  — the fn 0x21e2a90 registration walk that SH312 pinned as the "App" service gate DOES fire headlessly
  on this line.
- **Terminal**: SIGSEGV `guestpc=0x106240cb8 fault=0x0` = the FMOD
  `Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged` region (SH212-crash-A / accepted FMOD AAudio tail
  `bl 0x1d97414 -> tbnz w0` @0x6240cb4/8), then SIGABRT via leaked host-pc (fault=0x3e900272d36,
  SH320-class). This is the SAME run-variable wall SH330 documented downstream of the crossed gate
  (RUN2 was 0x106240cb8). NOT a new stable gate.

## What this means for Route B

- A genuine **new combined reach** (never measured together): the do-init done-path MAIN dispatch
  (SH320) + lifecycle/Settings cross (SH320/322/323) + app-start +0x408 cross (SH330) on one jit_run,
  with the registration-walk + lookup executing. This is cause-level SESSION-CTOR progress — the
  MAIN-path engine-settings-init line now demonstrably climbs into the app-start continuation that
  crosses +0x408.
- It does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false). The `"App"` tier-2
  controller-name cell ([0x107027170+i] fixidx -> "Runtime0" invariant, SH317/318) is unchanged; the
  lookup+walk executing is the *engine-side* of the registration, but the app-start continuation
  dies at the FMOD/AAudio live-object wall before the registry's "App" entry is written. Route-B
  live-DM structural gate UNCHANGED.
- The next wall is the SH320/SH212-class run-variable FMOD/AAudio host-pc leak (0x106240cb8) — a
  live-object (host-allocated unmaterialized device) / leaked-host-pc class, NOT a seedable cell
  (SH132/213 measured: FMOD does not dlopen libaaudio and the bridge never engages on this path;
  the crash is a direct-JNI `this` misconstruction via the JNICallProtocol registry). Do-not-re-derive.

## Honest

New combined reach + repro probe; does NOT advance the live-DM structural gate. The decision-relevant
finding: the do-init MAIN path now demonstrated to reach and cross the app-start +0x408 fork, and to
terminate at the FMOD/AAudio live-object wall — consistent with SH330/SH331's characterization (cross
at the fork is 100% deterministic; downstream run-variable). The "App"-registration lever (SH313/315/
316/317/318) remains the standing unblocked-next target, still gated on a real session populating the
tier-2 controller-name table.

## Verify

- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (lib 408/0, elfjit 154/0) — no
  Rust source changed this cycle, only the probe + docs.
- recon-v3 re-verified green: `runs/capture_taskv4_frame.sh` (24 frames, present #19..#23 swap
  Ok(0x1), 192 node pops, 0 json abort, EXIT 124).
- Repro: `bash runs/probe_sh332c_mainpath_regok.sh` (guard408=2, fork_501c=1, fork_5060=1,
  lifecycle_seed=22, sso_seed=2, lookup_hits=4, regwalk_hits=2, terminal 0x106240cb8).

Single-agent, default-inert (all guards opt-in env, unchanged). elfjit.rs byte-identical, under the
1MB pre-commit hook.

## SH333 addendum — A/B attribution of the +0x408 (and registration-walk) reach (same cycle)

Added runs/probe_sh333_bus_mainpath.sh, which stacks the SH315 bus route (`--v2boot-session-bus`)
with the MAIN-path stack + 408 cross on ONE non-skip run, and A/B's the guards:

- **Arm A (SH330+SH320 guards ON)**: `guard408=2`, `fork501c=1`, `fork5060=1` — the +0x408 app-start
  cross fires (SH332 parity), registration-walk `regwalk=2` + lookup `lookup=4` both execute. Run dies
  SIGSEGV `guestpc=0x101dcab68` -> SIGABRT leaked host-pc (SH320-class run-variable), BEFORE the
  post-ladder dump so no registry/DM-root readout is reached.
- **Arm B (SH330+SH320 guards OFF)**: `guard408=0`, `fork501c=0`, `fork5060=0` — the +0x408 cross does
  NOT fire; registration-walk + lookup still execute; run dies `guestpc=0x101db1d04` (the standing
  SH260/285 LSM insert-leaf reader wall, NOT the +0x408 path).

Conclusion: the +0x408 reach seen in SH332/SH333-arm-A is attributable to the SH330 guard (A/B clean)
and the registration-walk driving is independent (fires on both arms). Neither arm reaches the
post-ladder `dump()` (registry count "App", DM-root, tier-2 cell) because the continuation faults at the
run-variable FMOD/AAudio / LSM reader wall first — so the "App"-registration question stays open but
UNCHANGED: no new stable gate, no DM. Consistent with SH332 + SH330/331.