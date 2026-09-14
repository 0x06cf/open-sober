# SH149 — guarded-shim insurance tail (honestly-labeled, zero-reachability)

**Goal:** close the remaining raw string/mem pointer-deref imports that glibc
length-walks — the same SH97/SH135/SH139 crash class — as cheap DEFENSIVE
insurance. SH147 closed the proven memmove hole (~192x boot dispatch); these
have NO run-log dispatch evidence (zero reachability), so they are labeled
insurance, NOT memmove-class holes.

## Changes (crates/arm64jit/src/shims.rs)

Nine new `ptr_ok`-guarded shims (unsafe ptr -> safe default; valid -> glibc
bit-identical), registered via register_named (wins over resolve_common dlsym):
- `bionic_strncpy(dst,src,n)` / `bionic_strncat(dst,src,n)` — !ptr_ok or n==0 -> dst.
- `bionic_strpbrk(s,accept)` / `bionic_strnlen(s,maxlen)` / `bionic_memrchr(s,c,n)`
  — !ptr_ok -> 0 (memrchr walks from the end; GNU-extension-safe manual impl).
- `bionic_strtoull(s,endptr,base)` / `bionic_strtoll(...)` — !ptr_ok -> 0.
- `bionic_strtof(s,endptr)` — guard only (returns float in s0, which a plain
  HostCall can't satisfy; the guard's job is avoiding SIGSEGV, result dropped —
  documented limitation).
- `bionic_strftime(s,max,fmt,tm)` — the one benign-reachable item (1x in
  sh130-combined: valid rodata fmt + valid dest); guarded against garbage fmt/tm.

## Verification

- **Hermetic**: extended sh97_harden_guarded_string_ops_reject_garbage_no_segfault —
  all nine reject the garbage class (0xffffff80ffffffc8 incl. sub-image) with no
  SIGSEGV. Workspace green (537/0); arm64jit 359+56... all pass.
- **Real binary**: capture_sh130 re-run EXIT 0 / ladder done / 0 SIGSEGV — no
  boot regression (all valid pointers still bit-identical).

## Honesty / scope

Purely defensive. These are NOT proven live holes (unlike memmove in SH147) —
they are zero-reachability insurance that makes the reachable string/mem plane
deterministic if the do-init ever hands them a garbage map-field pointer. strtof
only guards (returns 0, no float passthrough — documented).