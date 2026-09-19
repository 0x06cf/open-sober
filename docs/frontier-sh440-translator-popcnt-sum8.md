# SH440 — hermetic coverage of the byte-COUNT + horizontal-SUM codegen pair
## (translate.rs SimdPopcnt `cnt` / SimdSum8 `uaddlv`)

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (SH439 baseline; capture_taskv4_frame.sh
frame deliverable unchanged — 24 real task-driven frames `present swap Ok(0x1)`,
0 json abort, 0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit
lib 590/0 incl. 2 new sh440 pins, was 588; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core body
byte-untouched; jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs
unchanged). Commit 72a6cbb.

## What SH440 pins

SimdPopcnt (`cnt Vd.8b, Vn.8b`) and SimdSum8 (`uaddlv h{rd}, Vn.8b`) had NO
direct byte tests. SH440 pins both to the exact emitted x86 with rd=1, rn=2
(Vn slot = VECTOR_BASE 0x110+2*16=0x130, Vd slot = 0x110+1*16=0x120):

1. **SimdPopcnt full buffer** — the SWAR popcount: `mov rax,[rbx+0x130]` (load
   Vn), `mov rdx,rax`, `shr rdx,1` + `and rdx,0x5555…` + `sub rax,rdx` (x −
   ((x>>1)&0x5555…)), then `(x&0x3333…)+((x>>2)&0x3333…)` (shr 2 + two ands +
   add), then `(x+(x>>4))&0x0f0f…` (shr 4 + add + and), `mov [rbx+0x120],rax`.
   The three masks (0x5555../0x3333../0x0f0f..) in ascending bit-half order and
   the shift ladder **1/2/4** are the byte-count discriminators a wrong mask or
   shift silently corrupts.
2. **SimdSum8 full buffer** — the horizontal byte sum (uaddlv): `mov rax,
   [rbx+0x130]` then a three-step widening pairwise sum — `(x+(x>>8))&0x00ff00ff..`,
   `(x+(x>>16))&0x0000ffff0000ffff`, `(x+(x>>32))&0x00000000ffffffff` — each
   `mov rdx,rax` + `shr rdx,imm8` + two `and`s + `add`, storing `mov [rbx+0x120],
   rax`. The shift ladder **8/16/32** (vs SimdPopcnt's 1/2/4) is the
   count-vs-sum discriminator; a byte error here mis-sums a lane-mask /
   popcount total.

Both tests assert the exact full emit AND the semantic discriminators
(individually: each mask constant, each shr imm8, and the negative assert that
the popcount buffer never uses shr-by-8 and the sum buffer never uses
shr-by-1 — so a count/sum ladder cross would fail).

Deterministic: synthetic `Inst` -> translate() -> CodeBuf.as_slice() (zero-pc
0x1000), [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. No image,
no env, parallel-safe. The trunk was verified by a one-off eprintln dump
(removed before commit) so the shipped tests pin the REAL emission.

## Honest

NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the complete
substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-surface
coverage completion on the byte-count + horizontal-reduce family, continuing
the SH427-439 translator-core lineage (distinct from SH432 SimdVLog/SminMax/
SimdSatAdd, SH435 fcvt, SH436 SimdSel/SimdHighNarrow, SH437 SimdInsD,
SH438 Ld2/St2, SH439 fcvtzu). No re-treads.

## Files

- docs/frontier-sh440-translator-popcnt-sum8.md (this file)
- crates/arm64jit/src/translate.rs (`#[cfg(test)]` only)