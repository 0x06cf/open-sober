# SH281 — cross SH280's config+56 NULL-deref; the engine's state=5 body now runs the full GlobalInit-reentry into the 0x2207118 continuation

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · extends `--v2boot-session-engine5` (default-inert) · workspace green (578/0; elfjit examples 114/0)

## What this is

SH280 left the state=5 settings body (0x2bd24b4) faulting at the GlobalInit-reentry's
`ldr x0,[x21,x8]` @0x10275a154 because x21=[config+56]=0 on the zeroed settings-config.
SH280 classified that as a structural live-object wall. Fresh disasm this cycle showed the
**callee 0x275a23c DROPS the read value** — it overwrites x0 with `operator_new(0x40)` and
never derefs what [config+56] actually pointed at. So [config+56] only needs a *valid buffer*,
NOT real content — a cheaper, seedable gate than "structural".

## The mechanism (fresh disasm + measured)

- state=5 body (0x2bd24b4) → config dispatch `bl 0x2bce0d4` (w2=1).
- 0x2bce0d4: `ldr x0,[x0,#56]` (=[config+56]) + `tbz w2,#0` (w2=1 → NOT taken) → `mov w2,wzr;
  b 275a0c4` (GlobalInit-reentry).
- GlobalInit-reentry 0x275a0c4: once-guard (already latched) → `bl 220671c` (gameGlobalInit sub)
  → `mov w8,#8; tst w0,#1; csel x8,x8,xzr,ne; ldr x0,[x21,x8]` @0x10275a154 — the SH280 fault
  (x21=[config+56]=0) → `bl 275a23c`.
- **Callee 0x275a23c**: `mov w0,#0x40; bl 1d96768` (operator_new 0x40) → fresh box; zeroes
  [+32]/[+48]/[+56]/[+60]; `bl 2206f04`; then `b 2207118` (continuation) with x0=the box.
  The incoming x0 (=[config+56]-derived) is NEVER read post-bl — dropped.

So seeding `[config+56]=leaked 0x100 zeroed buf` makes x21 non-null → the `ldr [x21,x8]`
reads 0 cleanly → control falls into 0x275a23c → tail-branch into the 0x2207118 continuation.

## Measured (real libroblox.so, full SH280 seed env, runs/capture_sh281_config56_seed.sh)

**Deterministic 3/3**: the SH281 seed fires, and the state=5 body now runs the FULL
GlobalInit-reentry — region-watch hits `0x10275a144/148/... /275a23c/... /2207118` — with NO
SIGSEGV (EXIT 124 clean timeout on every run; runs/sh281-e5-1.txt..3). This is the FIRST time
the engine's own settings-state path penetrates past 0x10275a148 into the game-global-init
continuation. A/B baseline (SH280 fund, runs/sh281 watch) previously terminated at 0x10275a148.

**The NEW terminal (park, not crash)**: the 0x2207118 continuation does `add x0,x0,#0xaa0;
bl 2b53a68 (mutex_lock)` — it treats the callee's 0x40 box as a MUCH larger object (offsets
+0xaa0/+0xa70/+0xac8). [box+0xaa0] is host-heap junk beyond the 0x40 alloc → the guest mutex
wrapper sees garbage state and parks (spins) until the 120s timeout. Never SIGSEGVs — a
SH174/204 live-object structural park (the box must be a real, correctly-sized settings salient
object, not operator_new(0x40)).

## Honest framing

Cause-not-symptom on the SEP-17 SESSION-CTOR primary lever: the engine's OWN settings-state
machine now advances 3→5→6 and runs its config dispatch into the real GlobalInit-reentry
continuation — one full machine-gate past SH280 (fault crossed, no crash). It still does NOT
manufacture a live DataModel (MH_FLAGS_LOADED=false, MH_APP_READY=false, DM-root 0). The new
terminal parks on the callee box being undersized for the +0xaa0 continuation offsets — the
SH174/204 live-object structural class, now reached from the state=5 path. SH174 capture-latch
stays the single forward hook.

## Next (honest, single-agent)

The continuation 0x2207118 expects the box the callee allocates to be a real, sized settings
salient object (offsets to +0xac8). The operator_new(0x40) fast-path gives it a 0x40 box that
parks the +0xaa0 mutex. Advancing past the park = supply a correctly-sized (≥0xac8+mutex)
settings object to the continuation — live-object-construction class, not a constant seed.
state=9 (0x2bd2668) remains the same class (also →0x2bce0d4 w2=1), not re-attacked as a fresh
lever. Route-B live-DM gate UNCHANGED.

## Verify

`cargo test -p arm64jit --example elfjit sh281` = 1 passed (real-image pins: fault
`ldr x0,[x21,x8]` @0x10275a154 + bl 0x275a23c + callee prologue/operator_new-0x40/tail-b +
continuation prologue/mutex bl 0x102207144). elfjit examples **114/0** (was 113); `cargo build
--workspace` + `cargo test --workspace` EXIT 0 (578). recon-v3 plane re-verified green at HEAD
(runs/g-recon281.txt: 24 task frames `present #` swap Ok(0x1), 195 node pops, 0 json abort,
0 crash, EXIT 124). elfjit.rs 1,052,600 B and HANDOFF.md 1,048,548 B — both under the 1MB
pre-commit hook. Repro runs/capture_sh281_config56_seed.sh + runs/sh281-e5-{1,2,3}.txt +
runs/sh281-watch.txt. Single-agent, default-inert.