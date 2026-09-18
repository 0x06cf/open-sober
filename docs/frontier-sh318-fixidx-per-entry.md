# Frontier SH318 — the tier-2 fixidx byte is PER-ENTRY, not a single fixed cell (corrects SH317)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh318` (real-image, 6 pins).
Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false). Recon-v3 re-verified green
at HEAD (24 task frames, swap Ok(0x1), 197 pops, 0 json abort).

## 1. The correction

SH317's probe read a **single fixed** index byte at guest [0x1070271d0] and concluded the
controller-name table always resolves "Runtime0". Fresh disasm of the two-tier lookup
(0x2168798) shows the tier-2 index byte is actually **per-registry-entry**:

```
21687d8: adrp x21, 6fe6000            ; registry array base page
21687dc: add  x21, x21, #0x180        ; x21 = 0x106fe6180 (entry base, stride 0x60)
21687e4: add  x8, x21, #0x3e, lsl #12 ; x8  = 0x107026180
21687f0: add  x24, x8, #0xff0         ; x24 = 0x107027170  <-- fixidx ARRAY base
2168828: add  x24, x24, #0x1          ; per-iteration: fixidx[i] = [0x107027170 + i]
216882c: add  x21, x21, #0x60         ; per-iteration: entry stride
```

So `fixidx[i] = [0x107027170 + i]`, and tier-2's controller cell is
`0x106fe2f00 + fixidx[i]*0x5c + 0x2078`.

SH317's probe read `[0x1070271d0]` — which is `0x107027170 + 0x60`, i.e. **entry index 96**,
far beyond the count of 12, always a stale 0. The "Runtime0 invariant" verdict was therefore
based on a mis-attributed address.

## 2. Measured (corrected probe, real so, bus route)

New SH318 runtime dump walks the REAL per-entry addressing for the 12 registered entries:

```
SH318 per-entry fixidx[0x107027170+i]->controller-cell:
  [e0]0->Runtime0 [e1]0->Runtime0 ... [e11]0->Runtime0
```

Every entry's fixidx byte is 0 and every controller cell resolves "Runtime0". So even with the
correct per-entry addressing, **the "App" invariant holds**: no registry entry can make the
DM-controller ctor's fast-path resolve "App" through a `.bss` seed.

## 3. What this means for Route B

- SH317's conclusion stands, but for the rigth reason and at the correct address. The controller
  table cells hold "Runtime0" — a runtime-generated name ABSENT from the 229MB .so (verified by
  byte-scan), so the table is **session-constructed** (no static writer reaches [0x106fe2f00+0x2078
  +*0x5c]; my whole-image stress scan found no STR/STP into that region).
- The SESSION-CTOR lever is unchanged and sharpened: the binder route (MessageBus.subscribe,
  --v2boot-session-bus) provably populates the registry 0→12; the "App" primary must be registered
  by a real session AND resolve through the controller-name table, both live-construction work.

## Repro

- `cargo test -p arm64jit --example elfjit -- sh318` (1 passed; 143/0 total).
- Bus-route probe: `runs/probe_sh315_bus_registry.sh` now prints the SH318 per-entry dump.
- Recon-v3 re-verified green at HEAD: `runs/capture_taskv4_frame.sh` (24 frames, swap Ok(0x1),
  197 pops, 0 json abort, EXIT 124).

Single-agent. Workspace green (arm64jit lib 405/0; elfjit examples 143/0; build EXIT 0).
elfjit.rs held 176 B under the 1MB pre-commit hook.