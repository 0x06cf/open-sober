# SH256 — "repair the map at its fixed root global [0x1067d16f0]" is a MEASURED FALSE POSITIVE
# (that global is the STACK CANARY, not a map-root holder; the repair only moved the failure mode)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## The genuinely-new angle (and why it was tried)
All 7 prior closures at the app-start live-object map wall (guestpc=0x1021dde34,
nativeAppBridgeAppStart / SH174/SH204 class) keyed the repair on the wrong object:
- SH248g static seed: targeted an ASLR host-heap pointer (no fixed cell).
- SH248h runtime repair: keyed on `x21`, the per-iteration cursor.
- SH250 count-clamp: clamped a live host-heap field.
A fresh JIT_DUMP_PC register dump at THIS HEAD (3/3 EXIT 134,
runs/sh256-maproot-dump.txt) showed:
  x24 = 0x561a62418060  STABLE across all 3 clean inserts, AND === [0x1067d16f0]
so it looked like the map root IS anchored at a fixed .bss global we could repair:
give it a coherent header + a LARGE pre-allocated zeroed bucket array so the insert
walk stays in-bounds and the never-run grow/rehash ctor (SH254's root cause) is
never needed. No prior closure ever keyed the repair on [0x1067d16f0].

## The lever (implemented, then MEASURED, then REVERTED)
Default-inert `routeb_appstart_maproot_seed_guard` (opt-in JIT_ROUTEB_APPSART_MAPROOT_SEED=1),
fires at the wall block entry 0x1021dde00..0x1021dde40, reads root=[0x1067d16f0], and
if the (alleged) map header is incoherent (count garbage, base unmapped) repoints
[root+0]=leaked 0x400-slot zeroed array and [root+8]=0x400. Hermetic sh256 test added.
Wired into the block-entry dispatch.

## MEASURED (real libroblox.so, full DMCONT env, 3 runs OFF + 3 runs ON)
                OFF (baseline)        ON (MAPROOT_SEED=1)
repair firings   0                     2 (root base==root itself, count=0x2f2a1a0a0e0f1011)
stack-smash      0                     3 ("*** stack smashing detected ***")
stable SIGSEGV   guestpc=0x1021dde34   GONE
crash sites      0x1021dde34 x3        (none stable)
exits            134/134/134           134/134/139 (SIGABRT from stack smash)
First sight this *looks* like an advance: the 8-closure wall SIGSEGV disappears and the
run only terminates because of a DIFFERENT, apparently-later abort.

## WHY IT IS A FALSE POSITIVE (the honest root cause)
"Stack smashing detected" is the guest's own `__stack_chk_fail` (0x21ddc8c): the app-start fn
prologue (0x21ddc44) does `adrp x20,67d1000; ldr x20,[x20,#1776]` => x20 = [0x1067d16f0], then
`saved_canary = [x20]` at [x29,#-8]; the epilogue re-reads `ldr x8,[x20]; cmp x8,saved; b.ne
__stack_chk_fail`. My guard's repair **wrote [root+0] = bucket-array ptr** — but root IS x20
([0x1067d16f0] is the CANARY pointer, and [root+0] is the canary VALUE at [0x1067d16f0]). So the
repair CHANGED the canary value between prologue and epilogue → deterministic __stack_chk_fail.

Therefore [0x1067d16f0] is exactly SH182's CANARY global (stack-canary pointer, file 0x67d16f0),
components a 8-byte canary WORD — NOT a map-root holder. The "repair moved past the wall" is an
artifact of corrupting the stack canary, which only *changed the failure mode* from a map SIGSEGV
to a canary SIGABRT. It does NOT advance app-start and does NOT manufacture a DataModel.

## What is genuinely new + measured (the value of this cycle, no over-claim)
1. Closes the 8th adjacent angle at 0x1021dde34 with a runtime A/B: "repair the map via a fixed
   root global [0x1067d16f0]" is a measured FALSE POSITIVE because that global is the stack canary,
   not a map holder. A future session must NOT re-tread a "[0x1067d16f0] root" seed.
2. CORRECTS SH250's register attribution: SH250 read x24 === [0x1067d16f0] as "stable map identity / 
   caller-side root global" — it is actually the stack-canary pointer (stable for exactly that reason:
   the canary global is a stable .bss cell). The real map `this` is live host-heap and ctor-uninitialized
   (SH174/SH204 live-object class, still the standing wall).
3. Re-confirms the standing structural gate: the app-start live-object map (and the whole
   nativeAppBridgeAppStart world-build) needs a REAL upstream make_shared<DataModel> ctor. SH174
   capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the single forward hook.

## Do-not-re-tread (now closed at this HEAD)
- "[0x1067d16f0] holds the map root; seed/repair it" — it is the stack canary pointer (this cycle).
- "repair the app-start map keyed on x21" — x21 is the in-image/cursor, SH248h closed, SH249
  segment-proof closed.
- "clamp the outer count" — SH250 closed (live host-heap field).
- "seed the 0x2a0 array allocator" — SH254 closed (code never runs headlessly).

## Files / code
- Guard + hermetic sh256 test + dispatch wiring: implemented, MEASURED, then REVERTED
  (a lever that only corrupts the stack canary adds no value; clean tree vs HEAD).
- Evidence logs (gitignored per project rule): runs/sh256-maproot-dump.txt (the fresh register dump),
  runs/sh256-ab-off.txt, runs/sh256-ab-on.txt (3-run A/B each arm).

## Verification
cargo test -p arm64jit --lib sh256 PASSED (before revert); tree clean vs HEAD after revert;
cargo build --workspace + cargo test --workspace green (re-verified); recon-v3 deliverables
re-verified green at HEAD (24 task-driven frames, swap Ok(0x1), 196 pops, 0 json abort, 0 crash).
No production code path changed (guard reverted). Single-agent.