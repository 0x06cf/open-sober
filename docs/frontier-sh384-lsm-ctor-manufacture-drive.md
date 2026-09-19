# Frontier SH384 — DRIVE the GENUINE LocalStorageManager ctor (0x101db0dfc) and MANUFACTURE a real vtable-owning manager (SH383's named next-step implemented + measured)

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start (cargo test --workspace EXIT 0, 438/0 arm64jit lib) and at end (439/0 lib).

## Why this cycle
recon-v3 immediate-priority deliverables (self-driven frame + JSON-abort) are committed
and green at HEAD. SH383 byte-PINNED the GENUINE LocalStorageManager ctor 0x1db0dfc as
the MIGRATION-directive manufacture target and explicitly left "the drive itself" as the
NEXT step — every prior persistence cycle was a SKIP (SH348 leaf-ret, SH349/350
sub-call-skip) or a SEED (SH267/285 map/nodes); NONE had ever run the genuine ctor.
SH384 implements + MEASURES that drive.

## The drive (default-inert JIT_ROUTEB_LSM_CTOR_MANUFACTURE=1, scoped to StartLuaAppDM entry)
Disasm (real libroblox.so) showed the ctor is NULL-tolerant in exactly the places the
SH384 zeroed-coherent-container premise needs:
- OUTER ctor 0x1db0dfc: `ldr x0,[x1,#8]` (container[8] sub-object) then `cbz x0,0x50`
  @0x1db0e3c — a NULL sub-object takes a benign skip, no vt dispatch. It writes
  [this+0]=vt, [this+16]=container[16] 16-byte payload, [this+32]=container[32] word,
  then `bl 0x1db0748` (inner ctor) @0x1db0e84.
- INNER ctor 0x1db0748: re-reads `[x1+8]` `cbz x9,0x78` @0x1db0778 -> `mov x19,xzr`
  @0x1db07c0 (NULL sub -> x19=0, no dispatch). Its stack-canary check reads the same
  [x22] twice @0x1db076c and @0x1db0810 (self-consistent, passes WITHOUT a canary seed).
So the drive leaks a zeroed 0x100 `this` + a zeroed 0x40 `container` (container[8]=NULL
takes both benign paths, [16]=0 payload, [32]=0 word) and runs
`run_guest_callback(0x101db0dfc, [this, container, 0,...], tp)` — the guest addr is
file 0x1db0dfc inside the r-x text segment guest [0x100000000,0x1062d8190) where ELF
file-offset == vaddr (NOT guest base 0x100000000 + the old broken +0x20 arithmetic).

## MEASURED (real libroblox.so, completing ladder + JIT_ROUTEB_LSM_CTOR_MANUFACTURE, 2/3)
```
[routeb-lsm-ctor] SH384: GENUINE LocalStorageManager ctor 0x101db0dfc DROVE ok ret x0=0x0;
  [this]=vt 0x10635bf88 0x0 this+0x20=0x0 this+0x28=0x10635bfb8 this+0x30=0x10635bff8
  (in-image vt=true inner 0x101db0748 ran through real code)
```
The manufacture SUCCEEDS: the genuine ctor ran through real relocated engine code with a
coherent container and produced a real vtable-owning manager — [this+0]=0x10635bf88 =
vtable page 0x10635b000 + 0xf88, [this+0x28]=+0xfb8, [this+0x30]=+0xff8, the exact word
offsets the ctor's `add x8,x9,#0x30/#0x70` computes. `in-image vt=true`, inner ctor
0x101db0748 entered and returned through real code. This is the first time the genuine
LocalStorageManager ctor has ever been DRIVEN headlessly (SH383 only pinned it).

## What is genuinely new + honest
- New real-image hermetic `sh384_lsm_ctor_null_tolerant_drive_path_pinned` (arm64jit lib
  438->439) pins the exact NULL-tolerant words (outer cbz 0x1db0e3c, payload/word reads,
  inner cbz 0x1db0778 -> mov xzr 0x1db07c0, self-consistent canary reads) that make the
  zeroed-container drive premise anchor to the real binary.
- This IS a genuine manufacture: a real vtable-owning LocalStorageManager object was
  constructed by running the engine's own ctor code, not by skipping or map-seeding — the
  MIGRATION-directive line "manufacture the genuine-vptr object, drive its ctor world-build
  further, INSIDE this JIT".
- Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
  AppBridgeV2 0); the run still terminates at the standing SH285 persistence-lane wall
  guestpc=0x101db1b08 (the SH383 doc correctly framed this: the ctor drive manufactures
  the manager, but wiring the manufactured object INTO the lane so the SH285 reader consumes
  it is the FURTHER step, gated on a real `container` whose sub-object survives dispatch —
  structural, not a fixed-.bss seed). SH174 capture-latch stays the single forward observer.
- recon-v3 deliverables unaffected (frame-plane drive is a separate opt-in rung, unchanged).

## Do-not-re-tread (unchanged closures, they all still stand)
LSM skips (SH349/350/358/373), EC reader-gate (SH355/356/374), 0x258b5d8/SetInitParams
(SH362/375), window-attach real (SH367), ALooper (SH365), governor gates full-ladder
(SH379), -9 string/0x102b504e4 (SH380), map-header repair (SH248h), once-lambda store
seeding (SH381). SH384 ADDS: the run_guest_callback drive of the genuine ctor is now
MEASURED (manufactures a real vtable-owning manager); re-driving must be via this real
ctor path, not a new skip/seed. The SH285-lane wiring-into-the-lane step is still open.

## Files
- crates/arm64jit/src/jit.rs: +`routeb_lsm_ctor_manufacture_drive` (default-inert,
  JIT_ROUTEB_LSM_CTOR_MANUFACTURE=1) wired into the block-entry hook (the SH189 slot);
  +hermetic `sh384_lsm_ctor_null_tolerant_drive_path_pinned` (arm64jit lib 438->439).
  jit.rs 1,011,782 B (<1MiB hook). No production path / JIT hook default / guest byte
  changed (the drive is pure opt-in; default leap unchanged for the standard ladder).
- runs/capture_sh384_lsm_ctor_manufacture.sh (repro; log sh384-lsm-ctor-manufacture.txt
  outside repo).