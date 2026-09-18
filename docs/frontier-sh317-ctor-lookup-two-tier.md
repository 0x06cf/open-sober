# Frontier SH317 — the DM-controller ctor's name→service lookup is TWO-TIER; the fast-path "App" match resolves through a controller-name table (newly pinned tier)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh317` (real-image, 6 byte-pins).
Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false). Recon-v3 re-verified green.

## 1. Why this tier was missed

SH312/313/315/316 pinned the ctor's `bl 0x2168798` name→service lookup as a single
strcasecmp against the service-registry array [0x106fe6180] (0x60 stride) + the
count-cell gate [0x106fe2f08]. Then the ctor fast-path `cbnz x0` @0x61e3124 returns
the matched service as the LIVE DM-controller, so "ONE 'App' entry" (SH313) seemed
sufficient. SH316 measured: once-lambda fast-path FIRES and stores 0x400000b (the
"Execute" scheduler service) into once-slot [0x106a68408] — but that is the fallback
pair member, not "App" (the DM), so DM-root stays 0.

Fresh disasm of the lookup (0x2168798) body shows the match is **two-tier**, which the
prior byte-pins never enumerated:

```
loop (0x21687f8):
  strncasecmp(x0=x1-arg "secondary", [x21]=entry, 0x3f)   // tier-1: walks array entries
  .zero: w8=[x24]                                          // x24 = FIXED byte, NOT per-entry
         umaddl x8, w8, 0x5c, 0x106fe2f00
         add    x1, x8, 0x2078
         strcasecmp(x0=x0-arg "primary", x1)               // tier-2: controller-name cell
  .zero: match -> return [x21-16]                          // service handle
```

- `tier-1` compares the **other/secondary** name (the once-lambda's x1="Execute")
  against each registry entry's leading bytes (stride 0x60, base [0x106fe6180]).
- `tier-2` (only on a tier-1 hit) reads a **fixed** index byte `[x24]` where
  x24 = 0x106fe6180 + 0x3e000 + 0xff0 = **guest [0x1070271d0]**, then resolves the
  **primary** name ("App") through a controller-name table cell
  `[0x1070271d0]*0x5c + 0x106fe2f00 + 0x2078`.

So a match is NOT just "an 'App'/Execute entry exists" — the primary "App" must also
resolve through that fixed-index controller cell. SH313's "ONE 'App' entry" is thereby
necessary but NOT sufficient.

## 2. Measured state on the real so (bus route, deterministic)

Running the SH315/316 session-bus probe (`probe_sh315_bus_registry.sh`) now also prints:

```
SH317 ctor-lookup controller-name table: fixidx[0x1070271d0]=0x0 ->
  cell[0x106fe4f78]="Runtime0" (tier-2 PRIMARY 'App' match must resolve here)
```

- The fixed index byte [0x1070271d0] = 0x0.
- The controller cell [0x106fe4f78] holds the string `"Runtime0"` — NOT "App".
- SH317b extension: dumping indices 0..7 of the same stride/offset shows EVERY index reads
  `"Runtime0"` — the cell region is **invariant**, so the primary "App" match cannot be linked
  through ANY index-byte value. This strengthens the closure: there is no index value that makes
  tier-2 resolve "App", so this specific lookup can never yield a real DM via a `.bss` seed.
- Therefore the final `strcasecmp("App", "Runtime0")` FAILS, so the lookup never returns a full
  match for "App"; the ctor falls to its count/register path and once-slot gets the tier-1
  "Execute" handle (0x400000b), which is a scheduler service, not a DM.

The cell contents are engine-owned runtime state ("Runtime0" — the name of a real
registered controller in the runtime's own table). Fixing it to "App" is **live-session
construction work**, not a `.bss` seed: SH313's whole-image scan already proved the
registry (and by extension this controller table) has no static writer — it is populated
only by a real app-start session run.

## 3. What this means for Route B (SESSION-CTOR)

- The do-init once-lambda DOES complete headlessly now (SH316: once-guard self-latches,
  once-slot non-NULL 0x400000b), but its controller is the wrong tier.
- The precise open gate is now pinned at the **tier-2 controller-name cell**:
  `cell = [0x1070271d0]*0x5c + 0x106fe2f00 + 0x2078` must equal "App" for the fast-path
  to yield the real DM controller. That cell's index byte [0x1070271d0] and the
  controller table at [0x106fe2f00+...] are session-constructed (no static writer).
- Same conclusion as SH312-316: the unlock is a REAL app-start session populating the
  service registry AND its controller-name table so "App" resolves. This is the
  SESSION-CTOR cause-not-symptom lever, now with the exact second-tier cell measured.

Honest: does NOT manufacture a DM; DM-root stays 0; MH_* false; Route-B live-DM
structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.

## Verify / files

- Hermetic `sh317_ctor_lookup_controller_table_two_tier_pinned` (elfjit.rs, real-image):
  6 byte-pins — `add x24,x8,#0xff0` @0x1021687f0=0x913fc118, `mov w26,#0x2078`
  @0x1021687f4=0x52840f1a, `ldrb w8,[x24]` @0x10216880c=0x39400308, `umaddl`
  @0x102168814=0x9bb76508, `add x1,x8,w26` @0x102168818=0x8b1a0101, `cbz w0`
  @0x102168820=0x34000200. (1 passed; examples 142/0.)
- Runtime probe: `probe_sh315_bus_registry.sh` now also prints the SH317 fixidx/cell line.
- `cargo test --workspace` EXIT 0 (405 lib + 142 examples + fsmap + others, 0 fail);
  `cargo build` EXIT 0; elfjit.rs held 163 B under the 1MB pre-commit hook (prose
  condensed — facts/addresses preserved).
- Commit: local `dev` only (operator pushes).

Single-agent. Route-B live-DM gate UNCHANGED.