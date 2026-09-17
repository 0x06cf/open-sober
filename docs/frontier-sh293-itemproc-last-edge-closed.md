# Frontier SH293 — item-PROCESSOR LAST edge ([item+48] -> 0x22193a0) mechanism-CLOSED

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(pins only; no production path edited, no opt-in rung added).

## tl;dr

SH292 drove item-proc 0x102207950's [item+32]->vt[+48] dispatch and left the LAST
never-driven edge: `[item+48] -> bl 0x22193a0` (0x22079e0). This cycle disassembles
that edge and byte-pins it as a measured mechanism: it is the authentic per-item
continuation and it converges on the SH273/SH174 live-object wall — it is NOT
driveable standalone. This fully closes item-proc's state machine: every edge is now
either driven+measured (once-build SH290, idempotent re-entry SH291, per-item
[/item+32] dispatch SH292) or mechanism-pinned ([/item+48] this edge).

## The mechanism (fresh disasm)

`0x22193a0` (the [item+48] continuation), entry `sub sp,#0x30`:

1. Prologue helper `bl 0x22076e8([frame], item+0x10, 0)` — but `[frame]` is
   0x22193a0's OWN stack pointer, and just before the call 0x22193cc wrote the
   stack CANARY into `[sp,#8]`. 0x22076e8 then does `ldr x8,[x19,#8]; blr x8`
   (0x2207734/738): it dispatches `[frame+8]` as a function pointer. In item-proc's
   frame that slot holds the canary VALUE — a data pointer — so the edge only runs
   when a REAL session thread has established a coroutine/TLS descriptor there. That
   is the SH174/SH204/TLS live-object class, not a controllable seed.
2. Then it marks the item consumed (`strb w9,[x19,#1]`) and splits on `item[0]`
   (0x22193e0 cbz):
   - `item[0]==0` -> `bl 0x2219428` (= `b 0x28506a4`)
   - `item[0]!=0` -> `bl 0x24993c8` (= `b 0x28508a8`)
3. Both targets (0x28506a4 / 0x28508a8) sit inside the `nativeOnDestroyed` symbol =
   the SH273 JNIActivityLifecycleCallbacks family. Their helpers 0x2850740/0x2850944
   build a frame via `bl 0x28528d4` then deref lifecycle-registry global
   `[0x1068266e8]` as a function pointer (`adrp x22,6826000; ldr x8,[x22,#744]`,
   `blr x8` when non-zero). Headlessly that registry is never populated -> live-object
   wall.

## Measured / pinned

Hermetic `sh293` (real-image guard, loads libroblox.so) byte-pins the whole chain:
the [item+48] call site, 0x22193a0's entry + frame-prep + descriptor-helper bl, the
consumed-mark + item[0] discriminator, the two tail-jumps (0x2219428->0x28506a4,
0x24993c8->0x28508a8), the two subpath entries + their frame/registry helpers, the
`[0x1068266e8]` fn-ptr read, and 0x22076e8's `[frame+8]` blr — plus an in-exec-seg
assert on the pinned pcs. All words verified against `aarch64-linux-gnu-objdump`.

## Honest (do-not-over-claim)

- This is the measured mechanism-closure of item-proc's last edge; it does NOT
  fabricate a DataModel, does NOT lift the Route-B live-DM gate, MH_* stay false,
  DM-root 0 — identical to SH290/291/292.
- It converts SH292's "gated on the SH273 wall" judgment for this edge into a
  byte-pinned mechanism (descriptor dispatch + lifecycle-registry deref), so a future
  cycle does not re-tread it as a seedable lever.
- SH174 capture-latch stays the single forward hook.

## Verify

- `cargo test -p arm64jit --example elfjit sh293` = 1 passed.
- `cargo test -p arm64jit --examples` = 122/0 (was 121 + this).
- `cargo build --workspace` + `cargo test --workspace` EXIT 0 (see session verify).
- elfjit.rs under the 1MB pre-commit hook (comment-prose trims batched in this cycle).

## Files

- `crates/arm64jit/examples/elfjit.rs` (+sh293.. sh292#).
- `docs/frontier-sh293-itemproc-last-edge-closed.md` (this).