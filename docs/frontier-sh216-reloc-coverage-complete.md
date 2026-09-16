# SH216 — Relocation-coverage completeness audit: Route-B is NOT provably
# dead on the relocation-synthesis axis (the operator's named proof-of-dead-end
# criterion is answered and boxed). Workspace green. Commit <SH216COMMIT>.

## Why (the operator's own bar)
The operator's doctrine ("MIGRATION IS NOT A STOPPING-POINT") requires Route-B
to keep being ground until a **concrete PROOF-of-dead-end** is shown — and names
the example: *"a specific gate provably unprocessable by this JIT, e.g. a
relocation type no loader pass can synthesize."* This cycle takes that criterion
directly and measures it against the real libroblox.so, instead of re-treading a
closed Route-B cone.

## Method (authoritative: the loader's OWN decoder, not a hand-rolled one)
Used `libloader::android_relocs::read_elf_relocations` (the exact code path the
JIT `load_elf_image` uses) against the 122,837,888-byte real
`libroblox.so`, and aggregated every relocation type it decodes. (A first ad-hoc
Python APS2 decode mis-parsed the stream because this .so's `DT_ANDROID_RELA` is
not raw "APS2"-magic — the loader is the truthful source.)

## Measured census (all in the DT_RELA data-relocation table)
    total decoded data relocs: 568,272
      R_AARCH64_RELATIVE (1027): 568,194   <- apply_relatives writes B+A (guest==host)
      R_AARCH64_GLOB_DAT (1025):      56   <- ptl bind_glob_dat -> dlsym/thunk/resolver
      R_AARCH64_ABS64    (257):      22   <- ptl bind_glob_dat -> dlsym/thunk/resolver
    (plus 534 R_AARCH64_JUMP_SLOT in the separate DT_JMPREL=PLT table,
     bound lazily by the PLT resolver)

**Every relocation type present in the real image is synthesized by the loader.**
There is NO relocation type a loader pass cannot synthesize: `apply_relatives`
(android_relocs.rs) covers RELATIVE; `bind_glob_dat` (arm64jit/plt.rs) covers
GLOB_DAT + ABS64 (imported data → `dlsym` host addr; imported fn/notype →
host-call thunk; GLES names `glGetShaderInfoLog`/`glGetProgramInfoLog` → resolver
host-thunk slots; libmediandk `AMEDIAFORMAT_KEY_*`/`AMediaFormat_delete`/
`AMediaCodec_delete` → resolver NDK shims; `__stack_chk_guard` data GOT bound so
the guest's very first prologue does not null-fault); the PLT resolver covers
JUMP_SLOT.

## The 78 symbol-based relocs (all UNDEF imports — not a gap)
The 56 GLOB_DAT + 22 ABS64 all reference **UNDEF(import)** symbols: libc/libm
(stderr, strcmp, atan/sin/cos/pow/fmod/log2/round/…, mmap/read/write/stat/
fcntl/unlink/mkdir/…, environ, stdin/stdout/stderr, __sF, optarg/optind,
tzname/daylight/timezone, longjmp, getentropy, __cxa_thread_atexit_impl,
__stack_chk_guard) + Android media/GLES (AMediaFormat_delete, AMediaCodec_delete,
10× AMEDIAFORMAT_KEY_*, glGetShaderInfoLog, glGetProgramInfoLog) + weak gcov
(__gcov_dump, __gcov_flush). These are the symbols the plt binder resolves at
load; they are not "left unapplied." (`apply_relatives` skipping them is correct
— they are the binder's job, not the loader's.)

## Conclusion (do-not-re-tread boxed)
1. Route-B is **not provably dead on the relocation-synthesis axis**: the
   foundational loader synthesizes the complete relocation type set of the real
   client. This is evidence *for* continuing the manufacture/DMCONT/PATH-B grind,
   not a proof-of-dead-end — per the operator's own criterion, the grind is not
   released by this axis.
2. No code-path fix falls out of the audit itself (nothing is unsynthesized). The
   standing structural gate (live-DM construction unreached headlessly, SH209)
   is unchanged.
3. The one angle this opens for future hardware of the DM line: if a puzzling
   NULL/host-pointer crash ever resurfaces in a region whose only imported data
   is one of these 78, check the binder's `(bound, unresolved)` return first —
   an unresolved data import leaves its GOT slot 0 → NULL deref, an easy-to-miss
   "SH55/64-looking" fault class that is actually a load-time bind gap.

## Code (this cycle)
- `crates/libloader/src/android_relocs.rs` + hermetic
  `real_image_relocation_types_all_loader_synthesizable` (real-image-guard family,
  same skip-if-absent convention as sh202/sh211/sh116b): asserts the real
  libroblox.so data-reloc type set ⊆ {1027,1025,257}; fails if a future .so ever
  introduces a relocation type the loader cannot synthesize. Workspace green.
- Removed the throwaway `reloc_audit` example (the hermetic test supersedes it).

## Verify
`cargo test --workspace` green (565 + this = 566 passed, 0 failed). The new test
prints the live census: `sh216 reloc census on <so>: 568272 data relocs {1025: 56,
1027: 568194, 257: 22} — ALL loader-synthesizable`.