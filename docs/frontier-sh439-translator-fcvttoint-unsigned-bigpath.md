# Frontier SH439 — translator: float→int UNSIGNED big-path (FcvtToInt) + fixed-point scale

Hermetic byte-pin coverage of the translate.rs `FcvtToInt` codegen for the
value range a single signed `cvttsd2si` silently corrupts: UNSIGNED
destinations over [2^63, 2^64) (fcvtzu), and the `fbits>0` fixed-point scale.
SH435 pinned the Fcvt / FcvtTzReg / FcvtHalf families and explicitly left "the
unsigned big-path" as next-forward; this closes that.

## Why it matters

The real client converts float→int on timing / URLOpen / color-of-light paths.
x86 `cvttsd2si` is SIGNED: exact in [0, 2^63), but saturates to INT64_MIN
(0x8000..0) for any value >= 2^63 — which, stored directly, corrupts the HIGH
half of an unsigned 64-bit destination. The FcvtToInt unsigned path therefore
cannot be a bare `cvttsd2si`; it must synthesize the range gate. A byte error
here (dropping the comisd gate, the big-path subsd/add, or the u64::MAX clamp)
silently mis-converts every large-magnitude unsigned float in the client.

## What SH439 pins (3 hermetic, exact-byte window asserts)

1. **fcvtzu (unsigned, mode=0) emits the full [0, 2^64) range gate** — the
   2^63 double constant (`mov rcx,0x43e0_0000_0000_0000`, `48 B9 .. E0 43`), the
   signed compare `comisd xmm0,xmm1` (`66 40 0F 2F C1` — note the REX byte is
   *always* present, a `66 0F 2F` window would miss it), the `JB`(0F 82) branch
   to the signed path, the big-path `subsd xmm0,xmm1` (`F2 0F 5C C1`, computes
   d-2^63) and `add rax,rcx` (+2^63 restore, `48 01 C8`), and the `u64::MAX`
   saturation (`mov rax,-1`, `48 B8 FF..`) for d>=2^64 — plus the `cmovs
   rax,rcx` (`48 0F 48 C1`) negative-to-0 clamp present on the signed arm.

2. **signed fcvtzs (mode=0, unsigned=false) is a BARE trunc** — movq_load +
   single `cvttsd2si rax,xmm0` (`F2 48 0F 2C C0`) + 64-bit dst store, with
   NONE of the unsigned machinery (asserts absence of comisd gate, subsd,
   cmovs clamp, u64::MAX). Presence of the comisd gate is the fcvtzu-vs-fcvtzs
   discriminator.

3. **fbits>0 fixed-point scales BEFORE truncating** — the 2^fbits double
   (2^4=16.0 = `0x4030_0000_0000_0000`, `48 B8 .. 30 40`) → `movq xmm1,rax`
   (`66 48 0F 6E C8`) → `mulsd xmm0,xmm1` (`F2 0F 59 C1`) prior to conversion;
   fbits=0 must NOT mulsd.

## Method

Synthetic `Inst::FcvtToInt {..}` -> `translate()` (pc 0x1000, deterministic) via
`tr_bytes()`; `[RBX]=CpuState`; vector slot `v[t]=VECTOR_BASE(0x110)+t*16`
(rn=1 -> v1 @ 0x120, d-src low 8B). Inverse/fixed-point windows asserted
against positive+negative. No image, no env, parallel-safe (local buffers).

## Scope honesty

BUILD-THE-RUNTIME codegen-surface coverage completion (FcvtToInt unsigned
big-path + fixed-point), distinct from SH435 (Fcvt/FcvtTzReg/FcvtHalf) and the
SH427-438 lineage. Prod code byte-untouched — `#[cfg(test)]` only in
translate.rs; jit.rs 1,048,390 B / elfjit.rs / session.rs untouched.
arm64jit lib was 585/0, now 588/0. Recon-v3 immediate-priority deliverables
unchanged-green at HEAD (verified last cycle; no production path touched this
cycle means no regression surface). HONEST: NOT a DM — Route-B live-DM gate
unchanged (DM-root [0x106a68818]=0 under the complete substrate requires a real
session ctor the JIT cannot reproduce headlessly).