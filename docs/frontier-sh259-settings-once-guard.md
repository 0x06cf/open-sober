# Frontier SH259 — seed the settings/registry factory once-guard to exit the deepest app-start reach

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed — no subagents).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. Workspace green, no default-config production path edited (guard is
opt-in env-gated + pc-gated, default-inert).

## The gate (measured, fresh at THIS HEAD)

SH258 (bd430ed) recorded the DEEPEST app-start reach ever: with the FULL combined
SH248c-f seed set, nativeAppBridgeAppStart's deep orchestrator fn 0x2339d0c
region-hits 0x102339d44 and then EXIT 134 at the standing live-object map wall
0x1021dde34 (the map-`this` 0x100548ca9 is inside the write-off R-E exec segment —
SH249). Region-watch showed the largest pc BEFORE the wall was 0x102339d44.

Fresh disasm this cycle isolates exactly what 0x2339d44 does:

```
0x2339d40  bl 0x22082d8                      ; (gameGlobalInit sub)
0x2339d44  bl 0x21dac2c   <-- DEEPEST REACH  ; settings/registry singleton factory
0x2339d48  ldrb w8,[x0,#3]                   ; (returns >> reads the registry flags)
```

Disasm of 0x21dac2c (the `bl` target):

```
0x21dac2c  stp x29,x30,[sp,#-16]!            ; entry
0x21dac34  adrp x8, 6a6f000
0x21dac38  add  x8, x8, #0x430               ; x8 = 0x6a6f430  (once-guard cell)
0x21dac3c  ldar w8,[x8]                      ; acquire-load the once-guard
0x21dac40  tbz  w8,#0, 0x21dac54             ; bit0 clear -> builder path
0x21dac44  adrp x0, 6a6f000                  ; (bit0 set)
0x21dac48  add  x0, x0, #0x3f0               ; x0 = 0x6a6f3f0 (registry object base)
0x21dac4c  ... ret                           ; EARLY-RETURN the (zeroed) registry
0x21dac54  (builder path) bl __call_once 0x284ce54 ; then bl 0x21dac80 ...
```

When bit0 is CLEAR (headless first call), 0x21dac2c falls into the builder path
which constructs the registry object/run-time singletons; per the region-watch key
a region hit at 0x102339d44, control then dives into the stride-0x2a0 live-object
map-construction chain (0x21ddc44 -> 0x21ddcac) = the SH174/SH204 wall, and the
ladder EXIT 134s at 0x1021dde34 without 0x2339d48 ever firing again.

## The lever (this cycle, opt-in JIT_ROUTEB_APPSART_SETTINGS_ONCE)

`routeb_appstart_settings_once_seed_guard` fires at block-entry
[0x102339d40,0x102339d4c) (immediately before the deepest `bl`) and ORs bit0 of
the once-guard cell [0x106a6f430] = 1. With bit0 set, 0x21dac2c takes its
early-return (`adrp+add #0x3f0; ret`) instead of the builder path, so control
returns to the orchestrator at 0x2339d48 and lets it walk its OWN real app-start
body — 0x233a804 / 0x233af10 / 0x233bbac / 0x233bf20 / 0x233d11c / 0x233d2bc /
0x233d2ec — a fresh Path-B surface never before reached headlessly (those bls
register the app with the app-bridge / surface / flags in the real flow).

Unlike SH248e (which seeds the -1 CELL pointed to by [0x106b0bdf0]) this seeds the
once-guard FLAG byte itself at a NEW cell [0x106a6f430] — the SH156 "flags-latch"
pattern applied to the settings/registry singleton factory. All 8 prior SH258
closures were at OTHER cells (jar 0x106ed7a20/28, once 0x106b0bdf0, adapter
0x106b0bde0, map-root 0x1067d16f0=canary, count-clamp, source-vector 0x106dca0ea8).

## Honest (do-not-over-claim)

- Does NOT manufacture a DataModel and does NOT lift the Route-B live-DM
  structural gate. Seeding the registry once-guard only exits the factory's own
  builder; whether the orchestrator then self-registers cleanly or faults at a NEW
  fencepost (live object / content) is the measurement that decides.
- This is a forward lever (never-tried cell + fresh surface), default-inert,
  regression-pinned. It is NOT a proof-of-dead-end and NOT a manufacture.

## Artefacts

- Code: crates/arm64jit/src/jit.rs — `routeb_appstart_settings_once_seed_guard`
  (+ dispatch wiring after the SH248f adapter guard).
- Hermetic: `sh259_settings_factory_once_guard_endpoint` (real-image guard family,
  skip-if-absent) pins the factory's 7 prologue words + the once-guard cell maps
  into the image.
- Repro: runs/capture_sh259_settings_once.sh (watch the fresh region
  [0x10233a000,0x102350000) for the orchestrator body bls + whether a NEW
  termination pc appears instead of 0x1021dde34).
- Verify: cargo test --workspace green; cargo test -p arm64jit --example elfjit
  sh259 = 1 passed.

## Route-B standing (unchanged)

Route-B live-DM = structural gate. SH174 capture-latch (arm *(0x106391908) at a
real make_shared<DataModel>) stays the single forward hook. recon-v3 deliverables
(type4 self-driven frames + json zero-fix) stay shipped + verified.

## Measured result (A/B, real libroblox.so, fresh at HEAD)

SEED FIRES + TERMINATION MOVED OFF THE STANDING WALL. 3/3 runs (EXIT 134) with
JIT_ROUTEB_APPSART_SETTINGS_ONCE=1 on top of the full SH258 seed set: the
settings/registry once-guard [0x106a6f430].bit0 is seeded (log line
`[routeb-sh259] seeded ... at pc=0x102339d0c`), and the orchestrator 0x2339d0c
region-hits 0x102339d44 *and* 0x102339d48/0x102339d54 (the instructions AFTER the
deepest `bl` that never fired in SH258), then walks its OWN app-start body:
60+ brand-new block-entry pcs in [0x10233a000,0x102350000) — fns 0x233a804,
0x233a84c, 0x233a884 ... 0x233ae38, then 0x102346f2 ... 0x1023474b — comprising
the real app-register/sub-step surface SH258 predicted was behind the wall. The
STANDING map-wall crash `guestpc=0x1021dde34` is **gone** (0 occurrences as the
terminal this run; the full-seed SH258 baseline terminated there 3/3).

NEW termination (fresh, previously headless-unreached): `SIGSEGV guestpc=0x101db1d04`
= fn 0x1db1cc8 (`mov x20,x3` entry; the 0x1db1d00 `bl 0x1db1d8c` return-check
`cbz x0` at 0x1db1d04) — the LocalStorageManager init / persistence map world
(global map ptr loaded at 0x1db1d14 `adrp x8,726f000; ldr x8,[x8,#2240]` =
[0x106a6f8c0], still NULL headlessly; the map-insert reads it via
`ldar x9,[x9]`). This is a DIFFERENT subsystem — the data-store line (objective
2b), reached for the first time headlessly because the orchestrator finally got
past the settings/registry factory's own builder. Register snapshot at the new
wall: x19=host-heap, x20=0x0, x8=0x10, x9=0x7f..9010 (map global), x0=0x2.

HONEST advance: does NOT manufacture a DataModel and does NOT lift the Route-B
live-DM structural gate (SH174 capture-latch UNCHANGED). What IS new + measured:
the app-start orchestrator's deepest-gate closure (the settings/registry factory
once-guard) is cleared and the real client now executes a ~60-block body of its
own app-start registration (0x233a804..0x233ae38, 0x102346f2..) it never reached
in 9 prior cycles — and dies instead at LocalStorageManager's map (a seedable
fixed-.bss-global-shaped expression at [0x106a6f8c0], next CANDIDATE to try:
seed a coherent empty lsm map there, same family as SH248d's jar seed).

## Post-measurement classification of the NEW wall (doc addendum)

The next angle "seed LocalStorageManager's map global [0x106a6f8c0]" was examined
and CLASSIFIED, not chased free-form:
- fn 0x1db1cc8 (crash fp) is INDIRECT-DISPATCH ONLY — 0 direct `bl 0x1db1cc8`
  callers over the whole .text (the fn the app-start body reaches via a
  loader-relocated/function-pointer slot). Its crash is the map-insert at
  `1db1d08 adrp x8,726f000 / 1db1d14 ldr x8,[x8,#2240]` reading the map base
  global [0x10726f8c0] (vaddr 0x726f8c0, in the RW seg [0x67d67c0,0x7333c3c),
  so it IS a writable fixed-.bss cell — seedable-by-fixed-global in principle).
- BUT the crash register snapshot shows the deref'd map base x9 =
  0x7fe2eb8f9010 = a HOST-HEAP pointer (top 16 bits 0x7fe2), NOT 0 — i.e. the
  LocalStorageManager map HEADER has already been constructed (host-heap) and its
  backing bucket array is garbage/never-built. That is the SH174/SH204/248g
  live-object family (a real session DM ctor must build the backing), NOT a
  NULL-fixed-global that a fixed-.bss seed can satisfy. A seed would have to
  fabricate the whole host-heap map + buckets at an address the engine later
  reads — the same live-object structural gate, now at the persistence layer.
- HONEST conclusion: LocalStorageManager init is the NEW terminal face of the
  SAME live-DM gate (SH174 latch = real make_shared<DataModel> still the single
  forward hook). The SH259 VALUE is upstream: the settings-once seed cleared the
  deepest app-start gate and proved the orchestrator+app-registration body
  (0x233a804..0x233ae38) executes headlessly — a measure, not a dead-end.