# Frontier SH313 — the service-registry is 100% SESSION-constructed; the DM-controller ctor FAST path needs ONE "App" entry (whole-image scan)

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed).

## What was measured (this cycle)

SH312 byte-pinned the do-init DM-controller NULL to the EMPTY service-registry COUNT
[0x106fe2f08]. This cycle extends that byte-pin three ways: (a) it identifies the
DM-controller ctor's FAST path (which needs only ONE registered entry, not the 0xa7e
fallback), (b) it locates what name that fast path looks up ("App"), and (c) it proves
via a WHOLE-IMAGE scan that the registry (count + array) has NO static writer — it is
100% session-constructed. All on the REAL libroblox.so.

### (1) The ctor is entered with NAME "App"

The do-init ONCE-lambda (0x2206d24..0x2206d64, the `bl 0x2173b3c` arg build) computes
two rodata addresses via adrp/add: x0 = 0x2d34ab and x1 = 0x3d1ba8. On disk those bytes
are `"App\0"` and `"Execute\0"`. `0x2173b3c` keeps x0 (name) / x1 (other) and tail-branches
`b 0x61e30bc` (the DM-controller ctor). So the ctor's name->service lookup
(`bl 0x2168798` @0x61e311c) is called to find the **"App"** service.

### (2) The ctor FAST path returns a LIVE controller on ONE match

- `bl 0x2168798` (0x61e311c) = name->service lookup: reads COUNT [0x106fe2f08]
  (`ldr w22,[x8,#3848]` @0x21687d0) and `cbz w22 -> 0x2168834` @0x21687d4 returns 0
  immediately when the registry is EMPTY. Headlessly nothing is registered -> lookup=0.
- On a match, the lookup walks the array (base 0x6fe6000+0x180 = guest [0x106fe6180],
  stride 0x60) doing `strcasecmp`; a hit returns the entry.
- Back in the ctor: `cbnz x0` @0x61e3124 -> `mov x0,x25` @0x61e32cc -> `ret` returns the
  matched service as the LIVE DM-controller. So **ONE registered "App" entry is enough to
  give do-init a non-NULL DM-root [0x106a68818].**
- `cmp w8,#0xa7e` @0x61e3150 is only the NOT-FOUND fallback: it increments the count
  (`str w8,[x27,#3848]` @0x61e32c0) and builds a fresh controller via 0x61e3aa4.

### (3) WHOLE-IMAGE scan: the registry has no static writer

- The count cell [0x106fe2f08] (base 0x6fe2000, offset 0xf08) is written ONLY by the ctor's
  own 0xa7e-gated increment (0x61e32c0). The "writers" SH312 attributed to 0x40faa1c etc.
  use a DIFFERENT base (0x6c2f000) — a separate table, not the service-registry count.
- The registry ARRAY [0x106fe6180] (base 0x6fe6000) has ZERO static writers over the whole
  exec segment: only 3 instructions reference base 0x6fe6000 (0x21687d8 lookup, 0x22546d8
  settings-sum, 0x61e697c telemetry), and all 3 are READERS.
- => the registry is populated ONLY by app-start's OWN session run as it walks its
  service-registration table. SH312 located that walk's caller (0x235678c `bl 0x21e2a90`
  inside nativeAppBridgeAppStart); it dies at the standing 0x1021dde34 live-object map wall
  (SH248g/h .. SH256) BEFORE registering even one service. Hence count stays 0 headlessly.

## Verdict

The Route-B live-DM structural gate is UNCHANGED (DM-root [0x106a68818]=0, MH_* false),
but this cycle REFINES where the gate lives: the DM-controller ctor is not gated on a
magical 0xa7e-count; it is gated on **one "App" service being registered**, and that
registration can only happen inside the real app-start session (no static writer exists).
This is byte+scan pinned on the real image, narrowing the SESSION-CTOR target from
"populate ~2694 services" to "get app-start to register the 'App' service", with the
fast-path return @0x61e32cc as the concrete measurement hook for when that happens.

Honest: does NOT manufacture a DM; Route-B live-DM structural gate UNCHANGED; SH174
capture-latch stays the single forward hook.

## Verify / files

- New hermetic `sh313_registry_no_static_writer_fastpath_needs_one_app` (elfjit.rs
  real-image guard, 3 byte-pins): ctor fast-path `cbnz x0` @0x106_1e3124 = 0xb5000d40,
  fast-tail `mov x0,x25;ret` @0x106_1e32cc = 0xf00067a0, fallback count++ str [x27,#3848]
  @0x106_1e32c0 = 0xb90f0b68. The "App" rodata (file 0x2d34ab) is noted in the comment
  (the container transform through host_addr_of is loader-dependent, not asserted).
- cargo test --workspace EXIT 0 (elfjit examples 138/0 = 137 + sh313; lib 405/0).
- elfjit.rs under the 1MB pre-commit hook (+405 B margin, funded by condensing verbose
  SH/doc prose — all addresses preserved).
- Commit: local `dev` only (operator pushes).