# SH442 — hermetic coverage of the scalar FcvtToInt ROUNDING-mode paths
## (translate.rs FcvtToInt mode 2 `fcvtau` / mode 3 `fcvtpu`,`fcvtps` / mode 4 `fcvtmu`,`fcvtms`)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (24 real task-driven frames `present swap Ok(0x1)`, 0 json
abort, 0 crash — SH441/440 baseline unchanged). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 599/0 incl. 5 new sh442 pins, was 594; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
1f04a2a.

## What SH442 pins

SH435 pinned FcvtToInt mode 0 (fcvtzs/fcvtzu trunc) and SH439 pinned fcvtzu's
[2^63,2^64) big-path. The ROUND-before-truncate modes had zero direct byte
tests. SH442 pins the exact emitted x86 (rd=0, rn=1 d-src; src slot =
VECTOR_BASE 0x110+1*16 = 0x120, dst slot g0):

1. **fcvtau (unsigned, mode 2)** = `roundsd xmm0,xmm0,0x00` (NEAREST-EVEN) +
   `cvttsd2si` — the imm8 0x00 vs 0x01/0x02 is the mode-2 discriminator.
   Notably it emits NO unsigned clamp (the source comment documents the
   accepted saturation edge for d>=2^63) — pinned negative so a later editor
   does not add one.
2. **fcvtpu (unsigned, mode 3, +inf)** = `roundsd ...,0x02` (CEIL) +
   `cvttsd2si` + the negative-clamp trio (`xor rcx,rcx`; `test rax,rax`;
   `cmovs rax,rcx` = 48 0f 48 c1).
3. **fcvtmu (unsigned, mode 4, -inf)** = `roundsd ...,0x01` (FLOOR) +
   `cvttsd2si` + same clamp. The imm8 0x01 (floor) vs 0x02 (ceil) is the
   fcvtmu-vs-fcvtpu discriminator.
4. **Signed mode 3/4 (fcvtps/fcvtms)** = identical roundsd but NO clamp — the
   unsigned-cmovs absence is the signed discriminator.

Each test pins the exact full emit AND the discriminator (roundsd imm8, the
cmovs clamp presence/absence), so a wrong rounding direction or a missing
negative-clamp silently mis-converts a render/lighting/normal value.
Deterministic: synthetic `Inst` -> translate() -> CodeBuf.as_slice() (zero-pc
0x1000), [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. No image,
no env, parallel-safe. Trunk verified by a one-off eprintln dump (removed
before commit) so pins match the REAL emission.

## Honest

NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the scalar-float-to-int rounding modes, completing the
SH435/439 mode-0 coverage with the mode 2/3/4 rounding family. No re-treads.

## Files

- docs/frontier-sh442-translator-fcvtoint-rounding-modes.md (this file)
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)