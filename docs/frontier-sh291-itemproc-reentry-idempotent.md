# Frontier SH291 — item-PROCESSOR guard-latched re-entry measured idempotent (closes SH290's "second half")

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert (same opt-in `--v2boot-session-itemproc`).

## tl;dr

SH290 drove item-proc 0x102207950 ONCE in isolation with a zeroed item, proving the
once-build completes + [0x106a63b00]=0x800000c. SH291 re-drives item-proc a SECOND
time with the once-guard **latched** (0x101) — the once-body is skipped (guard-check
`tbz w8,#0` at 0x2207af0 not taken since bit0 done-set) and the shared per-item tail
runs. MEASURED 3/3 on the real libroblox.so: re-entry returns Ok, **[0x106a63b00]
STAYS 0x800000c** (the string-map insert does NOT re-run) and **once-guard persists
0x101** — the engine's own once-cell is **stable/idempotent across re-entry**.

## Why (a genuinely open second half, not a re-tread)

SH290's doc explicitly deferred "the second half": its zeroed item made both indir
`blr x8` dispatches (0x2207cb0/0x2207ce0, gated on x19!=0 && x0==0) cbz-skip, so only
the ONCE path was ever measured. The unmeasured question: what does item-proc do on a
RE-ENTRY now that the once is built — does it re-run the once-body (corrupting the
cell) or branch straight to the shared per-item tail? SH291 answers with a real-guard
re-drive: the guard 0x100/0x1 "started+done" bits persist and the once-body is skipped.

## The measurement (real libroblox.so, runs/capture_sh290_itemproc.sh, 3/3)

- SH290 drive (guard cleared -> __call_once once-body): 
  `once-guard 0 -> 0x101`, `[0x106a63b00]=0x800000c`, item-proc Ok.
- SH291 drive (guard latched 0x101 -> tbz skips once-body):
  `item-proc re-entry returned Ok(...)  [0x106a63b00]=0x800000c  once-guard=0x101
  IDEMPOTENT(once-cell stable, guard done-set persisted)=true`
- Deterministic 3/3. MH_FLAGS_LOADED=false / MH_APP_READY=false (no DM).
- Terminal after the ladder = SH285-B LSM reader/pop live-object wall (baseline
  parity, NOT a regression).

## Honest (do-not-over-claim)

- Cause-not-symptom SESSION-CTOR: the ENGINE's OWN worker item-processor is now
  measured stable/idempotent across one full construct + re-enter cycle — a real
  state-machine property (the once body runs exactly once, the guard never re-fires).
- It does NOT manufacture a DataModel: MH_* false, DM-root[0x106a68818]=0, Route-B
  live-DM gate UNCHANGED. No GuiObject; no app-shell.
- The per-item tail's two `blr x8` (std::function dispatch from [sp]) still cbz-skip
  because the fabricated item stays zeroed — driving those is the NEXT gate past
  SH291 (needs a real dispatcher/coherent [sp] std::function, a live-object class).
- SH174 capture-latch stays the single forward hook.

## Verify

- `cargo build --workspace` + `cargo test --workspace` EXIT 0 (580 passed).
- `cargo test -p arm64jit --example elfjit sh291` = 1 passed (examples 120/0; was 119 +sh291).
- recon-v3 frame plane re-verified green at this HEAD (24 frames swap Ok(0x1)).
- elfjit.rs + HANDOFF.md held under the 1MB pre-commit hook.

## Files

- `crates/arm64jit/examples/elfjit.rs` (SH291 2nd drive in the `--v2boot-session-itemproc` rung
  + hermetic `sh291_itemproc_guardlatched_reentry_idempotent`).
- `docs/frontier-sh291-itemproc-reentry-idempotent.md` (this).