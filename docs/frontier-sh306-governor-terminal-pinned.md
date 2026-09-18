# Frontier SH306 — hermetic pin for the standing SendAppEventOnAppReady terminal

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Pure
hardening — a real-image guard for the current Route-B no-regression anchor,
NOT a DM construction attempt.

## Why

At HEAD (SH305 re-derivation), the canonical session-ctor repro (`runs/
capture_sh269_session_ctor_exec.sh`) lands deterministically at the same
terminal: MessageBus.subscribe Ok(0x3e8) x3, once-guard [0x106a68410] self-
latches 0x1, DM-root [0x106a68818] stays 0 (never owns a live DM), and
SendAppEventOnAppReady parks at the governor NULL app-DM-controller
(guestpc=0x102ea0b9c, EXIT 134); with the opt-in GOVFLAG seed it advances one
gate to the sh270-pinned preload-overrides wall 0x102bb803c.

sh270 pins the downstream preload-overrides wall and sh272 pins that getter's
two dead branches, but the **governor NULL-DM-controller fault site itself
(0x102ea0b9c) had NO hermetic byte-pin anywhere in the tree** — it was only
referred to in eprintln prose. That is the actual standing Route-B terminal,
the single most important no-regression anchor for the operator's
SESSION-CTOR frontier: if ANY future session-drive/seed shifts it, a pin should
fail loudly so the shift is not silently absorbed as "still parked".

## What SH306 adds

A compact real-image-guard test `sh306_governor_null_controller_terminal_pinned`
(elfjit.rs, sh115_tests module) that byte-pins:
- the governor fn prologue 0x2ea0b48 = 0xd10203ff (`sub sp,#0x80`);
- 0x2ea0b78 = 0xf9401015 (`ldr x21,[x0,#32]` -> app-DM controller);
- 0x2ea0ba8 = 0x39768108 (`ldr w8,[x8,#3488]` = reads the GOVFLAG byte
  [0x106a64da0]);
- 0x2ea0bd0 = 0xf94206a0 (`ldr x0,[x21,#1032]` -> controller);
- 0x2ea0bd4 = 0xf9400008 (`ldr x8,[x0]`) — the NULL-controller deref that
  fires at guestpc 0x102ea0b9c;
- the GOVFLAG cell 0x106a64da0 in the RW image window.

Skip-if-absent on machines without the real libroblox.so (same guard family as
sh270/272/273). All 5 bytes verified against `aarch64-linux-gnu-objdump` before
landing.

## Verify

- `cargo test -p arm64jit --example elfjit -- sh306` = 1 passed (131 examples
  total → 130 baseline + 1 new; 0 failed).
- `cargo test --workspace` EXIT 0.
- elfjit.rs 1,048,567 B = 9 B under the 1MB pre-commit hook (funded by
  condensing redundant doc-prose across SH 81/99/116b/118/119/120/122/128/131/
  153/160 — address grammar + shrink, no behavior or address changed).

## HONEST

Pure regression hardening — it does NOT manufacture a DataModel, does not lift
the Route-B live-DM structural gate (DM-root 0, MH_* false). The governor
terminal itself remains the SH269/SH270/SH272-measured live-object wall
(do-not-re-tread its getter/value-cell seeds). The value is an explicit,
fail-loudly pin for the exact no-regression anchor a future session-drive must
move. SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>)
stays the single forward hook. recon-v3 deliverables stay shipped + verified.