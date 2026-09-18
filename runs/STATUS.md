# Open-Sober run state (hermes-worker)

## HEAD: `dev` branch, SH316 (do-init once-guard SELF-LATCHES + the DM-ctor fast-path stores the
## "Execute" service handle on the populated-registry SESSION-CTOR run; corrects SH315's
## "never self-latches" and the once-slot vs DM-root cell distinction)

## — workspace green (elfjit examples 141/0; arm64jit lib 405/0; 585 total).

**State**: `cargo test --workspace` green (exit 0): arm64jit lib 405/0 + elfjit examples 141/0
(140 + sh316) + fsmap + others (585 total, 0 fail). `cargo build --workspace` EXIT 0. elfjit.rs
held UNDER the 1MB pre-commit hook. Commit (SH316) on local dev, not pushed (operator pushes).

## This session (SH316)

1. **SH316 — do-init once-guard SELF-LATCHES on the plain SESSION-CTOR bus run** (corrects SH315).
   With `--v2boot-skip-appstart + --v2boot-session-bus` (full seed set, real libroblox.so): the
   do-init once-guard [0x106a68410]=0x1 and once-slot [0x106a68408]=0x400000b — the DM-ctor
   fast-path fired on the populated registry and stored the matched "Execute" service handle.
   3/3 deterministic, EXIT 124. SH315's "once-guard NEVER self-latches / DM-root stays 0" was
   measured on its postbus re-drive, which CLEARS the guard + re-seeds main-id; the plain run's
   __call_once COMPLETES headlessly.
2. **CRITICAL CELL DISTINCTION**: the once-lambda `str x0,[x23,#1032] @0x2206d74` writes once-slot
   [0x106a68408], and the do-init DONE-path reads it (`ldr x1,[x8,#1032] @0x2206c8c`, pinned). The
   probe's "DM-root [0x106a68818]" is a SEPARATE cell (+0x410) with no static/once writer (SH155).
   So "DM-root 0" != once failed; the ctor matched "Execute" (task-scheduler tier), which is NOT a
   live DM — the live-DM structural gate still holds.
3. **Sharper open lever**: the DM-ctor fast-path (cbnz x0 @0x61e3124) CAN fire headlessly on a
   populated registry. The SESSION-CTOR lever is unchanged and precise: get "App" (the DM pair)
   registered via the real session -> fast-path returns the DM controller -> once-slot -> done-path.
4. **HONEST:** no DM (once-slot 0x400000b is a service handle, not a controller). Route-B live-DM
   structural gate UNCHANGED. SESSION-CTOR binder route (SH315) is the working lever. Probe SH155
   now also reads once-slot. +hermetic sh316 (4 image pins). Recon-v3 unchanged green.

## Standing (honest, unchanged across sessions)

- **Route-B live-DM structural gate UNCHANGED**: no make_shared<DataModel> fires headlessly;
  DM-root [0x106a68818] stays 0; MH_APP_READY stays false.
- **SH315+316 advance**: the service registry is provably populated headlessly (0→12) by the
  SESSION-CTOR binder route, AND do-init's once now completes (self-latches) with the ctor
  fast-path returning the matched "Execute" handle into once-slot. Next gap: register "App" (the
  DM pair) so the fast-path returns the DM controller.
- **SESSION-CTOR is the primary lever** (operator Sep-17): drive the real app-start/Activity
  session. The binder route (MessageBus.subscribe) is the concrete working path.
- Everything achievable headlessly (llvmpipe); GPU host for performance later.

## Next-forward candidates

(a) Register the "App" service (SH313: ONE "App" entry -> fast-path returns a live DM-controller)
    so the ctor's fast-path hits and once-slot gets a real DM controller -> done-path dispatches
    into the app-shell ctor. This is the precise open lever now that the fast-path provably fires.
(b) After DM-root/once-slot populates a live controller, the done-path dispatches -> app-shell/EC.
(c) R1 content path (synthetic CoreScript module) latent until a live DM requests rbxasset://.
(d) SH304 session-gated producer fires the instant a real session owns a live DM.

## Do-not-re-tread (added this session)

- "do-init's once-guard never self-latches (SH315)" — SH316 measured it SELF-LATCHES (0x1) on the
  plain bus run; SH315's claim was specific to its guard-cleared postbus re-drive.
- "DM-root 0 means the once-lambda failed" — the once-slot [0x106a68408] is the real once result /
  done-path controller source; DM-root [0x106a68818] is a distinct no-writer live-object cell (SH155).