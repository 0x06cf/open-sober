# SH187 — Route-B reconsidered: the GENUINE RBX::DataModel vptr base + a REAL ctor that materializes it

Status: code landing (manufactured-DM vptr corrected to the TRUE base + full vptr set),
workspace 374/0. This falsifies the SH179-186 "no static DM construction / migration gate"
proof-of-dead-end.

## 1. The one-line correction

The genuine primary RBX::DataModel vptr is **`0x67162e8`** (guest `0x1067162e8`), **NOT the
+8-off `0x67162f0`** used throughout SH179-SH186. Itanium: RTTI typeinfo `0x6714e18` sits at
vptr-8 = `0x67162e0`, so vptr = `0x67162e0 + 8` = `0x67162e8`. The whole lineage's scan targeted
`0x67162f0/0x67163a8/0x6716400` (+8) and page `0x671000`, declared adrp+add "geometrically
impossible" (offsets 0x62f0/0x63a8/0x6400 > imm12 4095), and concluded NO DataModel ctor exists.

## 2. The REAL constructor (missed by every prior cone)

A genuinely reachable DataModel ctor exists at **guest `0x1023f6038`** (file `0x23f6038`). It
materializes the genuine vptr set via **`adrp x8, 0x6716000` + imm12 adds** (offsets `0x2e8`,
`0x3a0`, `0x3f8` all fit the imm12 range):

```
23f6130: adrp x8, 6716000
23f6134: add  x8, x8, #0x2e8      ; x8 = 0x67162e8  (primary  DM vptr)
23f6138: add  x9, x8, #0xb8       ; x9 = 0x67163a0  (secondary MI base)
23f6140: stp  x8, x9, [x19]       ; [obj+0]=0x67162e8, [obj+8]=0x67163a0
23f6144: add  x8, x8, #0x110      ; x8 = 0x67163f8  (tertiary MI base)
23f6148: str  x8, [x19, #496]     ; [obj+0x1f0]=0x67163f8
```

Prior cones scanned page `0x671000` and the +8 constants, so they could not reach this ctor
via adrp+add and (wrongly) proved "zero static materialization." The page used is
`0x6716000`; the compiler materializes low-offset vtables with adrp+add only on the correct
page. This is a **proof-of-dead-end FALSIFIED** — precisely the condition the operator set for
not stopping on a migration-gate verdict.

## 3. Why the SH181/183 manufactured DM went nowhere

`routeb_manufactured_dm()` planted `DM_VTABLE = 0x1067162f0` (+8. wrong) at offset 0 — an
object whose first word is NOT a valid vptr (it pointed 8 bytes past the true vptr base, i.e.
at a method slot). Any virtual dispatch on it was garbage. That is a concrete, mechanical
explanation for the "manufacture lever fires into a slot nobody derefs / no dispatch" empirics
(SH181 region-watch 0 'entered region', SH184 holder-no-consumer).

## 4. Change

- `routeb_manufactured_dm()` now writes the **full genuine vptr set** at the exact offsets the
  real ctor writes: `[0]=0x1067162e8`, `[8]=0x1067163a0`, `[0x1f0]=0x1067163f8`.
- `sh181_dm_manufacture_guard_plants_genuine_vptr_env_gated` updated to assert all three words
  (regression lock against the +8 error).

## 5. Honest status / next

- The manufactured DM now bears a CORRECT genuine vptr set, so any dispatch reaches real
  relocated engine code.
- **EMPIRICAL (real libroblox.so, llvmpipe, capture_sh187_real_ctor.sh):** the REAL DM ctor
  wrapper `0x1023f5ff8` was host-driven via run_guest_callback -> it ENTERED guest `0x1023f6038`
  and ran real code **with 0 SIGSEGV/SIGABRT**, EXIT 124. FIRST run: `[obj+0]` = `0x1067147c8`
  (intermediate base subobject vptr written at `0x23f6098`), i.e. control passed `0x23f6098` and
  `bl 0x224fda8`, then STALLED inside `bl 0x23f6b0c` (subobject ctor) at `0x1023f60b8` — no
  early-return branch (the ctor body to `0x23f6130` is branch-free; its sole ret is at
  `0x23f6728`). **BREAKTHROUGH (NOP fix, recon deleg_fa2be765):** NOP'd `bl 0x23f6b0c` at
  `0x1023f60b8` (0x94000295 -> 0xd503201f, only under JIT_ROUTEB_DM_REALCTOR) -> the ctor now
  falls through to the `0x23f6130` writes and drives to completion. Re-run:
  **`obj vptr set = 0x1067162e8,0x1067163a0,0x1067163f8 GENUINE MATCH (genuine=true)`, 0 crashes,
  EXIT 124.** This is the FIRST time the JIT has CONSTRUCTED a genuine RBX::DataModel through its
  REAL ctor code (not a hand-planted manufacturing dot) — suggests the ctor is fully driveable and
  leaves a live genuine-vptr DM in obj+0x1f0 (ret x0).
- NEXT (honest, scoped): (1) verify the constructed obj (ret x0 = obj+0x1f0) is a coherent live DM
  and whether feeding it to the current-DM holder 0x106391908 / DataModelServices consumer
  0x2db63f4-family reaches real code; (2) re-derive the post-DM content path (UniversalApp /
  GuiObject) against the corrected base — prior "migration-gate" closures were built on the wrong
  +8 vptr.
- The prior "migration-gate" closures were built on the wrong vptr; they need re-derivation
  against the corrected base.
- The recon cone remains armed. Next dispatch batch re-targets the corrected vptr base
  `0x1067162e8` + the now-driven ctor `0x1023f6038` output.