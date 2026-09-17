# Frontier SH294 — [item+48] registry-cell + trampoline attribution CORRECTED

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(pins + attribution correction only; no prod path edited, no opt-in rung added).

## tl;dr

SH290-293 drove/closed the item-processor (0x102207950) cone: once-build (SH290),
idempotent re-entry (SH291), the REAL [item+32]->vt[+48] per-item dispatch (SH292),
and pinned the LAST edge [item+48]->0x22193a0 (SH293) as "not driveable standalone".
This cycle corrects TWO mis-attributions in SH293's pin — the same class of
wrong-address/label bug as SH243 (which cost 8 cycles) — so the ledger says what
the binary actually does and a future drive targets the RIGHT cell.

## The two corrections (fresh disasm vs aarch64-linux-gnu-objdump)

### (1) The lifecycle-registry fn-ptr cell is 0x1068262e8, NOT 0x1068266e8

All subpaths read it via `adrp x8,6826000; ldr x8,[x8,#744]`. The imm is #744 =
0x2e8, so the cell is guest **0x1068262e8** — a WRITABLE, file-backed .data cell
(file vaddr 0x68262e8 < LOAD-3 filesz end 0x6829e58). It is deref'd through
`cbz x8,skip; blr x8` at:

- 0x28506d0/6d4/6e4 (item[0]==0 subpath 0x28506a4)
- 0x28508d4/8d8/8e8 (item[0]!=0 subpath 0x28508a8)
- frame helpers 0x285290c/0x2852968

So the cell is a benign-leaf-FIREABLE dispatch (the SH292 pattern — seed a fixed
writable dispatch cell with a registered host leaf so the blr returns harmlessly).
That means SH293's "not driveable standalone" verdict is a JUDGMENT, not a proven
hard wall; it remains UNDRIVEN (a future cycle may seed [0x1068262e8] with a benign
leaf and drive item-proc with item[+48]!=0 so the subpath blr EXECUTES).

### (2) 0x22076e8 is a tail-TRAMPOLINE, not the canary-[frame+8] blr helper

SH293 described 0x22076e8 as the "prologue descriptor helper doing `ldr x8,[x19,#8];
blr x8`". Those two instructions are actually in the SEPARATE function **0x22076f8**
(entry `sub sp,#0x30`). 0x22076e8 itself is:

```
 22076e8: mov x3, xzr          (aa1f03e3)
 22076ec: b    0x2850ef0       (14192601)   <- tail into nativeOnDestroyed dispatcher
```

So 0x22193a0's `bl 0x22076e8` does NOT reach a stack-canary deref; it tail-flows
into the shared nativeOnDestroyed dispatcher, whose w2==0 arm `bl 0x28511c4`
(a body that derefs a live-object vt[+112]) is the real live-object gate. The
SH293 conclusion (the edge converges on the SH273/SH174 live-object wall) HOLDS —
but for the correct reason.

## Measured / pinned

Hermetic `sh294` (real-image guard, loads libroblox.so):
- adrp 0x6826000 + imm #744 -> 0x1068262e8 decomposition; assert 0x68262e8 != the
  SH293-labeled 0x68266e8 (a whole adrp-page apart).
- writable-.data membership of 0x1068262e8 (not .bss, not R-E).
- both subpath blr sites (0x28506d0/6d4/6e4 and 0x28508d4/8d8/8e8).
- trampoline words 0x1022076e8/76ec; real canary helper fn 0x22076f8 entry + its
  [frame+8] deref 0x10220734/738 (correctly attributed to 0x22076f8).
All verified against objdump.

## Honest (do-not-over-claim)

- Pure attribution/ledger correction (SH243-class wrong address). No production
  code path edited, no new harness drive. No DataModel (MH_* false, DM-root 0);
  Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
  forward hook.
- NEW forward affordance recorded: [0x1068262e8] is a benign-leaf-fireable cell on
  the item-proc [item+48] edge — a concrete next-drive target (SH290-293 drove
  item[+32]; the [item+48] subpath dispatch remains UNDRIVEN).

## Verify

- `cargo test -p arm64jit --example elfjit sh294` = 1 passed.
- `cargo test -p arm64jit --example elfjit --` = 123/0 (was 122 + this).
- `cargo build --workspace` + `cargo test --workspace` EXIT 0.
- elfjit.rs + HANDOFF.md under the 1MB pre-commit hook (prose trimmed this cycle).

## Files

- `crates/arm64jit/examples/elfjit.rs` (+sh294).
- `HANDOFF.md`, `runs/STATUS.md` (ledger).
- `docs/frontier-sh294-item48-cell-correction.md` (this).