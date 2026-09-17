# SH248g — the post-SH248f app-start wall is a LIVE host-heap hash-map insert
# (register-level determination; NOT a fixed-.bss seed)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## Context (SH248f, this session)
SH248f seeded the app-lifecycle adapter at [0x106b0bde0] (all-leaf vtable object),
which cleared the SIGSEGV at 0x102339020 and advanced the DMCONT continuation ~0x500
deeper through nativeAppBridgeAppStart — region hits to 0x102339d44 — to a NEW fault:
`SIGSEGV guestpc=0x1021dde34 fault=0x0`. SH248f left open the question: is this a
seedable fixed-.bss singleton (SH243 lesson) or a live-member wall (SH174/SH204)?

## This cycle: register-level determination (real libroblox.so)
Ran the canonical SH248f repro with JIT_DUMP_PC=0x1021dde34 (full x-file at block
entry) + JIT_DUMP_REGION. Evidence:

1. The block entry IS 0x1021dde34 (mid-function — the hash-find is reached via a
   `blr` function-pointer call into the middle of the visitor, not its entry). Three
   clean entries then a fault.
2. Each iteration is a live INSERT into a host-heap hash-map:
   - x19 (hash) differs per iteration: 0x6bfdfab08a4be46a, 0x3e66f7c296aef7de,
     0x40c29c7e746e86a1 — distinct keys being inserted.
   - x21 (map `this`/bucket cursor) is a NEW per-run ASLR host-heap address each
     iteration (~0x559c69497...), x24 = stable map identity 0x559c2d41f2d0 (host heap).
   - x0/x8/x9/x20/x26 are all host-heap pointers (0x7f..., 0x559c...) — the guest is
     genuinely allocating + walking its own map (via the operator_new path).
3. The guest's own builder (file 0x21dde00 family, the JNI_OnLoad+0x69e.. region)
   does a robin-hood grow/compute (sub x8,x23,#1 / lsl / csel) then
   `ldr x23,[x21,#8]` — reading the map's element-count field. On iteration 4 the
   container header is ctor-uninitialized (garbage count), so the index computation
   (udiv/msub on the count) produces a bucket out of the map's real bounds -> the
   `ldr x26,[x8, x25, lsl #3]`/walk reads an unmapped slot -> fault addr 0x0.
4. The map object is ASLR host-heap and its fields are set by a ctor that never ran
   headlessly. There is NO fixed-.bss holder for the map `this`:
   - 0x1021dde00 has ZERO direct bl/b callers (reached only via blr into mid-function).
   - No in-image RELATIVE-addend / reloc references 0x21dde00 (not a vtable slot).
   - x1=0x1072757e0 is constant across iterations but it is a KEY/ARG pointer, not the
     dereferenced map: the reader's map `this` lives on the heap.

## Verdict (do-not-re-tread)
The x1=.bss constant was the classic "SH243 holder tell", but the register dump proves
it is a key/arg, not the map holder — the dereferenced container is a host-heap
live-object whose init ctor never completed headlessly. This is deterministically the
SH174/SH204 live-object construction class (the SAME gate reached from one gate deeper
in app-start), NOT a fixed-.bss seed. Seeding an empty-map header is not possible at a
stable address (ASLR). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch
(arming *(0x106391908) at a real make_shared<DataModel>) stays the single forward hook.

## What is new + measured this cycle
- Confirms SH248f's verdict with full register-file evidence and CLOSES the "maybe a
  seedable .bss holder" residual (SH243-style) with a determination.
- The DMCONT continuation's app-start execution now runs its own host-heap map
  construction (3+ successful inserts) before the live-object wall — the farthest
  app-start line measured yet.

## Code / files
- crates/arm64jit/examples/elfjit.rs: +hermetic `sh248g_appstart_hashfind_wall_anchored`
  (real-image byte-pins on the 5 wall anchors 0x21dde00/0x21dde30/0x21dde34/0x21ddea8/
  0x21dd800, skip-if-absent, guest=file+0x100000000 + 4-align + distinct + window).
  cargo build --workspace EXIT 0; examples 90/0; lib 397/0; recon-v3 re-verified green
  this session (24 task-driven frames, swap Ok(0x1)x5, 197 pops, no json abort).
- repro: runs/batch_sh248f_adapter_seed.sh (same DMCONT env).