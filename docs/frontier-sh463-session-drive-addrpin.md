# Frontier SH463 — 3-axis HEAD re-verification + session-substrate <name,guest> contract pin

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace 850 passed/0 failed, incl. arm64jit lib 669/0; cargo
build --workspace + --example elfjit OK). recon-v3 immediate-priority
deliverables re-verified GREEN on the real libroblox.so at HEAD.

## Why this cycle
STATUS next-forward #5 names the "new family from a real-run decode gap" as the
only Route-B re-entry. Before deciding the next code target, this cycle re-
established ground truth on all three axes and independently closed the "is
there a genuinely-unpinned decoder family left" question, then landed the one
new contract hardening available on the SEP-17/18 session-drive lever without
re-treading the documented walls.

## Verify axes (all GREEN at HEAD)
1. **Runtime deliverable (recon-v3 §A)** — `runs/capture_taskv4_frame.sh`
   against the real 104MB `libroblox.so` (SH415 env): 24 real task-driven
   frames `present #N swap Ok(0x1)` (engine make-current 0x105b3b358 ->
   frame-fn 0x105b32c00 -> eglSwapBuffers 0x105b3b408 on RENDERCTX vtable
   0x106731ae0), 0 json abort (JIT_JSON_ZERO_FIX clean), 0 crash, EXIT 0.
   Seed markers present (`type4_frame_thunk registered`, `seeded dispatcher
   type-4 vector [0x106829ea8]`, `renderthunk published RENDERCTX`).
2. **Codegen coverage COMPLETE** — cross-referenced every `Inst::` variant the
   decoder produces (decode.rs) against translate.rs: the ONLY variant never
   constructed by a hermetic is `Inst::Unsupported` (not a real instruction
   family — it is the decode failure path). No real decoder family is unpinned.
3. **No real-run decode gap** — taskv4 + sh415/sh463b real-binary logs contain
   ZERO `Unsupported`/`illegal`/`not-implemented` markers; the translator never
   faults on the client's executed instructions (the two substrate stops are
   `outside image` branch-target lanes, not decode errors).
4. **Substrate baseline deterministic** — SH415 env reproduces exactly 11/16
   atoms non-zero Ok, the two "outside image" faulting atoms identical to the
   established baseline.

## The two faulting substrate atoms — confirmed pre-existing, not a regression
`nativeInitializeNativeFlags` (0x10232048c) and `nativeAppBridgeV2InitWithParams`
(0x102365c54) stop with garbage pcs from the session-drive. A clean boot-only
run (`--v2boot` without `--v2boot-session-drive`) drives the SAME fns in the
SAME order and they fault identically (then aborts EXIT 134 at the SH174 latch
without JIT_DM_ALLOC_CAPTURE_DELEGATE=1). The earlier sh415 "Ok(0x3e8) via boot
rung" was a later in-process re-drive that benefited from state the drive's
prior atoms warmed. This is the run-variable, order/warm-up-dependent nativeInit
'outside image' ladder lane recorded in memory — a documented pre-existing
Route-B lane, NOT a session-drive defect, and NOT a re-tread target per STATUS.

## The one new deliverable: session-substrate <name,guest> pin
The SEP-17/18 session-drive is dispatached by (name, guest). Existing locks:
- session.rs names-only set check (catches a name set change / fallthrough);
- jit.rs sh399 prologue-anchor (catches a guest address that stops resolving to
  a sub-sp/stp prologue).

Neither catches a **name<->guest address swap** (same names, addresses swapped to
two distinct valid fn entries). SH463 adds to session.rs
`substrate_atom_names_exact_and_no_fallthrough` a deterministic (no real binary,
parallel-safe) assert pinning the definitive 16-entry (name,guest) map, so any
drift fails at compile-test time. arm64jit lib 669/0 (no new test — extended the
existing names test).

## Honest
- NOT a DM; does NOT manufacture a DataModel. This is a re-verification cycle +
  a defensive contract pin on the session-drive lever the operator names.
- Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, no store reaches it,
  structural at the write site per SH462).
- No new decoder family exists to pin (coverage genuinely complete).

## Files / verify
- crates/arm64jit/src/session.rs (`substrate_atom_names_exact_and_no_fallthrough`
  extended with the definitive 16-atom <name,guest> address map). Test-only.
- Verify: `cargo test --workspace` 850/0; `cargo build --workspace` +
  `--example elfjit` OK.
- Commit: local `dev` only (operator pushes to origin).