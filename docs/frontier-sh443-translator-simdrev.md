# SH443 — hermetic coverage of the BYTE-REVERSE codegen family
## (translate.rs SimdRev `rev64`/`rev32`/`rev16`)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (24 real task-driven frames `present swap Ok(0x1)`, 0 json
abort, 0 crash — SH442/441 baseline unchanged). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 603/0 incl. 4 new sh443 pins, was 599; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
a31f13c.

## What SH443 pins

SimdRev (rev64/rev32/rev16 — byte-reverse within each granule) had no direct
byte tests. SH443 pins the exact emitted x86 with rd=1, rn=2 (Vn base =
VECTOR_BASE 0x110+2*16 = 0x130, Vd base = 0x110+1*16 = 0x120):

1. **rev64 (granule 8, q=0)** — one 8B granule: `mov rax,[rbx+0x130]` +
   `bswap r64` (`48 0f c8`) + `mov [rbx+0x120],rax`. The bswap-r64 opcode is
   the 8B-granule discriminator.
2. **rev64 q=1** — TWO 8B granules: the second reads Vn+8 (0x138), bswaps,
   stores Vd+8 (0x128). The q cross-lane +0x8 advance pins the whole-register
   reversal (a made-only-low-half buggy emit would never touch 0x138/0x128).
3. **rev32 (granule 4, q=0)** — TWO 4B granules: `mov eax,[rbx+0x130]` +
   `bswap r32` (`0f c8`, NO 0x48 REX.W) + `mov [rbx+0x120],eax`, then granule
   1 at 0x134 -> 0x124. The bswap-r32 (0f c8 without 48) vs bswap-r64
   (48 0f c8) opcode is the 4-vs-8-granule discriminator.
4. **rev16 (granule 2, q=0)** — FOUR 2B granules, each `movzx eax,[rbx+
   0x130+2i]` + `rol eax,8` (`66 c1 c0 08`, rotate-left-by-8 = byte-swap a
   halfword) + the 16-bit `66 89` store at 2-apart dst (0x120/0x122/0x124/
   0x126). The rol-imm8-by-8 vs a bswap is the 2-granule discriminator.

Each test pins the exact full emit AND the granule discriminator (bswap64 vs
bswap32 vs rol16, REX.W presence/absence, q cross-lane advance, 16-bit store
width), so a wrong granule silently reorders rendered pixel/color bytes.
Deterministic: synthetic `Inst` -> translate() -> CodeBuf.as_slice() (zero-pc
0x1000), [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. No image,
no env, parallel-safe. Trunk verified by a one-off eprintln dump (removed
before commit) so pins match the REAL emission.

## Honest

NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the byte-reverse family, continuing the SH440-442
translator-core lineage (distinct from SH442 fcvt-rounding, SH441 reduce/widen).
No re-treads.

## Files

- docs/frontier-sh443-translator-simdrev.md (this file)
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)