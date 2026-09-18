# SH316 — do-init once-guard SELF-LATCHES + the DM-ctor fast-path stores the "Execute" service handle (populated-registry plain run)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh316` (real-image, 4 pins).
Workspace green (arm64jit lib 405/0; elfjit examples 141/0). elfjit.rs held under the 1MB hook.
Recon-v3 re-verified green (unchanged). Route-B live-DM gate UNCHANGED (no DM; the once result
is a service handle, not a live controller).

## 1. The finding (measured, deterministic, plain SESSION-CTOR bus run)

Run: `--v2boot-skip-appstart --v2boot-session-bus` (the SH315 plain SESSION-CTOR route, full seed
set, real libroblox.so). New corrected probe readout (this cycle added `once-slot` to the SH155
probe):

    once-guard[0x106a68410] = 0x1          (SELF-LATCHED)
    once-slot[0x106a68408]  = 0x400000b    (the DM-ctor fast-path result = the "Execute" service handle)
    DM-root[0x106a68818]    = 0x0          (a DISTINCT live-object cell)
    service-registry-count[0x106fe2f08] = 12

Reproduced (3/3 deterministic, EXIT 124).

## 2. Why this corrects SH315

SH315's ledger line said "the once-guard NEVER self-latches and DM-root stays 0 — do-init's
`__call_once` doesn't complete on the ladder thread even with count=12". That was measured on its
`--v2boot-postbus-doinit` re-drive, which CLEARS the once-guard + re-seeds main-id before
re-driving StartLuaAppDM — a perturbed, guard-cleared state. On the PLAIN bus run the once-guard
SELF-LATCHES (0x1): do-init's `__call_once` COMPLETES headlessly now that the registry is
populated. The SH311/312 "DM-root stays 0 because the ctor returns NULL" picture is also refined:
the once-lambda's `str x0,[x23,#1032] @0x2206d74` (x23=adrp 6a68000) writes into once-slot
[0x106a68408], and that value is 0x400000b — the DM-ctor LOOKUP fast-path (`cbnz x0 @0x61e3124`)
fired and returned the matched "Execute" service handle, a real non-NULL result.

The probe's "DM-root [0x106a68818]" is a DIFFERENT cell (+0x410 from once-slot) with no
static/once writer (SH155: populated only by a live heap store during real app-launch). So
"DM-root 0" does not mean do-init's once failed; it means the ctor matched "Execute" (one of its
two registry names) but that service is the task-scheduler "Execute", NOT a live DataModel/app-shell
object — the live-DM structural gate still holds.

## 3. The open lever is unchanged (and sharper)

- The ctor's fast-path (0x61e3124) CAN fire headlessly on a populated registry and returns a real
  matched service handle. It currently matches "Execute" (registry count=12 includes Execute, not
  "App").
- SH313's "one 'App' entry gives a live DM-controller via the fast-path" remains the precise lever;
  the 12 task-scheduler services are a DIFFERENT tier from the DM "App"/"Execute" pair the ctor
  fast-path needs. Get "App" registered (a real session-side registration, the SESSION-CTOR cause)
  and the fast-path returns the DM controller -> once-slot -> done-path 0x2206c88 reads it
  (`ldr x1,[x8,#1032]` = once-slot, pinned: not DM-root) and dispatches.
- The done-path controller source is once-slot [0x106a68408], not [0x106a68818]; SH155's note that
  DM-root has no writer refers to the appbridge-level live-object cell, not the once-slot result.

## 4. What's committed / verified

- Probe: SH155 now also reads once-slot [0x106a68408] (the do-init once result / done-path
  controller source).
- Hermetic `sh316` (real-image, 4 pins): adrp 6a68000 (0x2206c74), add #0x410 guard (0x2206c78),
  done-path `ldr x1,[x8,#1032]` (0x2206c8c), once-lambda `str x0,[x23,#1032]` (0x2206d74).
- Corrrected SH311's comment (once-slot vs DM-root cell distinction).
- HONEST: no DM. once-slot=0x400000b is a service handle, not a live DM-controller. Route-B
  live-DM structural gate UNCHANGED; the SESSION-CTOR binder route (SH315) is the working lever.

## Repro

- `cargo test -p arm64jit --example elfjit -- sh316` (1 passed; 141/0 total).
- Plain run: `runs/probe_sh316_ctor_watch.sh` (region-watch: once-guard latches, ctor builder
  0x61e3aa4 fires) + the corrected SH155 once-slot readout.
- Single-agent. Workspace green.