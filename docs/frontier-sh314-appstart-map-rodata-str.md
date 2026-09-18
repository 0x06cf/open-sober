# SH314 — app-start map wall: the container is an address OF A RODATA STRING (live-object, do-not-re-tread)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh314` (real-image, 8 pins). elfjit examples 139/0.
Route-B live-DM structural gate UNCHANGED (no DM, MH_* false). Do-not-re-tread the "seed/fix the app-start
walk map" and "latch the once-guard to reach do-init's done-path" angles — measured-closed below.

## 1. Standing wall (re-derived at HEAD, 441d2bf)

Full-seed DMCONT ladder (SH248c-f + SH245 + SH257, `runs/capture_sh258_postgovtail.sh`) terminates 3/3 at
`SIGSEGV guestpc=0x1021dde34` with x20=x21=0x100548ca9 (in image), x19=0x40c29c7e746e86a1 (garbage hash),
x22=0x11, x23=0x2, lr=0x1021dde34. Same wall as SH248g/h/249. Fault instr `ldr x23,[x21,#8]` where `x21`
is the map container header.

## 2. NEW: what 0x100548ca9 actually is (the closure)

File offset 0x548ca9 (guest 0x100548ca9) is RODATA — a string region:
`...Id\0assetTypeId\0avatar_load_start\0texture...` (measured byte dump: `72496400617373657454797065...`).
So the map-container pointer is neither free-heap garbage nor an unapplied relocation addend — it is the
**address of an in-image RODATA string constant** that was type-punned into a slot the walk later reads as a
live map object. Cause: an upstream init (a real session/Activity ctor) that normally clears/constructs that
slot never ran headlessly. This is the SH174/204/248g live-object class. There is no fixed .bss cell to seed,
no count to clamp, and no repair that can make a string address a valid map.

## 3. The 5 STATIC walker callers are benign (proved not-the-crash-source)

The map-build helper fn 0x21ddbc8 (5 direct `bl` callers: 0x21dcd20/0x21dcffc/0x21dd178/0x21dd40c/0x21dd558)
is entered by the crash, but all 5 STATIC callers build into **stack** containers from a static descriptor
table:
- table adrp `0x66e7000` (x8) at 0x21dccf8, memcpy'd into the frame
- `x0 = sp+0xbd0+0x78` (stack container), `x1 = sp+0xa70`, `w2 = 0x1` (count=1) (0x21dcd14/1c/1d)

The crash register evidence (x22=0x11) does NOT match count=1, so the crash is a *dynamic* app-start map
build with a live/garbage container, not these static initializers. => no count-clamp on the static walk and
no fixed-cell seed can cross 0x1021dde34.

## 4. Measured: latching the once-guard cannot reach do-init's done-path (SH311's 0x2206c88)

Per SH311, do-init's done-path (0x2206c88) is NEVER entered; only the ONCE-lambda (0x2206d10) runs and stores
the ctor return (NULL headlessly -> empty service registry, SH312/313). To read a seeded DM-root, the once-guard
[0x106a68410].bit0 must be latched so the lambda is skipped. Measured (runs/sh314-donepath-full.txt,
full ladder + `JIT_ROUTEB_DM_SEED=1` + once-guard latch): **DONEPATH-latch fires (guard set), yet the ONCE
path still runs (0x102206d10 hit) and done-path stays 0-hit** — a prior rung (the SH310 re-seed rung,
documented in `runs/capture_sh311_once_lambda.sh`) actively clears the once-guard before do-init. No seed on
the guard sticks headlessly. The app-shell ctor 0x102207b50 still executes its body to ~0x1022081b8 via the
existing DMCONT/SH239 genuine-root path (consistent with SH257 — not a new frontier), and the ladder still
terminates at the same 0x1021dde34 map wall.

## Verdict

The narrowed SH312/313 lever ("get app-start to register the single 'App' service so the DM-controller ctor
fast-path yields a live DM-root") is blocked one step upstream: the registration walk crashes at 0x1021dde34
with a map container that is a *rodata-string address* — a live data-type-mismatch from an upstream session
ctor that never runs headlessly. No JIT seed/repair/clamp crosses it. This is an honest measured closure, not
a claimed DM. Route-B live-DM structural gate UNCHANGED.

Repro: `cargo test -p arm64jit --example elfjit -- sh314`. Local evidence: runs/sh314-donepath-*.txt
(not committed). Standing ROUTE-B direction unchanged (SEP-17 SESSION-CTOR is the primary lever).