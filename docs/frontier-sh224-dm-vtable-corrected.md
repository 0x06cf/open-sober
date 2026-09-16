# SH224 — genuine DataModel vtable rows re-derivation at the SH187-corrected base (reconciles SH186c's +8-slip "null-stub row" verdict)

Status: implemented + verified on the real binary (1 hermetic, elfjit example 72/0).
Frontier: Route-B — the operator's "keep driving the genuine-vptr DM / re-derive against
the corrected base" grind. No production path edited (pure corrected-base re-derivation
+ a real-image regression pin).

## Why
SH187 (frontier-sh187-dm-vptr-base-corrected.md) FALSIFIED the SH179-186 "no static DM
construction / migration gate" proof-of-dead-end: the genuine primary RBX::DataModel
vptr is **0x67162e8** (guest 0x1067162e8), NOT the +8-slipped 0x67162f0 used throughout
SH179-186. SH187 explicitly left an open NEXT: *"re-derive the post-DM content path /
GuiObject against the corrected base — prior 'migration-gate' closures were built on the
wrong +8 vptr."*

That re-derivation at the VTABLE-ROW level was not done since SH187. In particular
SH186c (frontier-sh186-routeb-fresh-reattack-jstring-slot.md, written BEFORE the SH187
correction) reached a load-bearing verdict on the +8 addresses: *"the SH181/182
manufacture lever used vt=0x67162f0 (the null-stub row) — which is why feeding it
through the cb is inert"* and classified primary slot-2 as 0x10229c2a4 (mov x0,xzr; ret).
That verdict is exactly the kind of +8-slip-derived closure the corrected base was meant
to re-check. This cycle does it with fresh decoded rows.

## Method
`crates/arm64jit/examples/dm_vtable_corrected.rs`: `load_elf_image` the real
libroblox.so (which applies the R_AARCH64_RELATIVE relocs to .data.rel.ro exactly as the
runtime does), `host_addr_of` each slot, and read all 16 slots of the three genuine DM
vtable rows at BOTH the corrected base (0x1067162e8 / 0x1067163a0 / 0x1067163f8) and the
+8-slipped base for contrast. Disassemble the notable dispatch targets with
`arm64jit::decode`.

## Measured (real libroblox.so; identical decode on both the robbox and android-env
## copies of the .so)

### Corrected primary row (base 0x1067162e8)
- **slot-2  @0x1067162f8 = 0x1057d19bc — a REAL method body** (prologue `sub sp,sp,64;
  stp x29,x30,[sp,16]; ... mov x20,x0` — stack frame + callee-saved, consumes x0). NOT
  the null-stub.
- slot-3  @0x106716300 = 0x10229c2a4 — **the null-stub** (mov x0,xzr; ret).
- slot-6  @0x106716318 = 0x1057d1b9c — vt+0x30 = tiny accessor (`ldr x8,[x0,#76];
  add x0,x8,#8; ret`).
- slot-7  @0x106716320 = 0x1057d6ef4 — **the SH182/187 "app-shell ctor"** (big frame:
  `sub sp,sp,640; adrp ...`). It sits at **vt+0x38**, not vt+0x30.
- slot-9  @0x106716330 = 0x103facf10 — the boolean registration-predicate family
  (SH189's create-path `blr [DM-vptr+0x1c0]` = 0x103facf10).

### Corrected secondary row (base 0x1067163a0, the MI-base subobject planted at [obj+8])
- slot-0  @0x1067163a0 = 0x101dcc83c
- **slot-1  @0x1067163a8 = 0x10240a8b8 — the SH186c "real DM consumer" thunk**
  (`add x0,x0,#0x758; b deep-body`). Confirms SH186c: this adjusts `this` and jumps into
  a deep body that derefs deep DM members — it dies on a shallow manufactured DM
  (SH182 PATH-B null-fault on a deep member), never producing a GuiObject.

### Tertiary row (base 0x1067163f8, planted at [obj+0x1f0])
- slot-1 @0x106716400 = 0x1057d07f8 — destructor path.
- slot-7 @0x106716430 = 0x10229c2a4 — null-stub again.

## Reconciliation (do-not-re-claim)
1. **SH186c's "manufacture primary row is a NULL-stub, hence inert" is REFUTED at the
   corrected base.** The corrected primary slot-2 is real DM method 0x1057d19bc;
   the null-stub 0x10229c2a4 SH186c placed at primary slot-2 is actually corrected slot-3.
   This is consistent with SH187c's earlier empirical finding that the corrected
   manufactured DM genuinely dispatches through a reachable consumer — it is NOT inert.
   Fresh body decode of slot-2 (16 instr): `add x0,x0,#0x1f0` (it walks into the DM's
   embedded tertiary subobject, the same +0x1f0 the ctor wires to 0x1067163f8), loads a
   GOT fn-ptr, `bl` a helper, then `and w8,w2,#255; cmp w8,#4` — a **command/opcode
   dispatcher**. Not a null-stub, and not a GuiObject factory: it dispatches on arg-2
   and forwards, does no Instance/scene construction of its own.
2. **SH187's own indexing carried a residual +8 slip:** 0x1057d6ef4 (the "app-shell
   ctor") is corrected-primary **vt+0x38 (slot-7)**, not vt+0x30. vt+0x30 is the accessor
   0x1057d1b9c. Any host path that dispatches `[DM_vt+0x30]` expecting the app-shell ctor
   would call the accessor. (SH182/187 drove 0x1057d6ef4 by direct address, so their
   *execution* empirics are unaffected — only the slot-index label is corrected.)
3. **No GuiObject producer exists at the corrected base either.** Slot-2 targets across
   all three rows are: a real DM method body (primary), a deep-member consumer thunk
   (secondary) that faults before GuiObject instantiation, and shared/destructor paths
   (tertiary). The create-path registration predicate (0x103facf10) is a boolean
   predicate, never a GuiObject factory (unchanged from SH189).
4. **Route-B no-GuiObject structural gate UNCHANGED** — but the closure is now the
   corrected-base re-derivation SH187 required, not a stale +8 verdict. This converts the
   SH186c "manufacture lever inert (null-stub)" sub-claim into the measured statement:
   *the corrected manufactured DM dispatches into real DM method code, but no DM vtable
   row produces a GuiObject, and the consumer row dies on a shallow-manufactured DM.*

## Code / repro
- `crates/arm64jit/examples/dm_vtable_corrected.rs` (repro tool; prints all 6 rows +
  dispatch-target disasm).
- Hermetic `sh224_dm_vtable_corrected_slots_pinned` (elfjit example test module,
  skip-if-absent real-image guard family as sh223/sh222/sh219/sh213): pins the six
  load-bearing corrected-base slots so a drift fails loudly.
- Repro: `cargo run --example dm_vtable_corrected --release -- \
  /home/hermes-worker/.cache/open-sober/robbox/libroblox.so`
  (also `runs/capture_sh224_dm_vtable.sh`).

## Verify
- `cargo test --example elfjit` = 72 passed / 0 failed (was 71; +1 sh224).
- `cargo test --workspace` = 567 passed / 0 failed (unchanged; sh224 is an example test).
- Tree clean at end of commit.