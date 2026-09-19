# Frontier SH383 — pin the GENUINE LocalStorageManager constructor (0x1db0dfc) as the manufacture target the MIGRATION directive demands

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(re-confirmed at start: cargo test --workspace EXIT 0, 617/0; arm64jit lib 437/0;
elfjit example 160/0 before +sh383, now 161/0). recon-v3 self-driven-frame +
JSON deliverables re-verified green at HEAD this cycle (capture_taskv4_frame.sh
attempt 1: 24 real task-driven frames swap Ok(0x1), 197 node pops, 0 json abort,
0 crash).

## Why this cycle
recon-v3's concrete deliverables are committed + green (the type4_frame_thunk
self-driven frame plane and the JIT_JSON_ZERO_FIX len-clamp at 0x102355d40). The
Route-B live-DM structural gate stands (DM-root 0, MH_* false), and the standing
map says the persistence lane is the funnel: the do-init/DMCONT/session drives all
drain into the LocalStorageManager reader/pop wall `guestpc=0x101db1b08`
(fault=0xffffffffffffffff, [obj+0x50]=uninitialized string buffer).

The operator's MIGRATION NOT-STOPPING directive is explicit: "Keep manufacturing
the genuine-vptr DM, driving its ctor world-build (DMCONT / app-shell init)
further ... INSIDE this JIT". Every prior cycle that touched the LSM lane used a
*skip* (SH348 leaf-`ret` initStorageManagerNative) or a *seed* (SH267/SH269 map +
SH285 per-node cells). NO prior cycle byte-pinned the REAL LocalStorageManager
CONSTRUCTOR that manufactures a genuine vtable-owning manager object whose
[obj+0x50] buffer is actually initialized. That constructor is the missing
manufacture artifact; this cycle pins it so a future drive has the exact ABI.

## The pinned target (real libroblox.so)
Fn **file/guest 0x1db0dfc** (`initStorageManagerNative` inner region, a genuine
`sub sp,#0x40` member ctor). Verified words (real-image, all in-window + 4-aligned):
```
0x1db0dfc: a9bc7bfd  stp x29,x30,[sp,#-64]!          <- ctor entry
0x1db0e10: f0022d48  adrp x8,635b000                 <- outer vtable page
0x1db0e14: 9139a108  add  x8,#0xe68                  -> [this+48]=vt
0x1db0e2c: 913562f7  add  x23,#0xd58                 -> [this+0]=vt  (0x10635b000+0xd58)
0x1db0e34: f9000277  str  x23,[x19]                  <+> this vt store
0x1db0e20: f9400420  ldr  x0,[x1,#8]                 <- container sub (this=x0, container=x1)
0x1db0e38: f8008e80  str  x0,[x20,#8]!               <+> this+8
0x1db0e4c: b94022a8  ldr  w8,[x21,#32]               <- container word -> this+32
0x1db0e84: 97fffe31  bl   0x1db0748                  <- inner ctor (vt[+24] dispatch on [x1+8])
0x1db0e98: d65f03c0  ret
inner ctor 0x1db0748: sub sp,#0x40 (0xd10103ff)
SH285 leaf 0x1d9a15c: ldr w8,[x2]                      (the byte-copy that faults on [obj+0x50])
```
**ABI for a drive**: `run_guest_callback(0x1db0dfc, [this, container, 0,0,0,0,0,0], tp)`
with `this` = a leaked zeroed 0x60+ guest-visible buffer, `container` = an object
whose [x1+8] is a non-null sub-object with vt[+24] dispatchable (or the cbz path
@0x1db0e3c handles NULL), [x1+16] a 16-byte payload, [x1+32] a word. On return the
manager owns genuine vtables at [this+0]/[this+48] and the inner ctor has run —
manufacturing the SH285 reader's required object rather than skipping/repairing.

## Honest
Does NOT manufacture a DataModel; this cycle adds byte-anchored instrumentation
(a hermetic + this doc), NO guest mutation, NO production-path change. The drive
itself (manufacturing a coherent `container` whose sub-object ctor-internals survive
the vt dispatch, then wiring the result into the lane so the reader consumes it)
is the NEXT step and remains gated on the container sub-object's own invariants
(vt[+24] on [x1+8] must return a coherent manager) — i.e. this target is
structural, not a fixed-.bss seed. It is the concrete manufacture lever the
MIGRATION directive names, now pinned so it is no longer vague.

Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
AppBridgeV2 0). SH174 capture-latch stays the single forward observer. recon-v3
deliverables re-verified green.

## Do-not-re-tread
Unchanged closures stand: LSM skips (SH349/350/358/373), EC reader-gate
(SH355/356/374), 0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367),
ALooper (SH365), governor gates full-ladder (SH379), -9 string/0x102b504e4 (SH380),
map-header repair (SH248h), once-lambda store seeding (SH381). SH383 ADDS the
pinned genuine-ctor target above; re-driving it must be via the real ctor, not a
skip or a map/seed repair of the same lane.

## Files
- crates/arm64jit/examples/elfjit.rs: +hermetic `sh383_lsm_ctor_manufacture_target_pinned`
  (sh115_tests module, 161/0) — real-image byte pins on 0x1db0dfc + inner 0x1db0748
  + SH285 leaf 0x1d9a15c, skip-if-absent.
- No production path / JIT hook / guest byte touched.