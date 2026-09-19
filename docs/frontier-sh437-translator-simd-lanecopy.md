# Frontier SH437 — SIMD lane-copy codegen (translate.rs SimdInsD)

**hermes-worker, Sep 19, 2026.** Single-agent (cone suppressed).

## What this closes

`SimdInsD` (`mov Vd.T[dst], Vn.T[src]` — the per-element LANE COPY) had zero
direct byte tests. It is the load-bearing splat/broadcast/shuffle primitive on
render data paths (color/texel lane packing, 8-bit channel moves, vector
broadcast). decode.rs pins the decode, jit.rs pins runtime, but the byte
EMISSION between them was unpinned for this family. SH437 pins it with 5
deterministic exact-byte hermetics.

## Byte-map (deterministic, no image/env, parallel-safe)

```
[RBX] = CpuState ;  vector slot v[t] = VECTOR_BASE(0x110) + t*16
src = 0x110 + rn*16   + src_idx*esize
dst = 0x110 + rd*16   + dst_idx*esize
```

The element size selects FOUR distinct load/store byte forms — a width flub
copies the wrong byte count and silently corrupts the lane:

| esize | load form                 | bytes        | store form              | bytes    |
|-------|---------------------------|--------------|-------------------------|----------|
| 8 (d) | `mov_load64`              | 48 8B        | `mov_store64`           | 48 89    |
| 4 (s) | `mov_load32` (zero-ext)   | 8B (no REX)  | `mov_store32`           | 89       |
| 2 (h) | `movzx_word_mem` (0F B7)  | 0F B7        | `mov_store16` (66 pre)  | 66 89    |
| 1 (b) | `movzx_byte_mem` (0F B6)  | 0F B6        | `mov_store8`            | 88       |

## The 5 pins

1. **d-lane** (esize=8, rd=0 rn=1 dst_idx=1 src_idx=0): src=0x120, dst=0x118 →
   `48 8B 83 20 01 00 00` + `48 89 83 18 01 00 00` (full-buffer equality).
2. **s-lane** (esize=4, rd=2 rn=3 dst_idx=1 src_idx=0): src=0x140, dst=0x134 →
   `8B 83 40 01 00 00` + `89 83 34 01 00 00`; asserts NO REX.W `48 8B` (must not
   emit the d-lane form).
3. **h-lane** (esize=2, rd=0 rn=1 dst_idx=3 src_idx=1): src=0x122, dst=0x116 →
   `0F B7 83 22 01 00 00` + `66 89 83 16 01 00 00` (the only esize whose store
   carries the 66 operand-size prefix).
4. **b-lane** (esize=1, rd=1 rn=0 dst_idx=4 src_idx=2): src=0x112, dst=0x124 →
   `0F B6 83 12 01 00 00` + `88 83 24 01 00 00`; asserts no 64-bit mov.
5. **lane-ADDRESSING discriminator** (esize=8, rn=3, dst_idx=0): src_idx 0→1
   moves the source disp 0x140→0x148 (exactly +esize) while the dest disp stays
   fixed at 0x110 — proving the address uses `Vn*16 + src_idx*es`,
   never a fixed/vt-stale stride.

## Honest

NOT a DM (Route-B live-DM gate UNCHANGED: DM-root [0x106a68818]=0 under the
complete substrate). BUILD-THE-RUNTIME codegen-surface coverage completion —
the lane-copy family, distinct from SH432 (SimdVLog/SminMax/SimdSatAdd), SH434
(shifts), SH435 (fcvt), SH436 (SimdSel/SimdHighNarrow). No re-treads.

## Verification

- `cargo test -p arm64jit --lib sh437` → 5/5 pass (first run; exact bytes
  predicted from `emit_mem` + mov-helper tables).
- `cargo test --workspace` → EXIT 0. arm64jit lib **582/0** (was 577).
- `cargo build --example elfjit` → OK.
- Production code ONLY in translate.rs `#[cfg(test)]` (of the 1MiB hooks, file
  ~578 KB); translator core body byte-untouched; jit.rs 1,048,390 B < 1 MiB
  hook; elfjit.rs/session.rs unchanged.

Files: docs/frontier-sh437-translator-simd-lanecopy.md +
crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit pending.