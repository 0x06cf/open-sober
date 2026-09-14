# SH147 — guarded-shim family extension: memmove/strrchr/strdup/strndup/atoi/atol/atoll/strtol/strtoul

**Goal:** close the last raw-bound pointer-deref libc imports on the reachable
boot path (the SH97/SH135 crash class). The deep do-init walk can hand an
unseeded map-field pointer (0xffffff80ffffffc8 sign-extended, 0, or a sub-image
small-int like 0x1800064) to a libc string/mem function; raw host glibc then
strlen/memreads the garbage and SIGSEGVs. Prior work (SH97, SH135-139) guarded
strlen/strcmp/strncmp/strstr/strchr/strcpy/strspn/strcspn/memchr/memcmp/
strcasecmp/strncasecmp/memcpy/memset + fortify variants. SH147 extends the same
`ptr_ok` guard to the nine still-raw-bound pointer-deref imports.

## Why these (evidence)

- **`memmove` is PROVEN reachable at boot** — dispatched ~192x in
  `runs/sh126-workers.txt` (same SH97/SH139 class as the already-guarded
  memcpy/memset). Highest-priority real hole.
- `strrchr/strdup/strndup/atoi/atol/atoll/strtol/strtoul` are the same class:
  all take a `char*` that glibc internally length-walks (or the host `strtol`
  family reads the string), listed raw in `resolve_common`
  (resolver.rs:1722/1727-39). Cheap drop-in insurance.
- **`strtod` deferred**: returns a `double` in d0 (AArch64), which a plain
  `HostCall` (u64->x0) can't satisfy — it needs a `HostGlesCall`-style v0-write
  bridge + a new `resolve_gles` bind branch, and has zero reachability
  evidence. Documented gap, not worth the heavier lift now.

## Changes (crates/arm64jit/src/shims.rs)

Nine new `extern "C" fn` shims mirroring the existing guarded idiom
(`!ptr_ok -> safe default; valid -> glibc bit-identical`):
- `bionic_memmove(dst,src,n)` — !ptr_ok or n==0 -> dst (no copy); else libc::memmove.
- `bionic_strrchr(s,c)` — !ptr_ok(s) -> 0; else libc::strrchr (reverse search).
- `bionic_strdup(s)` — !ptr_ok(s) -> 0 (NULL); else libc::strdup.
- `bionic_strndup(s,n)` — !ptr_ok(s) or n==0 -> 0; else libc::strndup.
- `bionic_atoi/atol/atoll(s)` — !ptr_ok(s) -> 0; else libc::atoi/atol/atoll.
- `bionic_strtol/strtoul(s,endptr,base)` — !ptr_ok(s) -> 0; else glibc (endptr is
  an output-only pointer arg; glibc writes it only when s is valid).

All registered in `register_shims()` with `register_named` (wins over the
resolve_common dlsym, same precedence as SH135-139).

## Verification

- **Hermetic**: extended the `sh97_harden_guarded_string_ops_reject_garbage_no_segfault`
  test — all nine new shims fed the garbage class (0xffffff80ffffffc8 + sub-image
  0x1800064) return their safe default with no SIGSEGV; valid canonical pointers
  (atoi("12345")=12345, strtol base10, strrchr('hello','l')->offset3,
  strdup->"hello", strndup(s,3)->"hel", memmove copies bytes, dst identity)
  reach real glibc bit-identically. `cargo test -p arm64jit` green (359+56...).
- **Real binary**: `runs/capture_sh130.sh` re-run EXIT=0 / ladder done / no
  SIGSEGV/SIGABRT — no boot-path regression from the new shims (all valid
  pointers still pass through bit-identical).
- Workspace green (537 lib + example sh14 15).

## Honesty / scope

Reachable-path hardening only — the shims divert only pointers that would
otherwise SIGSEGV; valid pointers are bit-identical to raw glibc. `strtod`
remains raw (documented, deferred). This reduces the deterministic-silent-SEGV
surface per SH131d direction (a).