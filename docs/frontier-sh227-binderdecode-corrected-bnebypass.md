# SH227 — do-init closure-build binder-dispatch DECODE CORRECTED (#32, not #4) + measured b.ne bypass (SR225/226 "next target" = measured dead-end)

Status: correction + regression pin, single-agent Route-B re-attack. +1 hermetic sh227
(elfjit example 75/0). frontier: Route-B. No production path edited — this cycle fixes a
decode error that the last two sessions propagated as "AUTHORITATIVE" and converts the
SH225/226 "fabricate a binder" next-target into a measured proof-of-dead-end on two grounds.

## 1. Why

SH225 and SH226 both pinned the GlobalInit do-init closure-build dispatch (guest
0x102206db8) as the next Route-B fabrication target. Their shared semantic decode was:

```
0x102206df4  ldr x0,[x19,#4]   (the "binder"/app-bridge object)
0x102206dfc  ldr x8,[x0]       (its vtable)
0x102206e00  ldr x1,[x8,#0x30] (vt+0x30)
0x102206e24  br x1             (DM-construction dispatch)
```

SH156 (the ORIGINAL decode, which SH225 "corrected") read that same load as
`ldr x0,[x19,#32]`. SH226 declared its `[x19,#4]` reading "AUTHORITATIVE / byte-for-byte
confirmed" and built the whole "deterministic-NULL binder at [union+4]; fill [union+8] to
fire it" mechanism on it. That mechanism is what the operator's "re-attack the
live-DataModel construction via do-init completion" directive would have driven next.

## 2. Measured (real libroblox.so; three independent authorities)

### (a) GNU objdump — the load is `[x19,#32]`, not `[x19,#4]`
```
2206df0: 540001c1  b.ne 2206e28     <-- thread-match gate (see (c))
2206df4: f9401260  ldr x0, [x19, #32]   <-- SH156 CORRECT; SH225/226 WRONG
2206df8: b4000560  cbz  x0, 2206ea4
2206dfc: f9400008  ldr  x8, [x0]
2206e00: f9401901  ldr  x1, [x8, #48]   (48 = 0x30)
2206e24: d61f0020  br   x1
```

### (b) arm64jit decode + translate — the AArch64 scale rule
`LdStrImm { rn:19, imm:4, size:8 }`, and translate.rs:1139 computes `address = rn +
imm*size` = `x19 + 4*8 = x19 + #32`. The imm12 field in an unsigned-immediate LDR is the
BYTE offset DIVIDED BY the access size; for a 64-bit (size=8) load, raw imm12 4 => offset
32. **SH225/226 applied the 32-bit size-4 scale** (imm12*4 = 16? no — they just read "4")
and printed "#4"; both the GNU disassembler and this codebase's own decoder compute #32.
SH156's `[x19,#32]` was correct all along; SH225 "corrected" a correct decode into an
error, and SH226 locked the error in.

### (c) Live region-watch on the completing ladder (3/3 clean runs, EXIT 124) — the dispatch NEVER executes
```
region hit at guest pc=0x102206db8   (closure-build entry)
region hit at guest pc=0x102206dec   (cmp x0,x20 = pthread_self vs stored main-id)
region hit at guest pc=0x102206e34   (NON-MATCH path: LocalStorageManager op-new 0x1d96768)
region hit at guest pc=0x102206e3c
```
The `bne` at 0x102206df0 (`cmp x0,x20` against stored main-thread id [0x106863a68]) is
TAKEN on the ladder thread -> jumps to 0x102206e28 (`mov x0,sp` -> LocalStorageManager
construction). The binder-dispatch block 0x102206df4..0x102206e24 (ldr/cbz/ldr/br x1)
is NEVER reached. The `cbz x0,0x102206ea4` soft-return SH226 claimed as the harness's
"deterministic-NULL binder" path is not even the taken route.

## 3. Conclusion — the SH225/226 next-target is a MEASURED dead-end on both grounds

1. **Wrong offset**: the load is `[x19,#32]` (SH156/objdump/arm64jit agree), so the
   "binder object at [union+4]" does not exist as decoded. The data actually read is the
   union's +0x30 (= [sp+32] = stack self-ref).
2. **Reachability bypass**: even with the correct offset, the dispatch block never runs
   headlessly because the `b.ne` thread-match gate routes the Closure-build to the
   LocalStorageManager construction on the (non-main) ladder thread.

So the operator's "next drive = fabricate a binder to fire do-init's dispatch" lever is
NOT a forward path: it targets a mis-decoded offset and a code path the runtime bypasses.
This is the operator's proof-of-dead-end standard (a specific gate/lever provably not
advanceable), not an ROI judgement — it is measured on the real binary both statically
(decode) and dynamically (region-watch).

**What this does NOT disprove**: the do-init -> app-shell ctor -> governor continuation
still executes (SH197), and the ladder DOES reach 0x1023eff4c via a DIFFERENT mechanism
(unrelated to this binder dispatch). Route-B live-DM world-build remains the standing
structural gate (SH209/218/223/226). SH227 removes ONE purported lever from the set with a
corrected decode rather than re-attacking a phantom binder.

## 4. Code

- `crates/arm64jit/examples/elfjit.rs` hermetic `sh227_doinit_binder_dispatch_decode_corrected_and_bne_bypass`
  (real-image guard family as sh222/sh223/sh224/sh225/sh226): asserts the load word, that
  `arm64jit::decode::decode(0xf9401260)` yields a 64-bit LDR with imm12*size == 32 (so a
  drift /re-label fails), the `b.ne` gate word 0x540001c1, the non-match target mov x0,sp,
  and the (byte-present-but-unexecuted) br x1.
- Corrected the sh225 + sh226 semantic labels: the load is now documented as `[x19,#32]`
  and the assert message now FAILS on SH225/226's `#4` mislabel rather than enshrining it.
  All byte pins unchanged (they were correct).

## 5. Verify

- `cargo test -p arm64jit --example elfjit` = 75 passed / 0 failed (was 74; +1 sh227).
- `cargo test --workspace` green (567/0 + arm64jit lib).
- Repro: `runs/capture_sh227_binder_bypass.sh` (hermetic + the 3-run region-watch showing
  the dispatch block never fires).

## 6. Next (honest, single-agent)

Standing Route-B structural gate UNCHANGED. The do-init binder-fabrication lever is now
closed as a measured dead-end (corrected decode + reachability bypass). The remaining
forward hooks are unchanged from SH226: the live-DM world-build (SH174 capture latch at a
real make_shared<DataModel>) or a genuine flags-loaded engine-init state — neither a
headless seed. recon-v3 render plane + SH210 wiring + cookie persistence unregressed.