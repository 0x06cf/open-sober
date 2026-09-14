# SH135 — Guarded string-op shims (SH97 hardening, reachable-path)

## What

The deep `gameGlobalInit` / `StartLuaAppDM` do-init walk hands **unseeded
map-field pointers** to libc string functions — the exact crash class SH97
eliminated for `strlen`/`vsnprintf` (0xffffff80ffffffc8 sign-extended garbage,
0, or a sub-image small-int/tag like 0x1800064). SH97 guarded only those two
ops; **8+ common string imports were still bound raw to host glibc**, so a
garbage pointer reaching any of them SIGSEGVs inside glibc's vectorized
read (same class as the SH97 guestpc=0x0 fault).

## Imports covered (verified UND@LIBC in libroblox.so)

`strcmp`, `strncmp`, `strstr`, `strchr`, `strcpy`, `strspn`, `strcspn`,
`memchr`, plus the fortified variants `__strchr_chk` and `__strcpy_chk`, and
`wmemchr` (left raw: wide-char, no string-pointer crash surface on the boot
path).

## Change (crates/arm64jit/src/shims.rs)

- New `ptr_ok(p)` pointer-domain predicate (mirrors `safe_cstr_len`: rejects
  0, `p < 0x100000000`, and non-canonical `(p >> 48) & 0xffff == 0xffff`; a
  valid empty string still has a canonical pointer, so this is a pure domain
  check distinct from a length).
- Ten guarded shims: on an unsafe pointer arg return a deterministic
  non-faulting default (0 for compare/search; `dst` unchanged for strcpy),
  else forward to the real glibc call — so valid image/host-heap pointers are
  bit-identical to before.
- Registered in `register_shims()` (register_named slots take precedence over
  the generic `resolve()` dlsym binding, same path as `bionic_strlen`).

## Why it advances the end goal

Reachable-path hardening (per SH131d + STATUS): removes the highest-probability
remaining silent SIGSEGV on the boot/do-init string path deterministically —
not a seed, not session-gated — turning the stable 24-frame artifact into a
bedrock for later session/cookie (SH129) work.

## Verification

- Hermetic `sh97_harden_guarded_string_ops_reject_garbage_no_segfault` (new):
  every guarded shim fed the 0xffffff80ffffffc8 class + sub-image int returns
  its safe default with **no SIGSEGV**; valid canonical strings still produce
  the real glibc result (equal/differ, strstr needle at offset, memchr hit).
- `cargo test --workspace`: **535 passed / 0 failed** (was 534).
- Real binary `runs/capture_sh130.sh`: **EXIT 0 / 24 real task-driven frames /
  0 SIGSEGV** — no boot-path regression.

## Scope / honesty

Default/product path unchanged for valid pointers (pure delegation);
behavior only differs for pointers that would fault. No session advance (still
behind the structural SH131d wall) — this hardens what the real client already
touches.