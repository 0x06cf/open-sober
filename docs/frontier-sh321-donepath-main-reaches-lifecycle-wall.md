# Frontier SH321 — do-init MAIN-path binder-dispatch now reaches REAL engine settings-init,
# terminates at the SH273 lifecycle-notifier live-object wall (guard-gated new reach)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh321` (elfjit real-image guard pin, 1
passed). This is a pure regression pin + reach-closure of the SH320 "next forward" — it does NOT
manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM structural gate
UNCHANGED.

## 1. The SH320 forward, executed

SH320 proved the do-init DONE-path dispatcher 0x2206db8 can be flipped (via the JIT block-entry
guard JIT_ROUTEB_DONEPATH_MAIN=1) from its default non-main box-build branch to the MAIN
binder-dispatch side (0x206df4 `ldr x0,[x19,#32]` -> vt+0x30 -> `br x1` @0x206e24). The doc's
"next forward work" said the vt+0x30 target must resolve to a live controller (DM-ctor). This
cycle MEASURES what that dispatch actually does headlessly on the real libroblox.so.

## 2. Measured (real libroblox.so, JIT_DRIVE_LIFECYCLE full seed set)

A/B (JIT_DUMP_PC=0x1021f3748, single-pc unperturbed probe, runs/sh321-ab-off.txt vs -on.txt):
- BASELINE (guard OFF): 0 hits at 0x1021f3748 (reach not present).
- CRATE-GUARD ON: guard seeds main-id (eprintl "[routeb-sh320] ... -> b.eq NOT taken -> MAIN
  binder-dispatch"), and the run faults `SIGSEGV guestpc=0x1021f3748 fault=0x50` — reach 0 -> 2
  hits, guard-gated.

## 3. What the run reaches (fresh disasm, authoritative)

tracking the vt+0x30 dispatch target: the MAIN binder-dispatch climbs into a REAL engine init
chain — caller frame at 0x2270024 (`stp x9,x8,[sp,#24]` stores the engine-controller's [x22] and
[x22+8]) -> 0x2270050 (`add x1,sp,#0x18`) -> `bl 0x21f3748` @0x2270060 — fn 0x21f3748 being the
SH273 shared lifecycle-notifier REGISTRY body. The fault mechanism: 0x21f3748 prologue
`0xd10243ff`; `ldr x8,[x1]` @0x21f376c then `ldrb w8,[x8,#80]` @0x21f3770 faults fault=0x50 because
the engine-controller object x22 is a HOST-HEAP LIVE object with first word [x22]=0 (the register
dump shows x22=0x5592...630 host heap, x23=[x22] a live vt pointer; the controller is not yet
fully constructed).

So the do-init MAIN bind dispatch does NOT land on a directly-constructible DM ctor — it reaches
engine settings/controller init that immediately depends on a REAL (not-yet-constructed) engine
controller object. This matches the standing SH273 finding ("the ENTIRE JNIActivityLifecycleCallbacks
family converges on ONE shared dispatcher; driving any sibling = the identical live-object wall
0x1021f3748 fault=0x50"). The SH320 MAIN path is a NEW ROUTE INTO that same wall (via real engine
init, not a fabricated JNI lifecycle entry).

## 4. Honest

No DM (DM-root 0, MH_* false). The concrete forward: SH320's claimed "vt+0x30 -> DM-ctor entry"
is REFINED — the dispatch reaches engine-settings-init (not a DM ctor) and dies at the SH273
live-object wall. That wall is the SESSION-CTOR cave (the engine controller object needs real
session construction, not a seed). This is guard-gated reach closure + regression pin, not a
forward DM advance. SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>)
stays the single forward hook.

## 5. Verify

- `cargo test -p arm64jit --example elfjit -- sh321` = 1 passed (real-image anchors: fault-callee
  prologue 0x1021_f3748=0xd10243ff, ldr 0x1021_f376c=0xf9400028 + ldrb 0x1021_f3770=0x39414108,
  caller stp 0x102_270024=0xa901a3e9 / add 0x102_270050=0x910063e1 / bl 0x102_270060=0x97fe0dba).
- Workspace green; cargo build EXIT 0; elfjit examples 145+1; arm64jit lib 406/0.
- Repro: runs/sh321-ab-off.txt / runs/sh321-ab-on.txt (JIT_DUMP_PC=0x1021f3748, guard A/B).

Single-agent, default-inert (guard opt-in only). No production code path altered.