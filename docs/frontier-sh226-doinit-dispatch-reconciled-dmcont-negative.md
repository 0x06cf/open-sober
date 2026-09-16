# SH226 — do-init DM-construction dispatch chain AUTHORITATIVELY reconciled (SH225 byte-correct) + DMCONT continuation MEASURED negative at HEAD

Status: implementation + verification on the real binary (+1 hermetic sh226, elfjit example
74/0). Frontier: Route-B (single-agent). No production path edited — this cycle (a) settles the
SH225-vs-SH156 dispatch decode discrepancy with a fresh authoritative decode and pins the full
provenance chain, and (b) empirically closes the operator's named "continueAfterFlagsLoaded_ ->
app-shell" re-attack lever (SH165-fwd "Next item 2") as a MEASURED not-reached-headlessly
negative, rather than the prior "routed-but-unverified" guess.

## 1. Why

Two conflicting records existed for the SAME do-init closure-build dispatch:

- SH156 (recon-sh156-startluaappdm-postdoinit.md §2, "MATCH 0x2206db8") decoded the dispatch as
  `ldr x0,[x19,#32]` -> `x8=[x0]` -> `x1=[x8,#48](table[+0x30])` -> `br` into 0x1023eff4c.
- SH225 (frontier-sh225) freshly re-decoded it as `ldr x0,[x19,#4]` (the binder) -> `x8=[x0]`
  (vtable) -> `x1=[x8,#0x30]` (vt+0x30) -> `br x1`, with NULL binder -> `cbz` -> benign soft-return
  Ok(0x3e8).

These disagree on the LOAD OFFSET, the SEMANTICS (union table vs binder vtable), and the resulting
branch. Getting this wrong matters: SH225's contract is the byte-anchored target for fabricating a
binder to advance do-init, while SH156's made do-init appear to already reach the governor
independently of any binder. This cycle resolves it with a fresh decode of the real .so and pins
the complete controller chain so a future drive starts from one correct map.

## 2. Measured (fresh authoritative decode of the real libroblox.so; guest = file + 0x100000000)

### The closure-build dispatch (SH225 byte-CORRECT)
```
0x102206db8 (closure-build):
  0x102206dd0  aa0103f3   mov x19, x1            ; x19 = closure-build arg1
  0x102206df4  f9401260   ldr x0, [x19, #4]      ; binder = [arg1 + 4]   <-- NOT [x19,#32]
  0x102206df8  b4000560   cbz x0, ->0x102206ea4  ; NULL binder -> benign soft-return Ok(0x3e8)
  0x102206dfc  f9400008   ldr x8, [x0]           ; x8 = *binder = binder vtable
  0x102206e00  f9401901   ldr x1, [x8, #0x30]    ; x1 = vt+0x30 slot
  0x102206e24  d61f0020   br x1                  ; DM-construction dispatch to vt+0x30
```
SH225's `[x19,#4]` (imm 4, word 0xf9401260) is confirmed; SH156's `[x19,#32]` / `table[+0x30] ->
0x1023eff4c` was a mis-decode. (0x1023eff4c / the AppBridgeV2 governor remains a real reachable
post-do-init body the ladder DOES enter via a different mechanism — SH197; the mis-decode was only
about attributing it to THIS closure-build binder dispatch.)

### The provenance chain (newly pinned this cycle) — what makes the binder deterministic-NULL
```
StartLuaAppDM 0x1023efe2c (builds a 40-byte stack union):
  0x1023efe98  910003f4   mov x20, sp                  ; x20 = union base
  0x1023efe9c  f90003e8   str x8, [sp]                 ; [sp+0]  = 0x635dd68 table addr
  0x1023efea0  f90013f4   str x20,[sp,#32]             ; [sp+32] = sp (stack self-ref)
  0x1023efeac  941efc10   bl 0x102baeeec (dispatcher)  ; x0=sp (union), w1=0
dispatcher 0x102baeeec:
  0x102baef04  2a0103f4   mov w20, w1
  0x102baef08  aa0003f3   mov x19, x0                  ; x19 = union base
  0x102baef54  aa1303e1   mov x1, x19                  ; x1 = union      (do-init arg1)
  0x102baef58  f9400500   ldr x0, [x8, #8]             ; gov "this"
  0x102baef6c  aa1f03e2   mov w2, wzr                  ; (w20 bit0==0 path)
  0x102baef70  97d95f34   bl 0x102206c40 (do-init)
do-init 0x102206c40:
  0x102206c5c  aa0203f3   mov x19, x2
  0x102206c60  aa0103f4   mov x20, x1                  ; x20 = union   (do-init arg1)
  0x102206cd4  aa1403e1   mov x1, x20                  ; closure-build arg1 = union
  0x102206cd8  aa1303e2   mov x2, x19
  0x102206cdc  94000037   bl 0x102206db8 (closure-build)
closure-build:
  0x102206dd0  mov x19, x1                              ; x19 = union
  0x102206df4  ldr x0, [x19, #4]                        ; binder = [union + 4]
```
So binder = `[closure-build arg1 + 4]` = `[do-init arg1 + 4]` = `[StartLuaAppDM union + 4]`. The
union is {[+0]=table, [+8..24]=0, [+32]=sp}; `[union+4]` reads bytes 4-11 = high-half of the
<2^32 table pointer (=0) coalesced with `[union+8]` (=0) => **binder == 0 deterministically** from
StartLuaAppDM's own construction. Whether the harness's ladder supplies a NONzero [union+8] (which
would be the "binder") determines whether this dispatch fires. SH197 measured the ladder DOES reach
0x1023eff4c/the governor, so at HEAD the binder (or the pathway through it) is non-zero in that
context — the point of the pin is that a future drive knows binder=[union+4] and that filling
[union+8] is the mechanism, not re-guessing the offset.

## 3. DMCONT continuation — MEASURED NEGATIVE at HEAD (converts a guess into a measured result)

The operator's re-attack lever "continueAfterFlagsLoaded_ -> app-shell ctor" (SH165-fwd "Next
item 2") was already BUILT as JIT_ROUTEB_DMCONT: `routeb_dm_manager_cont` (jit.rs) sizes M to 0x260,
builds flags-holder F (0x310), and routes `vt[+0x1f0]` to the REAL continueAfterFlagsLoaded_
(guest 0x102bd1d68) so 0x102bd8ce8's dispatch actually runs it to nativeAppBridgeAppStart. It was
speculated ("Expected empirical floor") that this reaches app-start. Measured this cycle:

```
[routeb-dmforce] SH165 manager singleton holder ... -> continuation-routed manager ... (vt[+0x1f0]
  = REAL continueAfterFlagsLoaded_ (0x102bd1d68)) at pc=0x102bd1b98 ...
[elfjit:v2boot] ... StartLuaAppDM Ok(0x3e8) ... ladder done
EXIT=124, 0 SIGSEGV/0 SIGABRT
JIT_REGION_WATCH=0x102bd1d68-0x102bd2600  =>  0 region hits   <-- continueAfterFlagsLoaded_ NEVER ENTERED
```
The guard fires (M routed to real 0x102bd1d68) but continueAfterFlagsLoaded_ is never entered on
the completing ladder. Reason (from the fresh decode of the engine-init dispatcher 0x102bd8ce8):
the +0xf8/+0x108 vt slots (network feature-flag fetch) are benign leaves that return no flags; the
pipeline takes the benign path (0x102bd8dac 2nd-frame -> calls 0x102bd9058 soft-return) and never
reaches the `blr` at 0x102bd8e28 (ldr x8,[x8,#0x1f0]; blr x8) that would dispatch vt[+0x1f0]. So
routing the flag-completion slot to the REAL continuation is required-but-insufficient: the
continuation is gated behind a genuine flags-loaded state that a fabricated all-leaf manager never
produces headlessly. This is a MEASURED negative (region-watch proved 0 hits), not a guess.

## 4. Honest boundary (do-not-over-claim)

- This cycle is recon + a regression pin + a measured negative. It does NOT fabricate a binder that
  fires the do-init dispatch, does NOT drive continueAfterFlagsLoaded_ (its +0xf8/+0x108 network
  feature-flag state is not headless-producible), and does NOT produce a live DataModel / GuiObject.
- The DMCONT lever is confirmed WIRED + ARMED (guard fires, M routed to the real continuation) but
  LATENT: it fires the instant a real flags-loaded state emerges, exactly as SH165-fwd's honest
  residual predicted.
- Route-B live-DM structural gate UNCHANGED (SH209/218/223). This cycle removes the last
  "routed-but-unverified" open item on the continueAfterFlagsLoaded_ lever with a measured result.

## 5. Code

- `crates/arm64jit/examples/elfjit.rs` hermetic `sh226_doinit_binder_dispatch_chain_reconciled`
  (real-image guard family as sh225/sh224/sh223): byte-pins the 14-word controller chain
  (StartLuaAppDM union-build + bl dispatcher, dispatcher -> do-init, do-init -> closure-build), the
  two decisive close-build dispatch words, a hard assert that binder load is [x19,#4] (imm 4, NOT
  SH156's imm 32), the 5 DMCONT continuation anchors (continueAfterFlagsLoaded_ prologue/state,
  engine-init dispatcher vt+0x1f0 load + blr, 2nd-frame), and alignment.
- `crates/arm64jit/examples/doinit_dispatch_reconcile.rs` (repro tool; prints the resolved disasm
  of the whole chain incl. continueAfterFlagsLoaded_ + engine-init dispatcher).
- `runs/capture_sh226_dmcont_negative.sh` (repro: sh226 pin + the DMCONT measured negative).

## 6. Verify

- `cargo test -p arm64jit --example elfjit sh226` = 1 passed (real-image guard, 0.08s).
- `cargo test --example elfjit` = 74 passed / 0 failed (was 73; +1 sh226).
- `cargo test --workspace` green.
- Repro (run/capture_sh226_dmcont_negative.sh): manager-cont guard fires, 0 region hits in
  0x102bd1d68-0x102bd2600, EXIT 124, 0 crash.

## 7. Next (honest, single-agent)

Standing Route-B structural gate UNCHANGED. The do-init binder dispatch is now a pinned
fabrication target ([union+4], non-NULL goes to vt+0x30) and the continueAfterFlagsLoaded_ lever is
measured-negative on the headless flags state. The unblocked route forward stays the live-DM
world-build (SH174 capture-latch at a real make_shared<DataModel> = the single forward hook) or a
real flags-loaded engine-init state — neither is a headless seed. recon-v3 render plane + SH210
wiring + cookie persistence stay as before, unregressed at HEAD.