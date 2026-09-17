# Frontier SH273 — the ENTIRE JNIActivityLifecycleCallbacks nativeOn* family converges on ONE shared dispatcher

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Route-B live-DM
structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
+1 hermetic sh273 (real-image guard). No production path edited; default-inert
(the test is a pure regression pin). Workspace green.

## Why (closing the SEP-17 directive's ambiguity)
The SEP-17 SESSION-CTOR directive names the REAL Android Activity-session init
state machine as the primary lever, and the SH184 lifecycle map lists
`JNIAppLifecycleNativeAdapter_setActive`, `initAppShellReporter`, the client-
settings receive, and the Activity-lifecycle callback family. SH264 measured only
TWO lifecycle receive entries headlessly (`nativeOnResumed` -> a real lifecycle-
registry live-object fault=0x50, and the setActive/initAppShellReporter benign
Ok(0x0) paths). That left the OTHER 11 lifecycle public entries — nativeOnPre
Created/Created/PreStarted/Started/PreResumed/PrePaused/Paused/PreStopped/Stopped/
PreDestroyed/Destroyed — as ambiguous "still-undriven candidates" a future cycle
might drive one-by-one as fresh levers.

## Measured (fresh disasm + authoritative symbol table, real libroblox.so)
All 12 public `Java_com_roblox_universalapp_activitylifecyclecallbacks_JMIActivity
LifecycleCallbacks_nativeOn*` entries are 44-byte JNI stubs of the IDENTICAL shape:
- prologue `stp x29,x30,[sp,#-16]!` (0xa9bf7bfd)
- load JNIEnv vtbl GetStringUTFChars (slot 169/offset 1352) — the SH186 identity
  shim, so a fabricated jstring resolves
- FINAL instruction: an unconditional `b 0x21f15a4` (tail branch), each with its
  per-entry state literal (0..8) in w0

0x102_21f15a4 is the SINGLE shared Activity-lifecycle notifier dispatcher
(`sub sp,#0x1c0`, huge state-dispatch body keyed on the w20 state code). Its
state dispatch reaches a real lifecycle-callback-registry object whose deref is
the SH264-measured live-object fault (guestpc=0x1021f3748 = `sub sp,#0x90`
callee prologue; the registry-access faults fault=0x50 there).

### The closure
There are NOT 12 independent lifecycle primitives to drive headlessly. Every
public lifecycle entry is a one-instruction tail into the SAME dispatcher body,
which immediately derefs a REAL registry live-object — the SH184/SH174 live-object
class. Driving any sibling (PreCreated, Created, Started, Resumed, Paused, ...)
produces the IDENTICAL wall as the already-measured nativeOnResumed. The SEP-17
"remaining lifecycle levers" are therefore a single closed gate, not a fan of
open ones. Do NOT re-drive a sibling lifecycle native as a fresh seed lever.

## Honest (do-not-over-claim)
Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED; the
SH174 capture-latch stays the single forward hook. What IS new + measured: the
12-entry convergence on the one shared dispatcher is now byte-pinned (prologue
word + tail-branch imm26 target recompute for EACH entry + the dispatcher
prologue + the fault-site callee prologue), so a future cycle has a loud
regression signal if this structural fact drifts, and the SEP-17 lifecycle
candidate list is closed as one wall.

## Code / verify / artefacts
- elfjit.rs hermetic `sh273_lifecycle_natives_converge_on_shared_dispatcher`
  (real-image guard, skip-if-absent): pins dispatcher 0x102_1f15a4=0xd10703ff,
  the 12 entry prologues (0xa9bf7bfd each), each entry's tail word is an
  unconditional `b` (opcode 0x5) whose imm26-recomputed target == 0x102_1f15a4,
  and the fault-site prologue 0x102_1f3748=0xd10243ff. 4-aligned + in-window.
- Verify: `cargo test -p arm64jit --example elfjit sh273` = 1 passed (real-image
  anchors asserted on libroblox.so). Full `--example elfjit` batch + workspace
  re-run green (see commit note).
- repro: none needed (static pin); SH269 session-ctor repro re-ran clean at this
  HEAD (MessageBus Ok(0x3e8), once-guard=1, DM-root=0).
- Commit: local `dev` only.