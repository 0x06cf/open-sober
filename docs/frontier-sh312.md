# Frontier SH312 — the do-init DM-controller NULL is the empty service registry (byte-pinned)

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed).

## What was measured (this cycle)

Re-ran the SH311 repro at HEAD (3/3, runs/sh311-once-*.txt) — deterministic confirmation:
ONCE-lambda 0x2206d10 fires every run, its `str x0,[x23,#1032]` @0x2206d74 executes
(storing bl 0x2173b3c's return), yet DM-root [0x106a68818] reads 0x0 after. The do-init
once-path is NOT the blocker; the value stored is NULL. That pinned the concrete next gate:
**why does the DM-controller ctor chain (0x2173b3c -> tail 0x61e30bc) return 0.**

This cycle disassembled the full chain and located the return-0 mechanism at byte level.

### The mechanism (authoritative, guest == file vaddr + 0x100000000)

The ctor 0x61e30bc's FIRST real action is `bl 0x2168798` @0x61e311c — a
name->service lookup. That lookup begins by reading the service-registry COUNT
GUEST cell **[0x106fe2f08]** (`adrp x8,6fe2000; ldr w22,[x8,#3848]` @0x21687d0) and
`cbz w22 -> 0x2168834` @0x21687d4 returns 0 immediately when the registry is empty.
Headlessly no service is ever registered, so:
- the lookup returns 0 (x25=0),
- `cbnz x0` @0x61e3124 falls through to the ctor's fallback, which is ALSO
  gated on the SAME cell: `cmp w8,#0xa7e` @0x61e3150 — it only constructs a real
  object when count==0xa7e (2694) AND [0x106fe2f90]==0 (the 0x61e3aa4 build).
So a live DM-controller requires the service registry populated, i.e. the
registry-count/version cell [0x106fe2f08] reaching 0xa7e via ~2694 REAL service
registrations — a real session ctor (SH174/204/SH251 live-object class). That is
never a `.bss` seed and never a fabricatable DM-root.

### Why this is the SESSION-CTOR gate (not a seed)

`grep` of all writers of the 0x6fe2000-f08 region: the registration fn **0x21e2a90**
(`bl 0x21e2a90` callers incl. **0x235678c, which lives INSIDE nativeAppBridgeAppStart**)
and static bulk-writers 0x1e2a988/0x1e2a9xx. So the registry is populated by
app-start's OWN session run as it walks its service-registration table — and
app-start dies headlessly at the standing **0x1021dde34 live-object map wall**
(SH248g/h/249/250/251/253/256/260), BEFORE it can register the ~2694 services.
Hence DM-root stays 0 and do-init's done-path never reads a live controller.

This is exactly the operator's SEP-17 SESSION-CTOR premise, now located at a
single byte cell: **the DM-controller ctor is gated on a registry that only a
real upstream session ctor populates.** Do-not-hand-seed a DM-root (SH251:
genuine-but-fake DM does NOT advance app-start's map construction).

## Verdict

SH311's "why 0x61e30bc returns NULL" is now byte-pinned to the empty
service-registry count [0x106fe2f08] (lookup `cbz w22` @0x21687d4) + the version-gated
fallback (`cmp w8,#0xa7e` @0x61e3150). The Route-B live-DM structural gate is
UNCHANGED and precisely located; the unblock is driving the real upstream session
ctor (per SESSION-CTOR directive), not a `.bss` seed.

## Honest (do-not-over-claim)

- Does NOT manufacture a DM: DM-root [0x106a68818]=0, MH_* false. Route-B live-DM
  structural gate UNCHANGED. SH174 capture-latch stays the single forward hook.
- This is a byte-level closing of WHERE the do-init ctor stops, consistent with
  and extending SH311; it converts SH311's "not re-tread beyond this map" into a
  pinned mechanism.

## Verify / files

- New hermetic `sh312_dmctor_null_is_service_registry_empty_pinned` (elfjit.rs
  real-image guard, 6 byte-pins): lookup count-read 0x1021687d0=0xb94f0916,
  cbz 0x1021687d4=0x34000316, ctor bl 0x101_61e311c=0x96fe159f, cbnz 0x101_61e3124=
  0xb5000d40, fallback cmp 0x101_61e3150=0x7129f91f, b.ne 0x101_61e3154=0x54000421.
  Skip-if-absent.
- elfjit.rs held under the 1MB pre-commit hook (condensed render/session prose;
  all addresses preserved).
- Repro: runs/capture_sh311_once_lambda.sh (3/3, deterministic at HEAD).
- Commit: local `dev` only (operator pushes).