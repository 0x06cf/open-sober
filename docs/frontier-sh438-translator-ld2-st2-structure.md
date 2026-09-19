# Frontier SH438 — structure-load/store codegen (translate.rs Ld2 / St2)

**hermes-worker, Sep 19, 2026.** Single-agent (cone suppressed).

## What this closes

`Ld2` / `St2` (ld2/st2 `{Vt, Vt1}, [Xn]` — interleaved structure pair moves) had
zero direct byte tests. They are the deinterleave primitive for interleaved
vertex-attribute / RG-z+texcoord data (2-vector register-list). decode.rs pins
the decode, jit.rs pins runtime, but the byte EMISSION was unpinned. SH438 pins
it with 3 deterministic exact-byte hermetics.

## Byte-map (deterministic, no image/env, parallel-safe)

Memory holds the DEINTERLEAVED layout `{V0.e0,V1.e0, V0.e1,V1.e1, ...}`: element
`i` of register `j` (j in 0..2) is at byte offset `i*(2*es) + j*es`. Ld2 reads
memory (base loaded via `ldg` into RDX) and writes vector slots
`VECTOR_BASE(0x110)+(rd+j)*16 + i*es`; St2 is the inverse (reads slots, stores to
memory). A byte-offset flub misdelivers which attribute/register lane goes where
— the whole point of the family.

`movzx_byte_mem` for disp 0 uses mod=0 (`0F B6 02`, no disp byte); disp 4/8/0xc
use mod=1 (`0F B6 42 04/08/0c`). The Vd-slot store is 6 bytes (`88 83` + disp32).

## The 3 pins

1. **Ld2 deinterleave** (esize=4, q=false 8B, rd=0 rn=1, post=0): loads advance
   j mem+es=4 (`0F B6 42 04`) and i mem+2*es=8 (`0F B6 42 08`); element 0 of the
   first structure reg reads mem 0 (mod=0 `0F B6 02`). Stores: reg j lands at
   `0x110 + j*16` (`V0.elt0 -> 88 83 10 01 00 00`, `V1.elt0 -> 88 83 20 01 00 00`
   = the 2nd structure reg at +16), element i advances by es=4 (`V0.elt1 ->
   88 83 14 01 00 00`).
2. **St2 inverse** (same case): the movzx SOURCE becomes the Vd slot `[rbx+...]`
   (`V1.elt0 read -> 0F B6 83 20 01 00 00`) and the STORE becomes `88` to
   `[rdx+mem-off]` (`V1.elt0 -> [rdx+4]`, `V0.elt1 -> [rdx+8] = i+2*es`) — the
   export/import direction flip is the Ld2-vs-St2 discriminator.
3. **q=true (16B) + post-increment**: q doubles nelems to 4 (`V0.elt3 ->
   88 83 1c 01 00 00`); `post=0x20` emits the rn register write-back
   `48 8B 43 08` (mov_load64) + `48 81 C0 20 00 00 00` (add imm32 — NOTE the
   imm32 form, not imm8) + `48 89 43 08` (store-back to [rbx+8]).

## Honest

NOT a DM (Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0 under the
complete substrate). BUILD-THE-RUNTIME codegen-surface coverage completion on the
structure pair family, continuing the SH427-437 translator-core lineage. Distinct
from SH437 SimdInsD (single-lane copy) — Ld2/St2 are the multi-register
interleaved pair move. No re-treads.

## Verification

- `cargo test -p arm64jit --lib sh438` → 3/3 pass. Two encoding details were
  corrected during development by dumping the emitted buffer (disp=0 uses mod=0
  `0F B6 02`; post-increment uses `48 81 C0` imm32, not imm8) — the shipped tests
  pin the real emission.
- `cargo test --workspace` → EXIT 0. arm64jit lib **585/0** (was 582).
- `cargo build --example elfjit` → OK.
- Production code ONLY in translate.rs `#[cfg(test)]` (off the 1MiB hooks);
  translator core body byte-untouched; jit.rs 1,048,390 B < 1 MiB hook (unchanged
  this cycle); elfjit.rs/session.rs unchanged.

Files: docs/frontier-sh438-translator-ld2-st2-structure.md +
crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit pending.