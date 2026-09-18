# Frontier SH330 — cross the SH329 app-start fork gate by one deterministic step (AppStarted+0x408)

Status: `dev`, single-agent (cone suppressed). This cycle adds a new default-inert block-entry guard
(`routeb_appstart_408_guard`, env `JIT_ROUTEB_APPSART_408SEED`) that seeds the AppStarted+0x408 live
member the SH329 fork gate reads, making the app-start continuation PASS 0x25f5050 deterministically
— the first measured advance of that gate since SH324. Work-in-progress confirmations: recon-v3 green
at HEAD re-verified (24 frames, EXIT 124).

## 1. Why this cycle

SH329 closed only the governor-flag *fork* (0x25f503c): no flag byte value swaps which null arm runs.
Both arms converge on `ldr x0,[x19,#1032]` (AppStarted+0x408) = [AppStarted+0x408]=0 -> `ldr x8,[x0]`
@0x25f5050 SIGSEGV (guestpc 0x1025f501c). The gate is SESSION-CTOR class but SH329's own disassembly
shows the target `@0x25f5058/5c` is `ldr x8,[x8,#136]; blr x8` — a PLAIN vt[+136] dispatch, NOT the
x8-out-param ABI that SH324 proved un-crossable by a fabricated object. So a fabricated object whose
vt[+136] is a benign host leaf should pass 0x25f5050 and reveal the NEXT gate — a legitimate forward
probe that SH329 had not run. (SH326 had flagged a determinism gap; this pins it.)

## 2. The implementation

The faulting base x19 is a RUNTIME heap AppStarted (measured 0x55d3ec1a5170 / 0x556eeaa79638), only
known in-process — a static seed is impossible. So the guard fires at block-entry into the fork
window (`pc` [0x1025f501c, 0x1025f5060]), reads x19, and if [x19+0x408] is 0/sub-image writes a leaked
benign object whose vtable (0x180, all slots) has vt[+136]=identity host leaf. Idempotent (only writes
when 0), preserves a real live pointer, default-inert (env-gated). Mirrors routeb_tail_dispatch_guard
(SH161). ~55 lines added to crates/arm64jit/src/jit.rs; examples/elfjit.rs unchanged net.

## 3. Measured (real libroblox.so, full SH328 seed set + JIT_ROUTEB_APPSART_408SEED=1)

3/3 runs the guard fires identically:
`[routeb-appstart408] SH330 seeded [x19+0x408] ({slot}) = {obj} (vt[+136]=leaf) at pc=0x1025f501c`.
The gate IS deterministically crossed — no SIGSEGV at 0x1025f501c anymore (vs SH329 3/3 SIGSEGV).

Downstream of the crossed gate is RUN-VARIABLE (not a stable gate), three distinct endpoints:
- RUN1 (clean): EXIT 0 — DUMPPC at 0x1025f5060 shows the vt[+136] leaf dispatched benignly (x8=
  0x7f00000001d8 leaf); the continuation completes the canary-check epilogue and the function returns.
- RUN2: SIGSEGV guestpc=0x106240cb8 fault=0x0 — the FMOD/AAudio region (0x6240cb8, the known
  SH212-crash-A site / FMOD AAudio init walk), reached by the deeper continuation.
- RUN3: SIGSEGV guestpc=0x40850fc085 (garbage = a LEAKED HOST POINTER became a guest pc) — the
  SH320-class ASLR-flaky host-ptr leak, run-variable.

So SH330 answers SH326's open question: the +0x408 zero was NOT a sync/determinism gap — seeding it
deterministically passes 0x25f5050, and the NEXT wall is the SH320-class run-variable host-ptr leak /
FMOD AAudio, not a stable gate. This is a fencepost-level advance: gate moved from a deterministic
SESSION-CTOR fork (0x25f5050) to a run-variable downstream wall.

## 4. Honest

No DM (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM structural gate UNCHANGED. This is NOT
a live DM; it crosses ONE deterministic app-start gate and documents that the next wall is run-variable
(host-ptr leak / FMOD AAudio), a different class from the stable fork — actionable for a future
deterministic-drive cycle but not a session boot. No production path changed; the guard is default-inert.

## 5. Verify

- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (arm64jit lib 408/0; elfjit
  examples 154/0 incl. new `sh330_appstart_408_vt136_dispatch_crossable` + `sh329` still passing).
- recon-v3 re-verified at HEAD: 24 task-driven frames, present #21..#23 swap Ok(0x1), dispatch
  ~#4750000, 0 json abort, EXIT 124.
- Repro: full seed set + JIT_ROUTEB_APPSART_408SEED=1 (probe cmds above); logs in /tmp/sh330-*.log.
- elfjit.rs held at 1048570 B < 1MB hook (SH-prose condensed across sh320-329 comments, facts kept).