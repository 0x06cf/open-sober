# Frontier SH417+SH418 — the persistent-tracker host input LOOP, then promoted to a first-class substrate step

Date: 2026-09-19, hermes-worker, single-agent (cone suppressed). Workspace green
before (cargo test --workspace EXIT 0, 651/0) and after (arm64jit lib 471/0 incl.
sh417 + sh418; full workspace re-verified). Real-binary capture runs/capture_sh417_host_input_loop.sh,
log /home/hermes-worker/runs/sh417-input-loop.txt (outside repo). Production code
in session.rs (+drive_host_input_loop / input_loop_iters, +sh417 +sh418 hermetics)
+ elfjit.rs (+--v2boot-input-loop rung); jit.rs untouched (byte-unchanged, at the
1MiB hook). elfjit.rs condensed SH-prose comments (addresses kept) to stay under
the 1MiB pre-commit hook (1,048,552 B < 1,048,576).

## Why this cycle

STATUS next-forward #3 ("Hang drive_host_input_poll on a real host loop — it is
currently one-shot opt-in, proven on real binary"). The SH416 one-shot poll built
a FRESH `PointerTracker` every call, so a press's DOWN in poll i and its MOVE in
poll i+1 were translated as two different pointers — the pointer-down / multi-touch
state that is exactly the input axis's job was thrown away between polls.

## What landed

- **session.rs `drive_host_input_loop`** (SH417): checks the three inert guards
  ONCE up front (JIT_AINPUT_BRIDGE + registered window XID + live image), then
  drains N=INPUT_LOOP_ITERS non-blocking batches through ONE PointerTracker
  (pointer state survives across polls), marshalling each translated MotionEvent
  into guest `nativePassInput` via the SH413/414 ABI. Bounded; a mid-loop X error
  stops the loop keeping events already delivered. +hermetic sh417.
- **session.rs substrate promotion** (SH418): `drive_host_input_loop(iimg, ib,
  tpidr, boot_sp, input_loop_iters())` is wired into `drive_routeb_session_substrate`
  right after the post-bus G3 content-surface step — the same first-class promotion
  SH411/412 gave the DM binder / nativeAppBridgeAppStart / content surface. A live
  session's ordered drive now includes the host-input source. +`input_loop_iters()`
  (bounded default 8) +hermetic sh418.
- **elfjit.rs**: `--v2boot-input-loop` rung (INPUT_LOOP_ITERS).

## Measured (real libroblox.so, SH416 capture env + INPUT_LOOP_ITERS=4)

- Window wired: `[elfjit:anativewindow] wired real X11 window XID=0x200000 on :308`.
- `[session-drive] host-input loop: polling real window 0x200000 on :308 for 4
  iterations (persistent tracker, bridge armed)` — loop ran all 4 iterations,
  `0 raw -> 0 translated -> 0 delivered`, `running total 0`,
  `done 4 iterations, 0 real events delivered`. No input-path crash (terminal
  SIGSEGV at guestpc 0x1029f3f7c = known nativeInit "outside image" Route-B lane,
  SH416 documented identically).

## Honest

Inert-by-construction like every runtime axis: 0 real events on a boot with no
live session — there is no constructed login/home screen to deliver to until a
completed do-init owns a live DataModel. It is the cause-not-symptom input runtime
surface (BUILD-THE-RUNTIME): the moment a live DM advances, the bounded loop
delivers real desktop pointer input with correct persisting pointer state across
frames. Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0,
MH_GAME_LOADED false, no make_shared). recon-v3 deliverables re-verified green at
HEAD. No re-treads.

## SH417b correction (this cycle): the boot/input loop's "known nativeInit Route-B
## lane SIGSEGV" was a MISSING-ENV artifact, not a distinct crash.

SH416/417/418 all documented the input/boot runs as terminating in a "known
run-variable nativeInit 'outside image' Route-B lane at guestpc=0x1029f3f7c" and
treating it as pre-existing. SH417 re-measured it: with the SAME input-loop boot
env plus the crossing hash-fix gates the full reaching ladder already carries
(`JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1` — the exact env
recon-v3/capture_taskv4 and sh415 set), the run is **deterministic-clean: EXIT 124
(stable idle), 0 SIGSEGV/SIGABRT**; hashfix fires 103×, the input loop still runs
all 4 iterations 0→0→0 delivered as designed. The 0x1029f3f7c fault is the SH83/SH91
string-hash-map `blr x8` into a garbage +0x18 hash-fn-2 slot (`x8=0x4741495241003635`)
— exactly the map crash the hashfix guard clears; the input-loop runbook simply
omitted the env, so the whole --v2boot boot raced into the unguarded map insert and
the terminal was MISATTRIBUTED as "a Route-B lane." Fix: capture_sh417_host_input_loop.sh
now carries the same crossing env as the reaching/recon-v3 runbooks. VERIFIED on real
libroblox.so (0 crash). The input axis itself was never the faulting path.
(SH417b scope: ONLY the promoted sh417 loop. The superseded SH416 one-shot poll
re-ran with the same env and hit a DIFFERENT run-variable lane, guestpc=0x101fd128c
LSM-family fault=0x18, hashfix 0× — a separate pre-existing wall, not cleansed by
this env; capture_sh416_input_poll.sh was reverted unchanged.)

## Files

- crates/arm64jit/src/session.rs (+drive_host_input_loop, +input_loop_iters,
  +sh417, +sh418; one-shot poll doc corrected to point at the loop)
- crates/arm64jit/examples/elfjit.rs (+--v2boot-input-loop rung; SH-prose condensed)
- runs/capture_sh417_host_input_loop.sh (SH417b: + crossing env -> deterministic-clean)
- docs/frontier-sh417-host-input-loop.md

## SH417c audit (this cycle): extend the SH417b missing-env audit to the FULL-BOOT
## --v2boot runbooks — capture_v2boot.sh (SH54) + capture_v2boot_gate.sh (SH81)

SH417b noted the crossing-env audit should run on every boot-path runbook. The
audit found two canonical stable-baseline `--v2boot` full-boot runbooks
(runs/capture_v2boot.sh, runs/capture_v2boot_gate.sh) that omitted the crossing
env and silently inherited the exact SH83/SH91 hashfix-lane crash SH417b
cleaned from the input loop. MEASURED on real libroblox.so:

- capture_v2boot.sh WITHOUT the env: SIGSEGV guestpc=0x1029f3f7c (fault=0x0,
  the string-hash-map `blr x8` into garbage hash-fn-2 — precisely the hashfix
  lane), EXIT 134, crash-count 2.
- Same runbook WITH `JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1`:
  routeb-hashfix fires (hundreds of lines incl. SH83/SH84/SH88/SH92 inserts),
  guestpc=0x1029f3f7c GONE, run ADVANCES into nativeGameGlobalInit's real body,
  then aborts at `*** stack smashing detected ***` — the documented SH97/98
  canary wall deeper in the ladder, a SEPARATE pre-existing wall (NOT claimed
  clean). crash-count 1.
- capture_v2boot_gate.sh (SH81, header claims "CRASH-FREE now") carried NO
  crossing env — same latent misattribution; env added to match.
- Out of scope (correctly untouched): capture_v2boot_sh82.sh is a DIAGNOSTIC
  runbook whose purpose is demonstrating exactly the hashfix lane progression;
  capture_sustain_loop.sh + capture_sh304_session_producer.sh + all render-plane
  runbooks (triangle/quad/mesh/tex/emitter) drive the harness-frame/startapp path
  that never reaches the hashfix insert lane (no full --v2boot ladder).

Honest: NOT a Route-B step, no DM (DM-root 0, MH_* false unchanged). This is an
artifact-quality/attribution correction in the same class as SH417b: it removes a
genuine crash from the canonical stable boot runbook (the "0 crash" guarantee a
usable headless session needs) and fixes the two SH54/SH81 headers' latent
misattribution of a hashfix-lane SIGSEGV as a Route-B/pre-existing lane. Route-B
live-DM structural gate UNCHANGED. recon-v3 deliverables re-verified green at
HEAD this cycle (capture_taskv4_frame.sh attempt 1: 24 real task frames `present
swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash).

## Next (standing)

Route B / do-init completeness stays top (STATUS #1): the substrate now REPORTS
the completion markers, the marker stays non-live until the engine's own session
ctor builds the DM world. Do-not-re-tread unchanged: LSM skips/rebuilds
(SH349/350/358/373/375/377/378/385/396), setDataModelToCurrent (SH388), EC reader
(SH355/356/374), window-attach real (SH367), ALooper (SH365), governor gates
(SH379). Continue the SEP-18 runtime build (real Activity/AppBridge/JNI-lifecycle/
GLES/session drive) as the one cause-not-symptom front.