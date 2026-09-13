# SH106 — stack-smash root-caused: unseeded pervasive `__stack_chk_guard` GOT slot (0x67d16f0)

## Finding

The SH104/105 "host-pointer leak into the guest stack canary slot" stack-smash
(`*** stack smashing detected ***`, glibc `__stack_chk_fail`, guestpc=0x0, on the
`--v2boot` ladder thread) has TWO distinct layers.

### Layer 1 (fixed in SH106): the pervasive canary GOT slot was never seeded.

The whole binary's stack-protected functions (48,831 `adrp xN, 67d1000; ldr
xN,[xN,#1776]` references = guest GOT slot **0x1067d16f0**) read their
`__stack_chk_guard` base from that slot. `plt::patch_stack_canary` ONLY patched
`0x631aa30` (deduced from JNI_OnLoad's prologue). So the pervasive slot was left
unresolved and held whatever the harness put there — in our runs a MUTABLE
pointer (the engine's render-ctx singleton, the very slot `--renderthunk`
publishes into: `[elfjit:renderinit] ctx 0x1067d16f0=0x...`). A canary function
that runs across that mutation (prologue `ldr x8,[x22]; stur x8,[x29,-16]` then a
call that re-publishes the ctx, then epilogue `ldr x8,[x22]`) reads a DIFFERENT
"canary" at prologue vs epilogue -> false `__stack_chk_fail`.

**Fix (plt.rs):** seed BOTH canary GOT slots (0x631aa30 AND 0x67d16f0) with the
same stable canary address (libc's real `__stack_chk_guard`, or the process-static
0x2f_2a_1a_0a_0e_0f_10_11 fallback), forcing the pervasive slot even when it
already holds a page pointer (its pre-existing value is the bug, not a bound
canary). Hoisted the `static CANARY` + resolve-once OUTSIDE the slot loop so both
slots point at the same variable (previously the loop-local static + `&canary_val`
would have addressed the local and produced two different canaries).

**Verified:** `[plt] patched __stack_chk_guard GOT 0x631aa30 ... -> <X>` AND
`0x67d16f0 ... -> <X>` both print with the SAME `<X>`. Productized baseline (no
`--v2boot`) UNREGRESSED: exit 124, 0 crash, persist byte-exact, frame-fn +
present swap Ok(0x1). Workspace **518/0**.

### Layer 2 (NOT yet fixed): a store still writes x29+0x30 into the canary slot.

With the guard now stable, the JIT_DUMP_PC probe at the canary fn epilogue
(0x102dae368) shows:
```
x22=0x5641e7455ca0 (== guardGOT, correct & stable now)
canarywin[..x29-0x10]=0x5642043fce50   (= x29+0x30, sp+0x180)
canarywin[..x29-0x08]=0x101d99ff0      (guest code addr)
guardGOT[0x1067d16f0]=0x5641e7455ca0 guardval=<same>  # guard now coherent
```
i.e. the prologue stored the CORRECT canary, but between prologue and epilogue a
store wrote `x29+0x30` (== **sp+0x180 = the address of the x24/x23 callee-save
slot**) into `[x29-16]` (the canary slot = sp+0x140). So a GUEST store computed
`&frame_local_at_x29+0x30` and landed it on the canary slot — a mis-targeted
store (register-indirect base, or a nested-frame x29 mixup), NOT a guard-value
problem. Recon (deleg_90bf59c2) confirmed the canary fn 0x2dadb2c has only ONE
fixed store to [x29,-16] (the prologue) and computes no `x29+0x30`, so the writer
is elsewhere (likely a nested call in the do-init that runs with the outer x29 but
writes to its own computed slot, or a register-indirect store whose base got the
canary-slot address).

**NEXT:** put a store-watch in the translator (`Inst::LdStrImm`/`LdStPair`
non-`ld` paths, and the register-offset store) keyed on the observed target
`[x29-16]` / self-stack-pointer class to name the exact guest `str`/`stp` writer,
or check whether the do-init's nested `nativeGameGlobalInit` call (0x2219170)
frame overlaps the outer frame's canary slot (its own canary is at `[x29-8]`,
0x40-byte frame).

## Repro
`timeout 60 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500
  JIT_ROUTEB_HASHFIX=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so
  0x2173ff4 --jni --startapp 0x258b144 --v2boot --renderinit 0x105b3a280 --renderthunk
  --renderframe --renderframe-drive --renderframe-seedgles --persist-roundtrip --kicker 0x106863af8`
Exit 134 (was stack-smash before; still exits 134 via layer-2 writer).
Probes: `JIT_DUMP_PC=0x102dae368` (epilogue), `=0x102dadb2c` (entry).