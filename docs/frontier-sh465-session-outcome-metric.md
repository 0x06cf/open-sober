# Frontier SH465 — substrate completion metric is outcome-aware (true session-boot health, not return-value filter)

Worker: hermes-worker · single-agent (cone suppressed). recon-v3 immediate-priority
deliverables re-verified green at this HEAD first (capture_taskv4_frame.sh: 24 real
task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort, 0 crash,
EXIT 0; and the full SH415 do-init capture re-confirms the Route-B baseline). This
change is session-drive observability-only (production runtime untouched on the boot
path; the metric is readout).

## Why this cycle

The SH400 ordered session substrate reports a completion count at the end of every
real-binary drive: `substrate complete: N/16 atoms returned non-zero Ok`. Reading the
actual sh415 run (kept on disk) the number was **11/16**, yet 14 of the 16 atoms
genuinely COMPLETED `jit_run` and returned. The metric only counted atoms whose
return value was non-zero, so three **void JNI natives** — `initAppShellReporter`,
`setActive`, `nativeActivity_onEngineSettingsReceived`, all legitimate `Ok(0x0)`
returns — were counted as failures. That understates a healthy session boot (14/16
complete) as 11/16 and misleads about which atoms truly fault: only the two
documented pre-existing nativeInit "outside image" atoms (`nativeInitializeNativeFlags`
0x10232048c, `V2InitWithParams` 0x102365c54) actually stop.

This is exactly the "measured verdict, not a guess" discipline the operator demands:
the runtime's own health number must report the true session-boot state. The fix makes
the atom's COMPLETION (did `jit_run` return?) the health bit, and keeps the non-zero
return sub-count for the cases where the return value is the meaningful signal.

## What landed

### crates/arm64jit/src/session.rs
- **`DriveOutcome` enum**: `Stopped` (`jit_run` picked the Err path — outside-image /
  unresolvable pc) vs `Completed(u64)` (guest function returned; carries w0, where 0 is
  a perfectly valid return for a void JNI native). `DriveOutcome::completed()` is the
  health bit.
- **`drive_atom`** now returns `DriveOutcome` (was `u64`). Internal-only (no external
  caller, no test pinned the old signature).
- **`drive_routeb_session_substrate`** tracks BOTH: `completed` (true health bit) and
  `nonzero` (the legacy non-zero-return count, preserved for continuity). The final
  summary reports `completed/total atoms completed jit_run (nonzero non-zero return);
  stopped/total stopped` — so a real run reads `14/16 completed jit_run (11 non-zero
  return); 2/16 stopped` instead of hiding the 3 void completions as failures. Returns
  the completed count (the true health bit).

### Tests
- New hermetic `sh465_substrate_completion_is_outcome_aware`: pins `Completed(0)`
  counts as a completion (the whole point — void JNI natives are completions), only
  `Stopped` is not counted, and the 16-atom real-run shape (2 documented nativeInit
  "outside image" Stopped + 11 non-zero + 3 void Completed) reads 14 completed / 11
  non-zero. Pure enum semantics, no jit_run, deterministic, parallel-safe.

## Verification
- `cargo build -p arm64jit --lib` OK (the workspace was already green before: 856/0
  total, arm64jit 675/0, re-verified this cycle). `DriveOutcome` lib count 675 -> 676
  with the new test.
- Full `cargo test --workspace` + `cargo build --workspace` + `--example elfjit`
  confirmed green (per the standing gate).
- **MEASURED on the real libroblox.so (sh415 capture re-run after the fix):**
  `substrate complete: 14/16 atoms completed jit_run (11 non-zero return); 2/16
  stopped` — exactly as predicted. The runtime now reports the true session-boot
  health (14/16 completed, the two Stopped being the documented pre-existing nativeInit
  "outside image" atoms). Route-B live-DM probe unchanged (once-guard bit0=1, DM-root
  [0x106a68818]=0x0, LIVE DM = false).

## Honest
- NOT a DM; does NOT advance the Route-B live-DM gate (DM-root [0x106a68818]=0x0 under
  the complete substrate, re-confirmed this cycle). This is a runtime-observability
  correctness fix: the session substrate now reports the TRUE number of atoms that
  completed `jit_run`, so a healthy session boot is measured as 14/16 (not 11/16) and
  the only real faults (the two documented pre-existing nativeInit "outside image"
  atoms) are isolated. It does not change any guest byte, default env, or the envgated
  ladder path (the metric is readout + eprintln).
- Not a re-tread: prior SH cycles measured/recorded the 11/16 baseline but never fixed
  the metric's conflation of `Completed(0)` with `Stopped`; this closes the observability
  defect. Aligned with the operator's data-persistence/measured-verdict discipline (the
  number the runtime reports must be correct so verdicts derived from it are sound).

## Files / verify
- crates/arm64jit/src/session.rs (DriveOutcome + drive_atom + substrate summary + test).
- Verify: `cargo test -p arm64jit --lib` 676/0; `cargo build --workspace` + `--example
  elfjit` OK.
- Commit: local `dev` only (operator pushes to origin).