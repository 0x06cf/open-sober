# SH453 — Hermetic coverage of the SIMD WIDEN-AND-ADD/SUB codegen family (translate.rs SimdAddl — saddl/uaddl/subl/usubl Vd.T, Vn.T, Vm.T)

## Summary
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 642/0 incl. 4 new sh453 pins, was 638; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs 1,048,390
B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit 7a75a4f +4.

## What SH453 pins
`SimdAddl` (widen-and-add/sub — widen each esrc-byte element of Vn and Vm (low
or upper half) sign-/zero-extended to 2*esrc, then add or subtract into a
DOUBLE-width dst; with the in-place-alias snapshot) had zero direct byte tests.
SH453 pins the exact emit (rd=1 rn=2 rm=3; Vn@0x130 Vm@0x140 Vd@0x120) with 4
exact-byte / window pins:

1. **THE signed widen-add (the load-bearing pin).** saddl V1.2s,V2.2s,V3.2s
   (esrc=4 sign=true sub=false): each 4-byte element SIGN-extends via `movsxd`
   (48 63 c0 / 48 63 c9 — the 4B source is widened to 64 bits) then `add
   rax,rcx` (48 01 c8) into an 8-byte dst (48 89 83). The movsxd + 8B store IS
   the widening: a zero-extend (no movsxd) flips a negative element's high bits,
   a 4B store truncates. 2-lane full-buffer exact-bytes. Signed sources must
   movsxd, unsigned must NOT (pinned in the word test).

2. **THE add-vs-sub opcode byte.** usubl V1.4s,V2.4h,V3.4h (esrc=2 sub=true):
   `sub rax,rcx` (48 29 c8) vs uaddl's `add rax,rcx` (48 01 c8) — 0x29-vs-0x01
   is the semantic (a transposed op adds instead of subtracting). Unsigned word
   sources zero-extend via movzx (0f b7), no movsxd. 4 lanes, lane3 dst 0x12c.

3. **THE upper high-half source offset.** saddl2 (upper=true): the source bases
   add +8 -> Vn byte source at 0x138 (not 0x130), Vm at 0x148 (not 0x140);
   signed byte sources SIGN-extend (movsx 48 0f be). Negative assert the upper
   form must NOT read the low-half 0x130 source.

4. **THE in-place-alias permute_source snapshot (the historic bug).** gcc's
   `saddl v1.4s, v1.4h, v2.4h` (rd==rn): the widened 2*esrc dst write at i*2*esrc
   OVERLAPS the narrow source bytes of lane i+1 (at (i+1)*esrc), so a naive
   read-then-write loop clobbers the still-needed source. The emit snapshots the
   full 16B dest/source to perm-scratch FIRST (mov_load64 [0x120]->[0x320] and
   [0x128]->[0x328], the two u64 halves) then reads the aliased Vn operands from
   scratch (0x320,0x322,...) — the [.., 0x320] perm-scratch reads are the
   in-place-alias signature; the non-aliased Vm==3 is read directly at 0x140.

## Method
Deterministic: synthetic `Inst::SimdAddl` -> `translate()` -> `CodeBuf.as_slice()`
(zero-pc 0x1000). [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16.
Emission captured precisely with a one-off probe test (eprintln dump, removed
before commit) so the pins match the real emission; the se=4 full-buffer
assert_eq matched the capture first try. Dev-time fixes (all corrected): the
movsx/movzx word+byte loads and the 2B stores are 7-8 bytes (0f b7 83 + dw /
48 0f be 83 + dw / 66 89 83 + dw), so those windows are windows(7)/windows(8),
adjacent to the 6-byte 32-bit forms. 4 pins, parallel-safe, no image/env.

## Honesty
NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the widen-and-add/sub family, adjacent to SH452
(pairwise-add-long) and SH441 (WidenShl). Distinct (3-operand widen-accumulate
with the alias-snapshot guard). No re-treads. Commit 7a75a4f +4.